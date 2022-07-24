use std::io::{Read, Seek};
use std::time::Duration;

use crate::Source;

use cks_dec::decoder;
use cks_dec::{decoder::Decoder, sample::info::SampleInfo, FormatType};

pub struct CksDecoder<R>
where
    R: Read + Seek,
{
    decoder: Decoder<R>,
    sample_info: SampleInfo,
    buf: FormatType,
    current_frame_offset: usize,
}

impl<R> CksDecoder<R>
where
    R: Read + Seek,
{
    pub fn new(mut data: R) -> Result<Self, R> {
        if Decoder::new(data.by_ref()).is_err() {
            return Err(data);
        }

        let mut decoder = Decoder::new(data).unwrap();
        let sample_info: SampleInfo = decoder.sample_info();
        //let current_frame_num = 0;
        let mut buf = match sample_info.format {
            decoder::DecoderType::Adpcm => FormatType::new_int16(),
            _ => FormatType::new_int32(),
        };

        let _ = decoder.decode(&mut buf, 1);

        Ok(CksDecoder {
            decoder,
            sample_info,
            buf,
            //current_frame_num,
            current_frame_offset: 0,
        })
    }
    pub fn into_inner(self) -> R {
        self.decoder.into_inner()
    }
}

impl<R> Source for CksDecoder<R>
where
    R: Read + Seek,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.sample_info.block_bytes as _)
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.sample_info.channels as _
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.sample_info.sample_rate as _
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        let frame = self.sample_info.blocks * self.sample_info.block_frames as i32;
        let dur = frame as f32 / self.sample_info.sample_rate as f32;
        Some(Duration::from_secs_f32(dur))
    }
}

impl<R> Iterator for CksDecoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if let cks_dec::decoder::DecoderType::Adpcm = self.decoder.sample_info().format {
            if self.current_frame_offset == self.buf.len() {
                if self.decoder.next(&mut self.buf).is_none() {
                    return None;
                }
                self.current_frame_offset = 0;
            }

            if let FormatType::Int16(v) = &self.buf {
                let v = v[self.current_frame_offset];
                self.current_frame_offset += 1;
                
                Some(v)
            } else {
                None
            }
        } else {
            if self.current_frame_offset == self.sample_info.block_bytes as usize / 4 {
                if self.decoder.next(&mut self.buf).is_none() {
                    return None;
                }
                self.current_frame_offset = 0;
            }

            match &self.buf {
                FormatType::Int32(v) => {
                    let v = v[self.current_frame_offset];
                    self.current_frame_offset += 1;

                    Some(v as _)
                }
                FormatType::Float(_) => None,
                _ => None,
            }
        }
    }
}

use std::io::{BufRead, Seek, SeekFrom};

use image::{ColorType, ImageDecoder, ImageError, ImageResult, error::{DecodingError, ImageFormatHint}};

use crate::pat::error::PatError;


mod error;

pub struct PatDecoder<R> {
    reader: R,
    width: u32, 
    height: u32,
    color_palette: Vec<[u8; 3]>,
    limit: u64,
    // metadata: Metadata,
    // x_resolution: u32, 
    // y_resolution: u32,
}

impl<R: BufRead + Seek> PatDecoder<R> {
    
    pub fn new(mut reader: R) -> Result<PatDecoder<R>, PatError> {

        reader.seek(SeekFrom::Start(0))?;

        let mut wh_buf = [0u8; 2];
        
        reader.read_exact(&mut wh_buf)?;
        let width = u16::from_be_bytes([wh_buf[0], wh_buf[1]]) as u32;
        reader.read_exact(&mut wh_buf)?;
        let height = u16::from_be_bytes([wh_buf[0], wh_buf[1]]) as u32;

        let mut color_palette = Vec::with_capacity(256);
        let mut color_data = [0u8; 0x300];

        reader.seek(SeekFrom::Start(0x200))?;
        reader.read_exact(&mut color_data)?;

        //Planar grb
        for i in 0..256 {
            color_palette.push([color_data[0x100 + i], color_data[i], color_data[0x200 + i] ]);
        }

        // let color_palette = palette;
        let limit = 0x600 + width as u64 * height as u64 - 1;

        Ok(Self { reader, width, height, color_palette, limit})

        
    }

    

    fn read_pat_image(&mut self, buf: &mut [u8]) -> Result<(), PatError> {
        let mut pos = self.reader.seek(SeekFrom::Start(0x600))?;

        let mut index = 0;

        let mut nextbyte = [0u8];

        while pos <= self.limit {
            // let nextbyte = self.reader.read

            self.reader.read_exact(&mut nextbyte)?;


            let color = self.color_palette[nextbyte[0] as usize];

            buf[index] = color[0];
            buf[index + 1] = color[1];
            buf[index + 2] = color[2];

            // println!("POS: {:X} | OFFSET: {:X} | VAL : {:X}", pos, pos - 0x600, nextbyte[0]);

            pos = self.reader.stream_position()?;
            index = (pos as usize - 0x600) * 3;
            
        }
        

        Ok(())

    
    }

}

impl<R: BufRead + Seek> ImageDecoder for PatDecoder<R> {
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn color_type(&self) -> image::ColorType {
        image::ColorType::Rgb8
    }

    fn read_image(mut self, buf: &mut [u8]) -> ImageResult<()>
    where
        Self: Sized {

        
        let (w,h) = self.dimensions();
        assert_eq!(buf.len(), (w * h * ColorType::Rgb8.bytes_per_pixel() as u32) as usize, "Invalid buffer size");

        self.read_pat_image(buf).map_err(convert_error)
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
       (*self).read_image(buf) 
    }
}

pub fn convert_error(err: PatError) -> ImageError {
    match err {
        PatError::IoError(error) => ImageError::IoError(error),
        PatError::UnkownError => ImageError::Decoding(DecodingError::from_format_hint(format_hint())),
    }
}

fn format_hint() -> ImageFormatHint {
    ImageFormatHint::Name("PAT".into())
}

#[cfg(test)]
mod tests{ 

    use std::{fs::File, io::BufReader};

    use super::*;

    #[test]
    fn test_read_header() {
        // let file = "yuma.pat";

        // let reader = BufReader::new(File::open(file).expect("error opening file"));

        // let mut decoder = PatDecoder::new(reader).expect("error creating decorder");

        // decoder.read_pat_header().expect("Error reading header");

        // println!("{:X?}", decoder);
    }


    #[test]
    fn test_read() {
        // let file = "yuma.pat";

        // let reader = BufReader::new(File::open(file).expect("error opening file"));

        // let mut decoder = PatDecoder::new(reader).expect("error creating decoder");

        // decoder.read_pat_header().expect("Error reading header");

        // // println!("SIZE: {}", decoder.total_size());

        // let mut buf = [0u8; 1755000];

        // decoder.read_pat_image(&mut buf).expect("error read");
    }

        #[test]
    fn test_decoder() {
        let file = "yuma.pat";

        let reader = BufReader::new(File::open(file).expect("error opening file"));

        let decoder = PatDecoder::new(reader).expect("error creating decoder");

        // println!("SIZE: {}", decoder.total_size());

        let mut buf = [0u8; 1755000];

        decoder.read_image(&mut buf).expect("error read");


    }


}
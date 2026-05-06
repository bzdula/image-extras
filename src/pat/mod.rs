use std::io::{BufRead, Seek, SeekFrom};

use image::{
    error::{DecodingError, ImageFormatHint},
    ColorType, ImageDecoder, ImageError, ImageResult,
};

use crate::pat::error::PatError;

mod error;

fn read_rgb_image<R: BufRead + Seek>(
    reader: &mut R,
    dim: (u32, u32),
    buf: &mut [u8],
) -> Result<(), PatError> {
    reader.seek(SeekFrom::Start(0x600))?;

    let (w, h) = dim;

    let mut row_plane_r = vec![0u8; w as usize];
    let mut row_plane_g = vec![0u8; w as usize];
    let mut row_plane_b = vec![0u8; w as usize];

    for row in 0..h {
        reader.read_exact(&mut row_plane_r)?;
        reader.read_exact(&mut row_plane_g)?;
        reader.read_exact(&mut row_plane_b)?;

        for column in 0..w as usize {
            let base = (row as usize * w as usize + column) * 3;
            buf[base] = row_plane_r[column];
            buf[base + 1] = row_plane_g[column];
            buf[base + 2] = row_plane_b[column];
        }
    }

    Ok(())
}

fn read_grb_image<R: BufRead + Seek>(
    reader: &mut R,
    dim: (u32, u32),
    buf: &mut [u8],
    color_palette: &Vec<[u8; 3]>,
) -> Result<(), PatError> {
    reader.seek(SeekFrom::Start(0x600))?;

    let (w, h) = dim;
    let pixel_count = (w * h) as usize;

    let mut indices = vec![0u8; pixel_count];

    reader.read_exact(&mut indices)?;

    for (i, &index) in indices.iter().enumerate() {
        let color = color_palette[index as usize];

        let base = i * 3;

        buf[base] = color[0];
        buf[base + 1] = color[1];
        buf[base + 2] = color[2];
    }

    Ok(())
}

pub enum ColorMode {
    Rgb,
    Planar(Vec<[u8; 3]>),
}

pub struct PatDecoder<R> {
    reader: R,
    width: u32,
    height: u32,
    color_mode: ColorMode,
}

impl<R: BufRead + Seek> PatDecoder<R> {
    pub fn new(mut reader: R) -> Result<PatDecoder<R>, PatError> {
        reader.seek(SeekFrom::Start(0))?;

        let mut wh_buf = [0u8; 2];

        reader.read_exact(&mut wh_buf)?;
        let width = u16::from_be_bytes([wh_buf[0], wh_buf[1]]) as u32;
        reader.read_exact(&mut wh_buf)?;
        let mut height = u16::from_be_bytes([wh_buf[0], wh_buf[1]]) as u32;

        let mut color_data = [0u8; 0x300];

        reader.seek(SeekFrom::Start(0x200))?;
        reader.read_exact(&mut color_data)?;

        let color_mode = if color_data.iter().all(|byte| byte == &0u8) {
            height /= 3;
            //No custom palette, regular rgb color
            ColorMode::Rgb
        } else {
            //Planar grb

            let mut color_palette = Vec::with_capacity(256);

            for i in 0..256 {
                color_palette.push([color_data[0x100 + i], color_data[i], color_data[0x200 + i]]);
            }

            ColorMode::Planar(color_palette)
        };

        Ok(Self {
            reader,
            width,
            height,
            color_mode,
        })
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
        Self: Sized,
    {
        let (w, h) = self.dimensions();

        println!("DIM: {:?}", self.dimensions());

        assert_eq!(
            buf.len(),
            (w * h * ColorType::Rgb8.bytes_per_pixel() as u32) as usize,
            "Invalid buffer size"
        );

        match self.color_mode {
            ColorMode::Rgb => read_rgb_image(&mut self.reader, (w, h), buf).map_err(convert_error),
            ColorMode::Planar(color_palette) => {
                read_grb_image(&mut self.reader, (w,h), buf, &color_palette)
                    .map_err(convert_error)
            }
        }
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        (*self).read_image(buf)
    }
}

pub fn convert_error(err: PatError) -> ImageError {
    match err {
        PatError::IoError(error) => ImageError::IoError(error),
        PatError::UnkownError => {
            ImageError::Decoding(DecodingError::from_format_hint(format_hint()))
        }
    }
}

fn format_hint() -> ImageFormatHint {
    ImageFormatHint::Name("PAT".into())
}

#[cfg(test)]
mod tests {

    use std::{fs::File, io::BufReader};

    use image::{ImageBuffer, Rgb};

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
        let file = "skan.pat";

        let reader = BufReader::new(File::open(file).expect("error opening file"));

        let decoder = PatDecoder::new(reader).expect("error creating decoder");

        // println!("SIZE: {}", decoder.total_size());
        let (w, h) = decoder.dimensions();
        let size = (w * h * decoder.color_type().bytes_per_pixel() as u32) as usize;

        let mut buf = vec![0u8; size];

        println!("{w}x{h} = {size}");

        decoder.read_image(&mut buf).expect("error read");

        let image = ImageBuffer::<Rgb<u8>, _>::from_raw(w, h, buf).expect("invalid buffer");
        image.save("test.png").expect("save failed");
    }
}

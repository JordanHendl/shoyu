use image::GenericImageView;
use super::error::*;
pub struct ImageInfo<T> {
   pub size: [u32; 2],
   pub format: dashi::Format,
   pub bytes: Vec<T>,
}

pub fn load_image_rgba8(path: &str) -> Result<ImageInfo<u8>, Error>{
    let img = image::open(&path)?;

    // Convert the image to RGBA8 format
    let rgba_image = img.to_rgba8();

    // Flip the image vertically (upside down)
//    let rgba_image = image::imageops::flip_vertical(&rgba_image);

    let (width, height) = rgba_image.dimensions();
    let bytes = rgba_image.into_raw();
    assert!((width*height*4) as usize == bytes.len());
    Ok(ImageInfo::<u8> {
        size: [width, height],
        format: dashi::Format::RGBA8,
        bytes,
    })
}

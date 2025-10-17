use image::ImageFormat;

pub fn detect_mime(input: &[u8]) -> String {
    match image::guess_format(input) {
        Ok(fmt) => match fmt {
            ImageFormat::Png => "image/png".to_string(),
            ImageFormat::Jpeg => "image/jpeg".to_string(),
            ImageFormat::Gif => "image/gif".to_string(),
            ImageFormat::Bmp => "image/bmp".to_string(),
            ImageFormat::Ico => "image/x-icon".to_string(),
            ImageFormat::Tiff => "image/tiff".to_string(),
            ImageFormat::WebP => "image/webp".to_string(),
            ImageFormat::Avif => "image/avif".to_string(),
            _ => "application/octet-stream".to_string(),
        },
        Err(_) => "application/octet-stream".to_string(),
    }
}

use image::{DynamicImage, Rgb, ImageBuffer};

pub fn adjust_brightness(img: &DynamicImage, brightness: f32) -> DynamicImage {
    let img = img.to_rgb8();
    let (width, height) = img.dimensions();
    let mut new_img = ImageBuffer::new(width, height);

    // Scale brightness from [-1, 1] to [-0.5, 0.5]
    let scaled_brightness = brightness * 0.5;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let r = (pixel[0] as f32 + scaled_brightness * 255.0).clamp(0.0, 255.0) as u8;
            let g = (pixel[1] as f32 + scaled_brightness * 255.0).clamp(0.0, 255.0) as u8;
            let b = (pixel[2] as f32 + scaled_brightness * 255.0).clamp(0.0, 255.0) as u8;
            new_img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    DynamicImage::ImageRgb8(new_img)
} 
use image::{DynamicImage, Rgb, ImageBuffer};

pub fn adjust_contrast(img: &DynamicImage, contrast: f32) -> DynamicImage {
    let img = img.to_rgb8();
    let (width, height) = img.dimensions();
    let mut new_img = ImageBuffer::new(width, height);

    // Convert contrast from [-1, 1] to [0.25, 4.0] for more pronounced effect
    let factor = if contrast >= 0.0 {
        1.0 + contrast * 3.0  // Maps [0, 1] to [1, 4]
    } else {
        1.0 / (1.0 - contrast * 3.0)  // Maps [-1, 0] to [0.25, 1]
    };

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let r = ((pixel[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            let g = ((pixel[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            let b = ((pixel[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            new_img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    DynamicImage::ImageRgb8(new_img)
} 
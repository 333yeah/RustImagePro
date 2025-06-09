use image::DynamicImage;

pub fn analyze_image(img: &DynamicImage) -> (f32, f32) {
    let img = img.to_rgb8();
    let (width, height) = img.dimensions();
    
    // Calculate average brightness and standard deviation
    let mut total_brightness = 0.0;
    let mut total_pixels = 0;
    let mut brightness_values = Vec::new();
    
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let brightness = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / (3.0 * 255.0);
            total_brightness += brightness;
            brightness_values.push(brightness);
            total_pixels += 1;
        }
    }
    
    let avg_brightness = total_brightness / total_pixels as f32;
    
    // Calculate standard deviation
    let variance = brightness_values.iter()
        .map(|&b| (b - avg_brightness).powi(2))
        .sum::<f32>() / total_pixels as f32;
    let std_dev = variance.sqrt();
    
    // Calculate auto brightness adjustment
    // Target brightness is 0.5 (middle gray)
    let brightness_adjust = (0.5 - avg_brightness) * 2.0; // Scale to [-1, 1] range
    
    // Calculate auto contrast adjustment
    // Target standard deviation is 0.2
    let contrast_adjust = if std_dev < 0.1 {
        // Low contrast image, increase contrast
        0.5
    } else if std_dev > 0.3 {
        // High contrast image, decrease contrast
        -0.3
    } else {
        // Normal contrast image, slight adjustment
        0.1
    };
    
    (brightness_adjust, contrast_adjust)
} 
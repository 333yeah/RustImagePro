use image::{DynamicImage, Rgb, ImageBuffer};
use rayon::prelude::*;

#[derive(Clone)]
pub struct ImageBlock {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub overlap: u32,
}

impl ImageBlock {
    pub fn new(x: u32, y: u32, width: u32, height: u32, overlap: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            data: vec![0; (width * height * 3) as usize],
            overlap,
        }
    }
}

pub fn split_image_into_blocks(img: &DynamicImage, block_size: u32) -> Vec<ImageBlock> {
    let img = img.to_rgb8();
    let (width, height) = img.dimensions();
    let mut blocks = Vec::new();
    let overlap = block_size / 4;

    for y in (0..height).step_by((block_size - overlap) as usize) {
        for x in (0..width).step_by((block_size - overlap) as usize) {
            let block_width = (width - x).min(block_size);
            let block_height = (height - y).min(block_size);
            
            let mut block = ImageBlock::new(x, y, block_width, block_height, overlap);
            
            for by in 0..block_height {
                for bx in 0..block_width {
                    let src_x = x + bx;
                    let src_y = y + by;
                    if src_x < width && src_y < height {
                        let pixel = img.get_pixel(src_x, src_y);
                        let idx = ((by * block_width + bx) * 3) as usize;
                        block.data[idx] = pixel[0];
                        block.data[idx + 1] = pixel[1];
                        block.data[idx + 2] = pixel[2];
                    }
                }
            }
            
            blocks.push(block);
        }
    }
    
    blocks
}

pub fn merge_blocks_into_image(blocks: Vec<ImageBlock>, width: u32, height: u32) -> DynamicImage {
    let mut img = ImageBuffer::new(width, height);
    let mut weights = vec![vec![0.0f32; width as usize]; height as usize];
    
    // Initialize image with zeros
    for y in 0..height {
        for x in 0..width {
            img.put_pixel(x, y, Rgb([0, 0, 0]));
        }
    }
    
    for block in blocks {
        for y in 0..block.height {
            for x in 0..block.width {
                let src_x = block.x + x;
                let src_y = block.y + y;
                if src_x < width && src_y < height {
                    let idx = ((y * block.width + x) * 3) as usize;
                    let pixel = Rgb([
                        block.data[idx],
                        block.data[idx + 1],
                        block.data[idx + 2],
                    ]);
                    
                    // Calculate weight
                    let weight = calculate_weight(x, y, block.width, block.height, block.overlap);
                    
                    // Accumulate weighted pixel values
                    let current_pixel: &Rgb<u8> = img.get_pixel(src_x, src_y);
                    let new_pixel = Rgb([
                        ((current_pixel[0] as f32 * weights[src_y as usize][src_x as usize] + 
                          pixel[0] as f32 * weight) / 
                         (weights[src_y as usize][src_x as usize] + weight)) as u8,
                        ((current_pixel[1] as f32 * weights[src_y as usize][src_x as usize] + 
                          pixel[1] as f32 * weight) / 
                         (weights[src_y as usize][src_x as usize] + weight)) as u8,
                        ((current_pixel[2] as f32 * weights[src_y as usize][src_x as usize] + 
                          pixel[2] as f32 * weight) / 
                         (weights[src_y as usize][src_x as usize] + weight)) as u8,
                    ]);
                    
                    img.put_pixel(src_x, src_y, new_pixel);
                    weights[src_y as usize][src_x as usize] += weight;
                }
            }
        }
    }
    
    // Handle zero-weight regions (boundaries)
    for y in 0..height {
        for x in 0..width {
            if weights[y as usize][x as usize] == 0.0 {
                // Use nearest valid pixel value
                let mut nearest_pixel = Rgb([0, 0, 0]);
                let mut min_dist = f32::MAX;
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let dist = (dx * dx + dy * dy) as f32;
                            if dist < min_dist && weights[ny as usize][nx as usize] > 0.0 {
                                min_dist = dist;
                                nearest_pixel = *img.get_pixel(nx as u32, ny as u32);
                            }
                        }
                    }
                }
                
                img.put_pixel(x, y, nearest_pixel);
            }
        }
    }
    
    DynamicImage::ImageRgb8(img)
}

fn calculate_weight(x: u32, y: u32, width: u32, height: u32, overlap: u32) -> f32 {
    let x_weight = if x < overlap {
        (x as f32 / overlap as f32).powf(1.5) // Use 1.5 power for smoother transition
    } else if x >= width - overlap {
        ((width - x - 1) as f32 / overlap as f32).powf(1.5)
    } else {
        1.0
    };
    
    let y_weight = if y < overlap {
        (y as f32 / overlap as f32).powf(1.5)
    } else if y >= height - overlap {
        ((height - y - 1) as f32 / overlap as f32).powf(1.5)
    } else {
        1.0
    };
    
    x_weight * y_weight
}

pub fn process_blocks_parallel<F>(blocks: Vec<ImageBlock>, process_fn: F) -> Vec<ImageBlock>
where
    F: Fn(&ImageBlock) -> ImageBlock + Send + Sync,
{
    blocks.par_iter().map(|block| process_fn(block)).collect()
}

pub fn process_image_parallel<F>(img: &DynamicImage, block_size: u32, process_fn: F) -> DynamicImage
where
    F: Fn(&ImageBlock) -> ImageBlock + Send + Sync,
{
    let blocks = split_image_into_blocks(img, block_size);
    let processed_blocks = process_blocks_parallel(blocks, process_fn);
    merge_blocks_into_image(processed_blocks, img.width(), img.height())
} 
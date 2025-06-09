use eframe::egui;
use eframe::egui::ViewportBuilder;
use image::{DynamicImage, ImageBuffer};
use rfd::FileDialog;

mod algorithms;
mod image_loader;

use algorithms::{denoise::*, brightness::*, contrast::*, sharpness::*, auto_adjust::*, parallel::*};
use image_loader::load_image;

fn main() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Image Processing",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );
}

struct MyApp {
    original_image: Option<DynamicImage>,
    denoised_image: Option<DynamicImage>,
    denoise_type: DenoiseType,
    kernel_size: usize,
    brightness: f32,
    contrast: f32,
    sharpness: f32,
    tv_lambda: f32,
    tv_iterations: usize,
    processing_time: Option<std::time::Duration>,
    use_parallel: bool,
    block_size: u32,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            original_image: None,
            denoised_image: None,
            denoise_type: DenoiseType::MeanFilter,
            kernel_size: 3,
            brightness: 0.0,
            contrast: 0.0,
            sharpness: 0.0,
            tv_lambda: 0.1,
            tv_iterations: 50,
            processing_time: None,
            use_parallel: false,
            block_size: 64,
        }
    }

    fn auto_optimize(&mut self) {
        if let Some(img) = &self.original_image {
            // Analyze image and get auto adjustments
            let (auto_brightness, auto_contrast) = analyze_image(img);
            
            // Apply auto adjustments
            self.brightness = auto_brightness;
            self.contrast = auto_contrast;
            self.sharpness = 1.0; // Default sharpness value
            self.kernel_size = 6; // Larger kernel size for better denoising
            
            // Apply denoising and adjustments using the same method as manual optimization
            let (denoised, duration) = self.apply_denoising(img, self.denoise_type, self.kernel_size);
            self.denoised_image = Some(denoised);
            self.processing_time = Some(duration);
        }
    }

    fn export_image(&self) {
        if let Some(img) = &self.denoised_image {
            if let Some(path) = FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .set_directory(".")
                .save_file()
            {
                let _ = img.save(path);
            }
        }
    }

    fn apply_denoising(
        &self,
        img: &DynamicImage,
        denoise_type: DenoiseType,
        kernel_size: usize,
    ) -> (DynamicImage, std::time::Duration) {
        let start_time = std::time::Instant::now();
        let mut current_img = img.clone();

        if self.use_parallel {
            current_img = process_image_parallel(&current_img, self.block_size, |block| {
                let mut block_img = DynamicImage::ImageRgb8(ImageBuffer::from_raw(
                    block.width,
                    block.height,
                    block.data.clone(),
                ).unwrap());
                
                block_img = denoise_image(
                    &block_img,
                    denoise_type,
                    kernel_size,
                    self.tv_lambda,
                    self.tv_iterations
                );

                if self.brightness != 0.0 {
                    block_img = adjust_brightness(&block_img, self.brightness);
                }

                if self.contrast != 1.0 {
                    block_img = adjust_contrast(&block_img, self.contrast);
                }

                if self.sharpness > 0.0 {
                    block_img = sharpen_image(&block_img, self.sharpness);
                }

                let rgb = block_img.to_rgb8();
                ImageBlock {
                    x: block.x,
                    y: block.y,
                    width: block.width,
                    height: block.height,
                    data: rgb.into_raw(),
                    overlap: block.overlap,
                }
            });
        } else {
            current_img = denoise_image(
                &current_img, 
                denoise_type, 
                kernel_size,
                self.tv_lambda,
                self.tv_iterations
            );

            if self.brightness != 0.0 {
                current_img = adjust_brightness(&current_img, self.brightness);
            }

            if self.contrast != 1.0 {
                current_img = adjust_contrast(&current_img, self.contrast);
            }

            if self.sharpness > 0.0 {
                current_img = sharpen_image(&current_img, self.sharpness);
            }
        }

        let duration = start_time.elapsed();
        (current_img, duration)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(25.0);
            ui.horizontal(|ui| {
                ui.add_space(25.0);
                ui.vertical(|ui| {
                    ui.heading(egui::RichText::new("Image Processing").size(30.0));

                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("Select Image").size(16.0)).min_size(egui::vec2(120.0, 40.0))).clicked() {
                            self.original_image = load_image();
                            self.denoised_image = None;
                            self.processing_time = None;
                        }

                        if self.denoised_image.is_some() {
                            ui.add_space(300.0);
                            if ui.add(egui::Button::new(egui::RichText::new("Export Image").size(16.0)).min_size(egui::vec2(120.0, 40.0))).clicked() {
                                self.export_image();
                            }
                        }
                    });

                    if let Some(original) = &self.original_image {
                        let original_width = original.width();
                        let original_height = original.height();
                        let original_data = original.to_rgba8().to_vec();

                        ui.horizontal(|ui| {
                            // Left side - Original image
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Original Image:").size(18.0));
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [original_width as usize, original_height as usize],
                                    &original_data,
                                );
                                let texture_handle = ctx.load_texture("original", color_image, Default::default());
                                let scale = 400.0 / original_height as f32;
                                let size = egui::vec2(original_width as f32 * scale, 400.0);
                                ui.image((texture_handle.id(), size));
                            });

                            // Add spacing between images
                            ui.add_space(20.0);

                            // Right side - Denoised image
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Denoised Image:").size(18.0));

                                if let Some(denoised) = &self.denoised_image {
                                    let denoised_width = denoised.width();
                                    let denoised_height = denoised.height();
                                    let denoised_data = denoised.to_rgba8().to_vec();
                                    
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        [denoised_width as usize, denoised_height as usize],
                                        &denoised_data,
                                    );
                                    let texture_handle = ctx.load_texture("denoised", color_image, Default::default());
                                    let scale = 400.0 / denoised_height as f32;
                                    let size = egui::vec2(denoised_width as f32 * scale, 400.0);
                                    ui.image((texture_handle.id(), size));

                                    if let Some(duration) = self.processing_time {
                                        ui.label(egui::RichText::new(format!("Processing Time: {:.3} seconds", duration.as_secs_f64())).size(16.0));
                                    }
                                }
                            });
                        });

                        // Image adjustments section
                        ui.separator();
                        ui.horizontal(|ui| {
                            // Denoising parameters
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Denoising Parameters:").size(16.0));
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Denoise type:").size(16.0));
                                    egui::ComboBox::from_id_source("denoise_type")
                                        .selected_text(format!("{:?}", self.denoise_type))
                                        .show_ui(ui, |ui| {
                                            for denoise_type in [
                                                DenoiseType::MeanFilter,
                                                DenoiseType::GaussianFilter,
                                                DenoiseType::MedianFilter,
                                                DenoiseType::BilateralFilter,
                                                DenoiseType::NonLocalMeans,
                                                DenoiseType::TotalVariation,
                                            ] {
                                                ui.selectable_value(&mut self.denoise_type, denoise_type, format!("{:?}", denoise_type));
                                            }
                                        });
                                });

                                if self.denoise_type != DenoiseType::NonLocalMeans {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Kernel size:").size(16.0));
                                        ui.add(egui::Slider::new(&mut self.kernel_size, 3..=9).text("size"));
                                    });
                                }

                                // Parallel processing options
                                ui.vertical(|ui| {
                                    ui.checkbox(&mut self.use_parallel, egui::RichText::new("Use Parallel Processing").size(16.0));
                                    if self.use_parallel {
                                        ui.horizontal(|ui| {
                                            ui.add_space(20.0);
                                            ui.label(egui::RichText::new("Block Size:").size(16.0));
                                            ui.add(egui::Slider::new(&mut self.block_size, 32..=256).step_by(32.0).text("pixels"));
                                        });
                                    }
                                });
                            });

                            // Image adjustments
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(150.0);
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new("Image Adjustments:").size(16.0));
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Brightness:").size(16.0));
                                            ui.add(egui::Slider::new(&mut self.brightness, -1.0..=1.0).step_by(0.01));
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Contrast:").size(16.0));
                                            ui.add(egui::Slider::new(&mut self.contrast, -1.0..=1.0).step_by(0.01));
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Sharpness:").size(16.0));
                                            ui.add(egui::Slider::new(&mut self.sharpness, -1.0..=1.0).step_by(0.01));
                                        });
                                    });
                                });
                            });
                        });

                        // Action buttons
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("Apply Denoising").size(16.0)).min_size(egui::vec2(120.0, 40.0))).clicked() {
                                if let Some(img) = &self.original_image {
                                    let (denoised, duration) = self.apply_denoising(img, self.denoise_type, self.kernel_size);
                                    self.denoised_image = Some(denoised);
                                    self.processing_time = Some(duration);
                                }
                            }

                            if ui.add(egui::Button::new(egui::RichText::new("Auto Optimize").size(16.0)).min_size(egui::vec2(120.0, 40.0))).clicked() {
                                self.auto_optimize();
                            }
                        });
                    }
                });
            });
        });
    }
} 
use image::DynamicImage;
use rfd::FileDialog;

pub fn load_image() -> Option<DynamicImage> {
    if let Some(path) = FileDialog::new().pick_file() {
        if let Ok(img) = image::open(path) {
            return Some(img);
        }
    }
    None
} 
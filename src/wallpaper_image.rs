use std::path::PathBuf;

use iced::widget::image;

#[derive(Debug, Clone)]
pub struct WallpaperImage {
    pub name: String,
    pub img_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub thumbnail_handle: Option<image::Handle>,
    pub is_visible: bool,
    pub is_loading: bool,
}

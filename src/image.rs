use iced::widget::image;

#[derive(Debug, Clone)]
pub struct WallpaperImage {
    pub name: String,
    pub img_path: String,
    pub thumbnail_path: String,
    pub thumbnail_handle: Option<image::Handle>,
    pub is_visible: bool,
    pub has_thumbnail: bool,
}

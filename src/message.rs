use iced::{
    keyboard::Key,
    widget::{image as iced_image, scrollable},
};

#[derive(Debug, Clone)]
pub enum Message {
    WallpaperDiscovered(crate::image::WallpaperImage),
    ScrolledTo(scrollable::Viewport),
    LoadVisibleThumbnails,
    ThumbnailLoaded(usize, iced_image::Handle),
    ImageHovered(Option<usize>),
    KeyPressed(Key),
    WallpaperSelected,
    ScrollToVisible,
}

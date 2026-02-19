use iced::{Size, window::Settings};

use crate::ui::AppView;

mod config;
mod constants;
mod events;
mod message;
mod thumbnail;
mod ui;
mod util;
mod wallpaper;
mod wallpaper_image;

fn main() -> iced::Result {
    iced::application(AppView::new, AppView::update, AppView::view)
        .title("Wallity - Wallpaper Manager")
        .subscription(AppView::subscription)
        .centered()
        .window(Settings {
            resizable: false,
            size: Size::new(896.0, 800.0),
            ..Settings::default()
        })
        .run()
}

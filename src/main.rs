use iced::{Size, window::Settings};

use crate::ui::AppView;

mod config;
mod events;
mod image;
mod message;
mod thumbnail;
mod ui;
mod util;
mod wallpaper;

fn main() -> iced::Result {
    iced::application(AppView::new, AppView::update, AppView::view)
        .subscription(AppView::subscription)
        .centered()
        .window(Settings {
            resizable: false,
            size: Size::new(896.0, 800.0),
            ..Settings::default()
        })
        .run()
}

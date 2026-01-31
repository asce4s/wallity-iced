use iced::{Subscription, futures::SinkExt};
use std::sync::mpsc;

use crate::{message::Message, wallpaper::load_wallpapers};

pub fn wallpaper_stream() -> Subscription<Message> {
    use iced::futures::channel::mpsc as futures_mpsc;
    use iced::stream;

    Subscription::run(|| {
        stream::channel(
            100,
            |mut output: futures_mpsc::Sender<Message>| async move {
                let (tx, rx) = mpsc::channel();

                std::thread::spawn(move || {
                    // This move captures tx
                    let _ = load_wallpapers(move |img: crate::image::WallpaperImage| {
                        tx.send(img).ok();
                    });
                });

                while let Ok(img) = rx.recv() {
                    let _ = output.send(Message::WallpaperDiscovered(img)).await;
                }
                // Keep the stream alive
                loop {
                    std::future::pending::<()>().await;
                }
            },
        )
    })
}

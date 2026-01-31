use iced::futures::StreamExt;
use iced::{Subscription, futures::SinkExt};
use std::sync::mpsc;

use crate::{message::Message, wallpaper::load_wallpapers};
use iced::futures::channel::mpsc as futures_mpsc;
use iced::stream;

pub fn wallpaper_stream() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(
            500,
            |mut output: futures_mpsc::Sender<Message>| async move {
                let (tx, rx) = mpsc::channel();
                let (bridge_tx, mut bridge_rx) = futures_mpsc::channel(500);
                let bridge_tx_clone = bridge_tx.clone();

                std::thread::spawn(move || {
                    while let Ok(img) = rx.recv() {
                        // Use try_send to avoid blocking, or use a runtime with block_on
                        // For simplicity, we'll use try_send and accept that it might fail if full
                        if bridge_tx_clone
                            .clone()
                            .try_send(Message::WallpaperDiscovered(img))
                            .is_err()
                        {
                            // Channel closed or full - this shouldn't happen normally
                            break;
                        }
                    }
                });

                let _ = load_wallpapers(tx);

                while let Some(img) = bridge_rx.next().await {
                    let _ = output.send(img).await;
                }

                loop {
                    std::future::pending::<()>().await;
                }
            },
        )
    })
}

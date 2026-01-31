use crate::{
    config::CONFIG,
    image::WallpaperImage,
    thumbnail::{gen_thumbnail, list_thumbnails},
};

use iced::widget::image;
use rayon::prelude::*;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub fn load_wallpapers<F>(callback: F) -> Result<(), String>
where
    F: FnMut(WallpaperImage) + Send + 'static,
{
    let exts = ["png", "jpg", "jpeg", "webp"];
    let callback = Arc::new(Mutex::new(callback));

    std::thread::spawn(move || {
        let thumbnails: HashSet<String> = list_thumbnails();
        let thumbnail_path = CONFIG
            .cache_path
            .clone()
            .unwrap()
            .to_string_lossy()
            .trim_end_matches("/")
            .to_string();

        let absolute_path = CONFIG
            .wallpaper_path
            .as_ref()
            .expect("Wallpaper Path not found");

        if let Ok(entries) = std::fs::read_dir(absolute_path) {
            let entries: Vec<_> = entries
                .flatten()
                .filter(|entry| entry.file_type().map(|f| f.is_file()).unwrap_or(false))
                .collect();

            // Collect valid wallpaper file stems for cleanup later
            let valid_stems: HashSet<String> = entries
                .par_iter()
                .filter_map(|entry| {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if exts.contains(&ext.as_str()) {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            // Separate entries into those with/without thumbnails
            let (with_thumbnails, without_thumbnails): (Vec<_>, Vec<_>) =
                entries.into_par_iter().partition(|entry| {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| thumbnails.contains(s))
                        .unwrap_or(false)
                });

            // Process images WITH existing thumbnails first (instant UI feedback)
            with_thumbnails.par_iter().for_each(|entry| {
                let path = entry.path();

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !exts.contains(&ext.as_str()) {
                    return;
                }

                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let img_path = path.to_string_lossy().to_string();
                let thumbnail_path = format!("{}/{}.webp", &thumbnail_path, &file_stem);
                let image = WallpaperImage {
                    name: file_name,
                    img_path,
                    thumbnail_path,
                    thumbnail_handle: None,
                    is_visible: false,
                };
                if let Ok(mut cb) = callback.lock() {
                    cb(image);
                }
            });

            // Then generate missing thumbnails and emit (doesn't block above)
            without_thumbnails.par_iter().for_each(|entry| {
                let path = entry.path();

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !exts.contains(&ext.as_str()) {
                    return;
                }

                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let img_path = path.to_string_lossy().to_string();
                let thumbnail_path = format!("{}/{}.webp", &thumbnail_path, &file_stem);

                if gen_thumbnail(&img_path, &thumbnail_path).is_ok() {
                    let image = WallpaperImage {
                        name: file_name,
                        img_path,
                        thumbnail_handle: None,
                        thumbnail_path,
                        is_visible: false,
                    };

                    if let Ok(mut cb) = callback.lock() {
                        cb(image);
                    }
                } else {
                    eprintln!("Failed to generate thumbnail for: {}", img_path);
                }
            });

            // Clean up orphaned thumbnails after main processing
            for thumbnail_stem in thumbnails {
                if !valid_stems.contains(&thumbnail_stem) {
                    let thumbnail_file = format!("{}/{}.webp", &thumbnail_path, &thumbnail_stem);
                    if let Err(e) = std::fs::remove_file(&thumbnail_file) {
                        eprintln!(
                            "Failed to remove orphaned thumbnail {}: {}",
                            thumbnail_file, e
                        );
                    }
                }
            }
        }
    });

    Ok(())
}

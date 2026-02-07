use crate::{
    config::CONFIG,
    image::WallpaperImage,
    thumbnail::{gen_thumbnail, list_thumbnails},
};

use rayon::prelude::*;
use std::{collections::HashSet, path::PathBuf, sync::mpsc};

pub fn load_wallpapers(tx: mpsc::SyncSender<WallpaperImage>) -> Result<(), String> {
    let exts = ["png", "jpg", "jpeg", "webp"];

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

            // Single scan: collect valid stems and split by thumbnail presence
            let mut valid_stems: HashSet<String> = HashSet::new();
            let mut with_thumbnails: Vec<_> = Vec::new();
            let mut without_thumbnails: Vec<_> = Vec::new();

            for entry in entries {
                let path = entry.path();
                let ext_ok = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| exts.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
                    .unwrap_or(false);
                if !ext_ok {
                    continue;
                }

                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    valid_stems.insert(stem.to_string());
                    if thumbnails.contains(stem) {
                        with_thumbnails.push(entry);
                    } else {
                        without_thumbnails.push(entry);
                    }
                }
            }

            // Process images WITH existing thumbnails first (instant UI feedback)
            with_thumbnails
                .par_iter()
                .for_each_with(tx.clone(), |tx, entry| {
                    let path = entry.path();

                    let ext_ok = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| exts.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
                        .unwrap_or(false);
                    if !ext_ok {
                        return;
                    }

                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let img_path = path;
                    let thumbnail_path =
                        PathBuf::from(format!("{}/{}.jpeg", &thumbnail_path, &file_stem));
                    let image = WallpaperImage {
                        name: file_name,
                        img_path,
                        thumbnail_path,
                        thumbnail_handle: None,
                        is_visible: false,
                        is_loading: false,
                    };
                    let _ = tx.send(image);
                });

            // Then generate missing thumbnails and emit (doesn't block above)
            without_thumbnails
                .par_iter()
                .for_each_with(tx.clone(), |tx, entry| {
                    let path = entry.path();

                    let ext_ok = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| exts.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
                        .unwrap_or(false);
                    if !ext_ok {
                        return;
                    }

                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let img_path = path;
                    let thumbnail_path =
                        PathBuf::from(format!("{}/{}.jpeg", &thumbnail_path, &file_stem));

                    let image = WallpaperImage {
                        name: file_name,
                        img_path,
                        thumbnail_handle: None,
                        thumbnail_path,
                        is_visible: false,
                        is_loading: false,
                    };

                    if gen_thumbnail(&image.img_path, &image.thumbnail_path).is_ok() {
                        let _ = tx.send(image);
                    } else {
                        eprintln!("Failed to generate thumbnail for: {}", image.name);
                    }
                });

            // Clean up orphaned thumbnails after main processing
            for thumbnail_stem in thumbnails {
                if !valid_stems.contains(&thumbnail_stem) {
                    let thumbnail_file = format!("{}/{}.jpeg", &thumbnail_path, &thumbnail_stem);
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

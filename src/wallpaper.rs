use crate::{
    config::CONFIG,
    image::WallpaperImage,
    thumbnail::{gen_thumbnail, list_thumbnails},
};

use rayon::prelude::*;
use std::{collections::HashSet, path::Path, path::PathBuf, sync::mpsc};

const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|&supported| ext.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}

pub fn load_wallpapers(tx: mpsc::SyncSender<WallpaperImage>) -> Result<(), String> {
    std::thread::spawn(move || {
        let thumbnails: HashSet<String> = list_thumbnails();
        let Some(ref thumbnail_path_base) = CONFIG.cache_path else {
            eprintln!("Cache path not configured");
            return;
        };
        let thumbnail_path_str = thumbnail_path_base.to_string_lossy();

        let Some(ref absolute_path) = CONFIG.wallpaper_path else {
            eprintln!("Wallpaper Path not configured");
            return;
        };

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
                if !is_supported_extension(&path) {
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
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let thumbnail_path =
                        PathBuf::from(format!("{}/{}.jpeg", &thumbnail_path_str, &file_stem));
                    let image = WallpaperImage {
                        name: file_name,
                        img_path: path,
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
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let thumbnail_path =
                        PathBuf::from(format!("{}/{}.jpeg", &thumbnail_path_str, &file_stem));

                    let image = WallpaperImage {
                        name: file_name,
                        img_path: path,
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
                    let thumbnail_file =
                        format!("{}/{}.jpeg", &thumbnail_path_str, &thumbnail_stem);
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

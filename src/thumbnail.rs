use std::{
    collections::HashSet,
    fs::{self},
};

use crate::config::CONFIG;

use image::ImageFormat;
use resolve_path::PathResolveExt;

pub fn gen_thumbnail(input: &str, output: &str) -> anyhow::Result<()> {
    let img = image::open(input)?;
    let thumb = img.thumbnail(320, 150);
    thumb.save_with_format(output, ImageFormat::WebP)?;
    Ok(())
}

pub fn list_thumbnails() -> HashSet<String> {
    let path = CONFIG.cache_path.as_ref().unwrap().try_resolve().unwrap();
    let dir = fs::read_dir(&path);

    dir.unwrap()
        .filter_map(|entry| {
            entry
                .ok()
                .map(|v| v.path().file_stem().unwrap().to_string_lossy().to_string())
        })
        .collect()
}

use std::{
    collections::HashSet,
    fs::{self},
    path::Path,
};

use crate::config::CONFIG;

use image::ImageFormat;

pub fn gen_thumbnail(input: &Path, output: &Path) -> anyhow::Result<()> {
    let img = image::open(input)?;
    let thumb = img.thumbnail(320, 150);
    thumb.save_with_format(output, ImageFormat::Jpeg)?;
    Ok(())
}

pub fn list_thumbnails() -> HashSet<String> {
    let Some(ref path) = CONFIG.cache_path else {
        return HashSet::new();
    };

    let Ok(dir) = fs::read_dir(path) else {
        return HashSet::new();
    };

    dir.filter_map(|entry| {
        entry
            .ok()
            .and_then(|v| v.path().file_stem().map(|s| s.to_string_lossy().to_string()))
    })
    .collect()
}

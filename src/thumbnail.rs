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

    list_thumbnails_from_path(path)
}

fn list_thumbnails_from_path(path: &Path) -> HashSet<String> {
    let Ok(dir) = fs::read_dir(path) else {
        return HashSet::new();
    };

    dir.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str())? == "jpeg" {
            path.file_stem().map(|s| s.to_string_lossy().to_string())
        } else {
            None
        }
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use image::{RgbImage, ImageFormat};

    #[test]
    fn test_gen_thumbnail() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.png");
        let output_path = dir.path().join("output.jpeg");

        // Create a dummy image
        let img = RgbImage::new(100, 100);
        img.save_with_format(&input_path, ImageFormat::Png).unwrap();

        let result = gen_thumbnail(&input_path, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify it's a valid image
        let thumb = image::open(&output_path).unwrap();
        // 100x100 image scaled to fit 320x150 should be 150x150
        assert_eq!(thumb.width(), 150);
        assert_eq!(thumb.height(), 150);
    }

    #[test]
    fn test_list_thumbnails_from_path() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("thumb1.jpeg")).unwrap();
        File::create(dir.path().join("thumb2.jpeg")).unwrap();
        File::create(dir.path().join("not_a_thumb.txt")).unwrap();

        let thumbnails = list_thumbnails_from_path(dir.path());
        assert_eq!(thumbnails.len(), 2);
        assert!(thumbnails.contains("thumb1"));
        assert!(thumbnails.contains("thumb2"));
        assert!(!thumbnails.contains("not_a_thumb"));
    }
}

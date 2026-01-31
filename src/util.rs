use std::{fs, path::PathBuf};

use resolve_path::PathResolveExt;

pub fn get_absolute_path(path: &str) -> anyhow::Result<PathBuf> {
    let resolved = path.try_resolve()?.to_path_buf();
    Ok(resolved)
}
pub fn resolve_file_path(path: &str) -> anyhow::Result<PathBuf> {
    let resolved = get_absolute_path(path).expect("Failed to resolve file path");
    let parent = resolved.parent().unwrap();

    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    Ok(resolved)
}

pub fn resolve_dir_path(path: &str) -> anyhow::Result<PathBuf> {
    let resolved = get_absolute_path(path).expect("Failed to resolve file path");
    if !resolved.exists() {
        fs::create_dir_all(&resolved)?;
    }

    Ok(resolved)
}

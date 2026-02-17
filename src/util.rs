use anyhow::{Context, Result};
use resolve_path::PathResolveExt;
use std::{fs, path::PathBuf};

pub fn get_absolute_path(path: &str) -> Result<PathBuf> {
    path.try_resolve()
        .map(|p| p.to_path_buf())
        .with_context(|| format!("Failed to resolve path: {}", path))
}

pub fn resolve_file_path(path: &str) -> Result<PathBuf> {
    let resolved = get_absolute_path(path)?;
    if let Some(parent) = resolved.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory for: {}", path))?;
        }
    }

    Ok(resolved)
}

pub fn resolve_dir_path(path: &str) -> Result<PathBuf> {
    let resolved = get_absolute_path(path)?;
    if !resolved.exists() {
        fs::create_dir_all(&resolved)
            .with_context(|| format!("Failed to create directory: {}", path))?;
    }

    Ok(resolved)
}

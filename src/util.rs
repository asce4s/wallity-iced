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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_absolute_path() {
        // Test with a regular relative path
        let path = "src/main.rs";
        let resolved = get_absolute_path(path).unwrap();
        assert!(resolved.is_absolute());
        assert!(resolved.ends_with("src/main.rs"));

        // Test with tilde
        let tilde_path = "~/some_file";
        let resolved = get_absolute_path(tilde_path).unwrap();
        assert!(resolved.is_absolute());
        // We don't necessarily know the exact home dir, but it should be absolute and contain "some_file"
        assert!(resolved.to_string_lossy().contains("some_file"));
    }

    #[test]
    fn test_resolve_file_path() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sub/dir");
        let file_path = sub_dir.join("test.txt");
        let file_path_str = file_path.to_string_lossy();

        assert!(!sub_dir.exists());

        let resolved = resolve_file_path(&file_path_str).unwrap();
        assert_eq!(resolved, file_path);
        assert!(sub_dir.exists());
        assert!(sub_dir.is_dir());
    }

    #[test]
    fn test_resolve_dir_path() {
        let dir = tempdir().unwrap();
        let new_dir = dir.path().join("new_dir");
        let new_dir_str = new_dir.to_string_lossy();

        assert!(!new_dir.exists());

        let resolved = resolve_dir_path(&new_dir_str).unwrap();
        assert_eq!(resolved, new_dir);
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }
}

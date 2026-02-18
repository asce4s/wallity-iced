use crate::util::{get_absolute_path, resolve_dir_path, resolve_file_path};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load configuration: {}. Using defaults.", e);
        AppConfig::default()
    })
});

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub wallpaper_path: Option<PathBuf>,
    pub current_wallpaper: Option<PathBuf>,
    pub post_script: Option<String>,
    pub cache_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wallpaper_path: get_absolute_path("~/Pictures/wallpapers").ok(),
            current_wallpaper: get_absolute_path("~/.config/wallity/.current_wallpaper").ok(),
            post_script: Some(String::from("")),
            cache_path: get_absolute_path("~/.cache/wallity/thumbnails").ok(),
        }
    }
}

impl AppConfig {
    fn merge(mut self, other: AppConfig) -> Self {
        if let Some(path) = other.wallpaper_path {
            self.wallpaper_path = get_absolute_path(&path.to_string_lossy()).ok();
        }
        if let Some(path) = other.current_wallpaper {
            self.current_wallpaper = get_absolute_path(&path.to_string_lossy()).ok();
        }
        if other.post_script.is_some() {
            self.post_script = other.post_script;
        }
        if let Some(path) = other.cache_path {
            self.cache_path = get_absolute_path(&path.to_string_lossy()).ok();
        }
        self
    }

    pub fn empty() -> Self {
        Self {
            wallpaper_path: None,
            current_wallpaper: None,
            post_script: None,
            cache_path: None,
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        let config_path = resolve_file_path("~/.config/wallity/wallity.toml")?;
        let config_str = fs::read_to_string(&config_path).unwrap_or_default();

        let user_config: AppConfig = toml::from_str(&config_str).unwrap_or_else(|_| Self::empty());
        let merged_config = Self::default().merge(user_config);

        if let Some(ref current) = merged_config.current_wallpaper {
            let _ = resolve_file_path(&current.to_string_lossy());
        }
        if let Some(ref cache) = merged_config.cache_path {
            let _ = resolve_dir_path(&cache.to_string_lossy());
        }

        Ok(merged_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.post_script.is_some());
    }

    #[test]
    fn test_empty_config() {
        let config = AppConfig::empty();
        assert!(config.wallpaper_path.is_none());
        assert!(config.current_wallpaper.is_none());
        assert!(config.post_script.is_none());
        assert!(config.cache_path.is_none());
    }

    #[test]
    fn test_merge_config() {
        let mut config = AppConfig::empty();
        let other = AppConfig {
            wallpaper_path: Some(PathBuf::from("/tmp")),
            current_wallpaper: None,
            post_script: Some("test".to_string()),
            cache_path: None,
        };
        config = config.merge(other);
        assert!(config.wallpaper_path.is_some());
        assert_eq!(config.post_script, Some("test".to_string()));
    }

    #[test]
    fn test_merge_all_fields() {
        let mut config = AppConfig::empty();
        let other = AppConfig {
            wallpaper_path: Some(PathBuf::from("/wallpapers")),
            current_wallpaper: Some(PathBuf::from("/current")),
            post_script: Some("script".to_string()),
            cache_path: Some(PathBuf::from("/cache")),
        };
        config = config.merge(other);
        assert!(config.wallpaper_path.is_some());
        assert!(config.current_wallpaper.is_some());
        assert_eq!(config.post_script, Some("script".to_string()));
        assert!(config.cache_path.is_some());
    }

    #[test]
    fn test_merge_none() {
        let original = AppConfig::default();
        let merged = original.clone().merge(AppConfig::empty());

        assert_eq!(original.wallpaper_path, merged.wallpaper_path);
        assert_eq!(original.current_wallpaper, merged.current_wallpaper);
        assert_eq!(original.post_script, merged.post_script);
        assert_eq!(original.cache_path, merged.cache_path);
    }
}

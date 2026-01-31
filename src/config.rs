use crate::util::{get_absolute_path, resolve_dir_path, resolve_file_path};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub static CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().expect("Failed to load configuration"));

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub wallpaper_path: Option<PathBuf>,
    pub current_wallpaper: Option<PathBuf>,
    pub post_script: Option<String>,
    pub cache_path: Option<PathBuf>,
}

impl AppConfig {
    pub fn default() -> Self {
        Self {
            wallpaper_path: Some(get_absolute_path("~/Pictures/wallpapers").unwrap()),
            current_wallpaper: Some(
                get_absolute_path("~/.config/wallity/.current_wallpaper").unwrap(),
            ),
            post_script: Some(String::from("")),
            cache_path: Some(get_absolute_path("~/.cache/wallity/thumbnails").unwrap()),
        }
    }

    fn merge(mut self, other: AppConfig) -> Self {
        if other.wallpaper_path.is_some() {
            self.wallpaper_path =
                Some(get_absolute_path(other.wallpaper_path.unwrap().to_str().unwrap()).unwrap());
        }
        if other.current_wallpaper.is_some() {
            self.current_wallpaper = Some(
                get_absolute_path(other.current_wallpaper.unwrap().to_str().unwrap()).unwrap(),
            );
        }

        if other.post_script.is_some() {
            self.post_script = other.post_script;
        }

        if other.cache_path.is_some() {
            self.cache_path =
                Some(get_absolute_path(other.cache_path.unwrap().to_str().unwrap()).unwrap());
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

        let user_config: AppConfig = toml::from_str(&config_str).unwrap_or(AppConfig::empty());
        let user_config = Self::default().merge(user_config);

        let _ = resolve_file_path(
            user_config
                .current_wallpaper
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        let _ = resolve_dir_path(user_config.cache_path.as_ref().unwrap().to_str().unwrap());
        Ok(user_config)
    }
}

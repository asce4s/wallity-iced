use super::*;
use crate::message::Message;
use crate::wallpaper_image::WallpaperImage;
use iced::keyboard::key::Named;
use std::path::PathBuf;

fn create_dummy_image(name: &str) -> WallpaperImage {
    WallpaperImage {
        name: name.to_string(),
        img_path: PathBuf::from(name),
        thumbnail_path: PathBuf::from(name),
        thumbnail_handle: None,
        is_visible: false,
        is_loading: false,
    }
}

#[test]
fn test_app_view_new() {
    let view = AppView::new();
    assert!(view.images.is_empty());
    assert_eq!(view.selected_idx, 0);
    assert_eq!(view.visible_range, (0, 20));
}

#[test]
fn test_app_view_update_discovered() {
    let mut view = AppView::new();
    let image = create_dummy_image("test.jpg");

    let _ = view.update(Message::WallpaperDiscovered(image));
    assert_eq!(view.images.len(), 1);
    assert_eq!(view.images[0].name, "test.jpg");
}

#[test]
fn test_app_view_update_hover() {
    let mut view = AppView::new();
    view.images.push(create_dummy_image("1"));
    view.images.push(create_dummy_image("2"));

    let _ = view.update(Message::ImageHovered(Some(1)));
    assert_eq!(view.selected_idx, 1);

    let _ = view.update(Message::ImageHovered(None));
    assert_eq!(view.selected_idx, 1);
}

#[test]
fn test_app_view_update_key_navigation() {
    let mut view = AppView::new();
    for i in 0..10 {
        view.images.push(create_dummy_image(&i.to_string()));
    }

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowDown)));
    assert_eq!(view.selected_idx, IMAGES_PER_ROW);

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowRight)));
    assert_eq!(view.selected_idx, IMAGES_PER_ROW + 1);

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowUp)));
    assert_eq!(view.selected_idx, 1);

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowLeft)));
    assert_eq!(view.selected_idx, 0);

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowLeft)));
    assert_eq!(view.selected_idx, 0);
}

#[test]
fn test_app_view_update_key_navigation_edge_cases() {
    let mut view = AppView::new();
    for i in 0..2 {
        view.images.push(create_dummy_image(&i.to_string()));
    }

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowDown)));
    assert_eq!(view.selected_idx, 1);

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowRight)));
    assert_eq!(view.selected_idx, 1);

    let _ = view.update(Message::KeyPressed(key::Key::Named(Named::Enter)));
    assert_eq!(view.selected_idx, 1);
}

#[test]
fn test_app_view_update_thumbnail_loaded() {
    let mut view = AppView::new();
    view.images.push(create_dummy_image("1"));

    let handle = iced_image::Handle::from_rgba(1, 1, vec![0, 0, 0, 0]);
    let _ = view.update(Message::ThumbnailLoaded(0, handle));

    assert!(view.images[0].thumbnail_handle.is_some());
    assert!(view.images[0].is_visible);
    assert!(!view.images[0].is_loading);
}

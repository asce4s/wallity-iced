#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::process::Command;

use iced::{
    Alignment, Border, Color, ContentFit, Element, Length, Pixels, Subscription, Task, exit,
    keyboard::{self, key},
    widget::{
        Image, column, container, grid, image as iced_image, mouse_area,
        operation::{self, AbsoluteOffset},
        scrollable, text,
    },
};

use crate::{config::CONFIG, events::wallpaper_stream, image::WallpaperImage, message::Message};

const IMAGES_PER_ROW: usize = 4;
const THUMBNAIL_WIDTH: f32 = 320.0;
const THUMBNAIL_HEIGHT: f32 = 150.0;
const ROW_HEIGHT: f32 = 155.0;
const VIEWPORT_HEIGHT: f32 = 600.0;

pub struct AppView {
    images: Vec<WallpaperImage>,
    visible_range: (usize, usize),
    placeholder_handle: iced_image::Handle,
    selected_idx: usize,
    scroll_offset: f32,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            visible_range: (0, 20),
            placeholder_handle: iced_image::Handle::from_rgba(1, 1, vec![240, 240, 240, 255]),
            selected_idx: 0,
            scroll_offset: 0.0,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            wallpaper_stream(),
            keyboard::listen().filter_map(|event| match event {
                keyboard::Event::KeyReleased { key, .. } => Some(Message::KeyPressed(key)),
                _ => None,
            }),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.images.is_empty() {
            return container(text("Loading wallpapers..."))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        let mut g = grid![]
            .columns(IMAGES_PER_ROW)
            .spacing(Pixels(5.0))
            .height(Length::Shrink);

        let total_rows = self.images.len().div_ceil(IMAGES_PER_ROW);
        let start_row = self.visible_range.0 / IMAGES_PER_ROW;
        let end_row = self
            .visible_range
            .1
            .div_ceil(IMAGES_PER_ROW)
            .min(total_rows);

        let start_idx = start_row * IMAGES_PER_ROW;
        let end_idx = (end_row * IMAGES_PER_ROW).min(self.images.len());

        for idx in start_idx..end_idx {
            let img_data = &self.images[idx];
            let img_widget = if let Some(ref handle) = img_data.thumbnail_handle
                && img_data.is_visible
            {
                Image::new(handle.clone())
                    .width(THUMBNAIL_WIDTH)
                    .height(THUMBNAIL_HEIGHT)
                    .content_fit(ContentFit::Fill)
            } else {
                Image::new(self.placeholder_handle.clone())
                    .width(THUMBNAIL_WIDTH)
                    .height(THUMBNAIL_HEIGHT)
                    .content_fit(ContentFit::Fill)
            };

            // Create container with orange border when hovered
            let container_widget = container(img_widget)
                .width(THUMBNAIL_WIDTH)
                .height(THUMBNAIL_HEIGHT)
                .padding([5, 5]);

            // Apply orange border style when hovered/selected
            let styled_container = if self.selected_idx == idx {
                container_widget.style(|_theme| {
                    container::Style {
                        border: Border {
                            color: Color::from_rgb(1.0, 0.447, 0.0), // Orange color
                            width: 3.0,
                            radius: 0.0.into(),
                        },
                        ..container::Style::default()
                    }
                })
            } else {
                container_widget
            };

            let final_widget = mouse_area(styled_container)
                .on_enter(Message::ImageHovered(Some(idx)))
                .on_exit(Message::ImageHovered(None))
                .on_press(Message::WallpaperSelected);

            g = g.push(final_widget);
        }
        let top_spacer = container(text("")).height(Length::Fixed(ROW_HEIGHT * start_row as f32));
        let bottom_spacer =
            container(text("")).height(Length::Fixed(ROW_HEIGHT * (total_rows - end_row) as f32));

        let content = column![top_spacer, container(g).padding(10), bottom_spacer];

        let scroll = scrollable(content)
            .on_scroll(Message::ScrolledTo)
            .id("scrollable-id");

        let selected_wallpaper_name = self
            .images
            .get(self.selected_idx)
            .map(|img| img.name.as_str())
            .unwrap_or("");

        let footer = container(
            text(format!("{}", selected_wallpaper_name))
                .size(16)
                .color(Color::from_rgb(0.8, 0.8, 0.8)),
        )
        .width(Length::Fill)
        .padding(10)
        .align_x(Alignment::Center);

        column![
            container(scroll).width(Length::Fill).height(Length::Fill),
            footer
        ]
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WallpaperDiscovered(image) => {
                self.images.push(image);

                let idx = self.images.len() - 1;
                //load if in visible range
                if idx >= self.visible_range.0 && idx < self.visible_range.1 {
                    return Task::done(Message::LoadVisibleThumbnails);
                }

                Task::none()
            }
            Message::ScrolledTo(viewport) => {
                let scroll_offset = viewport.absolute_offset().y;
                self.scroll_offset = scroll_offset;
                let viewport_height = viewport.bounds().height;

                let start_row = (scroll_offset / ROW_HEIGHT).floor() as usize;
                let end_row = ((scroll_offset + viewport_height) / ROW_HEIGHT).ceil() as usize;

                let start_idx = start_row * IMAGES_PER_ROW;
                let end_idx = ((end_row + 1) * IMAGES_PER_ROW).min(self.images.len());

                let buffer = 10;
                let new_range = (
                    start_idx.saturating_sub(buffer),
                    (end_idx + buffer).min(self.images.len()),
                );

                // unload images not in visible range

                if new_range != self.visible_range {
                    self.visible_range = new_range;
                    let unload_distance = buffer + 10;
                    for (idx, img_data) in self.images.iter_mut().enumerate() {
                        if (idx < self.visible_range.0.saturating_sub(unload_distance)
                            || idx > self.visible_range.1 + unload_distance)
                            && img_data.thumbnail_handle.is_some()
                        {
                            img_data.thumbnail_handle = None;
                            img_data.is_visible = false;
                        }
                    }

                    Task::done(Message::LoadVisibleThumbnails)
                } else {
                    Task::none()
                }
            }
            Message::LoadVisibleThumbnails => {
                let mut tasks = Vec::new();

                for idx in self.visible_range.0..self.visible_range.1 {
                    if let Some(img_data) = self.images.get_mut(idx)
                        && !img_data.is_visible
                        && !img_data.is_loading
                        && img_data.thumbnail_handle.is_none()
                    {
                        img_data.is_loading = true;
                        let thumbnail_path = img_data.thumbnail_path.clone();

                        tasks.push(Task::perform(
                            async move { iced_image::Handle::from_path(thumbnail_path) },
                            move |handle| Message::ThumbnailLoaded(idx, handle),
                        ));
                    }
                }

                Task::batch(tasks)
            }
            Message::ThumbnailLoaded(idx, handle) => {
                if let Some(img_data) = self.images.get_mut(idx) {
                    img_data.thumbnail_handle = Some(handle);
                    img_data.is_visible = true;
                    img_data.is_loading = false;
                }
                Task::none()
            }
            Message::ImageHovered(idx) => {
                if let Some(index) = idx {
                    self.selected_idx = index;
                }
                Task::none()
            }
            Message::KeyPressed(key) => {
                let len = self.images.len();

                match key {
                    key::Key::Named(named) => match named {
                        key::Named::ArrowUp => {
                            if self.selected_idx >= IMAGES_PER_ROW {
                                self.selected_idx -= IMAGES_PER_ROW;
                                return Task::done(Message::ScrollToVisible);
                            }
                            Task::none()
                        }
                        key::Named::ArrowDown => {
                            let target_idx = self.selected_idx + IMAGES_PER_ROW;
                            if target_idx < len {
                                self.selected_idx = target_idx;
                            } else {
                                self.selected_idx = len - 1;
                            }
                            Task::done(Message::ScrollToVisible)
                        }
                        key::Named::ArrowLeft => {
                            if self.selected_idx > 0 {
                                self.selected_idx -= 1;
                                return Task::done(Message::ScrollToVisible);
                            }
                            Task::none()
                        }
                        key::Named::ArrowRight => {
                            let target_idx = self.selected_idx + 1;
                            if target_idx < len {
                                self.selected_idx = target_idx;
                                return Task::done(Message::ScrollToVisible);
                            }
                            Task::none()
                        }
                        key::Named::Enter => Task::done(Message::WallpaperSelected),
                        key::Named::Escape => exit(),
                        _ => Task::none(),
                    },
                    _ => Task::none(),
                }
            }
            Message::WallpaperSelected => {
                if let Some(img_data) = self.images.get(self.selected_idx) {
                    let current_wallpaper = CONFIG.current_wallpaper.clone();
                    let post_script = CONFIG.post_script.clone();

                    if let Some(current_wallpaper) = current_wallpaper {
                        let img_path = img_data.img_path.clone();

                        // Spawn completely detached thread to prevent any UI blocking
                        std::thread::spawn(move || {
                            // Remove existing symlink/file (ignore error if doesn't exist)
                            let _ = std::fs::remove_file(&current_wallpaper);

                            // Create new symlink using native Rust API
                            #[cfg(unix)]
                            if let Err(e) = symlink(&img_path, &current_wallpaper) {
                                eprintln!("Failed to create symlink: {}", e);
                                return;
                            }

                            if let Some(post_script) = post_script {
                                if post_script.is_empty() {
                                    return;
                                }
                                // Execute post script
                                // Always use sh -c to handle both files (with args) and inline scripts
                                match Command::new("sh").arg("-c").arg(&post_script).status() {
                                    Ok(status) if !status.success() => {
                                        eprintln!(
                                            "Post script exited with non-zero status: {}",
                                            status
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to execute post script: {}", e);
                                    }
                                    _ => {}
                                }
                            }
                        });
                    } else {
                        eprintln!("Current wallpaper path not configured");
                    }
                }
                Task::none()
            }
            Message::ScrollToVisible => {
                // Calculate actual visible bounds based on scroll position
                let start_row = (self.scroll_offset / ROW_HEIGHT).floor() as usize;
                let end_row = ((self.scroll_offset + VIEWPORT_HEIGHT) / ROW_HEIGHT).ceil() as usize;

                let actual_visible_start = start_row * IMAGES_PER_ROW;
                let actual_visible_end = ((end_row + 1) * IMAGES_PER_ROW).min(self.images.len());

                // Only scroll if selected_idx is outside actual visible bounds
                if self.selected_idx >= actual_visible_end
                    || self.selected_idx < actual_visible_start
                {
                    let selected_row = self.selected_idx / IMAGES_PER_ROW;
                    let new_offset = selected_row as f32 * ROW_HEIGHT;
                    return operation::scroll_to(
                        "scrollable-id",
                        AbsoluteOffset {
                            x: 0.0,
                            y: new_offset,
                        },
                    );
                }

                Task::none()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::WallpaperImage;
    use crate::message::Message;
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

        // Down
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowDown)));
        assert_eq!(view.selected_idx, IMAGES_PER_ROW);

        // Right
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowRight)));
        assert_eq!(view.selected_idx, IMAGES_PER_ROW + 1);

        // Up
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowUp)));
        assert_eq!(view.selected_idx, 1);

        // Left
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowLeft)));
        assert_eq!(view.selected_idx, 0);

        // Left at 0
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowLeft)));
        assert_eq!(view.selected_idx, 0);
    }

    #[test]
    fn test_app_view_update_key_navigation_edge_cases() {
        let mut view = AppView::new();
        for i in 0..2 {
            view.images.push(create_dummy_image(&i.to_string()));
        }

        // ArrowDown at row 0 with only 2 images (IMAGES_PER_ROW is 4)
        // target_idx = 0 + 4 = 4. 4 >= 2, so selected_idx = 2 - 1 = 1.
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowDown)));
        assert_eq!(view.selected_idx, 1);

        // ArrowRight at the end
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::ArrowRight)));
        assert_eq!(view.selected_idx, 1);

        // Enter
        let _ = view.update(Message::KeyPressed(key::Key::Named(Named::Enter)));
        // Should just return a task, no state change here
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

}

#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::process::Command;

use iced::{
    Border, Color, ContentFit, Element, Length, Pixels, Subscription, Task, exit,
    keyboard::{self, key},
    widget::{Image, column, container, grid, image as iced_image, mouse_area, scrollable, text},
    window,
};

use crate::{config::CONFIG, events::wallpaper_stream, image::WallpaperImage, message::Message};

pub struct AppView {
    images: Vec<WallpaperImage>,
    visible_range: (usize, usize),
    images_per_row: usize,
    row_height: f32,
    placeholder_handle: iced_image::Handle,
    selected_idx: usize,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            visible_range: (0, 20),
            images_per_row: 4,
            row_height: 155.0,
            placeholder_handle: iced_image::Handle::from_rgba(1, 1, vec![240, 240, 240, 255]),
            selected_idx: 0,
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
            .columns(self.images_per_row)
            .spacing(Pixels(5.0))
            .height(Length::Shrink);

        let total_rows = self.images.len().div_ceil(self.images_per_row);
        let start_row = self.visible_range.0 / self.images_per_row;
        let end_row = self
            .visible_range
            .1
            .div_ceil(self.images_per_row)
            .min(total_rows);

        let start_idx = start_row * self.images_per_row;
        let end_idx = (end_row * self.images_per_row).min(self.images.len());

        for idx in start_idx..end_idx {
            let img_data = &self.images[idx];
            let img_widget = if let Some(ref handle) = img_data.thumbnail_handle
                && img_data.is_visible
            {
                Image::new(handle.clone())
                    .width(320)
                    .height(150)
                    .content_fit(ContentFit::Fill)
            } else {
                Image::new(self.placeholder_handle.clone())
                    .width(320)
                    .height(150)
                    .content_fit(ContentFit::Fill)
            };

            // Create container with orange border when hovered
            let container_widget = container(img_widget).width(320).height(150).padding([5, 5]);

            // Apply orange border style when hovered
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
        let top_spacer =
            container(text("")).height(Length::Fixed(self.row_height * start_row as f32));
        let bottom_spacer = container(text("")).height(Length::Fixed(
            self.row_height * (total_rows - end_row) as f32,
        ));

        let content = column![top_spacer, container(g).padding(10), bottom_spacer];

        let scroll = scrollable(content).on_scroll(Message::ScrolledTo);

        container(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
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
                let viewport_height = viewport.bounds().height;

                let start_row = (scroll_offset / self.row_height).floor() as usize;
                let end_row = ((scroll_offset + viewport_height) / self.row_height).ceil() as usize;

                let start_idx = start_row * self.images_per_row;
                let end_idx = ((end_row + 1) * self.images_per_row).min(self.images.len());

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
                let per_row = self.images_per_row;

                match key {
                    key::Key::Named(named) => match named {
                        key::Named::ArrowUp => {
                            let row = self.selected_idx / per_row;
                            if row > 0 {
                                self.selected_idx -= per_row;
                            }
                            Task::none()
                        }
                        key::Named::ArrowDown => {
                            let target_idx = self.selected_idx + per_row;
                            if target_idx < len {
                                self.selected_idx = target_idx;
                            } else {
                                self.selected_idx = len - 1;
                            }
                            Task::none()
                        }
                        key::Named::ArrowLeft => {
                            let target_idx = self.selected_idx.saturating_sub(1);
                            self.selected_idx = target_idx;
                            Task::none()
                        }
                        key::Named::ArrowRight => {
                            let target_idx = self.selected_idx + 1;

                            if target_idx < len {
                                self.selected_idx = target_idx;
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
                    let current_wallpaper = CONFIG
                        .current_wallpaper
                        .clone()
                        .expect("Current wallpaper path not configured");

                    let post_script = CONFIG
                        .post_script
                        .clone()
                        .expect("Post script not configured");

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

                        // Execute post script
                        // Always use sh -c to handle both files (with args) and inline scripts
                        match Command::new("sh").arg("-c").arg(&post_script).status() {
                            Ok(status) if !status.success() => {
                                eprintln!("Post script exited with non-zero status: {}", status);
                            }
                            Err(e) => {
                                eprintln!("Failed to execute post script: {}", e);
                            }
                            _ => {}
                        }
                    });
                }
                Task::none()
            }
        }
    }
}

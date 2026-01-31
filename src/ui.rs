use iced::{
    ContentFit, Element, Length, Pixels, Subscription, Task,
    widget::{Image, container, grid, image as iced_image, scrollable, text},
};

use crate::{events::wallpaper_stream, image::WallpaperImage, message::Message};

pub struct AppView {
    images: Vec<WallpaperImage>,
    visible_range: (usize, usize),
    images_per_row: usize,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            visible_range: (0, 20),
            images_per_row: 4,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        wallpaper_stream()
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

        for img_data in self.images.iter() {
            let img_widget = if let Some(ref handle) = img_data.thumbnail_handle
                && img_data.is_visible
            {
                Image::new(handle.clone())
                    .width(320)
                    .height(150)
                    .content_fit(ContentFit::Fill)
            } else {
                Image::new(iced_image::Handle::from_rgba(
                    1,
                    1,
                    vec![240, 240, 240, 255],
                ))
                .width(320)
                .height(150)
                .content_fit(ContentFit::Fill)
            };
            g = g.push(container(img_widget).width(320).height(150).padding([5, 5]));
        }

        let scroll = scrollable(g)
            .height(Length::Fill)
            .on_scroll(Message::ScrolledTo);

        container(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([10, 10])
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WallpaperDiscovered(image) => {
                let mut tasks = Vec::new();
                self.images.push(image);

                let idx = self.images.len() - 1;
                //load if in visible range
                if idx >= self.visible_range.0 && idx < self.visible_range.1 {
                    tasks.push(Task::done(Message::LoadVisibleThumbnails));
                }

                Task::batch(tasks)
            }
            Message::ScrolledTo(viewport) => {
                let scroll_offset = viewport.absolute_offset().y;
                let viewport_height = viewport.bounds().height;

                let row_height = 155.0;
                let start_row = (scroll_offset / row_height).floor() as usize;
                let end_row = ((scroll_offset + viewport_height) / row_height).ceil() as usize;

                let start_idx = start_row * self.images_per_row;
                let end_idx = ((end_row + 1) * self.images_per_row).min(self.images.len());

                let buffer = 10;
                let new_range = (
                    start_idx.saturating_sub(buffer),
                    (end_idx + buffer).min(self.images.len()),
                );

                // unload images not in visible range

                if new_range != self.visible_range {
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
                    self.visible_range = new_range;
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
                        && img_data.thumbnail_handle.is_none()
                    {
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
                }
                Task::none()
            }
            Message::ThumbnailGenerated(idx) => {
                if let Some(img_data) = self.images.get_mut(idx) {
                    img_data.has_thumbnail = true;

                    if idx >= self.visible_range.0 && idx < self.visible_range.1 {
                        return Task::done(Message::LoadVisibleThumbnails);
                    }
                }
                Task::none()
            }
        }
    }
}

use iced::Task;

use crate::{constants::IMAGES_PER_ROW, message::Message};

use super::AppView;

pub(super) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl AppView {
    pub(super) fn move_selection(&mut self, direction: Direction) -> Task<Message> {
        let len = self.images.len();

        if self.images.is_empty() {
            return Task::none();
        }

        match direction {
            Direction::Up => {
                if self.selected_idx >= IMAGES_PER_ROW {
                    self.selected_idx -= IMAGES_PER_ROW;
                    return Task::done(Message::ScrollToVisible);
                }
                Task::none()
            }
            Direction::Down => {
                let target_idx = self.selected_idx + IMAGES_PER_ROW;
                if target_idx < len {
                    self.selected_idx = target_idx;
                } else {
                    self.selected_idx = len - 1;
                    return Task::done(Message::ScrollToVisible);
                }
                Task::none()
            }
            Direction::Left => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                    return Task::done(Message::ScrollToVisible);
                }
                Task::none()
            }
            Direction::Right => {
                let target_idx = self.selected_idx + 1;
                if target_idx < len {
                    self.selected_idx = target_idx;
                    return Task::done(Message::ScrollToVisible);
                }
                Task::none()
            }
        }
    }
}

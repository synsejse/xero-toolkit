//! UI widgets for command execution dialog.

use super::types::TaskStatus;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Label, ScrolledWindow, Window};

/// Container for all command execution dialog widgets
pub struct CommandExecutionWidgets {
    pub window: Window,
    pub title_label: Label,
    #[allow(dead_code)]
    pub task_list_container: GtkBox,
    pub scrolled_window: ScrolledWindow,
    pub cancel_button: Button,
    pub close_button: Button,
    pub task_items: Vec<TaskItem>,
}

/// A single task item in the task list
pub struct TaskItem {
    pub container: GtkBox,
    pub status_icon: Image,
    pub spinner_icon: Image,
}

impl TaskItem {
    /// Create a new task item
    pub fn new(description: &str) -> Self {
        let container = GtkBox::new(gtk4::Orientation::Horizontal, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        // Task description label
        let label = Label::new(Some(description));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_wrap(true);

        // Spinner icon for running state (using circle-noth-symbolic)
        let spinner_icon = Image::new();
        spinner_icon.set_icon_name(Some("circle-noth-symbolic"));
        spinner_icon.set_pixel_size(24);
        spinner_icon.set_visible(false);
        spinner_icon.add_css_class("spinning");

        // Status icon image
        let status_icon = Image::new();
        status_icon.set_pixel_size(24);
        status_icon.set_visible(false);

        container.append(&label);
        container.append(&spinner_icon);
        container.append(&status_icon);

        Self {
            container,
            status_icon,
            spinner_icon,
        }
    }

    /// Update the status of this task item
    pub fn set_status(&self, status: TaskStatus) {
        match status {
            TaskStatus::Pending => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Running => {
                self.spinner_icon.set_visible(true);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Success => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-check"));
                self.status_icon.set_visible(true);
            }
            TaskStatus::Failed => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-xmark"));
                self.status_icon.set_visible(true);
            }
        }
    }
}

impl CommandExecutionWidgets {
    /// Scroll to a specific task in the list, but only if it's outside the visible area
    fn scroll_to_task(&self, index: usize) {
        if self.task_items.get(index).is_none() {
            return;
        }

        // Get the vertical adjustment from the scrolled window
        let vadjustment = self.scrolled_window.vadjustment();

        let current_scroll = vadjustment.value();
        let page_size = vadjustment.page_size();
        let upper = vadjustment.upper();

        // Calculate the approximate position and height of each task
        let total_tasks = self.task_items.len() as f64;
        let content_height = upper;
        let task_height = content_height / total_tasks;

        // Calculate where this task is positioned
        let task_top = (index as f64) * task_height;
        let task_bottom = task_top + task_height;

        // Check if task is already visible in the current viewport
        let viewport_top = current_scroll;
        let viewport_bottom = current_scroll + page_size;

        // Only scroll if the task is outside the visible area
        if task_bottom > viewport_bottom {
            // Task is below the visible area - scroll down to show it
            let target_value = (task_bottom - page_size).max(0.0).min(upper - page_size);
            vadjustment.set_value(target_value);
        } else if task_top < viewport_top {
            // Task is above the visible area - scroll up to show it
            let target_value = task_top.max(0.0);
            vadjustment.set_value(target_value);
        }
        // If task is already visible, don't scroll
    }

    /// Update the status of a specific task
    pub fn update_task_status(&self, index: usize, status: TaskStatus) {
        if let Some(task_item) = self.task_items.get(index) {
            task_item.set_status(status);
            // Scroll to the task when its status changes
            self.scroll_to_task(index);
        }
    }

    /// Set the dialog title
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Disable the cancel button
    pub fn disable_cancel(&self) {
        self.cancel_button.set_sensitive(false);
    }

    /// Enable the close button and hide cancel button
    pub fn enable_close(&self) {
        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
        self.close_button.set_sensitive(true);
    }

    /// Show completion state (not needed for task-based UI, kept for compatibility)
    pub fn show_completion(&self, _success: bool, _message: &str) {
        // In task-based UI, completion is shown via task statuses
        // Enable the close button
        self.enable_close();
    }
}

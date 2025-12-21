//! UI widgets for task runner dialog.
//!
//! This module provides the UI components for displaying command execution progress,
//! including task items, status icons, and scroll management.

use super::command::TaskStatus;
use adw::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Label, ScrolledWindow, TextBuffer, TextView, ToggleButton, Window};

/// Container for all task runner dialog widgets.
pub struct TaskRunnerWidgets {
    pub window: Window,
    pub title_label: Label,
    #[allow(dead_code)]
    pub task_list_container: GtkBox,
    pub scrolled_window: ScrolledWindow,
    pub cancel_button: Button,
    pub close_button: Button,
    pub task_items: Vec<TaskItem>,
    pub sidebar_toggle: ToggleButton,
    pub split_view: adw::OverlaySplitView,
    pub output_text_view: TextView,
    pub output_text_buffer: TextBuffer,
}

impl TaskRunnerWidgets {
    /// Create a new TaskRunnerWidgets instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window: Window,
        title_label: Label,
        task_list_container: GtkBox,
        scrolled_window: ScrolledWindow,
        cancel_button: Button,
        close_button: Button,
        task_items: Vec<TaskItem>,
        sidebar_toggle: ToggleButton,
        split_view: adw::OverlaySplitView,
        output_text_view: TextView,
        output_text_buffer: TextBuffer,
    ) -> Self {
        Self {
            window,
            title_label,
            task_list_container,
            scrolled_window,
            cancel_button,
            close_button,
            task_items,
            sidebar_toggle,
            split_view,
            output_text_view,
            output_text_buffer,
        }
    }

    /// Bind the sidebar toggle button to the split view.
    pub fn setup_sidebar_toggle(&self) {
        // Bind toggle button's active state to split view's show-sidebar
        self.sidebar_toggle
            .bind_property("active", &self.split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        // Update tooltip based on state
        let toggle = self.sidebar_toggle.clone();
        self.split_view.connect_show_sidebar_notify(move |split_view| {
            let tooltip = if split_view.shows_sidebar() {
                "Hide command output"
            } else {
                "Show command output"
            };
            toggle.set_tooltip_text(Some(tooltip));
        });
    }
}

/// A single task item in the task list.
pub struct TaskItem {
    pub container: GtkBox,
    pub status_icon: Image,
    pub spinner_icon: Image,
}

impl TaskItem {
    /// Create a new task item.
    pub fn new(description: &str) -> Self {
        let container = GtkBox::new(gtk4::Orientation::Horizontal, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        let label = Label::new(Some(description));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_wrap(true);

        // Spinner icon for running state
        let spinner_icon = Image::new();
        spinner_icon.set_icon_name(Some("circle-noth-symbolic"));
        spinner_icon.set_pixel_size(24);
        spinner_icon.set_visible(false);
        spinner_icon.add_css_class("spinning");

        // Status icon for success/failure
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

    /// Update the status of this task item.
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
            TaskStatus::Cancelled => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-stop"));
                self.status_icon.set_visible(true);
            }
        }
    }
}

impl TaskRunnerWidgets {
    /// Scroll to a specific task in the list (only if outside visible area).
    fn scroll_to_task(&self, index: usize) {
        if self.task_items.get(index).is_none() {
            return;
        }

        let vadjustment = self.scrolled_window.vadjustment();
        let viewport_top = vadjustment.value();
        let viewport_bottom = viewport_top + vadjustment.page_size();

        let total_tasks = self.task_items.len() as f64;
        if total_tasks == 0.0 {
            return;
        }

        let content_height = vadjustment.upper();
        let estimated_task_height = content_height / total_tasks;
        let task_top = (index as f64) * estimated_task_height;
        let task_bottom = task_top + estimated_task_height;

        if task_bottom > viewport_bottom {
            let target = (task_bottom - vadjustment.page_size())
                .max(0.0)
                .min(vadjustment.upper() - vadjustment.page_size());
            vadjustment.set_value(target);
        } else if task_top < viewport_top {
            vadjustment.set_value(task_top.max(0.0));
        }
    }

    /// Update the status of a specific task.
    pub fn update_task_status(&self, index: usize, status: TaskStatus) {
        if let Some(task_item) = self.task_items.get(index) {
            task_item.set_status(status);
            self.scroll_to_task(index);
        }
    }

    /// Set the dialog title.
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Disable the cancel button.
    pub fn disable_cancel(&self) {
        self.cancel_button.set_sensitive(false);
    }

    /// Enable the close button and hide cancel button.
    pub fn enable_close(&self) {
        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
        self.close_button.set_sensitive(true);
    }

    /// Show completion state with a final message.
    pub fn show_completion(&self, success: bool, message: &str) {
        self.set_title(message);

        if success {
            self.close_button.add_css_class("suggested-action");
            self.title_label.remove_css_class("error");
            self.title_label.add_css_class("success");
        } else {
            self.close_button.remove_css_class("suggested-action");
            self.title_label.remove_css_class("success");
            self.title_label.add_css_class("error");
        }

        self.enable_close();
    }

    /// Update the output text view with command output.
    pub fn set_output(&self, output: &str) {
        self.output_text_buffer.set_text(output);
        let mut end = self.output_text_buffer.end_iter();
        let _ = self.output_text_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
    }

    /// Append text to the output text view.
    pub fn append_output(&self, text: &str) {
        let mut end = self.output_text_buffer.end_iter();
        self.output_text_buffer.insert(&mut end, text);
        let mut new_end = self.output_text_buffer.end_iter();
        let _ = self.output_text_view.scroll_to_iter(&mut new_end, 0.0, false, 0.0, 0.0);
    }

    /// Toggle the sidebar visibility.
    pub fn toggle_sidebar(&self) {
        self.sidebar_toggle.set_active(!self.sidebar_toggle.is_active());
    }

    /// Initialize sidebar to collapsed state.
    pub fn init_sidebar_collapsed(&self) {
        self.sidebar_toggle.set_active(false);
        self.split_view.set_show_sidebar(false);
    }
}

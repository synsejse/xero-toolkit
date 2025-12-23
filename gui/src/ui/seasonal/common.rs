//! Common utilities for seasonal effects.

use adw::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea, EventControllerMotion, Widget};
use std::cell::RefCell;
use std::rc::Rc;

/// Mouse position context for seasonal effects.
/// Provides mouse coordinates that effects can use.
pub struct MouseContext {
    position: Rc<RefCell<(f64, f64)>>,
}

impl MouseContext {
    /// Get a clone of the internal Rc<RefCell<(f64, f64)>> for sharing.
    pub fn position_internal(&self) -> Rc<RefCell<(f64, f64)>> {
        self.position.clone()
    }
}

/// Set up mouse tracking for the window and return a MouseContext.
pub fn setup_mouse_tracking(window: &ApplicationWindow) -> MouseContext {
    let mouse_pos = Rc::new(RefCell::new((0.0f64, 0.0f64)));

    let motion = EventControllerMotion::new();
    let mouse_pos_clone = mouse_pos.clone();
    motion.connect_motion(move |_, x, y| {
        *mouse_pos_clone.borrow_mut() = (x, y);
    });
    window.add_controller(motion);

    MouseContext {
        position: mouse_pos,
    }
}

/// Trait for effect states that need to handle window resizing.
pub trait ResizableEffectState {
    /// Handle window resize by adjusting particle positions to fit new dimensions.
    fn handle_resize(&mut self, new_width: f64, new_height: f64);
}

/// Set up a resize handler for a drawing area that calls the state's handle_resize method.
pub fn setup_resize_handler<F>(drawing_area: &DrawingArea, state: Rc<RefCell<Option<F>>>)
where
    F: ResizableEffectState + 'static,
{
    let state_clone = state.clone();
    drawing_area.connect_resize(move |da, width, height| {
        // Ignore invalid sizes to prevent division by zero or weird math later
        if width <= 0 || height <= 0 {
            return;
        }

        let mut state_ref = state_clone.borrow_mut();
        if let Some(s) = state_ref.as_mut() {
            s.handle_resize(width as f64, height as f64);
        }
        da.queue_draw();
    });
}

/// Helper function to add a drawing area as an overlay to the window.
pub fn add_overlay_to_window(window: &ApplicationWindow, drawing_area: &DrawingArea) -> bool {
    use log::info;

    let adw_window = match window.downcast_ref::<adw::ApplicationWindow>() {
        Some(w) => w,
        None => {
            info!("Window is not an AdwApplicationWindow, cannot add overlay");
            return false;
        }
    };

    let content_widget = match adw_window.content() {
        Some(w) => w,
        None => {
            info!("Window has no content");
            return false;
        }
    };

    // Verify it's a ToolbarView
    if content_widget.downcast_ref::<adw::ToolbarView>().is_none() {
        info!("Window content is not a ToolbarView, overlay may not work correctly");
    }

    // Check if the content is already wrapped in an overlay
    if let Some(existing_overlay) = content_widget.downcast_ref::<gtk4::Overlay>() {
        info!("Found existing overlay at window level, adding drawing area");
        existing_overlay.add_overlay(drawing_area);
        true
    } else {
        info!("Wrapping window content in overlay to cover entire window including navbar");
        // Create overlay that will wrap the entire content
        let overlay = gtk4::Overlay::new();

        // Remove the content from the window
        adw_window.set_content(Option::<&Widget>::None);

        // Add content as the main child of the overlay
        overlay.set_child(Some(&content_widget));

        // Add the drawing area as an overlay
        overlay.add_overlay(drawing_area);

        // Set the overlay as the window content
        adw_window.set_content(Some(&overlay));
        true
    }
}

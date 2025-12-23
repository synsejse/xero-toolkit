//! Seasonal overlay effects for the application window.
//!
//! This module provides animated overlay effects that appear during specific
//! times of the year (e.g., snow for December, Halloween effects for October).

mod common;
mod halloween;
mod snow;

use crate::ui::seasonal::common::MouseContext;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use halloween::HalloweenEffect;
pub use snow::SnowEffect;

/// Global state for whether seasonal effects are enabled.
static EFFECTS_ENABLED: AtomicBool = AtomicBool::new(true);

/// Global registry of active drawing areas for seasonal effects.
/// SAFETY: GTK operations must be on the main thread, so this RefCell is safe to use.
/// We use unsafe to implement Send+Sync, which is safe because GTK is single-threaded.
struct DrawingAreaVec(RefCell<Vec<Rc<DrawingArea>>>);

// SAFETY: Safe because GTK operations are single-threaded (main thread only).
unsafe impl Send for DrawingAreaVec {}
unsafe impl Sync for DrawingAreaVec {}

static DRAWING_AREAS: std::sync::OnceLock<DrawingAreaVec> = std::sync::OnceLock::new();

fn get_drawing_areas() -> &'static RefCell<Vec<Rc<DrawingArea>>> {
    &DRAWING_AREAS.get_or_init(|| DrawingAreaVec(RefCell::new(Vec::new()))).0
}

/// Check if seasonal effects are currently enabled.
pub fn are_effects_enabled() -> bool {
    EFFECTS_ENABLED.load(Ordering::Relaxed)
}

/// Set whether seasonal effects are enabled and update visibility of drawing areas.
pub fn set_effects_enabled(enabled: bool) {
    EFFECTS_ENABLED.store(enabled, Ordering::Relaxed);
    
    let drawing_areas = get_drawing_areas();
    for area in drawing_areas.borrow().iter() {
        area.set_visible(enabled);
    }
}

/// Check if any seasonal effect is currently active.
pub fn has_active_effect() -> bool {
    let effects: Vec<Box<dyn SeasonalEffect>> = vec![
        Box::new(SnowEffect),
        Box::new(HalloweenEffect),
    ];

    effects.iter().any(|e| e.is_active())
}

/// Register a drawing area so its visibility can be controlled by the toggle.
pub fn register_drawing_area(area: Rc<DrawingArea>) {
    let drawing_areas = get_drawing_areas();
    drawing_areas.borrow_mut().push(area);
}

/// Trait for seasonal effects that can be applied to application windows.
pub trait SeasonalEffect {
    /// Check if this effect should be active at the current time.
    fn is_active(&self) -> bool;

    /// Get the name of this seasonal effect (for logging).
    fn name(&self) -> &'static str;

    /// Apply this effect to the given window.
    /// The mouse_context provides mouse position if the effect needs it.
    /// Returns the drawing area if the effect was successfully applied.
    fn apply(&self, window: &ApplicationWindow, mouse_context: Option<&MouseContext>) -> Option<Rc<DrawingArea>>;
}

/// Apply any active seasonal effects to the window.
pub fn apply_seasonal_effects(window: &ApplicationWindow) {
    if !are_effects_enabled() {
        info!("Seasonal effects are disabled");
        return;
    }

    info!("Checking for active seasonal effects...");

    let mouse_context = common::setup_mouse_tracking(window);

    let effects: Vec<Box<dyn SeasonalEffect>> = vec![
        Box::new(SnowEffect),
        Box::new(HalloweenEffect),
    ];

    for effect in effects {
        if effect.is_active() {
            info!("Active seasonal effect detected: {}", effect.name());
            if let Some(drawing_area) = effect.apply(window, Some(&mouse_context)) {
                register_drawing_area(drawing_area);
                info!("Successfully applied {} effect", effect.name());
            } else {
                info!("Failed to apply {} effect", effect.name());
            }
        }
    }
}


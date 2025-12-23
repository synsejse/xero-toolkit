//! Shared dialog helpers used across the UI pages.

use adw::prelude::*;
use adw::AlertDialog;
use gtk4::ApplicationWindow;

/// Show an error message dialog transient for the provided window.
pub fn show_error(window: &ApplicationWindow, message: &str) {
    let dialog = AlertDialog::builder()
        .heading("Error")
        .body(message)
        .build();

    dialog.present(Some(window));
}

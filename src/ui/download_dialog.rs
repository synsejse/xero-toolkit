//! Download dialog for showing download progress

use super::download::{
    download_file, fetch_arch_iso_info, format_bytes, format_speed, format_time_remaining,
    DownloadState,
};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Button, Entry, Label, ProgressBar, Window};
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Show the download setup dialog for Arch ISO
pub fn show_download_dialog(parent: &Window) {
    info!("Opening Arch ISO download setup dialog");

    // Load the setup UI
    let builder = gtk4::Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/download_setup_dialog.ui");

    let window: adw::Window = builder.object("download_setup_window").expect("Failed to get download_setup_window");
    let month_entry: Entry = builder.object("month_entry").expect("Failed to get month_entry");
    let day_entry: Entry = builder.object("day_entry").expect("Failed to get day_entry");
    let year_entry: Entry = builder.object("year_entry").expect("Failed to get year_entry");
    let download_path_entry: Entry = builder.object("download_path_entry").expect("Failed to get download_path_entry");
    let browse_button: Button = builder.object("browse_button").expect("Failed to get browse_button");
    let cancel_button: Button = builder.object("cancel_button").expect("Failed to get cancel_button");
    let start_download_button: Button = builder.object("start_download_button").expect("Failed to get start_download_button");

    window.set_transient_for(Some(parent));

    // State to hold ISO info
    let iso_info: Arc<std::sync::Mutex<Option<(String, String)>>> = Arc::new(std::sync::Mutex::new(None));
    let selected_path: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));

    // Setup cancel button
    let window_clone = window.clone();
    cancel_button.connect_clicked(move |_| {
        window_clone.close();
    });

    // Create a channel for ISO info fetching
    let (tx, rx) = std::sync::mpsc::channel::<Result<(String, String), String>>();

    // Clone for the receiver
    let month_entry_clone = month_entry.clone();
    let day_entry_clone = day_entry.clone();
    let year_entry_clone = year_entry.clone();
    let browse_button_clone = browse_button.clone();
    let start_download_button_clone = start_download_button.clone();
    let download_path_entry_clone = download_path_entry.clone();
    let iso_info_clone = iso_info.clone();
    let selected_path_clone = selected_path.clone();
    let parent_clone = parent.clone();

    // Poll for ISO info result
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(result) => {
                match result {
                    Ok((iso_name, download_url)) => {
                        info!("Fetched ISO info: {}", iso_name);

                        // Parse date from filename (archlinux-YYYY.MM.DD-x86_64.iso)
                        if let Some(date_part) = iso_name.strip_prefix("archlinux-").and_then(|s| s.split('-').next()) {
                            if let Some((year, month, day)) = parse_date_parts(date_part) {
                                let month_name = get_month_name(month);
                                month_entry_clone.set_text(month_name);
                                day_entry_clone.set_text(&day.to_string());
                                year_entry_clone.set_text(&year.to_string());
                            } else {
                                month_entry_clone.set_text("Unknown");
                                day_entry_clone.set_text("--");
                                year_entry_clone.set_text("----");
                            }
                        }

                        // Store ISO info
                        *iso_info_clone.lock().unwrap() = Some((iso_name.clone(), download_url));

                        // Enable browse button
                        browse_button_clone.set_sensitive(true);

                        // Set default download path
                        let default_path = format!("{}/Downloads/{}", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()), iso_name);
                        download_path_entry_clone.set_text(&default_path);
                        *selected_path_clone.lock().unwrap() = Some(default_path);

                        // Enable start button
                        start_download_button_clone.set_sensitive(true);
                    }
                    Err(e) => {
                        error!("Failed to fetch ISO info: {}", e);
                        month_entry_clone.set_text("Error");
                        day_entry_clone.set_text("--");
                        year_entry_clone.set_text("----");
                        show_error_dialog(&parent_clone, "Failed to fetch ISO information", &e);
                    }
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                glib::ControlFlow::Continue
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                error!("Channel disconnected unexpectedly");
                month_entry_clone.set_text("Error");
                day_entry_clone.set_text("--");
                year_entry_clone.set_text("----");
                glib::ControlFlow::Break
            }
        }
    });

    // Spawn thread to fetch ISO info
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            fetch_arch_iso_info().await
        });
        let result: Result<(String, String), String> = result.map_err(|e| e.to_string());
        let _ = tx.send(result);
    });

    // Setup browse button
    let download_path_entry_clone = download_path_entry.clone();
    let start_download_button_clone = start_download_button.clone();
    let selected_path_clone = selected_path.clone();
    let window_clone = window.clone();
    let iso_info_clone = iso_info.clone();

    browse_button.connect_clicked(move |_| {
        let iso_info_guard = iso_info_clone.lock().unwrap();
        if let Some((iso_name, _)) = iso_info_guard.as_ref() {
            let dialog = gtk4::FileDialog::new();
            dialog.set_initial_name(Some(iso_name));

            let download_path_entry = download_path_entry_clone.clone();
            let start_download_button = start_download_button_clone.clone();
            let selected_path = selected_path_clone.clone();
            let window = window_clone.clone();

            glib::spawn_future_local(async move {
                match dialog.save_future(Some(&window)).await {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            download_path_entry.set_text(&path_str);
                            *selected_path.lock().unwrap() = Some(path_str);
                            start_download_button.set_sensitive(true);
                        }
                    }
                    Err(_) => {
                        // User cancelled
                    }
                }
            });
        }
    });

    // Setup start download button
    let window_clone = window.clone();
    let parent_clone = parent.clone();

    start_download_button.connect_clicked(move |_| {
        let iso_info_guard = iso_info.lock().unwrap();
        let selected_path_guard = selected_path.lock().unwrap();

        if let (Some((iso_name, download_url)), Some(save_path)) = (iso_info_guard.as_ref(), selected_path_guard.as_ref()) {
            info!("Starting download: {} -> {}", iso_name, save_path);
            window_clone.close();
            start_download(&parent_clone, iso_name.clone(), download_url.clone(), save_path.clone());
        }
    });

    window.present();
}

/// Start the actual download with progress dialog
fn start_download(parent: &Window, iso_name: String, download_url: String, save_path: String) {
    // Load the UI
    let builder = gtk4::Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/download_dialog.ui");

    let window: adw::Window = builder.object("download_window").expect("Failed to get download_window");
    let filename_label: Label = builder.object("filename_label").expect("Failed to get filename_label");
    let progress_bar: ProgressBar = builder.object("progress_bar").expect("Failed to get progress_bar");
    let speed_label: Label = builder.object("speed_label").expect("Failed to get speed_label");
    let downloaded_label: Label = builder.object("downloaded_label").expect("Failed to get downloaded_label");
    let time_remaining_label: Label = builder.object("time_remaining_label").expect("Failed to get time_remaining_label");
    let pause_button: Button = builder.object("pause_button").expect("Failed to get pause_button");
    let cancel_button: Button = builder.object("cancel_button").expect("Failed to get cancel_button");

    window.set_transient_for(Some(parent));

    // Set filename
    filename_label.set_text(&iso_name);

    // Create control flags
    let pause_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Setup pause button
    let pause_flag_clone = pause_flag.clone();
    let pause_button_clone = pause_button.clone();
    pause_button.connect_clicked(move |_| {
        let is_paused = pause_flag_clone.load(Ordering::Relaxed);
        pause_flag_clone.store(!is_paused, Ordering::Relaxed);

        if is_paused {
            pause_button_clone.set_label("Pause");
        } else {
            pause_button_clone.set_label("Resume");
        }
    });

    // Setup cancel button
    let cancel_flag_clone = cancel_flag.clone();
    let window_clone = window.clone();
    cancel_button.connect_clicked(move |_| {
        cancel_flag_clone.store(true, Ordering::Relaxed);
        window_clone.close();
    });

    // Use a channel to send progress updates from download thread to UI thread
    let (tx, rx) = std::sync::mpsc::channel::<DownloadMessage>();

    // Clone for the result callback
    let window_clone = window.clone();
    let pause_button_clone = pause_button.clone();
    let cancel_button_clone = cancel_button.clone();
    let parent_clone = parent.clone();

    // Set up a timer to check for messages
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        // Try to receive all pending messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                DownloadMessage::Progress(state) => {
                    // Update progress bar
                    let fraction = if state.total > 0 {
                        state.downloaded as f64 / state.total as f64
                    } else {
                        0.0
                    };
                    progress_bar.set_fraction(fraction);
                    progress_bar.set_text(Some(&format!("{:.1}%", fraction * 100.0)));

                    // Update speed
                    speed_label.set_text(&format_speed(state.speed));

                    // Update downloaded
                    downloaded_label.set_text(&format!(
                        "{} / {}",
                        format_bytes(state.downloaded),
                        format_bytes(state.total)
                    ));

                    // Update time remaining
                    let time_remaining = if state.speed > 0.0 {
                        let remaining_bytes = state.total.saturating_sub(state.downloaded);
                        (remaining_bytes as f64 / state.speed) as u64
                    } else {
                        0
                    };
                    time_remaining_label.set_text(&format_time_remaining(time_remaining));
                }
                DownloadMessage::Completed => {
                    info!("Download completed successfully");
                    pause_button_clone.set_sensitive(false);
                    cancel_button_clone.set_label("Close");
                    return glib::ControlFlow::Break;
                }
                DownloadMessage::Error(e) => {
                    error!("Download failed: {}", e);
                    if !e.contains("cancelled") {
                        show_error_dialog(&parent_clone, "Download Failed", &e);
                    }
                    window_clone.close();
                    return glib::ControlFlow::Break;
                }
            }
        }
        glib::ControlFlow::Continue
    });

    // Start download in background thread
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let tx_progress = tx.clone();

            let result = download_file(
                download_url,
                save_path.clone(),
                move |state: DownloadState| {
                    let _ = tx_progress.send(DownloadMessage::Progress(state));
                },
                pause_flag.clone(),
                cancel_flag.clone(),
            )
            .await;

            // Send completion message
            match result {
                Ok(_) => {
                    let _ = tx.send(DownloadMessage::Completed);
                }
                Err(e) => {
                    let _ = tx.send(DownloadMessage::Error(e.to_string()));
                }
            }
        });
    });

    window.present();
}

/// Messages sent from download thread to UI thread
enum DownloadMessage {
    Progress(DownloadState),
    Completed,
    Error(String),
}

/// Show an error dialog
fn show_error_dialog(parent: &Window, title: &str, message: &str) {
    use adw::prelude::*;

    let dialog = adw::AlertDialog::new(Some(title), Some(message));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(parent));
}

/// Parse date parts from ISO filename date string (YYYY.MM.DD)
fn parse_date_parts(date_str: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() == 3 {
        let year = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let day = parts[2].parse::<u32>().ok()?;
        Some((year, month, day))
    } else {
        None
    }
}

/// Get month name from month number (1-12)
fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}


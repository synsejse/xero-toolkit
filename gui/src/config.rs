//! Centralized configuration and constants for the application.

/// Application information constants.
pub mod app_info {
    pub const NAME: &str = "xero-toolkit";
    pub const ID: &str = "xyz.xerolinux.xero-toolkit";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Sidebar configuration.
pub mod sidebar {
    pub const MIN_WIDTH: i32 = 200;
    pub const MAX_WIDTH: i32 = 400;
}

/// External links.
pub mod links {
    pub const DISCORD: &str = "https://discord.xerolinux.xyz/";
    pub const YOUTUBE: &str = "https://www.youtube.com/@XeroLinux";
    pub const WEBSITE: &str = "https://xerolinux.xyz/";
    pub const DONATE: &str = "https://ko-fi.com/xerolinux";
}

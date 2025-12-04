//! Application setup and initialization.

use crate::config;
use crate::core;
use crate::ui::context::AppContext;
use crate::ui::context::UiComponents;
use crate::ui::{navigation, pages};
use adw::prelude::*;
use adw::Application;
use gtk4::glib;
use gtk4::{gio, ApplicationWindow, Box as GtkBox, Builder, CssProvider};
use log::{info, warn};

/// Initialize and set up main application UI.
pub fn setup_application_ui(app: &Application) {
    info!("Initializing application components");

    setup_resources_and_theme();

    let builder = Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/main.ui");
    let window = create_main_window(app, &builder);

    window.present();

    info!("Checking system dependencies");
    if !core::check_system_requirements(&window) {
        warn!("Dependency check failed - application will not continue");
        return;
    }

    // Initialize AUR helper after dependency checks pass
    if core::aur::init() {
        info!("AUR helper initialized successfully");
    }
    info!("Dependency check passed");

    info!("Loading individual page UI components");
    load_page_contents(&builder);

    let ctx = setup_ui_components(&builder);

    navigation::setup(&ctx.ui);

    info!("Setting initial view to main page");
    ctx.navigate_to_page("main_page");
    info!("Xero Toolkit application startup complete");
}

/// Set up resources and theme.
fn setup_resources_and_theme() {
    info!("Setting up resources and theme");

    gio::resources_register_include!("xyz.xerolinux.xero-toolkit.gresource")
        .expect("Failed to register gresources");

    if let Some(display) = gtk4::gdk::Display::default() {
        info!("Setting up UI theme and styling");

        let theme = gtk4::IconTheme::for_display(&display);
        // Don't inherit system icon themes
        theme.set_search_path(&[]);
        theme.add_resource_path("/xyz/xerolinux/xero-toolkit/icons");
        info!("Icon theme paths configured");

        let css_provider = CssProvider::new();
        css_provider.load_from_resource("/xyz/xerolinux/xero-toolkit/css/style.css");
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("UI theme and styling loaded successfully");
    } else {
        warn!("No default display found - UI theming may not work properly");
    }
}

/// Create main application window.
fn create_main_window(app: &Application, builder: &Builder) -> ApplicationWindow {
    let window: ApplicationWindow = builder
        .object("app_window")
        .expect("Failed to get app_window");

    window.set_application(Some(app));
    info!("Setting window icon to xero-toolkit");
    window.set_icon_name(Some("xero-toolkit"));
    info!("Main application window created from UI resource");

    window
}

/// Helper to extract widgets from builder with consistent error handling.
pub fn extract_widget<T: IsA<glib::Object>>(builder: &Builder, name: &str) -> T {
    builder
        .object(name)
        .unwrap_or_else(|| panic!("Failed to get widget with id '{}'", name))
}

/// Set up UI components and return application context.
fn setup_ui_components(builder: &Builder) -> AppContext {
    let stack = extract_widget(builder, "stack");
    let tabs_container = extract_widget(builder, "tabs_container");
    let main_paned = extract_widget(builder, "main_paned");

    info!("All UI components successfully initialized from UI builder");

    let ui = UiComponents::new(stack, tabs_container, main_paned);

    // Configure sidebar with size constraints from config
    ui.configure_sidebar(config::sidebar::MIN_WIDTH, config::sidebar::MAX_WIDTH);

    AppContext::new(ui)
}

/// Load page content from separate UI files into page containers.
fn load_page_contents(main_builder: &Builder) {
    let pages = [
        (
            "main_page",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/main_page.ui",
            "page_main_page_container",
        ),
        (
            "drivers",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/drivers.ui",
            "page_drivers_container",
        ),
        (
            "customization",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/customization.ui",
            "page_customization_container",
        ),
        (
            "gaming_tools",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/gaming_tools.ui",
            "page_gaming_tools_container",
        ),
        (
            "containers_vms",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/containers_vms.ui",
            "page_containers_vms_container",
        ),
        (
            "multimedia_tools",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/multimedia_tools.ui",
            "page_multimedia_tools_container",
        ),
        (
            "kernel_manager_scx",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/kernel_manager_scx.ui",
            "page_kernel_manager_scx_container",
        ),
        (
            "servicing_system_tweaks",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/servicing_system_tweaks.ui",
            "page_servicing_system_tweaks_container",
        ),
    ];

    for (page_name, resource_path, container_id) in pages {
        match load_page_from_resource(main_builder, page_name, resource_path, container_id) {
            Ok(_) => info!("Successfully loaded {} page", page_name),
            Err(e) => {
                warn!("Failed to load {} page: {}", page_name, e);
                if let Some(container) = main_builder.object::<GtkBox>(container_id) {
                    let fallback_label = gtk4::Label::builder()
                        .label(format!("{} page content not available", page_name))
                        .build();
                    container.append(&fallback_label);
                }
            }
        }
    }
}

/// Load a single page from a UI resource file.
fn load_page_from_resource(
    main_builder: &Builder,
    page_name: &str,
    resource_path: &str,
    container_id: &str,
) -> anyhow::Result<()> {
    let page_builder = Builder::from_resource(resource_path);

    let page_widget: gtk4::Widget = page_builder
        .object(format!("page_{}", page_name))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find page_{} widget in {}",
                page_name,
                resource_path
            )
        })?;

    let container: GtkBox = main_builder
        .object(container_id)
        .ok_or_else(|| anyhow::anyhow!("Could not find container {} in main UI", container_id))?;

    container.append(&page_widget);

    match page_name {
        "main_page" => pages::main_page::setup_handlers(&page_builder, main_builder),
        "drivers" => pages::drivers::setup_handlers(&page_builder, main_builder),
        "gaming_tools" => pages::gaming_tools::setup_handlers(&page_builder, main_builder),
        "containers_vms" => pages::containers_vms::setup_handlers(&page_builder, main_builder),
        "multimedia_tools" => pages::multimedia_tools::setup_handlers(&page_builder, main_builder),
        "servicing_system_tweaks" => pages::servicing::setup_handlers(&page_builder, main_builder),
        "customization" => pages::customization::setup_handlers(&page_builder, main_builder),
        _ => {}
    }

    Ok(())
}

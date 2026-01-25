//! `TerminalG` - GPU-accelerated terminal with integrated artifact viewing

mod settings;
mod terminal;
mod theme;
mod ui;
mod viewer;

use anyhow::Result;
use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Render, WindowBounds, WindowOptions,
};
use settings::SettingsStore;
use theme::Theme;
use tracing_subscriber::EnvFilter;

/// Empty view for initial GPUI bootstrap
struct EmptyView;

impl Render for EmptyView {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        // Get theme and settings from global state
        let theme = cx.global::<Theme>();
        let settings = cx.global::<SettingsStore>();

        // Convert our theme color to GPUI colors (RGB 0-255 -> 0xRRGGBB)
        let bg_color = rgb(
            u32::from(theme.ui.background.r) << 16
                | u32::from(theme.ui.background.g) << 8
                | u32::from(theme.ui.background.b),
        );
        let fg_color = rgb(
            u32::from(theme.ui.foreground.r) << 16
                | u32::from(theme.ui.foreground.g) << 8
                | u32::from(theme.ui.foreground.b),
        );

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(bg_color)
            .size_full()
            .text_color(fg_color)
            .child(div().text_2xl().child("TerminalG"))
            .child(
                div()
                    .text_sm()
                    .child(format!("Theme: {}", settings.settings().ui.theme)),
            )
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    tracing::info!("TerminalG starting...");

    // Load settings
    let settings_store = SettingsStore::new()?;
    tracing::info!("Settings loaded from {:?}", settings_store.settings_path());

    // Load theme
    let theme_name = &settings_store.settings().ui.theme;
    let theme = Theme::by_name(theme_name).unwrap_or_else(Theme::dark);
    tracing::info!("Loaded theme: {}", theme.name);

    // Initialize GPUI application
    App::new().run(move |cx: &mut gpui::AppContext| {
        // Store settings and theme in global state
        cx.set_global(settings_store);
        cx.set_global(theme);

        // Calculate centered window bounds (1200x800)
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        // Open main window
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("TerminalG".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |cx| cx.new_view(|_cx| EmptyView),
        )
        .expect("Failed to open window");

        cx.activate(true);

        tracing::info!("TerminalG window opened successfully");
    });

    Ok(())
}

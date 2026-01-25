//! `TerminalG` - GPU-accelerated terminal with integrated artifact viewing

mod settings;
mod terminal;
mod theme;
mod ui;
mod viewer;

use anyhow::Result;
use gpui::{prelude::*, px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use settings::SettingsStore;
use theme::Theme;
use tracing_subscriber::EnvFilter;
use ui::WorkspaceView;

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
    Application::new().run(move |cx: &mut App| {
        // Store settings and theme in global state
        cx.set_global(settings_store);
        cx.set_global(theme);

        // Quit application when all windows are closed
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        // Calculate centered window bounds (1200x800)
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        // Open main window with WorkspaceView
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("TerminalG".into()),
                    ..Default::default()
                }),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(WorkspaceView::new),
        )
        .expect("Failed to open window");

        cx.activate(true);

        tracing::info!("TerminalG window opened successfully");
    });

    tracing::info!("TerminalG shutdown complete");

    Ok(())
}

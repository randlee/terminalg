//! `TerminalG` - GPU-accelerated terminal with integrated artifact viewing

// Legacy modules (deprecated - kept for reference during migration)
#[allow(dead_code)]
mod settings;
#[allow(dead_code)]
mod theme;

// Adapter modules for Zed integration
mod settings_adapter;
mod theme_adapter;

// Active modules
mod terminal;
mod ui;
mod viewer;

use anyhow::Result;
use gpui::{prelude::*, px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use tracing_subscriber::EnvFilter;
use ui::WorkspaceView;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    tracing::info!("TerminalG starting...");

    // Initialize GPUI application
    Application::new().run(move |cx: &mut App| {
        // Initialize Zed's settings system first
        ::settings::init(cx);
        tracing::info!("Zed SettingsStore initialized");

        // Register TerminalG settings adapter
        settings_adapter::init(cx);

        // Initialize Zed's theme system
        theme_adapter::init(cx);

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

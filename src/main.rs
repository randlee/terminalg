//! TerminalG - GPU-accelerated terminal with integrated artifact viewing

mod settings;
mod terminal;
mod theme;
mod ui;
mod viewer;

use anyhow::Result;
use settings::SettingsStore;
use tracing_subscriber::EnvFilter;

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
    let theme = theme::Theme::by_name(theme_name)
        .unwrap_or_else(|| theme::Theme::dark());
    tracing::info!("Loaded theme: {}", theme.name);

    // TODO: Initialize GPUI app
    // Phase 1.5: Basic GPUI App

    tracing::info!("TerminalG initialized successfully");

    Ok(())
}

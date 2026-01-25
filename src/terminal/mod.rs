//! Terminal emulation components
//!
//! This module provides the terminal pane and tab management for `TerminalG`,
//! wrapping Zed's terminal crate for PTY management and rendering.

mod pane;
mod tab;

pub use pane::{TerminalPane, TerminalPaneEvent};
#[allow(unused_imports)] // Exported for future use
pub use tab::TerminalTab;

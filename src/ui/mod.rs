//! UI layout and components

mod workspace;
mod workspace_config;

pub use workspace::WorkspaceView;
// Re-export for potential use by other modules
#[allow(unused_imports)]
pub use workspace_config::{WorkspaceConfig, WorkspaceConfigStore, WorkspacesConfig};

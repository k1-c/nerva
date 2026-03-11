mod capture_screen;
mod clipboard_read;
mod launch_app;
mod list_windows;
mod run_command;

use std::sync::Arc;

use nerva_core::ToolRegistry;

pub use capture_screen::CaptureScreenSkill;
pub use clipboard_read::ClipboardReadSkill;
pub use launch_app::LaunchAppSkill;
pub use list_windows::ListWindowsSkill;
pub use run_command::RunCommandSafeSkill;

pub async fn register_all_skills(registry: &ToolRegistry) {
    registry.register(Arc::new(LaunchAppSkill)).await;
    registry.register(Arc::new(ListWindowsSkill)).await;
    registry.register(Arc::new(CaptureScreenSkill)).await;
    registry.register(Arc::new(RunCommandSafeSkill)).await;
    registry.register(Arc::new(ClipboardReadSkill)).await;
}

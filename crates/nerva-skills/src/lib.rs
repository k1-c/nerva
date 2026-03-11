mod active_window;
mod capture_screen;
mod clipboard_read;
mod focus_window;
mod gather_context;
mod launch_app;
mod list_windows;
mod notify;
mod ocr_screen;
mod open_path;
pub mod plugin_loader;
mod run_command;
pub mod script_skill;
mod summarize_screen;

use std::sync::Arc;

use nerva_core::ToolRegistry;

pub use active_window::GetActiveWindowSkill;
pub use capture_screen::CaptureScreenSkill;
pub use clipboard_read::ClipboardReadSkill;
pub use focus_window::FocusWindowSkill;
pub use gather_context::GatherContextSkill;
pub use launch_app::LaunchAppSkill;
pub use list_windows::ListWindowsSkill;
pub use notify::NotifySkill;
pub use ocr_screen::OcrScreenSkill;
pub use open_path::OpenPathSkill;
pub use run_command::RunCommandSafeSkill;
pub use summarize_screen::SummarizeScreenSkill;

pub async fn register_all_skills(registry: &ToolRegistry) {
    registry.register(Arc::new(LaunchAppSkill)).await;
    registry.register(Arc::new(ListWindowsSkill)).await;
    registry.register(Arc::new(CaptureScreenSkill)).await;
    registry.register(Arc::new(RunCommandSafeSkill::default())).await;
    registry.register(Arc::new(ClipboardReadSkill)).await;
    registry.register(Arc::new(NotifySkill)).await;
    registry.register(Arc::new(OpenPathSkill)).await;
    registry.register(Arc::new(GetActiveWindowSkill)).await;
    registry.register(Arc::new(FocusWindowSkill)).await;
    registry.register(Arc::new(OcrScreenSkill)).await;
    registry.register(Arc::new(GatherContextSkill)).await;
    registry.register(Arc::new(SummarizeScreenSkill::new(None))).await;
}

pub async fn register_all_skills_with_config(
    registry: &ToolRegistry,
    config: &nerva_core::config::NervaConfig,
) {
    registry.register(Arc::new(LaunchAppSkill)).await;
    registry.register(Arc::new(ListWindowsSkill)).await;
    registry.register(Arc::new(CaptureScreenSkill)).await;
    registry
        .register(Arc::new(RunCommandSafeSkill::with_allowed(
            config.commands.allowed.iter().cloned().collect(),
        )))
        .await;
    registry.register(Arc::new(ClipboardReadSkill)).await;
    registry.register(Arc::new(NotifySkill)).await;
    registry.register(Arc::new(OpenPathSkill)).await;
    registry.register(Arc::new(GetActiveWindowSkill)).await;
    registry.register(Arc::new(FocusWindowSkill)).await;
    registry.register(Arc::new(OcrScreenSkill)).await;
    registry.register(Arc::new(GatherContextSkill)).await;

    // Create VLM client if configured
    let vlm_client = if config.vlm.enabled {
        Some(nerva_core::vlm::VlmClient::default_ollama(&config.vlm.model))
    } else {
        None
    };
    registry
        .register(Arc::new(SummarizeScreenSkill::new(vlm_client)))
        .await;
}

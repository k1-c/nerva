use serde::{Deserialize, Serialize};

use crate::types::WindowInfo;

/// Assembled context representing the current desktop state.
///
/// Combines active window info, clipboard contents, and screen-extracted text
/// into a single struct for use by the agent runtime.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesktopContext {
    /// Currently focused window, if any.
    pub active_window: Option<WindowInfo>,
    /// Current clipboard text content, if any.
    pub clipboard: Option<String>,
    /// Text extracted from the screen via OCR, if available.
    pub screen_text: Option<String>,
    /// Path to the latest screenshot, if captured.
    pub screenshot_path: Option<String>,
}

impl DesktopContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_active_window(mut self, window: WindowInfo) -> Self {
        self.active_window = Some(window);
        self
    }

    pub fn with_clipboard(mut self, text: String) -> Self {
        self.clipboard = Some(text);
        self
    }

    pub fn with_screen_text(mut self, text: String) -> Self {
        self.screen_text = Some(text);
        self
    }

    pub fn with_screenshot_path(mut self, path: String) -> Self {
        self.screenshot_path = Some(path);
        self
    }

    /// Produce a text summary of the context for LLM consumption.
    pub fn to_prompt_text(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref window) = self.active_window {
            parts.push(format!(
                "[Active Window] app={} title=\"{}\"",
                window.app_id, window.title
            ));
        }

        if let Some(ref clipboard) = self.clipboard {
            let truncated = if clipboard.len() > 500 {
                format!("{}... ({} chars)", &clipboard[..500], clipboard.len())
            } else {
                clipboard.clone()
            };
            parts.push(format!("[Clipboard] {truncated}"));
        }

        if let Some(ref screen_text) = self.screen_text {
            let truncated = if screen_text.len() > 1000 {
                format!("{}... ({} chars)", &screen_text[..1000], screen_text.len())
            } else {
                screen_text.clone()
            };
            parts.push(format!("[Screen Text] {truncated}"));
        }

        if parts.is_empty() {
            "No context available.".to_string()
        } else {
            parts.join("\n")
        }
    }
}

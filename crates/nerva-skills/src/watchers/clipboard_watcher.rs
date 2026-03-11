use std::time::Duration;

use nerva_core::error::NervaError;
use nerva_core::watcher::{SuggestedAction, Suggestion, Watcher};
use tokio::sync::Mutex;

/// Watches clipboard changes and suggests actions based on content.
///
/// - URLs → offer to open or summarize
/// - File paths → offer to open
pub struct ClipboardWatcher {
    last_content: Mutex<String>,
}

impl ClipboardWatcher {
    pub fn new() -> Self {
        Self {
            last_content: Mutex::new(String::new()),
        }
    }
}

#[async_trait::async_trait]
impl Watcher for ClipboardWatcher {
    fn id(&self) -> &str {
        "clipboard_watcher"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(2)
    }

    async fn poll(&self) -> Result<Vec<Suggestion>, NervaError> {
        let content = match nerva_os::clipboard::read_clipboard().await {
            Ok(c) => c.trim().to_string(),
            Err(_) => return Ok(vec![]),
        };

        if content.is_empty() {
            return Ok(vec![]);
        }

        let mut last = self.last_content.lock().await;
        if *last == content {
            return Ok(vec![]);
        }
        *last = content.clone();

        // Analyze content and generate suggestions
        let mut suggestions = Vec::new();

        if looks_like_url(&content) {
            suggestions.push(Suggestion {
                source: self.id().into(),
                title: "URL detected in clipboard".into(),
                body: format!("Open {content}?"),
                action: Some(SuggestedAction {
                    tool_id: "open_path".into(),
                    input: serde_json::json!({ "path": content }),
                }),
            });
        } else if looks_like_file_path(&content) {
            suggestions.push(Suggestion {
                source: self.id().into(),
                title: "File path detected in clipboard".into(),
                body: format!("Open {content}?"),
                action: Some(SuggestedAction {
                    tool_id: "open_path".into(),
                    input: serde_json::json!({ "path": content }),
                }),
            });
        }

        Ok(suggestions)
    }
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
}

fn looks_like_file_path(s: &str) -> bool {
    (s.starts_with('/') || s.starts_with("~/")) && !s.contains('\n') && s.len() < 512
}

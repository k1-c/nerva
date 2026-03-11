use std::time::Duration;

use nerva_core::error::NervaError;
use nerva_core::watcher::{Suggestion, Watcher};
use tokio::sync::Mutex;

/// Watches active window changes and suggests contextual actions.
pub struct ActiveWindowWatcher {
    last_app_id: Mutex<String>,
}

impl ActiveWindowWatcher {
    pub fn new() -> Self {
        Self {
            last_app_id: Mutex::new(String::new()),
        }
    }
}

#[async_trait::async_trait]
impl Watcher for ActiveWindowWatcher {
    fn id(&self) -> &str {
        "window_watcher"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(3)
    }

    async fn poll(&self) -> Result<Vec<Suggestion>, NervaError> {
        let window = match nerva_os::wayland::get_active_window().await {
            Ok(Some(w)) => w,
            _ => return Ok(vec![]),
        };

        let mut last = self.last_app_id.lock().await;
        if *last == window.app_id {
            return Ok(vec![]);
        }

        let prev = last.clone();
        *last = window.app_id.clone();

        // Only suggest on actual app switches (not initial detection)
        if prev.is_empty() {
            return Ok(vec![]);
        }

        let mut suggestions = Vec::new();

        // Context-aware suggestions based on app
        let app = window.app_id.to_lowercase();
        if app.contains("firefox") || app.contains("chromium") || app.contains("chrome") {
            suggestions.push(Suggestion {
                source: self.id().into(),
                title: format!("Switched to browser: {}", window.title),
                body: "You can ask me to summarize the page or take a screenshot.".into(),
                action: None,
            });
        } else if app.contains("code") || app.contains("nvim") || app.contains("vim") {
            suggestions.push(Suggestion {
                source: self.id().into(),
                title: format!("Switched to editor: {}", window.title),
                body: "You can ask me to help with code or run commands.".into(),
                action: None,
            });
        } else if app.contains("terminal")
            || app.contains("kitty")
            || app.contains("alacritty")
            || app.contains("foot")
        {
            suggestions.push(Suggestion {
                source: self.id().into(),
                title: format!("Switched to terminal: {}", window.title),
                body: "You can ask me to run a safe command or capture the screen.".into(),
                action: None,
            });
        }

        Ok(suggestions)
    }
}

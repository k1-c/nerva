pub mod clipboard_watcher;
pub mod window_watcher;

use std::sync::Arc;

use nerva_core::watcher::WatcherManager;

/// Register all built-in watchers.
pub fn register_all_watchers(manager: &mut WatcherManager) {
    manager.register(Arc::new(clipboard_watcher::ClipboardWatcher::new()));
    manager.register(Arc::new(window_watcher::ActiveWindowWatcher::new()));
}

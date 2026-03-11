use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::policy::PolicyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NervaConfig {
    pub daemon: DaemonConfig,
    pub policy: PolicyConfig,
    pub commands: CommandsConfig,
    pub plugins: PluginsConfig,
    pub vlm: VlmConfig,
    pub llm: LlmConfig,
    pub watchers: WatchersConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub socket_path: Option<PathBuf>,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandsConfig {
    pub allowed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    pub skills_dir: Option<PathBuf>,
    pub enabled: bool,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            skills_dir: None,
            enabled: true,
        }
    }
}

impl PluginsConfig {
    pub fn skills_dir(&self) -> PathBuf {
        self.skills_dir.clone().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("nerva/skills")
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VlmConfig {
    pub enabled: bool,
    pub ollama_url: String,
    pub model: String,
}

impl Default for VlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ollama_url: "http://localhost:11434".into(),
            model: "moondream".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    pub enabled: bool,
    /// Provider: "claude", "openai", or "ollama"
    pub provider: String,
    pub model: String,
    /// API key (for claude/openai). Can also be set via ANTHROPIC_API_KEY or OPENAI_API_KEY env vars.
    pub api_key: Option<String>,
    /// Base URL override (for openai-compatible endpoints or ollama)
    pub base_url: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "claude".into(),
            model: "claude-sonnet-4-20250514".into(),
            api_key: None,
            base_url: None,
        }
    }
}

impl LlmConfig {
    /// Resolve the API key from config or environment variable.
    pub fn resolve_api_key(&self) -> Option<String> {
        self.api_key.clone().or_else(|| {
            match self.provider.as_str() {
                "claude" => std::env::var("ANTHROPIC_API_KEY").ok(),
                "openai" => std::env::var("OPENAI_API_KEY").ok(),
                _ => None,
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchersConfig {
    pub enabled: bool,
    /// Send suggestions as desktop notifications.
    pub notify: bool,
}

impl Default for WatchersConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            notify: true,
        }
    }
}

impl Default for NervaConfig {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig::default(),
            policy: PolicyConfig::default(),
            commands: CommandsConfig::default(),
            plugins: PluginsConfig::default(),
            vlm: VlmConfig::default(),
            llm: LlmConfig::default(),
            watchers: WatchersConfig::default(),
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: None,
            log_level: "info".into(),
        }
    }
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            allowed: vec![
                "ls".into(),
                "pwd".into(),
                "whoami".into(),
                "date".into(),
                "uname".into(),
                "cat".into(),
                "head".into(),
                "tail".into(),
                "wc".into(),
                "df".into(),
                "free".into(),
                "uptime".into(),
                "hostname".into(),
            ],
        }
    }
}

impl NervaConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        let candidates = config_paths();
        for path in &candidates {
            if path.exists() {
                match Self::load(path) {
                    Ok(config) => {
                        tracing::info!(path = %path.display(), "Loaded config");
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "Failed to load config, using defaults");
                    }
                }
            }
        }
        Self::default()
    }

    pub fn socket_path(&self) -> PathBuf {
        self.daemon.socket_path.clone().unwrap_or_else(default_socket_path)
    }
}

pub fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime_dir).join("nerva/nervad.sock")
}

fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("nerva/config.toml"));
    }

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".config/nerva/config.toml"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NervaConfig::default();
        assert_eq!(config.daemon.log_level, "info");
        assert!(config.daemon.socket_path.is_none());
        assert!(config.policy.auto_approve_safe);
        assert!(!config.policy.auto_approve_caution);
        assert!(!config.commands.allowed.is_empty());
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[daemon]
log_level = "debug"

[policy]
auto_approve_safe = true
auto_approve_caution = true

[commands]
allowed = ["ls", "pwd", "echo"]
"#;
        let config: NervaConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.daemon.log_level, "debug");
        assert!(config.policy.auto_approve_caution);
        assert_eq!(config.commands.allowed, vec!["ls", "pwd", "echo"]);
    }
}

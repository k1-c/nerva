use std::collections::HashSet;
use std::sync::LazyLock;

use nerva_core::{NervaError, RiskTier, Skill, ToolMetadata};

static METADATA: LazyLock<ToolMetadata> = LazyLock::new(|| ToolMetadata {
    id: "run_command_safe".into(),
    name: "Run Safe Command".into(),
    description: "Execute a command from the allowlist".into(),
    risk: RiskTier::Caution,
    confirmation_required: true,
});

static DEFAULT_ALLOWED: LazyLock<HashSet<String>> = LazyLock::new(|| {
    HashSet::from([
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
    ])
});

pub struct RunCommandSafeSkill {
    allowed: HashSet<String>,
}

impl Default for RunCommandSafeSkill {
    fn default() -> Self {
        Self {
            allowed: DEFAULT_ALLOWED.clone(),
        }
    }
}

impl RunCommandSafeSkill {
    pub fn with_allowed(allowed: HashSet<String>) -> Self {
        Self { allowed }
    }
}

#[async_trait::async_trait]
impl Skill for RunCommandSafeSkill {
    fn metadata(&self) -> &ToolMetadata {
        &METADATA
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let cmd = input
            .get("cmd")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NervaError::InvalidInput("missing 'cmd' field".into()))?;

        let args: Vec<String> = input
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let (stdout, exit_code) =
            nerva_os::process::run_command_safe(cmd, &args, &self.allowed).await?;

        Ok(serde_json::json!({
            "stdout": stdout,
            "exit_code": exit_code,
        }))
    }
}

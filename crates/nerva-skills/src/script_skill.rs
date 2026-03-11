use std::path::PathBuf;

use nerva_core::error::NervaError;
use nerva_core::types::ToolMetadata;
use nerva_core::Skill;

/// A skill backed by an external executable script.
///
/// The script receives JSON input on stdin and must write JSON output to stdout.
/// Exit code 0 indicates success; non-zero indicates failure.
pub struct ScriptSkill {
    metadata: ToolMetadata,
    executable: PathBuf,
}

impl ScriptSkill {
    pub fn new(metadata: ToolMetadata, executable: PathBuf) -> Self {
        Self {
            metadata,
            executable,
        }
    }
}

#[async_trait::async_trait]
impl Skill for ScriptSkill {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, NervaError> {
        let input_bytes = serde_json::to_vec(&input)
            .map_err(|e| NervaError::InvalidInput(format!("Failed to serialize input: {e}")))?;

        let output = tokio::process::Command::new(&self.executable)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                NervaError::ExecutionError(format!(
                    "Failed to spawn plugin '{}': {e}",
                    self.metadata.id
                ))
            })?;

        use tokio::io::AsyncWriteExt;
        let mut child = output;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&input_bytes).await.map_err(|e| {
                NervaError::ExecutionError(format!("Failed to write stdin: {e}"))
            })?;
        }

        let output = child.wait_with_output().await.map_err(|e| {
            NervaError::ExecutionError(format!("Failed to wait for plugin: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NervaError::ExecutionError(format!(
                "Plugin '{}' exited with {}: {}",
                self.metadata.id,
                output.status,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|e| {
            NervaError::ExecutionError(format!(
                "Plugin '{}' returned invalid JSON: {e}",
                self.metadata.id
            ))
        })?;

        Ok(result)
    }
}

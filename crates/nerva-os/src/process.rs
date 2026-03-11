use std::collections::HashSet;

use nerva_core::NervaError;
use tokio::process::Command;

pub async fn launch_app(app_name: &str) -> Result<(), NervaError> {
    let output = Command::new("gtk-launch")
        .arg(app_name)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to launch app: {e}")))?;

    if !output.status.success() {
        // Fallback to xdg-open
        let output = Command::new("xdg-open")
            .arg(app_name)
            .output()
            .await
            .map_err(|e| NervaError::OsError(format!("Failed to launch app: {e}")))?;

        if !output.status.success() {
            return Err(NervaError::OsError(format!(
                "Failed to launch '{}': {}",
                app_name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    Ok(())
}

pub async fn run_command_safe(
    cmd: &str,
    args: &[String],
    allowed_commands: &HashSet<String>,
) -> Result<(String, i32), NervaError> {
    if !allowed_commands.contains(cmd) {
        return Err(NervaError::PolicyDenied(format!(
            "Command '{}' is not in the allowlist",
            cmd
        )));
    }

    let output = Command::new(cmd)
        .args(args)
        .output()
        .await
        .map_err(|e| NervaError::OsError(format!("Failed to run command: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, exit_code))
}

use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;

use nerva_core::types::{RiskTier, ToolMetadata};
use nerva_core::{Skill, ToolRegistry};

use crate::script_skill::ScriptSkill;

/// Manifest format for a script-based skill plugin.
///
/// Each plugin is a directory under `skills_dir` containing:
/// - `skill.toml` — this manifest
/// - An executable file referenced by `executable` (default: `run`)
#[derive(Debug, Deserialize)]
struct SkillManifest {
    id: String,
    name: String,
    description: String,
    #[serde(default = "default_risk")]
    risk: RiskTier,
    #[serde(default)]
    executable: Option<String>,
}

fn default_risk() -> RiskTier {
    RiskTier::Caution
}

/// Scan a directory for plugin subdirectories and register them.
///
/// Directory structure:
/// ```text
/// skills_dir/
///   my-skill/
///     skill.toml
///     run          (executable)
///   another-skill/
///     skill.toml
///     main.py      (referenced in skill.toml)
/// ```
pub async fn load_plugins(skills_dir: &Path, registry: &ToolRegistry) -> Vec<String> {
    let mut loaded = Vec::new();

    let entries = match std::fs::read_dir(skills_dir) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    path = %skills_dir.display(),
                    error = %e,
                    "Failed to read plugin directory"
                );
            }
            return loaded;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("skill.toml");
        if !manifest_path.exists() {
            tracing::debug!(dir = %path.display(), "Skipping directory without skill.toml");
            continue;
        }

        match load_single_plugin(&path, &manifest_path).await {
            Ok(skill) => {
                let id = skill.metadata().id.clone();
                if registry.contains(&id).await {
                    tracing::warn!(
                        skill_id = %id,
                        "Plugin skill ID conflicts with existing skill, skipping"
                    );
                    continue;
                }
                registry.register(Arc::new(skill)).await;
                tracing::info!(skill_id = %id, dir = %path.display(), "Loaded plugin skill");
                loaded.push(id);
            }
            Err(e) => {
                tracing::warn!(
                    dir = %path.display(),
                    error = %e,
                    "Failed to load plugin"
                );
            }
        }
    }

    loaded
}

async fn load_single_plugin(
    plugin_dir: &Path,
    manifest_path: &Path,
) -> anyhow::Result<ScriptSkill> {
    let content = tokio::fs::read_to_string(manifest_path).await?;
    let manifest: SkillManifest = toml::from_str(&content)?;

    let executable_name = manifest.executable.as_deref().unwrap_or("run");
    let executable = plugin_dir.join(executable_name);

    if !executable.exists() {
        anyhow::bail!(
            "Executable '{}' not found in {}",
            executable_name,
            plugin_dir.display()
        );
    }

    let metadata = ToolMetadata {
        id: manifest.id,
        name: manifest.name,
        description: manifest.description,
        risk: manifest.risk,
        confirmation_required: manifest.risk != RiskTier::Safe,
    };

    Ok(ScriptSkill::new(metadata, executable))
}

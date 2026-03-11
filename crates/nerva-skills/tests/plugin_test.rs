use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use nerva_core::ToolRegistry;

fn create_temp_plugin_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();

    // Create a valid plugin: "echo-plugin"
    let plugin_dir = dir.path().join("echo-plugin");
    std::fs::create_dir(&plugin_dir).unwrap();

    std::fs::write(
        plugin_dir.join("skill.toml"),
        r#"
id = "echo_test"
name = "Echo Test"
description = "Echoes input back as output"
risk = "safe"
"#,
    )
    .unwrap();

    let script = r#"#!/bin/sh
cat
"#;
    let run_path = plugin_dir.join("run");
    std::fs::write(&run_path, script).unwrap();
    std::fs::set_permissions(&run_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Create a plugin with custom executable name
    let plugin_dir2 = dir.path().join("custom-exec");
    std::fs::create_dir(&plugin_dir2).unwrap();

    std::fs::write(
        plugin_dir2.join("skill.toml"),
        r#"
id = "custom_exec_test"
name = "Custom Exec"
description = "Plugin with custom executable"
risk = "caution"
executable = "main.sh"
"#,
    )
    .unwrap();

    let script2 = r#"#!/bin/sh
echo '{"result": "custom"}'
"#;
    let exec_path = plugin_dir2.join("main.sh");
    std::fs::write(&exec_path, script2).unwrap();
    std::fs::set_permissions(&exec_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Create an invalid plugin (missing executable)
    let bad_dir = dir.path().join("bad-plugin");
    std::fs::create_dir(&bad_dir).unwrap();
    std::fs::write(
        bad_dir.join("skill.toml"),
        r#"
id = "bad_plugin"
name = "Bad"
description = "Missing executable"
"#,
    )
    .unwrap();

    // Create a non-plugin directory (no skill.toml)
    let ignored_dir = dir.path().join("not-a-plugin");
    std::fs::create_dir(&ignored_dir).unwrap();
    std::fs::write(ignored_dir.join("README.md"), "not a plugin").unwrap();

    dir
}

#[tokio::test]
async fn test_load_plugins() {
    let dir = create_temp_plugin_dir();
    let registry = Arc::new(ToolRegistry::new());

    let loaded =
        nerva_skills::plugin_loader::load_plugins(dir.path(), registry.as_ref()).await;

    // Should load echo-plugin and custom-exec, skip bad-plugin and not-a-plugin
    assert_eq!(loaded.len(), 2);
    assert!(loaded.contains(&"echo_test".to_string()));
    assert!(loaded.contains(&"custom_exec_test".to_string()));

    // Verify they're in the registry
    assert!(registry.contains("echo_test").await);
    assert!(registry.contains("custom_exec_test").await);
    assert!(!registry.contains("bad_plugin").await);
}

#[tokio::test]
async fn test_load_plugins_nonexistent_dir() {
    let registry = Arc::new(ToolRegistry::new());
    let loaded = nerva_skills::plugin_loader::load_plugins(
        std::path::Path::new("/nonexistent/path"),
        registry.as_ref(),
    )
    .await;
    assert!(loaded.is_empty());
}

#[tokio::test]
async fn test_script_skill_execute() {
    let dir = create_temp_plugin_dir();
    let registry = Arc::new(ToolRegistry::new());

    nerva_skills::plugin_loader::load_plugins(dir.path(), registry.as_ref()).await;

    let skill = registry.get("echo_test").await.unwrap();
    let input = serde_json::json!({"hello": "world"});
    let output = skill.execute(input.clone()).await.unwrap();
    assert_eq!(output, input);
}

#[tokio::test]
async fn test_script_skill_custom_exec() {
    let dir = create_temp_plugin_dir();
    let registry = Arc::new(ToolRegistry::new());

    nerva_skills::plugin_loader::load_plugins(dir.path(), registry.as_ref()).await;

    let skill = registry.get("custom_exec_test").await.unwrap();
    let output = skill.execute(serde_json::json!({})).await.unwrap();
    assert_eq!(output["result"], "custom");
}

#[tokio::test]
async fn test_plugin_no_conflict_with_builtin() {
    let dir = tempfile::tempdir().unwrap();

    // Create a plugin that tries to use a builtin skill ID
    let plugin_dir = dir.path().join("conflict-plugin");
    std::fs::create_dir(&plugin_dir).unwrap();
    std::fs::write(
        plugin_dir.join("skill.toml"),
        r#"
id = "launch_app"
name = "Conflicting"
description = "Tries to override builtin"
"#,
    )
    .unwrap();

    let run_path = plugin_dir.join("run");
    std::fs::write(&run_path, "#!/bin/sh\necho '{}'").unwrap();
    std::fs::set_permissions(
        &run_path,
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();

    let registry = Arc::new(ToolRegistry::new());
    nerva_skills::register_all_skills(registry.as_ref()).await;

    let loaded =
        nerva_skills::plugin_loader::load_plugins(dir.path(), registry.as_ref()).await;

    // Should be skipped due to ID conflict
    assert!(loaded.is_empty());
}

#[tokio::test]
async fn test_registry_unregister() {
    let registry = ToolRegistry::new();
    nerva_skills::register_all_skills(&registry).await;

    assert!(registry.contains("launch_app").await);
    assert!(registry.unregister("launch_app").await);
    assert!(!registry.contains("launch_app").await);
    assert!(!registry.unregister("launch_app").await); // already removed
}

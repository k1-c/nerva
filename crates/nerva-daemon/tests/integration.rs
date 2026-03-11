use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use nerva_core::{CapabilityBus, PolicyEngine, ToolRegistry};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_socket_path() -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("nerva_test_{}_{n}.sock", std::process::id()))
}

async fn start_test_server() -> (PathBuf, tokio::task::JoinHandle<()>) {
    let socket_path = unique_socket_path();
    let _ = tokio::fs::remove_file(&socket_path).await;

    let registry = Arc::new(ToolRegistry::new());
    nerva_skills::register_all_skills(registry.as_ref()).await;

    let policy = PolicyEngine::default();
    let bus = Arc::new(CapabilityBus::new(registry, policy));

    let path = socket_path.clone();
    let handle = tokio::spawn(async move {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }
        let listener = tokio::net::UnixListener::bind(&path).unwrap();

        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let bus = bus.clone();

            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();

                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => {
                            let req: serde_json::Value =
                                serde_json::from_str(line.trim()).unwrap();

                            let response = match req["command"].as_str().unwrap() {
                                "status" => serde_json::json!({
                                    "ok": true,
                                    "data": {
                                        "status": "running",
                                        "tools_registered": bus.registry().list().await.len(),
                                    }
                                }),
                                "list_tools" => {
                                    let tools = bus.registry().list().await;
                                    serde_json::json!({
                                        "ok": true,
                                        "data": { "tools": tools }
                                    })
                                }
                                "execute" => {
                                    let tool_id = req["tool_id"].as_str().unwrap();
                                    let input = req
                                        .get("input")
                                        .cloned()
                                        .unwrap_or(serde_json::json!({}));
                                    let exec_req =
                                        nerva_core::ExecutionRequest::new(tool_id, input);
                                    let result = bus.execute(exec_req).await;
                                    serde_json::json!({
                                        "ok": true,
                                        "data": result,
                                    })
                                }
                                _ => {
                                    serde_json::json!({"ok": false, "error": "unknown command"})
                                }
                            };

                            let mut bytes = serde_json::to_vec(&response).unwrap();
                            bytes.push(b'\n');
                            if writer.write_all(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    });

    // Wait for socket to be ready
    for _ in 0..100 {
        if socket_path.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    (socket_path, handle)
}

async fn send_request(socket_path: &PathBuf, request: serde_json::Value) -> serde_json::Value {
    let stream = UnixStream::connect(socket_path).await.unwrap();
    let (reader, mut writer) = stream.into_split();

    let mut req_bytes = serde_json::to_vec(&request).unwrap();
    req_bytes.push(b'\n');
    writer.write_all(&req_bytes).await.unwrap();
    writer.shutdown().await.unwrap();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

async fn cleanup(socket_path: &PathBuf, handle: tokio::task::JoinHandle<()>) {
    handle.abort();
    let _ = tokio::fs::remove_file(socket_path).await;
}

#[tokio::test]
async fn test_status() {
    let (socket_path, handle) = start_test_server().await;
    let resp = send_request(&socket_path, serde_json::json!({"command": "status"})).await;
    assert_eq!(resp["ok"], true);
    assert_eq!(resp["data"]["status"], "running");
    assert!(resp["data"]["tools_registered"].as_u64().unwrap() >= 5);
    cleanup(&socket_path, handle).await;
}

#[tokio::test]
async fn test_list_tools() {
    let (socket_path, handle) = start_test_server().await;
    let resp = send_request(&socket_path, serde_json::json!({"command": "list_tools"})).await;
    assert_eq!(resp["ok"], true);
    let tools = resp["data"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 9);

    let ids: Vec<&str> = tools.iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"launch_app"));
    assert!(ids.contains(&"list_windows"));
    assert!(ids.contains(&"clipboard_read"));
    assert!(ids.contains(&"notify"));
    assert!(ids.contains(&"get_active_window"));
    assert!(ids.contains(&"focus_window"));
    assert!(ids.contains(&"open_path"));
    cleanup(&socket_path, handle).await;
}

#[tokio::test]
async fn test_execute_run_command() {
    let (socket_path, handle) = start_test_server().await;
    let resp = send_request(
        &socket_path,
        serde_json::json!({
            "command": "execute",
            "tool_id": "run_command_safe",
            "input": {"cmd": "uname", "args": ["-s"]}
        }),
    )
    .await;
    assert_eq!(resp["ok"], true);
    assert_eq!(resp["data"]["status"], "success");
    let output = &resp["data"]["output"];
    assert_eq!(output["exit_code"], 0);
    assert!(output["stdout"].as_str().unwrap().contains("Linux"));
    cleanup(&socket_path, handle).await;
}

#[tokio::test]
async fn test_execute_nonexistent_tool() {
    let (socket_path, handle) = start_test_server().await;
    let resp = send_request(
        &socket_path,
        serde_json::json!({
            "command": "execute",
            "tool_id": "nonexistent",
        }),
    )
    .await;
    assert_eq!(resp["ok"], true);
    assert_eq!(resp["data"]["status"], "failed");
    assert!(resp["data"]["error"]
        .as_str()
        .unwrap()
        .contains("not found"));
    cleanup(&socket_path, handle).await;
}

#[tokio::test]
async fn test_execute_blocked_command() {
    let (socket_path, handle) = start_test_server().await;
    let resp = send_request(
        &socket_path,
        serde_json::json!({
            "command": "execute",
            "tool_id": "run_command_safe",
            "input": {"cmd": "rm", "args": ["-rf", "/"]}
        }),
    )
    .await;
    assert_eq!(resp["ok"], true);
    assert_eq!(resp["data"]["status"], "failed");
    assert!(resp["data"]["error"]
        .as_str()
        .unwrap()
        .contains("allowlist"));
    cleanup(&socket_path, handle).await;
}

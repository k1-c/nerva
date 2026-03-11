use std::path::Path;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use nerva_core::CapabilityBus;

use crate::handler;
use crate::protocol::{Request, Response};

pub async fn run_server(bus: Arc<CapabilityBus>, socket_path: &Path) -> anyhow::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Remove stale socket
    let _ = tokio::fs::remove_file(socket_path).await;

    let listener = UnixListener::bind(socket_path)?;
    tracing::info!(path = %socket_path.display(), "Daemon listening");

    loop {
        let (stream, _) = listener.accept().await?;
        let bus = bus.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let response = match serde_json::from_str::<Request>(line.trim()) {
                            Ok(req) => handler::handle(&bus, req).await,
                            Err(e) => Response::error(format!("Invalid request: {e}")),
                        };

                        let mut resp_bytes =
                            serde_json::to_vec(&response).unwrap_or_default();
                        resp_bytes.push(b'\n');

                        if writer.write_all(&resp_bytes).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Read error: {e}");
                        break;
                    }
                }
            }
        });
    }
}

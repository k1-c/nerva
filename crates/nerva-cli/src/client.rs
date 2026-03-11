use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub async fn send_request(
    socket_path: &Path,
    request: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let stream = UnixStream::connect(socket_path).await.map_err(|e| {
        anyhow::anyhow!(
            "Failed to connect to daemon at {}: {e}\nIs nervad running?",
            socket_path.display()
        )
    })?;

    let (reader, mut writer) = stream.into_split();

    let mut req_bytes = serde_json::to_vec(&request)?;
    req_bytes.push(b'\n');
    writer.write_all(&req_bytes).await?;
    writer.shutdown().await?;

    let mut reader = BufReader::new(reader);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())?;
    Ok(response)
}

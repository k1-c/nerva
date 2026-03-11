mod client;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nerva", about = "Nerva AI Desktop Agent CLI")]
struct Cli {
    /// Path to daemon socket
    #[arg(long, global = true)]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a skill
    Exec {
        /// Tool ID to execute
        tool_id: String,
        /// JSON input for the tool
        #[arg(long, default_value = "{}")]
        input: String,
    },
    /// List available tools
    Tools,
    /// Show recent execution log
    Log {
        /// Number of records to show
        #[arg(long, default_value_t = 10)]
        count: usize,
    },
    /// Show daemon status
    Status,
}

fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime_dir).join("nerva/nervad.sock")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let socket_path = cli.socket.unwrap_or_else(default_socket_path);

    let request = match cli.command {
        Commands::Exec { tool_id, input } => {
            let input: serde_json::Value = serde_json::from_str(&input)
                .map_err(|e| anyhow::anyhow!("Invalid JSON input: {e}"))?;
            serde_json::json!({
                "command": "execute",
                "tool_id": tool_id,
                "input": input,
            })
        }
        Commands::Tools => serde_json::json!({ "command": "list_tools" }),
        Commands::Log { count } => serde_json::json!({
            "command": "get_log",
            "count": count,
        }),
        Commands::Status => serde_json::json!({ "command": "status" }),
    };

    let response = client::send_request(&socket_path, request).await?;

    // Pretty print
    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

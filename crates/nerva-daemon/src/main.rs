mod handler;
mod protocol;
mod server;

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use nerva_core::{CapabilityBus, PolicyEngine, ToolRegistry};

#[derive(Parser)]
#[command(name = "nervad", about = "Nerva AI Desktop Agent Daemon")]
struct Args {
    /// Path to the Unix socket
    #[arg(long)]
    socket: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn default_socket_path() -> PathBuf {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime_dir).join("nerva/nervad.sock")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let socket_path = args.socket.unwrap_or_else(default_socket_path);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| args.log_level.parse().unwrap_or_default()),
        )
        .init();

    tracing::info!("Starting nervad");

    let registry = Arc::new(ToolRegistry::new());
    nerva_skills::register_all_skills(&registry).await;

    let policy = PolicyEngine::default();
    let bus = Arc::new(CapabilityBus::new(registry, policy));

    let tool_count = bus.registry().list().await.len();
    tracing::info!(tool_count, "Skills registered");

    server::run_server(bus, &socket_path).await
}

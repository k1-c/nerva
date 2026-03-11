mod handler;
mod protocol;
mod server;

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use nerva_core::config::NervaConfig;
use nerva_core::{CapabilityBus, PolicyEngine, ToolRegistry};

#[derive(Parser)]
#[command(name = "nervad", about = "Nerva AI Desktop Agent Daemon")]
struct Args {
    /// Path to the Unix socket
    #[arg(long)]
    socket: Option<PathBuf>,

    /// Path to config file
    #[arg(long, short)]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut config = match &args.config {
        Some(path) => NervaConfig::load(path)?,
        None => NervaConfig::load_or_default(),
    };

    // CLI args override config
    if let Some(log_level) = &args.log_level {
        config.daemon.log_level = log_level.clone();
    }
    if args.socket.is_some() {
        config.daemon.socket_path = args.socket;
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.daemon.log_level.parse().unwrap_or_default()),
        )
        .init();

    tracing::info!("Starting nervad");

    let registry = Arc::new(ToolRegistry::new());
    nerva_skills::register_all_skills_with_config(&registry, &config).await;

    // Load plugin skills from plugin directory
    if config.plugins.enabled {
        let skills_dir = config.plugins.skills_dir();
        let loaded = nerva_skills::plugin_loader::load_plugins(&skills_dir, &registry).await;
        if !loaded.is_empty() {
            tracing::info!(count = loaded.len(), "Plugin skills loaded");
        }
    }

    let policy = PolicyEngine::new(config.policy.clone());
    let bus = Arc::new(CapabilityBus::new(registry, policy));

    let tool_count = bus.registry().list().await.len();
    tracing::info!(tool_count, "Skills registered");

    let socket_path = config.socket_path();

    // Graceful shutdown on SIGTERM/SIGINT
    let shutdown = async {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler");
        let sigint = tokio::signal::ctrl_c();

        tokio::select! {
            _ = sigterm.recv() => tracing::info!("Received SIGTERM"),
            _ = sigint => tracing::info!("Received SIGINT"),
        }
    };

    tokio::select! {
        result = server::run_server(bus, &socket_path) => {
            result?;
        }
        _ = shutdown => {
            tracing::info!("Shutting down gracefully");
            // Clean up socket file
            let _ = tokio::fs::remove_file(&socket_path).await;
        }
    }

    Ok(())
}

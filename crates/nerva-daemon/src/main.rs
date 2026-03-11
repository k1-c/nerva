mod handler;
mod protocol;
mod server;

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use nerva_core::agent::AgentRuntime;
use nerva_core::config::NervaConfig;
use nerva_core::llm::{ClaudeBackend, OllamaBackend, OpenAiBackend};
use nerva_core::watcher::Suggestion;
use nerva_core::{CapabilityBus, PolicyEngine, ToolRegistry};
use tokio::sync::Mutex;

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

    // Initialize agent runtime if LLM is configured
    let agent = if config.llm.enabled {
        let provider = config.llm.provider.as_str();
        let model = &config.llm.model;

        let backend: Option<Box<dyn nerva_core::llm::LlmBackend>> = match provider {
            "claude" => {
                let api_key = config.llm.resolve_api_key().unwrap_or_else(|| {
                    tracing::warn!("No API key for Claude. Set api_key in [llm] or ANTHROPIC_API_KEY env var.");
                    String::new()
                });
                Some(Box::new(ClaudeBackend::new(api_key, model)))
            }
            "openai" => {
                let api_key = config.llm.resolve_api_key().unwrap_or_else(|| {
                    tracing::warn!("No API key for OpenAI. Set api_key in [llm] or OPENAI_API_KEY env var.");
                    String::new()
                });
                Some(Box::new(OpenAiBackend::new(api_key, model, config.llm.base_url.clone())))
            }
            "ollama" => {
                let base_url = config.llm.base_url.as_deref().unwrap_or("http://localhost:11434");
                Some(Box::new(OllamaBackend::new(base_url, model)))
            }
            other => {
                tracing::error!(provider = other, "Unknown LLM provider, agent disabled");
                None
            }
        };

        if let Some(backend) = backend {
            let runtime = AgentRuntime::new(backend, bus.clone());
            tracing::info!(provider, model = %config.llm.model, "Agent runtime enabled");
            Some(Arc::new(runtime))
        } else {
            None
        }
    } else {
        tracing::info!("Agent runtime disabled (configure [llm] to enable)");
        None
    };

    // Initialize watcher system
    let suggestions: Arc<Mutex<Vec<Suggestion>>> = Arc::new(Mutex::new(Vec::new()));

    if config.watchers.enabled {
        let mut manager = nerva_core::watcher::WatcherManager::new(64);
        nerva_skills::watchers::register_all_watchers(&mut manager);
        tracing::info!(count = manager.count(), "Watchers started");

        let suggestions_for_watcher = suggestions.clone();
        let notify_enabled = config.watchers.notify;

        tokio::spawn(async move {
            loop {
                match manager.next_suggestion().await {
                    Some(suggestion) => {
                        tracing::debug!(
                            source = %suggestion.source,
                            title = %suggestion.title,
                            "Suggestion received"
                        );

                        // Send desktop notification if enabled
                        if notify_enabled {
                            let _ = nerva_os::notification::send_notification(
                                &suggestion.title,
                                Some(suggestion.body.as_str()),
                            )
                            .await;
                        }

                        // Buffer the suggestion
                        let mut store = suggestions_for_watcher.lock().await;
                        store.push(suggestion);

                        // Keep buffer bounded
                        const MAX_SUGGESTIONS: usize = 100;
                        if store.len() > MAX_SUGGESTIONS {
                            let drain_count = store.len() - MAX_SUGGESTIONS;
                            store.drain(..drain_count);
                        }
                    }
                    None => {
                        tracing::info!("Watcher channel closed");
                        break;
                    }
                }
            }
        });
    } else {
        tracing::info!("Watchers disabled (configure [watchers] to enable)");
    }

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
        result = server::run_server(bus, agent, suggestions, &socket_path) => {
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

# Nerva

> **⚠️ Under Development — This is a personal R&D project. Not ready for production use.**

A resident AI desktop agent for Linux. An orchestrator that bridges OS, applications, screen context, and automation through natural language.

Target environment: **NixOS + Wayland (Hyprland)**

## What is Nerva?

Nerva aims to be an OS-level AI agent layer that understands the user's current context, orchestrates native capabilities, and delivers the shortest path to action.

- Launcher experience like Raycast / Alfred
- OS-level agent like Open Interpreter
- Designed to coexist with Wayland's security model

## Architecture

```
Launcher UI  →  Agent Runtime  →  Capability Bus  →  Skills  →  OS Integration
  (thin)         (LLM/planner)    (policy/audit)    (actions)   (DBus/Wayland/PW)
```

See [docs/architecture.md](docs/architecture.md) for details.

## Project Structure

```
crates/
├── nerva-core/       # CapabilityBus, ToolRegistry, PolicyEngine, Skill trait
├── nerva-os/         # OS integration (process, clipboard, screenshot, wayland)
├── nerva-skills/     # Built-in skill implementations + plugin loader
├── nerva-daemon/     # nervad — resident daemon
└── nerva-cli/        # nerva — CLI client
```

## Getting Started

### Prerequisites

- Rust 1.85+
- Linux (Wayland recommended)
- Optional: `grim` (screenshot), `wl-paste` (clipboard), `hyprctl` (window management), `notify-send` (notifications)

### Build & Test

```bash
cargo build
cargo test
```

### Run

```bash
# Terminal 1: Start the daemon
cargo run --bin nervad

# Terminal 2: Use the CLI
cargo run --bin nerva -- status
cargo run --bin nerva -- tools
cargo run --bin nerva -- exec launch_app --input '{"app": "firefox"}'
cargo run --bin nerva -- exec clipboard_read
cargo run --bin nerva -- exec list_windows
cargo run --bin nerva -- exec run_command_safe --input '{"cmd": "uname", "args": ["-a"]}'
cargo run --bin nerva -- log
```

## Documentation

| Document                                     | Description                                                     |
| -------------------------------------------- | --------------------------------------------------------------- |
| [Architecture](docs/architecture.md)         | 5-layer architecture, crate structure, protocol                 |
| [Roadmap](docs/roadmap.md)                   | Development roadmap and non-goals                               |
| [Design Decisions](docs/design-decisions.md) | Key decisions with rationale (Wayland, NixOS, risk tiers, etc.) |
| [NixOS Integration](docs/nix-integration.md) | NixOS module structure, systemd service, flake design           |
| [Development Guide](docs/development.md)     | VM setup, build instructions, testing strategy                  |

## License

MIT

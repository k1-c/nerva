# Nerva

A resident AI desktop agent for Linux. An orchestrator that bridges OS, applications, screen context, and automation through natural language.

> **Status:** Phase 1 — Foundation (daemon + CLI + base skills)

## What is Nerva?

Nerva aims to be an OS layer that understands the user's current context, orchestrates native capabilities, and delivers the shortest path to action.

- Launcher experience like Raycast / Alfred
- OS-level agent like Open Interpreter
- Designed to coexist with Wayland's security model

Target environment: **NixOS + Wayland (Hyprland / KDE Plasma)**

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
├── nerva-skills/     # Built-in skill implementations
├── nerva-daemon/     # nervad — resident daemon
└── nerva-cli/        # nerva — CLI client
```

## Getting Started

### Prerequisites

- Rust 1.85+
- Linux (Wayland recommended)
- Optional: `grim` (screenshot), `wl-paste` (clipboard), `hyprctl` (window list)

### Build

```bash
cargo build
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

## Available Skills

| Skill              | Risk    | Description                          |
| ------------------ | ------- | ------------------------------------ |
| `launch_app`       | Safe    | Launch a desktop application         |
| `list_windows`     | Safe    | List all open windows                |
| `capture_screen`   | Safe    | Take a screenshot                    |
| `clipboard_read`   | Safe    | Read clipboard contents              |
| `run_command_safe` | Caution | Execute a command from the allowlist |

## Communication Protocol

The daemon and client communicate over a Unix domain socket (`$XDG_RUNTIME_DIR/nerva/nervad.sock`) using a JSON-lines protocol.

```json
{"command": "execute", "tool_id": "launch_app", "input": {"app": "firefox"}}
{"command": "list_tools"}
{"command": "get_log", "count": 10}
{"command": "status"}
```

## Documentation

| Document                                     | Description                                                     |
| -------------------------------------------- | --------------------------------------------------------------- |
| [Architecture](docs/architecture.md)         | 5-layer architecture, crate structure, protocol                 |
| [Roadmap](docs/roadmap.md)                   | Phase 1-8 roadmap and non-goals                                 |
| [Design Decisions](docs/design-decisions.md) | Key decisions with rationale (Wayland, NixOS, risk tiers, etc.) |
| [NixOS Integration](docs/nix-integration.md) | NixOS module structure, systemd service, flake design           |
| [Development Guide](docs/development.md)     | VM setup, build instructions, testing strategy                  |

## License

MIT

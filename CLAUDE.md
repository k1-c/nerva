# CLAUDE.md

## Project Overview

Nerva is a Linux AI Desktop Agent Orchestrator — a Raycast-like resident AI agent targeting NixOS + Wayland + Hyprland. Written in Rust.

## Architecture

5-layer architecture: Shell/UI → Agent Runtime → Capability Bus → Skills/Adapters → OS Integration

Key principle: "Separate reasoning from execution" — LLM decides what, deterministic tools execute.

## Project Structure

```
nerva/
├── crates/
│   ├── nerva-core/       # CapabilityBus, ToolRegistry, PolicyEngine, Skill trait, types, config
│   ├── nerva-os/         # OS integration (process, clipboard, screenshot, wayland, notification)
│   ├── nerva-skills/     # Built-in skills (9) + plugin loader (script-based plugins)
│   ├── nerva-daemon/     # Unix socket daemon (bin: nervad)
│   └── nerva-cli/        # CLI client (bin: nerva)
├── dist/                 # systemd service, example config
├── docs/                 # Architecture, roadmap, design decisions, nix integration
└── .claude/skills/       # Claude Code skills
```

## Tech Stack

- Rust (edition 2024, MSRV 1.85), tokio, serde, async-trait, clap, tracing
- Unix domain socket with JSON-lines protocol
- External tools: grim, wl-paste/xclip, hyprctl, notify-send, xdg-open

## Build & Test

```sh
cargo build          # Build all crates
cargo test           # Run all tests (unit + integration + plugin)
cargo run --bin nervad  # Start daemon
cargo run --bin nerva   # CLI client
```

## Key Conventions

- Tools have risk tiers: Safe / Caution / Dangerous
- Skill trait: `fn metadata() -> &ToolMetadata` + `async fn execute(input: Value) -> Result<Value, NervaError>`
- Plugin skills: `~/.config/nerva/skills/<name>/skill.toml` + executable
- Config: `~/.config/nerva/config.toml` (TOML)
- License: MIT

## Commit Rules

**IMPORTANT: Always use the `conventional-commit` skill (located at `.claude/skills/conventional-commit/`) when creating commits.**

When asked to commit changes, invoke the skill via `/conventional-commit` instead of committing directly. This ensures:

- Commits follow the [Conventional Commits](https://www.conventionalcommits.org/) specification
- Changes spanning multiple concerns are split into atomic commits
- The user is presented with a commit plan for review before execution
- No auto-generated footers (Co-Authored-By, etc.) are appended

Do NOT use `git commit` directly. Always delegate to the conventional-commit skill.

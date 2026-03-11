# Nerva Architecture

## Vision

Nerva is a resident AI desktop agent for Linux. It functions as an orchestrator that bridges the local OS, applications, screen context, and automation through natural language. It is an evolution of Raycast and closely related to Open Interpreter-style "OS-facing agents."

The target environment is **NixOS + Wayland (Hyprland / KDE Plasma)**. To coexist with Wayland's security model, Nerva maximizes the use of official APIs and treats GUI automation as a last resort.

## Design Principles

1. **Separate reasoning from execution** — The LLM decides "what to do" and "which capability to use"; deterministic tools handle actual execution
2. **Observe richly, act conservatively** — Context acquisition is rich; actions are conservative
3. **Official API first** — Priority order: DBus > portals > CLI > screen OCR > input injection
4. **Risk-tiered execution** — Every tool carries a risk level; the policy engine decides whether execution is allowed

## 5-Layer Architecture

```
┌─────────────────────────────┐
│         Launcher UI         │  ← Layer 1: Shell / UI
│   palette / HUD / overlay   │
└─────────────┬───────────────┘
              │
┌─────────────▼───────────────┐
│        Agent Runtime        │  ← Layer 2: NL interpretation & task planning
│ intent parse / planner /    │
│ memory / context assembly   │
└─────────────┬───────────────┘
              │
┌─────────────▼───────────────┐
│       Capability Bus        │  ← Layer 3: Core orchestrator
│ registry / policy / logs /  │
│ cancel / confirm / audit    │
└──────┬──────────┬───────────┘
       │          │
 ┌─────▼────┐ ┌───▼────────┐
 │ Skills   │ │ Watchers   │   ← Layer 4: Operation units
 │ actions  │ │ triggers   │
 └─────┬────┘ └───┬────────┘
       │          │
 ┌─────▼──────────▼─────────┐
 │     OS Integration       │   ← Layer 5: Raw OS layer
 │ DBus / portal / PW /     │
 │ shell / uinput / wm api  │
 └──────────────────────────┘
```

### Layer 1: Shell / UI

The user's entry point. Kept thin — no intelligence lives here.

- Text input / candidate display
- Current context display
- Execution confirmation / cancellation
- Execution log view

Will be implemented with GTK4 or Tauri in the future.

### Layer 2: Agent Runtime

Natural language interpretation and task planning. Consists of three subcomponents:

- **Interpreter** — Converts natural language into structured intents
- **Planner** — Builds execution plans (tool call sequences) from intents
- **Orchestrator** — Executes plans sequentially, stopping or retrying on failure

> Not yet implemented. Will be built alongside LLM integration in Phase 3.

### Layer 3: Capability Bus

The heart of the system. A mediation layer that prevents the agent from touching the OS directly.

| Component | Role |
|---|---|
| **ToolRegistry** | Manages and looks up registered skills |
| **PolicyEngine** | Controls execution based on risk tiers |
| **ExecutionState** | Tracks running / pending / awaiting confirmation / failed / cancelled |
| **AuditLog** | Records all operations |

### Layer 4: Skills / Adapters

Individual operation units. Each skill implements the `Skill` trait:

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    fn metadata(&self) -> &ToolMetadata;
    async fn execute(&self, input: Value) -> Result<Value, NervaError>;
}
```

#### Risk Tiers

| Tier | Policy | Examples |
|---|---|---|
| **Safe** | Execute without confirmation | app launch, clipboard read, screenshot, window list |
| **Caution** | May require confirmation | shell command, browser navigation, clipboard write |
| **Dangerous** | Confirmation required by default | input injection, file delete, external send, sudo |

### Layer 5: OS Integration

The lowest layer: Wayland, DBus, systemd, PipeWire, portals, uinput. The Agent Runtime has no direct knowledge of this layer.

#### Operation Priority

1. **Official APIs** — DBus, CLI, app-specific API, portals, filesystem
2. **State observation** — window metadata, screen capture, OCR, VLM
3. **Last resort** — ydotool, uinput, compositor-specific fake input

## Crate Structure

```
nerva/
├── crates/
│   ├── nerva-core/       # CapabilityBus, ToolRegistry, PolicyEngine, Skill trait, types
│   ├── nerva-os/         # OS integration (process, clipboard, screenshot, wayland)
│   ├── nerva-skills/     # Built-in skill implementations
│   ├── nerva-daemon/     # nervad — Unix socket daemon (bin: nervad)
│   └── nerva-cli/        # CLI client (bin: nerva)
```

### Dependency Graph

```
nerva-cli ──────► nerva-core
                      ▲
nerva-daemon ─────────┤
       │              │
       ▼              │
nerva-skills ─────────┤
       │              │
       ▼              │
nerva-os ─────────────┘
```

## Communication Protocol

The daemon and client communicate over a Unix domain socket (`$XDG_RUNTIME_DIR/nerva/nervad.sock`) using a JSON-lines (newline-delimited JSON) protocol.

### Request

```json
{"command": "execute", "tool_id": "launch_app", "input": {"app": "firefox"}}
{"command": "list_tools"}
{"command": "get_log", "count": 10}
{"command": "status"}
```

### Response

```json
{"ok": true, "data": {...}}
{"ok": false, "error": "..."}
```

## Task Classification

| Type | Description | Example |
|---|---|---|
| **Intent** | One-shot user request | "Open Slack", "Summarize this screen" |
| **Watcher** | State monitoring & suggestions | URL in clipboard → offer to summarize |
| **Workflow** | Multi-step execution | Read issue → create branch → open editor |

## Memory Design

| Type | Scope | Storage |
|---|---|---|
| **Execution memory** | Short-term execution state (current task, previous step results) | Volatile |
| **Preference memory** | User preferences (frequent apps, confirmation overrides) | Persistent |

## Target Environment

| Layer | Technology |
|---|---|
| OS | NixOS (unstable) |
| Compositor | Hyprland / KDE Plasma |
| Audio/Video | PipeWire + WirePlumber |
| IPC | DBus (session bus) |
| Portals | xdg-desktop-portal-hyprland / xdg-desktop-portal-kde |
| Agent | Rust (tokio) + systemd user service |

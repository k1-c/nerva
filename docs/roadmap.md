# Nerva Roadmap

## Phase 1: Foundation (Current)

**Goal:** Build the intent → tool execution skeleton and verify it works with the daemon + CLI.

### Done

- [x] Cargo workspace + 5 crate structure
- [x] `nerva-core`: CapabilityBus, ToolRegistry, PolicyEngine, Skill trait
- [x] `nerva-os`: process, clipboard, screenshot, wayland integration stubs
- [x] `nerva-skills`: 9 built-in skills
  - `launch_app` (Safe)
  - `list_windows` (Safe)
  - `capture_screen` (Safe)
  - `clipboard_read` (Safe)
  - `run_command_safe` (Caution)
  - `notify` (Safe)
  - `open_path` (Safe)
  - `get_active_window` (Safe)
  - `focus_window` (Caution)
- [x] `nerva-daemon`: Unix socket server (nervad)
- [x] `nerva-cli`: CLI client (nerva)
- [x] JSON-lines protocol
- [x] PolicyEngine with risk tiers
- [x] Execution audit log
- [x] Configuration file support (TOML) — `~/.config/nerva/config.toml`
- [x] Graceful shutdown (SIGTERM/SIGINT signal handling)
- [x] systemd user service unit file (`dist/nervad.service`)
- [x] Integration tests (daemon + CLI end-to-end, 5 tests)
- [x] Unit tests (CapabilityBus, PolicyEngine, config, 9 tests)

### TODO

- [x] Dynamic skill registration / plugin directory
  - Script-based plugins: `skill.toml` manifest + executable
  - Plugin directory: `~/.config/nerva/skills/`
  - ID conflict detection (plugins cannot override built-in skills)
  - Configurable via `[plugins]` section in config

---

## Phase 2: Screen Context

**Goal:** Understand screen state and enable context-aware operations.

- [x] `get_active_window` skill via compositor API (hyprctl)
- [x] Improved `capture_screen` — portal (xdg-desktop-portal) with grim fallback, region/window capture
- [x] OCR integration — `ocr_screen` skill via tesseract
- [x] VLM integration — Ollama HTTP client with vision model support (moondream, llava, etc.)
- [x] `summarize_screen` skill — capture → VLM (with OCR fallback)
- [x] Context assembly — `DesktopContext` struct + `gather_context` skill (active window + clipboard + screen text)

---

## Phase 3: Agent Runtime (LLM Integration)

**Goal:** Automate the natural language → tool invocation pipeline with an LLM.

- [ ] Interpreter — natural language → structured intent
- [ ] Planner — intent → tool execution plan
- [ ] Orchestrator — sequential plan execution with error handling
- [ ] LLM backend abstraction (local Ollama / cloud API)
- [ ] Confirmation flow — interactive confirmation for Dangerous tier operations
- [ ] Execution memory — short-term memory during task execution

---

## Phase 4: Watchers & Suggestions

**Goal:** React to state changes and provide suggestions as a resident agent.

- [ ] Watcher framework — common event monitoring infrastructure
- [ ] Clipboard watcher — URL → offer summary, code → offer explanation
- [ ] Active window watcher — suggest relevant actions on app switch
- [ ] Suggestion mode — display as suggestions, not auto-execute
- [ ] Notification integration — show suggestions as desktop notifications

---

## Phase 5: Workflow Automation

**Goal:** Safely execute multi-step workflows.

- [ ] Workflow definition format
- [ ] Browser control via Chrome DevTools Protocol
- [ ] Editor integration — VS Code / Neovim
- [ ] Extended safe shell — dynamic allowlist management
- [ ] Limited input automation via ydotool / uinput
- [ ] Killer workflows
  - "Read the error on screen and suggest causes + next command"
  - "Summarize the current article/PDF in 3 lines and save to notes"
  - "Pre-meeting setup" (check calendar → show notes → copy meeting URL)

---

## Phase 6: Launcher UI

**Goal:** Build a graphical launcher UI.

- [ ] Command palette UI with GTK4 / Tauri
- [ ] Global shortcut (XDG Desktop Portal GlobalShortcuts)
- [ ] Candidate display / incremental search
- [ ] Execution confirmation dialog
- [ ] Execution log viewer
- [ ] Overlay mode

---

## Phase 7: Preference Memory & Personalization

**Goal:** Learn user preferences and optimize operations.

- [ ] Preference store — frequent apps, preferred operations, confirmation overrides
- [ ] Usage analytics — track skill usage frequency
- [ ] Smart defaults — optimize candidate ordering based on frequency

---

## Phase 8: NixOS Module & Distribution

**Goal:** Package as a distributable NixOS module.

- [ ] Nix flake
- [ ] NixOS module (`services.nerva.enable = true`)
- [ ] Home Manager module (keybindings, watcher rules, agent config)
- [ ] NixOS VM test (`testScript` to verify agentd startup)
- [ ] CI/CD pipeline

---

## Non-Goals (for now)

The following are intentionally deferred:

- Fully autonomous agent (executing anything without human confirmation)
- Arbitrary bash execution
- Dependence on click-coordinate-based automation
- Elaborate memory / RAG systems
- Complex multi-LLM router
- Building a custom compositor

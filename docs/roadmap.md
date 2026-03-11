# Nerva Roadmap

## Phase 1: Foundation (Current)

**Goal:** Build the intent → tool execution skeleton and verify it works with the daemon + CLI.

### Done

- [x] Cargo workspace + 5 crate structure
- [x] `nerva-core`: CapabilityBus, ToolRegistry, PolicyEngine, Skill trait
- [x] `nerva-os`: process, clipboard, screenshot, wayland integration stubs
- [x] `nerva-skills`: 5 MVP skills
  - `launch_app` (Safe)
  - `list_windows` (Safe)
  - `capture_screen` (Safe)
  - `clipboard_read` (Safe)
  - `run_command_safe` (Caution)
- [x] `nerva-daemon`: Unix socket server (nervad)
- [x] `nerva-cli`: CLI client (nerva)
- [x] JSON-lines protocol
- [x] PolicyEngine with risk tiers
- [x] Execution audit log

### TODO

- [ ] Integration tests (daemon + CLI end-to-end)
- [ ] Configuration file support (TOML)
- [ ] Graceful shutdown (signal handling)
- [ ] systemd user service unit file
- [ ] Additional safe skills
  - `notify` — send desktop notifications
  - `open_path` — open a file or URL
  - `get_active_window` — get current active window info
  - `focus_window` — focus a specific window
- [ ] Dynamic skill registration / plugin directory

---

## Phase 2: Screen Context

**Goal:** Understand screen state and enable context-aware operations.

- [ ] `get_active_window` skill via compositor API
- [ ] Improved `capture_screen` via PipeWire / portal for secure capture
- [ ] OCR integration — extract text from screenshots with tesseract / PaddleOCR
- [ ] VLM integration — interpret screen images with a local VLM
- [ ] `summarize_screen` skill — capture → OCR/VLM → summarize
- [ ] Context assembly — combine active window + clipboard + screen text

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

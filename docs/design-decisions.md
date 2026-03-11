# Design Decisions

This document captures key design decisions and their rationale. Much of this context originated from early architectural discussions.

## Why Wayland (not X11)?

X11 allows unrestricted access to other applications' input, window state, and screen content — making AI agent development trivially easy. However:

- X11 is in end-of-life mode. GNOME, KDE, Ubuntu, and Fedora are all migrating to Wayland.
- X11's openness is a security liability (keyloggers, unauthorized screen capture, arbitrary click injection).
- Building on X11 means building on a dying platform.

**Decision:** Design for Wayland from the start. Accept the constraints and work within them.

## Why Hyprland / KDE over GNOME?

For an AI desktop agent, the compositor must support:

| Capability | GNOME | KDE | Hyprland |
|---|---|---|---|
| Custom automation | Limited | Good | Excellent |
| Window control API | Limited | Rich | Rich |
| Input injection support | Blocked | Supported | Supported |
| Wayland maturity | Excellent | Excellent | Good |
| Developer extensibility | Limited | Good | Excellent |

GNOME intentionally restricts the escape hatches that AI agents need. KDE and Hyprland provide better APIs for window control, input automation, and event hooks.

**Decision:** Target Hyprland (primary) and KDE Plasma (secondary). GNOME is validation-only.

## Why NixOS?

The agent integrates deeply with OS services: systemd, DBus, PipeWire, compositor config, portal permissions, uinput groups. On traditional distros, this state accumulates implicitly and is fragile.

NixOS makes all of this declarative and reproducible:
- Agent daemon = systemd user service in Nix config
- Compositor + keybindings + permissions = version-controlled
- Dev environment = `nix develop`
- Distribution = `imports = [ nerva.module ]`

**Decision:** NixOS as the primary target. The entire AI OS layer is designed to be a Nix module.

## Why Separate Reasoning from Execution?

A common anti-pattern in AI agents:

- LLM generates shell commands → executed directly
- LLM outputs click coordinates → injected immediately
- LLM manages permissions, state, and execution in one pass

This works for demos but fails for a resident OS layer because:
- No audit trail
- No permission enforcement
- No cancellation or retry logic
- Single point of failure

**Decision:** The LLM decides *what* to do (intent → plan). Deterministic tools handle *how* to do it (execute via Capability Bus). The bus validates every action against the policy engine.

## Why Risk Tiers?

Not all operations are equal. Listing windows is harmless; deleting files is destructive.

**Decision:** Every tool declares a risk tier in its metadata:

| Tier | Behavior | Examples |
|---|---|---|
| Safe | Auto-approve | app launch, clipboard read, screenshot |
| Caution | May require confirmation | shell command, browser nav, clipboard write |
| Dangerous | Always require confirmation | input injection, file delete, external post, sudo |

The PolicyEngine evaluates risk at execution time, not at registration time.

## Why Official API First?

Wayland isolates applications from each other. Screen scraping and input injection fight this model.

**Decision:** Operation priority order:

1. **Official APIs** — DBus, CLI tools, app-specific APIs, XDG portals, filesystem
2. **State observation** — Window metadata, screen capture, OCR, VLM
3. **Last resort** — ydotool, uinput, compositor-specific fake input

This is summarized as: **observe richly, act conservatively**.

## Policy Engine Examples

Concrete policy rules for reference:

| Rule | Tier | Behavior |
|---|---|---|
| Shell execution | Caution | Only commands in the allowlist |
| Input injection | Dangerous | Explicit user confirmation required |
| File deletion | Dangerous | Blocked by default |
| Browser form submission | Dangerous | Dry-run preview before execution |
| Screenshot | Safe | Auto-approved |
| Clipboard read | Safe | Auto-approved |

## Why Not a Monolith?

The agent could be a single process. However:

- Different layers have different stability requirements (UI can crash; daemon should not)
- The daemon must outlive the UI
- Skills should be independently testable
- The bus must be auditable independently of the LLM

**Decision:** Separate daemon (`nervad`) from UI and CLI. Communicate via Unix socket with a JSON-lines protocol. Skills are registered at daemon startup.

## Watcher Design: Suggestion Mode First

Watchers monitor system events (clipboard changes, window focus, etc.) and could auto-execute actions. However, autonomous execution is dangerous and annoying.

**Decision:** Watchers start in **suggestion mode** — they propose actions but do not execute them. Examples:

- URL appears in clipboard → "Summarize this page?"
- VS Code becomes foreground → "Open related repo?"
- GitHub issue detected in browser → "Create branch from issue?"

Auto-execution can be unlocked per-rule once trust is established through preference memory.

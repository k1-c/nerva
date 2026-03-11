# Development Guide

## Recommended Setup

Since this project touches OS-level concerns (Wayland, compositor, input injection, systemd, DBus, PipeWire), developing in a sandboxed environment is strongly recommended.

### Phase 1: VM Development (Start Here)

The most practical starting point. Develop everything inside a VM.

```
Host OS (e.g. Ubuntu)
   ↓
VM (NixOS)
   ↓
Wayland / Hyprland / KDE
   ↓
AI agent development
```

#### Why VM?

- **Snapshot rollback** — revert to a known good state when things break
- **Compositor experiments** — freely try Hyprland / KDE / Sway
- **Input injection testing** — ydotool, uinput, fake input are fragile; VM keeps it safe
- **NixOS rebuild freely** — `nixos-rebuild switch` failures are trivially recoverable

#### Recommended VM Software

| VM | Notes |
|---|---|
| **virt-manager + KVM** | Best for Linux hosts |
| VMware | Stable GPU passthrough |
| VirtualBox | Easy but limited |

#### Minimum VM Specs

| Resource | Minimum | With local LLM |
|---|---|---|
| CPU | 6 cores | 6 cores |
| RAM | 8 GB | 16 GB |
| Disk | 60 GB | 60 GB |
| GPU | virtio | GPU passthrough preferred |

#### VM Limitation

GPU acceleration is limited in VMs, which affects:
- PipeWire screen capture performance
- VLM inference speed
- Local LLM inference speed

This is acceptable for Phase 1 and Phase 2. GPU-dependent features should be validated on bare metal in Phase 3.

### Phase 2: Host Linux Testing

Once the agent is functional in the VM:

1. Package the project as a Nix flake
2. Install on the host Linux via `nix profile install` or `nix run`
3. Validate on the host desktop environment

### Phase 3: Bare Metal Testing

For GPU-dependent features (PipeWire capture, VLM, local LLM), test on real hardware:
- Screen capture performance
- VLM inference latency
- End-to-end workflow timing

## NixOS VM Testing

NixOS has built-in VM test infrastructure:

```nix
# Build a development VM
nix build .#nixosConfigurations.dev.vm
```

Integration tests can verify OS-level behavior:

```nix
testScript = ''
  machine.wait_for_unit("agentd.service")
  machine.succeed("nerva status")
'';
```

This enables **OS-level CI** — automated testing of daemon startup, skill execution, and systemd integration.

### Advanced: Dual VM Setup

For mature development:

```
dev VM    → manual development and experimentation
test VM   → automated CI tests for agent workflows
```

## Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run the daemon
cargo run --bin nervad

# Run the CLI
cargo run --bin nerva -- status
```

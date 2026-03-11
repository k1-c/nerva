# NixOS Integration Design

## Why NixOS?

NixOS is not just a package manager — it allows the entire OS state to be codified. For an AI desktop agent that touches systemd, DBus, PipeWire, compositor, portals, and uinput, this is a significant advantage.

### Key Benefits

1. **Reproducible OS state** — All system services, compositor config, agent daemon, and permissions are declarative and version-controlled
2. **Agent as a systemd user service** — Natively managed with restart, logging, and dependency ordering
3. **Declarative compositor management** — Hyprland/KDE config, keybindings, and agent integration all live in Git
4. **Reproducible dev environment** — `nix develop` provides identical toolchains across machines
5. **Distribution-ready** — The entire AI OS layer can be packaged as a Nix module that users import with a single line

## Three-Layer Nix Structure

### System Layer (NixOS Module)

OS-level configuration managed via NixOS modules:

```nix
# nixosModules/default.nix
{
  services.pipewire.enable = true;
  services.dbus.enable = true;
  programs.hyprland.enable = true;

  # uinput group for input injection
  users.groups.uinput = {};

  # Portal for screen capture
  xdg.portal.enable = true;

  # Fonts, system packages, etc.
}
```

### User Layer (Home Manager)

Per-user configuration managed via Home Manager:

```nix
# homeModules/default.nix
{
  # Launcher keybindings
  # Agent configuration (API keys path, notes path)
  # Watcher rules
  # Per-user secrets path
}
```

### App Layer (Flake Packages)

The agent binaries and tools as Nix packages:

```nix
# In flake.nix packages
{
  nervad = ...;      # Agent daemon
  nerva = ...;       # CLI client
  nerva-ui = ...;    # Launcher UI (future)
}
```

## Agent as systemd User Service

```nix
systemd.user.services.nervad = {
  description = "Nerva AI Desktop Agent";

  serviceConfig = {
    ExecStart = "${pkgs.nerva}/bin/nervad";
    Restart = "always";
    RestartSec = 3;
  };

  wantedBy = [ "default.target" ];
};
```

Logs via: `journalctl --user -u nervad`

## Declarative Compositor Integration

Example with Hyprland:

```nix
wayland.windowManager.hyprland = {
  enable = true;
  settings = {
    bind = [
      "SUPER, SPACE, exec, nerva-ui"  # Launch agent UI
    ];
  };
};
```

Everything — compositor, keybindings, agent, permissions — is Git-managed.

## Dev Shell

```nix
devShells.default = pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    grim        # screenshot
    wl-clipboard
    ydotool     # input injection
    ollama      # local LLM
  ];
};
```

New machine setup: `nix develop` — done.

## Future: Distribution

The end goal is that users can enable the full AI OS layer with:

```nix
imports = [ nerva.nixosModules.default ];

services.nerva.enable = true;
```

This requires the three-layer structure above to be stable and well-separated.

## Planned Nix File Structure

```
nerva/
├── flake.nix
├── flake.lock
├── nixosModules/
│   ├── default.nix       # Main module entry point
│   ├── nervad.nix        # Daemon service definition
│   ├── ui.nix            # Launcher UI integration
│   └── permissions.nix   # uinput, portal, DBus permissions
├── homeModules/
│   ├── default.nix       # Main home-manager entry point
│   ├── keybindings.nix   # Global shortcuts
│   └── watchers.nix      # Watcher rule configuration
├── crates/               # Rust workspace (existing)
└── docs/
```

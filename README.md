# Focusd 🎯
> A minimal, privacy-respecting screen time tracker for Linux (Hyprland & X11).
> Built in Rust, with an Iced-based dashboard.

![Screenshot](screenshot_placeholder.png)

## Features
- **Privacy First**: Data stored locally in SQLite (`~/.local/share/focusd/`).
- **Compositor Native**: First-class support for **Hyprland** (JSON IPC) and X11.
- **Visual Dashboard**: Modern, dark-themed GUI for Today/Week trends.

## Installation
You can build from source using the provided installer:

```bash
git clone https://github.com/revanthlol/focusd.git
cd focusd/focusd_installer/
chmod +x install.sh
./install.sh
```

## Configuration
Edit `~/.config/focusd/config.toml` to map ugly app IDs to human names:

```toml
[alias]
"code" = "VS Code"
"firefox" = "Browser"
"com.mitchellh.ghostty" = "Ghostty"
```

## Usage

```
Binary will be installed to:
```bash
~/.cargo/bin/focusd
```
Make sure it’s in your PATH.

## ▶ Usage
```bash
focusd [command]
```
```bash
Commands:
  daemon  
  today   
  week    
  export  
  listen  
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
- **GUI**: Run `focusd-dashboard` (or find **Focusd** in your app menu).
---

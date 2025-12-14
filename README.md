# Focusd 
> A minimal, lightweight screen time tracker for Linux (Hyprland & X11).
> Built in Rust, with an Iced-based dashboard.

![Screenshot](screenshot_placeholder.png)

## Features
- **Storage**: Data stored locally in SQLite (`~/.local/share/focusd/`).
- **Compositor Native**: Support for **Hyprland** (JSON IPC) and X11.
- **Visual Dashboard**: Modern, dark-themed GUI .

## Installation
You can install from source using the provided installer:

```bash
git clone https://github.com/revanthlol/focusd.git
cd focusd/focusd_installer/
chmod +x install.sh
./install.sh
```
**Check out Releases for better installation options.**
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

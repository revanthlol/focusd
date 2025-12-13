# focusd

`focusd` is a lightweight, daemon-style CLI tool for Linux that tracks
**focused window time** to help you understand where your attention goes.

It works across different desktop environments by observing
the currently active window and recording usage locally.

---

## ✨ Features

- ⏱ Tracks time spent on the currently focused window
- 🖥 Works across DEs (X11 / Wayland where supported)
- 🧠 No cloud, no telemetry — data stays local
- 📊 Stores data in a local SQLite database
- ⚡ Minimal resource usage
- 🧩 Designed as a daemon + CLI interface

---


## 🚀 Installation
```bash
- Extract: tar -xzvf focusd_setup.tar.gz
- Enter: cd focusd_installer
- Run: ./install.sh
```
## 🛠 Requirements
### System dependencies
- `libxcb`
- `sqlite3`

#### Arch Linux
```bash
sudo pacman -S libxcb sqlite
```
#### Ubuntu / Debian
```bash
sudo apt install libxcb1 libsqlite3-0
```

From source 
```bash
git clone https://github.com/revanthlol/focusd.git
cd focusd
cargo install --path .
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
## ⚙ Configuration
Config file location:
```bash
~/.config/focusd/config.toml
```
## 🧪 Development
```bash
cargo run
cargo test
```

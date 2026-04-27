# Focusd

> A minimal, privacy-respecting screen time tracker for Linux.
> Built in Rust. Local-only. No telemetry.

<p align="center">
  <img src="assets/focusd.svg" width="80" />
</p>

## Features

- **Local-first** — All data stored in SQLite (`~/.local/share/focusd/`). Nothing leaves your machine.
- **Compositor native** — Hyprland (JSON IPC) and X11 (`_NET_ACTIVE_WINDOW`) support.
- **Idle-aware** — Respects system idle state via `loginctl`. Configurable timeout.
- **Dashboard** — Modern Tauri-based GUI with today/week/month views, hourly timeline, calendar heatmap, and trend comparisons.
- **Themed** — Dynamic theming via [matugen](https://github.com/InioX/matugen) (Material You from your wallpaper) or custom hex colors. Falls back to a clean dark theme.
- **CLI-first** — Beautiful terminal reports with Unicode bars, sparklines, percentages, and `--json` output.
- **Fast** — Sub-second daemon polling. WAL-mode SQLite. Minimal resource usage.

## Screenshots

<!-- Add your screenshots here -->
<!-- ![Today View](assets/screenshot-today.png) -->
<!-- ![Week View](assets/screenshot-week.png) -->

## Installation

### From source

```bash
git clone https://github.com/revanthlol/focusd.git
cd focusd

# Build CLI
cargo build --release -p focusd_cli
cp target/release/focusd ~/.cargo/bin/

# Build GUI (requires Node.js + npm)
cd gui
npm install
npm run tauri build
```

### Using the installer

```bash
cd focusd/focusd_installer/
chmod +x install.sh
./install.sh
```

> Check [Releases](https://github.com/revanthlol/focusd/releases) for pre-built binaries.

## Usage

### Daemon

Start the background tracker:

```bash
focusd daemon
```

> Add to your Hyprland/i3/sway config for autostart:
> ```
> exec-once = focusd daemon
> ```

### CLI Commands

```
focusd <command>

Commands:
  daemon              Start background tracking
  today               Today's usage report
  week                This week's report
  month               This month's report
  range               Custom date range (--from, --to)
  top                 Top apps (--days, --limit)
  stats               All-time statistics
  watch               Live terminal dashboard
  export              Export data (--format json|csv)
  doctor              System diagnostics
  config              Config management (path|edit|init)
  listen              Debug: print active window
  help                Print help

Flags:
  --json              Output as JSON (today, week, month, top, range)
  --theme <name>      Override theme (dark, matugen, custom, none)
  --verbose           Verbose daemon logging
```

### GUI

```bash
focusd-dashboard
```

Or find **Focusd** in your application menu.

### Example output

```
focusd today

  Active now  : Firefox

  Today — 4h 20m
  ↑ 1h 54m vs yesterday

  Last 7d  ▁▃▅▇█▅▃

  Firefox          ████████████████▌░░░░░░░  1h 49m   42%
  kitty            █████░░░░░░░░░░░░░░░░░░░    40m   15%
  VS Code          ████░░░░░░░░░░░░░░░░░░░░    37m   14%
  Telegram         ████░░░░░░░░░░░░░░░░░░░░    37m   14%
  Discord          ███░░░░░░░░░░░░░░░░░░░░░    26m   10%
```

## Configuration

Config file: `~/.config/focusd/config.toml`

```toml
# Polling interval in seconds
interval = 1

# Seconds of inactivity before pausing tracking
idle_timeout = 300

# Theme: "dark" | "matugen" | "custom"
theme = "dark"

# Simple alias mapping
[alias]
"code" = "VS Code"
"firefox" = "Firefox"
"com.mitchellh.ghostty" = "Ghostty"
"vesktop" = "Discord"
"org.telegram.desktop" = "Telegram"

# Advanced per-app config (takes priority over [alias])
[apps.code]
name = "VS Code"
category = "Development"

[apps.firefox]
name = "Firefox"
category = "Browser"

# Custom theme colors (only when theme = "custom")
[theme_colors]
primary = "#5eead4"
accent = "#a78bfa"
background = "#0a0a0f"
```

### Matugen integration

For automatic wallpaper-based theming:

1. Install [matugen](https://github.com/InioX/matugen)
2. Set `theme = "matugen"` in config
3. Run matugen — focusd picks up colors from `~/.cache/matugen/colors.json`

## Architecture

```
focusd/
├── core/           Shared library — DB, config, theme
├── cli/            CLI binary — daemon, reports, watch
├── gui/            Tauri + React dashboard
│   ├── src/        React frontend (shadcn/ui + recharts)
│   └── src-tauri/  Tauri backend
└── assets/         Icons, desktop file
```

- **Data**: SQLite with WAL mode, daily + hourly aggregation
- **Hyprland**: `hyprctl activewindow -j`
- **X11**: `x11rb` crate via `_NET_ACTIVE_WINDOW` + `WM_CLASS`
- **Idle**: `loginctl show-session` idle hint

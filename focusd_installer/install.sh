#!/bin/bash
set -e

# Configuration
APP_CLI="focusd"
APP_GUI_PKG="focusd_gui"     # Cargo package name
APP_GUI_BIN="focusd-dashboard" # System binary name

INSTALL_BIN="$HOME/.local/bin"
INSTALL_SHARE="$HOME/.local/share"
CONFIG_DIR="$HOME/.config/$APP_CLI"
SERVICE_DIR="$HOME/.config/systemd/user"
APPS_DIR="$HOME/.local/share/applications"
ICONS_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

clear
echo -e "${BLUE}=== Focusd Installer ===${NC}"
echo "Select installation mode:"
echo "1) CLI Only (Daemon + Terminal Tool)"
echo "2) Full Desktop Experience (CLI + GUI Dashboard + Icons)"
read -p "Enter choice [1/2]: " CHOICE

if [[ "$CHOICE" != "1" && "$CHOICE" != "2" ]]; then
    echo "Invalid choice. Exiting."
    exit 1
fi

echo ""
echo -e "${GREEN}[1/5] Compiling source...${NC}"

# --- Step 1: Build & Stop Service ---
# Always build CLI
cargo build --release -p focusd_cli

if [ "$CHOICE" == "2" ]; then
    cargo build --release -p focusd_gui
fi

echo -e "${GREEN}[2/5] Stopping existing background service...${NC}"
systemctl --user stop $APP_CLI.service 2>/dev/null || true


# --- Step 2: Install CLI (Common) ---
echo -e "${GREEN}[3/5] Installing Core binaries...${NC}"
mkdir -p "$INSTALL_BIN"
cp "target/release/$APP_CLI" "$INSTALL_BIN/$APP_CLI"
chmod +x "$INSTALL_BIN/$APP_CLI"


# --- Step 3: Install GUI (Conditional) ---
if [ "$CHOICE" == "2" ]; then
    echo -e "${GREEN}[3.5/5] Installing Dashboard & Assets...${NC}"
    
    # 1. Install GUI Binary
    cp "target/release/focusd_gui" "$INSTALL_BIN/$APP_GUI_BIN"
    chmod +x "$INSTALL_BIN/$APP_GUI_BIN"

    # 2. Install Assets
    if [ -d "assets" ]; then
        mkdir -p "$ICONS_DIR"
        mkdir -p "$APPS_DIR"
        
        # Icon
        cp "assets/focusd.svg" "$ICONS_DIR/focusd.svg" 2>/dev/null || echo -e "${YELLOW}Warning: Icon not found${NC}"
        
        # Desktop Entry
        cp "assets/focusd.desktop" "$APPS_DIR/focusd.desktop" 2>/dev/null || echo -e "${YELLOW}Warning: .desktop file not found${NC}"
        
        # Notify OS
        update-desktop-database "$APPS_DIR" 2>/dev/null || true
    else
        echo -e "${YELLOW}Warning: './assets' directory missing. Run setup_assets.sh first!${NC}"
    fi
else
    # Cleanup if downgrading to CLI only
    rm -f "$INSTALL_BIN/$APP_GUI_BIN" 2>/dev/null
    rm -f "$APPS_DIR/focusd.desktop" 2>/dev/null
fi


# --- Step 4: Configuration & Service (Common) ---
echo -e "${GREEN}[4/5] Checking configuration...${NC}"
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cat <<EOF > "$CONFIG_DIR/config.toml"
interval = 1
[alias]
"code" = "VS Code"
"google-chrome" = "Chrome"
"kitty" = "Terminal"
EOF
fi

echo -e "${GREEN}[5/5] Configuring Daemon Service...${NC}"
mkdir -p "$SERVICE_DIR"
cat <<EOF > "$SERVICE_DIR/$APP_CLI.service"
[Unit]
Description=Focusd Screen Tracker
After=graphical-session.target

[Service]
ExecStart=$INSTALL_BIN/$APP_CLI daemon
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

# --- Step 5: Start ---
systemctl --user daemon-reload
systemctl --user enable $APP_CLI.service
systemctl --user restart $APP_CLI.service

echo ""
echo -e "${BLUE}=== Installation Complete! ===${NC}"
if [ "$CHOICE" == "2" ]; then
    echo "You can launch 'Focusd Dashboard' from your app menu."
fi
echo "The tracking daemon is running in the background."
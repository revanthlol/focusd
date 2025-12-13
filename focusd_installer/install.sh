#!/bin/bash
set -e

APP_NAME="focusd"
BIN_SOURCE="./bin/$APP_NAME"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/$APP_NAME"
SERVICE_DIR="$HOME/.config/systemd/user"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== Installing $APP_NAME (Binary Mode) ===${NC}"

# 1. Check if the binary exists in the package
if [ ! -f "$BIN_SOURCE" ]; then
    echo -e "${RED}Error: Binary not found in ./bin/${NC}"
    echo "Make sure you extracted the full zip archive."
    exit 1
fi

# 2. Basic Dependency Check
# We built with bundled SQLite, so we mainly check for X11 libs if X11 fallback is used.
if ! ldconfig -p | grep -q libxcb; then
    echo -e "${RED}Warning: libxcb not found.${NC} X11 backend might fail."
    echo "If you are on pure Hyprland, this is fine."
fi

# 3. Install Binary
echo "[-] Installing binary to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"

# Stop service if running to release file lock
systemctl --user stop $APP_NAME.service 2>/dev/null || true

cp "$BIN_SOURCE" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$APP_NAME"

# 4. Create Config
echo "[-] Checking configuration..."
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cat <<EOF > "$CONFIG_DIR/config.toml"
interval = 1
[alias]
"code" = "VS Code"
"google-chrome" = "Chrome"
"kitty" = "Terminal"
"vesktop" = "Discord"
"firefox" = "Firefox"
"spotify" = "Spotify"
EOF
    echo -e "${GREEN}    Created default config at $CONFIG_DIR/config.toml${NC}"
else
    echo "    Config exists, skipping."
fi

# 5. Setup Service
echo "[-] configuring background service..."
mkdir -p "$SERVICE_DIR"
cat <<EOF > "$SERVICE_DIR/$APP_NAME.service"
[Unit]
Description=Focusd Screen Time Tracker
After=graphical-session.target

[Service]
ExecStart=$INSTALL_DIR/$APP_NAME daemon
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

# 6. Start
systemctl --user daemon-reload
systemctl --user enable $APP_NAME.service
systemctl --user restart $APP_NAME.service

echo -e "${GREEN}=== Success! ===${NC}"
echo "Run '$APP_NAME today' to see stats."

# PATH check
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo -e "${RED}Note: $HOME/.local/bin is not in your PATH.${NC}"
    echo "You might need to restart your terminal or add it to .bashrc/.zshrc"
fi
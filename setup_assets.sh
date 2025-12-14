#!/bin/bash
set -e

# Create assets folder
mkdir -p assets

echo "Creating Focusd Icon..."
cat <<EOF > assets/focusd.svg
<svg width="64" height="64" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
  <circle cx="32" cy="32" r="30" fill="#1a202c" />
  <circle cx="32" cy="32" r="25" fill="none" stroke="#00b4d8" stroke-width="4" />
  <circle cx="32" cy="32" r="8" fill="#00b4d8" />
  <rect x="28" y="5" width="8" height="8" fill="#cbd5e0" rx="2" />
</svg>
EOF

echo "Creating Focusd .desktop file..."
# Note: Exec path will be handled by the installer or system resolution
cat <<EOF > assets/focusd.desktop
[Desktop Entry]
Name=Focusd Dashboard
Comment=Privacy Respecting Screen Time Tracker
Exec=focusd-dashboard
Icon=focusd
Terminal=false
Type=Application
Categories=Utility;
Keywords=Time;Tracker;Rust;
StartupWMClass=focusd
EOF

echo "✅ Assets created in ./assets/"
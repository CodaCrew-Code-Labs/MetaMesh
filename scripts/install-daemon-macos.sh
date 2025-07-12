#!/bin/bash

# MetaMesh Daemon Installation Script for macOS
set -e

DAEMON_NAME="metamesh-daemon"
INSTALL_DIR="/usr/local/bin"
PLIST_DIR="$HOME/Library/LaunchAgents"
LOG_DIR="$HOME/Library/Logs/metamesh"

# Build the daemon
echo "Building MetaMesh daemon..."
cargo build --release --bin $DAEMON_NAME

# Create log directory
mkdir -p $LOG_DIR

# Install binary
echo "Installing daemon binary..."
sudo cp target/release/$DAEMON_NAME $INSTALL_DIR/
sudo chmod +x $INSTALL_DIR/$DAEMON_NAME

# Create LaunchAgent plist
echo "Creating LaunchAgent..."
mkdir -p $PLIST_DIR
cat > $PLIST_DIR/com.metamesh.daemon.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.metamesh.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/$DAEMON_NAME</string>
        <string>--port</string>
        <string>50051</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/daemon.error.log</string>
</dict>
</plist>
EOF

# Load the service
launchctl load $PLIST_DIR/com.metamesh.daemon.plist

echo "Installation complete!"
echo "Service is now running on port 50051"
echo "Check status with: launchctl list | grep metamesh"
echo "View logs at: $LOG_DIR/"
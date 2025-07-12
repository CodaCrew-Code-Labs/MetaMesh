#!/bin/bash

# MetaMesh Daemon Installation Script
set -e

DAEMON_NAME="metamesh-daemon"
INSTALL_DIR="/usr/local/bin"
SERVICE_DIR="/etc/systemd/system"
LOG_DIR="/var/log/metamesh"

# Build the daemon
echo "Building MetaMesh daemon..."
cargo build --release --bin $DAEMON_NAME

# Create log directory
sudo mkdir -p $LOG_DIR
sudo chmod 755 $LOG_DIR

# Install binary
echo "Installing daemon binary..."
sudo cp target/release/$DAEMON_NAME $INSTALL_DIR/
sudo chmod +x $INSTALL_DIR/$DAEMON_NAME

# Create systemd service file
echo "Creating systemd service..."
sudo tee $SERVICE_DIR/metamesh-daemon.service > /dev/null <<EOF
[Unit]
Description=MetaMesh Daemon Service
After=network.target

[Service]
Type=simple
User=nobody
Group=daemon
ExecStart=$INSTALL_DIR/$DAEMON_NAME --port 50051
Restart=always
RestartSec=10
StandardOutput=append:$LOG_DIR/daemon.log
StandardError=append:$LOG_DIR/daemon.error.log

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable service
sudo systemctl daemon-reload
sudo systemctl enable metamesh-daemon.service

echo "Installation complete!"
echo "Start the service with: sudo systemctl start metamesh-daemon"
echo "Check status with: sudo systemctl status metamesh-daemon"
echo "View logs with: sudo journalctl -u metamesh-daemon -f"
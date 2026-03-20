# NEXUS Server — VPS Deployment Guide

## Prerequisites

- Linux VPS (Ubuntu 22+ recommended)
- SSH access
- At least 2 GB RAM, 10 GB disk

## Setup (one time)

```bash
# Clone the workspace
git clone https://github.com/DRVR777/workspace-blueprint.git
cd workspace-blueprint/programs/game_engine/world

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Build (release mode for production performance)
cargo build --release

# Binary is at: target/release/nexus-node
```

## Run

```bash
# Start on port 9001 (default)
./target/release/nexus-node

# Or specify port
./target/release/nexus-node 9001
```

## What it does

- Opens WebSocket server on the specified port
- Runs a 50Hz tick loop with Rapier physics
- Accepts client connections
- Broadcasts entity position updates after each tick

## Client connection

The R3F client (running locally or hosted) connects to:
```
ws://YOUR_VPS_IP:9001
```

## Firewall

Open port 9001:
```bash
sudo ufw allow 9001/tcp
```

## Process management (production)

```bash
# Install pm2 or use systemd
# Example systemd service:
sudo tee /etc/systemd/system/nexus.service << 'EOF'
[Unit]
Description=NEXUS Game Server
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/workspace-blueprint/programs/game_engine/world
ExecStart=/home/ubuntu/workspace-blueprint/programs/game_engine/world/target/release/nexus-node 9001
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable nexus
sudo systemctl start nexus
```

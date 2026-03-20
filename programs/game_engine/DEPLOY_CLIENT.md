# Deploy Client to VPS

The client is a static site (HTML + JS + assets). Nginx serves it and proxies WebSocket connections to the Rust server.

## Step 1: Build the client locally

```bash
cd programs/game_engine/engine/programs/renderer
npm install
npx vite build
```

Output is in `dist/` (~1MB JS + 26MB plane.glb + textures).

## Step 2: Copy dist/ to VPS

```bash
scp -r dist/ root@65.108.67.204:/var/www/nexus/
```

Or on the VPS, pull from git and build there:
```bash
cd /root/workspace-blueprint/programs/game_engine/engine/programs/renderer
npm install
npx vite build
ln -sf $(pwd)/dist /var/www/nexus
```

## Step 3: Nginx config on VPS

```nginx
server {
    listen 80;
    server_name 65.108.67.204;

    # Serve the built client
    root /var/www/nexus;
    index index.html;

    # SPA fallback — all routes serve index.html
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets aggressively
    location /assets/ {
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # Cache 3D models
    location /models/ {
        expires 7d;
        add_header Cache-Control "public";
    }
}
```

## Step 4: Enable and restart nginx

```bash
sudo apt install nginx -y
sudo cp nexus.conf /etc/nginx/sites-available/nexus
sudo ln -sf /etc/nginx/sites-available/nexus /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl restart nginx
```

## Step 5: Open firewall

```bash
sudo ufw allow 80/tcp
sudo ufw allow 9001/tcp  # already open for WebSocket
```

## How it works

```
Browser visits http://65.108.67.204
  → Nginx serves index.html + JS bundle from /var/www/nexus/
  → Client JS detects hostname is not localhost
  → Auto-connects to ws://65.108.67.204:9001
  → Rust server handles game state
  → Player sees the world and can move
```

No env vars needed. No localhost. Just visit the URL and play.

## Testing multiplayer

Open http://65.108.67.204 in two different browsers (or two devices).
Each gets its own WebSocket connection → own player entity → they see each other.

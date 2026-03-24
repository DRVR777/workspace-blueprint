#!/bin/bash
# test-connection.sh - Run this from an EXTERNAL network (cellular, friend's house, etc.)
# This tests if your server is reachable from the public internet

SERVER_IP="${1:-YOUR_PUBLIC_IP}"  # Pass your public IP as first argument
PORT="${2:-9001}"

echo "========================================"
echo " NEXUS Connection Diagnostic Tool"
echo "========================================"
echo ""
echo "Testing connection to: $SERVER_IP:$PORT"
echo "Your external IP: $(curl -s ifconfig.me 2>/dev/null || echo 'unknown')"
echo ""
echo "---"

# Test 1: TCP connectivity
echo "[Test 1] TCP Connection Test"
echo "  Command: nc -vz $SERVER_IP $PORT"
nc -vz "$SERVER_IP" $PORT 2>&1
result=$?

echo ""
echo "---"

if [ $result -eq 0 ]; then
    echo "[PASS] TCP connection successful!"
    echo ""
    echo "[Test 2] WebSocket Handshake"
    echo "  Sending HTTP upgrade request..."
    
    response=$(echo -e "GET / HTTP/1.1\r\nHost: $SERVER_IP\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n" | nc -w3 "$SERVER_IP" $PORT 2>&1)
    
    if echo "$response" | grep -q "101 Switching Protocols"; then
        echo "[PASS] WebSocket upgrade successful!"
        echo "  Server response: 101 Switching Protocols"
        echo ""
        echo "========================================"
        echo " NETWORK LAYER: PASS"
        echo " If players still don't see each other,"
        echo " the bug is in APPLICATION LAYER"
        echo "========================================"
    else
        echo "[FAIL] WebSocket handshake failed"
        echo "  Response: $response"
        echo ""
        echo "========================================"
        echo " Check WebSocket endpoint path"
        echo "========================================"
    fi
else
    echo "[FAIL] TCP connection failed!"
    echo ""
    echo "========================================"
    echo " ROOT CAUSE: Network infrastructure issue"
    echo "========================================"
    echo ""
    echo "Likely causes (check in order):"
    echo "  1. Port forwarding NOT configured on router"
    echo "  2. Firewall blocking port $PORT"
    echo "  3. Wrong public IP address"
    echo "  4. Server not running"
    echo ""
    echo "Quick fixes:"
    echo "  1. Router: Forward external $PORT → internal $(hostname -I | awk '{print $1}'):$PORT"
    echo "  2. Firewall: sudo ufw allow $PORT/tcp"
    echo "  3. Get public IP: curl ifconfig.me"
fi

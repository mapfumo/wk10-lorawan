#!/bin/bash
# Start all LoRaWAN data pipeline services
# Services: InfluxDB, Grafana, Mosquitto, MQTT Bridge

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Starting LoRaWAN data pipeline services..."
docker compose up -d

echo ""
echo "=== Service Status ==="
docker compose ps

echo ""
echo "=== MQTT Bridge Logs (last 10 lines) ==="
sleep 2  # Give container time to start
docker compose logs mqtt-bridge --tail 10

echo ""
echo "Services available at:"
echo "  Grafana:  http://localhost:3000  (admin/admin)"
echo "  InfluxDB: http://localhost:8086  (admin/admin123456)"

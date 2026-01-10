#!/bin/bash
# Stop all LoRaWAN data pipeline services

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Stopping LoRaWAN data pipeline services..."
docker compose down

echo ""
echo "All services stopped."

# LoRaWAN Sensor Network - User Guide

A complete guide to setting up and running a 2-node LoRaWAN sensor network using STM32WL55 microcontrollers, RAK7268V2 gateway, and a Grafana visualization dashboard.

**This project is fully standalone** - everything needed to run the data pipeline is included.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [System Architecture](#system-architecture)
3. [Prerequisites](#prerequisites)
4. [Quick Start](#quick-start)
5. [Hardware Setup](#hardware-setup)
6. [Firmware Setup](#firmware-setup)
7. [Gateway Configuration](#gateway-configuration)
8. [Data Pipeline Setup](#data-pipeline-setup)
9. [Docker Services](#docker-services)
10. [Grafana Dashboard](#grafana-dashboard)
11. [Troubleshooting](#troubleshooting)
12. [Quick Reference](#quick-reference)

---

## Project Overview

This project creates a LoRaWAN sensor network with:

- **2 STM32WL55 sensor nodes** transmitting environmental data
- **RAK7268V2 LoRaWAN gateway** receiving and forwarding data
- **Python MQTT bridge** decoding payloads and storing in InfluxDB
- **Grafana dashboard** for real-time visualization

### What Each Node Measures

| Node | Sensor | Measurements |
|------|--------|--------------|
| LoRa-1 | SHT41 | Temperature, Humidity |
| LoRa-2 | BME680 | Temperature, Humidity, Pressure, Gas Resistance |

---

## System Architecture

### Network Topology

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              YOUR NETWORK                                    │
│                                                                             │
│  ┌──────────────┐     LoRaWAN      ┌──────────────┐                        │
│  │   LoRa-1     │    (915 MHz)     │  RAK7268V2   │                        │
│  │  STM32WL55   │ ─────────────────│   Gateway    │                        │
│  │   + SHT41    │                  │              │                        │
│  │  (Temp/Hum)  │                  │ 10.10.10.254 │                        │
│  └──────────────┘                  │              │                        │
│                                    │   MQTT       │                        │
│  ┌──────────────┐                  │   Broker     │                        │
│  │   LoRa-2     │ ─────────────────│   :1883      │                        │
│  │  STM32WL55   │                  └──────┬───────┘                        │
│  │   + BME680   │                         │                                │
│  │ (Temp/Hum/   │                         │ MQTT (TCP)                     │
│  │  Press/Gas)  │                         │                                │
│  └──────────────┘                         ▼                                │
│                                    ┌──────────────┐                        │
│                                    │ Your Computer│                        │
│                                    │  (Docker)    │                        │
│                                    └──────────────┘                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
┌─────────┐    ┌─────────┐    ┌─────────────┐    ┌──────────┐    ┌─────────┐
│ Sensor  │───▶│ STM32WL │───▶│  RAK7268V2  │───▶│  MQTT    │───▶│InfluxDB │
│ Reading │    │  Radio  │    │   Gateway   │    │  Bridge  │    │         │
└─────────┘    └─────────┘    └─────────────┘    └──────────┘    └────┬────┘
                                                                      │
                                                                      ▼
                                                                ┌─────────┐
                                                                │ Grafana │
                                                                │Dashboard│
                                                                └─────────┘
```

### Docker Container Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       Docker Network: lorawan-network                    │
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   mqtt-bridge   │  │    influxdb     │  │     grafana     │         │
│  │                 │  │                 │  │                 │         │
│  │ Python script   │─▶│  Time-series    │─▶│  Visualization  │         │
│  │ decodes MQTT    │  │  database       │  │  dashboard      │         │
│  │ payloads        │  │                 │  │                 │         │
│  │                 │  │  Port: 8086     │  │  Port: 3000     │         │
│  └────────┬────────┘  └─────────────────┘  └─────────────────┘         │
│           │                                                             │
│           │ Connects to external MQTT                                   │
└───────────┼─────────────────────────────────────────────────────────────┘
            │
            ▼
    ┌───────────────┐
    │  RAK Gateway  │
    │  MQTT Broker  │
    │ 10.10.10.254  │
    │    :1883      │
    └───────────────┘
```

---

## Prerequisites

### Software Requirements

| Software | Version | Purpose |
|----------|---------|---------|
| Docker | 20.10+ | Container runtime |
| Docker Compose | 2.0+ | Multi-container orchestration |
| Rust | 1.70+ | Firmware compilation |
| probe-rs | Latest | Firmware flashing |

### Install Docker (Ubuntu/Debian)

```bash
# Update package index
sudo apt update

# Install Docker
sudo apt install docker.io docker-compose-plugin

# Add your user to docker group (logout/login required)
sudo usermod -aG docker $USER

# Verify installation
docker --version
docker compose version
```

### Install Rust Toolchain

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment (or restart terminal)
source ~/.cargo/env

# Add embedded target for STM32WL55
rustup target add thumbv7em-none-eabihf

# Verify
rustc --version
```

### Install probe-rs (Firmware Flashing Tool)

```bash
# Install probe-rs
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Verify installation
probe-rs --version
```

### Hardware Requirements

| Item | Quantity | Notes |
|------|----------|-------|
| STM32WL55 Nucleo Board | 2 | NUCLEO-WL55JC1 or similar |
| SHT41 Sensor | 1 | I2C temperature/humidity |
| BME680 Sensor | 1 | I2C environmental sensor |
| SSD1306 OLED (128x32) | 1 | For LoRa-1 display |
| SH1106 OLED (128x64) | 1 | For LoRa-2 display |
| RAK7268V2 Gateway | 1 | Or compatible LoRaWAN gateway |
| USB Cables | 2 | For programming/power |

---

## Quick Start

For those who want to get running quickly:

```bash
# 1. Clone/copy the project
cd ~/dev/4-month-plan/wk10-lorawan

# 2. Start all services (InfluxDB, Grafana, MQTT Bridge)
./start_services.sh

# 3. Flash firmware to nodes
cd firmware/lora-1 && cargo run --release
cd firmware/lora-2 && cargo run --release

# 4. Configure Grafana datasource (first time only)
curl -X POST http://localhost:3000/api/datasources \
  -H "Content-Type: application/json" \
  -u admin:admin \
  -d '{
    "name": "LoRaWAN InfluxDB",
    "type": "influxdb",
    "url": "http://influxdb:8086",
    "access": "proxy",
    "jsonData": {
      "version": "Flux",
      "organization": "my-org",
      "defaultBucket": "lorawan"
    },
    "secureJsonData": {
      "token": "my-super-secret-auth-token"
    }
  }'

# 5. Open Grafana
# http://localhost:3000 (admin/admin)
```

---

## Hardware Setup

### LoRa-1 Wiring (SHT41 + SSD1306)

```
STM32WL55 Nucleo          SHT41 Sensor       SSD1306 OLED
─────────────────         ────────────       ────────────
PA12 (I2C2_SCL)  ─────────  SCL  ────────────  SCL
PA11 (I2C2_SDA)  ─────────  SDA  ────────────  SDA
3.3V             ─────────  VCC  ────────────  VCC
GND              ─────────  GND  ────────────  GND
```

**I2C Addresses:**
- SHT41: `0x44`
- SSD1306: `0x3C`

### LoRa-2 Wiring (BME680 + SH1106)

```
STM32WL55 Nucleo          BME680 Sensor      SH1106 OLED
─────────────────         ─────────────      ───────────
PA12 (I2C2_SCL)  ─────────  SCL  ────────────  SCL
PA11 (I2C2_SDA)  ─────────  SDA  ────────────  SDA
3.3V             ─────────  VCC  ────────────  VCC
GND              ─────────  GND  ────────────  GND
```

**I2C Addresses:**
- BME680: `0x76` or `0x77`
- SH1106: `0x3C`

### Identifying Your Boards

Each STM32WL55 Nucleo has a unique probe serial number. List connected probes:

```bash
probe-rs list
```

Example output:
```
The following debug probes were found:
[0]: STLink V3 -- 0483:374e:003E00463234510A33353533 (ST-LINK)
[1]: STLink V3 -- 0483:374e:0026003A3234510A33353533 (ST-LINK)
```

**Board Assignment (this project):**
| Node | Probe Serial |
|------|--------------|
| LoRa-1 | `003E00463234510A33353533` |
| LoRa-2 | `0026003A3234510A33353533` |

---

## Firmware Setup

### Project Structure

```
wk10-lorawan/
├── firmware/
│   ├── lora-1/          # LoRa-1 firmware (SHT41)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── lora-2/          # LoRa-2 firmware (BME680)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── lora-phy-patched/    # Patched LoRa PHY library
├── lorawan-device-patched/  # Patched LoRaWAN library
└── ...
```

### Building Firmware

**LoRa-1:**
```bash
cd firmware/lora-1

# Debug build
cargo build

# Release build (smaller, optimized)
cargo build --release
```

**LoRa-2:**
```bash
cd firmware/lora-2
cargo build --release
```

### Flashing Firmware

**LoRa-1 (auto-detect probe):**
```bash
cd firmware/lora-1

# Build, flash, and attach RTT logs
cargo run --release
```

**LoRa-2 (explicit probe):**
```bash
cd firmware/lora-2

# If multiple probes connected, specify which one
probe-rs run --chip STM32WL55JCIx \
  --probe 0483:374e:0026003A3234510A33353533 \
  target/thumbv7em-none-eabihf/release/lora-2
```

### Viewing Debug Logs

Once flashed, RTT (Real-Time Transfer) logs appear in your terminal:

```
INFO  LoRa-2 Environmental Monitor
INFO  I2C initialized on PA12 (SCL) / PA11 (SDA)
INFO  Starting OTAA join...
INFO  Join successful!
INFO  Temp: 28C  Hum: 60%  Press: 1020 hPa  Gas: 134 kOhm
INFO  Tx #1 complete, RSSI: -15, SNR: 14.0
```

To attach to logs without reflashing:
```bash
probe-rs attach --chip STM32WL55JCIx
```

---

## Gateway Configuration

### RAK7268V2 Setup Overview

The gateway should be configured for:

- **Region:** AU915
- **Sub-band:** 2 (channels 8-15: 915.2-916.6 MHz)
- **Network Server:** Built-in LoRa Server (not ChirpStack)
- **MQTT Broker:** Enabled on port 1883

### LoRaWAN Credentials

Credentials are pre-registered in the gateway under application "TOT".

| Node | DevEUI | AppEUI |
|------|--------|--------|
| LoRa-1 | `23ce1bfeff091fac` | `b130a864c5295356` |
| LoRa-2 | `24ce1bfeff091fac` | `b130a864c5295356` |

See `LORAWAN_CREDENTIALS.md` for complete credential details including AppKeys.

### MQTT Topics

The gateway publishes to these MQTT topics:

| Topic | Description |
|-------|-------------|
| `application/TOT/device/+/rx` | Uplink messages (sensor data) |
| `application/TOT/device/+/join` | Join events |
| `gateway/+/stats` | Gateway statistics |

---

## Data Pipeline Setup

### Step 1: Start All Services

```bash
cd ~/dev/4-month-plan/wk10-lorawan
./start_services.sh
```

Or manually:
```bash
docker compose up -d
```

Verify containers are running:
```bash
docker compose ps
```

Expected output:
```
NAME          IMAGE                    STATUS
grafana       grafana/grafana:latest   Up
influxdb      influxdb:2               Up
mosquitto     eclipse-mosquitto:2      Up
mqtt-bridge   python:3.11-slim         Up
```

### Step 2: Verify MQTT Bridge

Check bridge logs:
```bash
docker compose logs mqtt-bridge
```

Expected output:
```
============================================================
LoRaWAN MQTT → InfluxDB Bridge
Gateway: 10.10.10.254:1883
InfluxDB: influxdb:8086/lorawan
============================================================
Connected to MQTT broker at 10.10.10.254:1883
Subscribed to: application/#
Waiting for LoRaWAN uplinks...
```

### Step 3: Configure Grafana Datasource (First Time Only)

```bash
curl -X POST http://localhost:3000/api/datasources \
  -H "Content-Type: application/json" \
  -u admin:admin \
  -d '{
    "name": "LoRaWAN InfluxDB",
    "type": "influxdb",
    "url": "http://influxdb:8086",
    "access": "proxy",
    "jsonData": {
      "version": "Flux",
      "organization": "my-org",
      "defaultBucket": "lorawan"
    },
    "secureJsonData": {
      "token": "my-super-secret-auth-token"
    }
  }'
```

### Step 4: Import Grafana Dashboard (Optional)

```bash
curl -X POST http://localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -u admin:admin \
  -d "{\"dashboard\": $(cat grafana/lorawan-dashboard.json), \"overwrite\": true}"
```

---

## Docker Services

### Service Overview

| Container | Image | Port | Purpose |
|-----------|-------|------|---------|
| influxdb | influxdb:2 | 8086 | Time-series database |
| grafana | grafana/grafana | 3000 | Visualization |
| mosquitto | eclipse-mosquitto:2 | 1883 | Local MQTT broker (optional) |
| mqtt-bridge | python:3.11-slim | - | Gateway MQTT → InfluxDB |

### Starting Services

```bash
# Using helper script
./start_services.sh

# Or manually
docker compose up -d
```

### Stopping Services

```bash
# Using helper script
./stop_services.sh

# Or manually
docker compose down
```

### Viewing Logs

```bash
# MQTT bridge logs (live)
docker compose logs -f mqtt-bridge

# All service logs
docker compose logs

# Specific service
docker compose logs influxdb
```

### Restarting Services

```bash
# Restart all
docker compose restart

# Restart specific service
docker compose restart mqtt-bridge
```

---

## Grafana Dashboard

### Accessing Grafana

- **URL:** http://localhost:3000
- **Username:** `admin`
- **Password:** `admin` (change on first login)

### Dashboard URL

Once set up, access the dashboard at:
```
http://localhost:3000/d/lorawan-sensors/lorawan-sensor-network
```

### First-Time Setup (Required After Fresh Install)

The Grafana container starts empty. You must create the datasource and import the dashboard.

**Step 1: Create the InfluxDB Datasource**

The dashboard JSON expects a datasource with UID `lorawan-influxdb`. This UID must match exactly.

```bash
curl -X POST http://localhost:3000/api/datasources \
  -H "Content-Type: application/json" \
  -u admin:admin \
  -d '{
    "name": "LoRaWAN InfluxDB",
    "type": "influxdb",
    "uid": "lorawan-influxdb",
    "url": "http://influxdb:8086",
    "access": "proxy",
    "jsonData": {
      "version": "Flux",
      "organization": "my-org",
      "defaultBucket": "lorawan"
    },
    "secureJsonData": {
      "token": "my-super-secret-auth-token"
    }
  }'
```

**Step 2: Import the Dashboard**

```bash
curl -X POST http://localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -u admin:admin \
  -d "{\"dashboard\": $(cat grafana/lorawan-dashboard.json), \"overwrite\": true}"
```

Expected response:
```json
{"status":"success","uid":"lorawan-sensors","url":"/d/lorawan-sensors/lorawan-sensor-network"}
```

### Verifying Setup

**Check if datasource exists:**
```bash
curl -s http://localhost:3000/api/datasources -u admin:admin | python3 -m json.tool
```

**Check if dashboard exists:**
```bash
curl -s "http://localhost:3000/api/search?query=lorawan" -u admin:admin
```

**Test datasource connection:**
```bash
curl -s http://localhost:3000/api/datasources/uid/lorawan-influxdb -u admin:admin
```

### Why the UID Matters

The dashboard JSON file (`grafana/lorawan-dashboard.json`) references the datasource by UID in every panel:

```json
"datasource": {
  "type": "influxdb",
  "uid": "lorawan-influxdb"
}
```

If you create a datasource with a different UID (e.g., auto-generated), all panels will show "Datasource not found". Always use `"uid": "lorawan-influxdb"` when creating the datasource.

### Recovery After Container Reset

If you recreate the Grafana container (e.g., `docker compose down` then `up`), you lose all dashboards and datasources. Re-run the setup commands:

```bash
# 1. Create datasource
curl -X POST http://localhost:3000/api/datasources \
  -H "Content-Type: application/json" \
  -u admin:YOUR_PASSWORD \
  -d '{
    "name": "LoRaWAN InfluxDB",
    "type": "influxdb",
    "uid": "lorawan-influxdb",
    "url": "http://influxdb:8086",
    "access": "proxy",
    "jsonData": {"version": "Flux", "organization": "my-org", "defaultBucket": "lorawan"},
    "secureJsonData": {"token": "my-super-secret-auth-token"}
  }'

# 2. Import dashboard
curl -X POST http://localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -u admin:YOUR_PASSWORD \
  -d "{\"dashboard\": $(cat grafana/lorawan-dashboard.json), \"overwrite\": true}"
```

**Note:** Replace `YOUR_PASSWORD` with your Grafana password (default: `admin`).

### Dashboard Panels

| Panel | Description |
|-------|-------------|
| Temperature | Time series of both nodes |
| Humidity | Time series of both nodes |
| Pressure | LoRa-2 only (BME680) |
| Gas Resistance | LoRa-2 only (BME680) |
| RSSI | Signal strength (dBm) |
| SNR | Signal-to-noise ratio (dB) |
| Stat Panels | Current readings for each node |

### Manual Flux Queries

**All sensor data (last hour):**
```flux
from(bucket: "lorawan")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "lorawan_sensor")
```

**Temperature only:**
```flux
from(bucket: "lorawan")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "lorawan_sensor")
  |> filter(fn: (r) => r._field == "temperature")
```

**Specific node:**
```flux
from(bucket: "lorawan")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "lorawan_sensor")
  |> filter(fn: (r) => r.node == "lora1")
```

---

## Troubleshooting

### MQTT Bridge Issues

**Problem:** Bridge can't connect to gateway MQTT
```
Error: Connection refused
```

**Solutions:**
1. Verify gateway IP address is correct (default: `10.10.10.254`)
2. Check gateway MQTT broker is enabled (port 1883)
3. Ensure your computer can reach the gateway:
   ```bash
   ping 10.10.10.254
   nc -zv 10.10.10.254 1883
   ```

**Problem:** Bridge connects but no data
```
Waiting for LoRaWAN uplinks...
(nothing appears)
```

**Solutions:**
1. Check nodes are powered and transmitting (LED blinks)
2. Verify nodes have joined successfully (check gateway web UI)
3. Subscribe manually to verify MQTT:
   ```bash
   docker run --rm -it eclipse-mosquitto \
     mosquitto_sub -h 10.10.10.254 -t "application/#" -v
   ```

### InfluxDB Issues

**Problem:** Data not appearing in InfluxDB

**Solutions:**
1. Check bridge logs for write errors:
   ```bash
   docker compose logs mqtt-bridge | grep -i error
   ```
2. Verify bucket exists:
   ```bash
   docker exec influxdb influx bucket list \
     --org my-org --token my-super-secret-auth-token
   ```
3. Query data directly:
   ```bash
   docker exec influxdb influx query \
     'from(bucket: "lorawan") |> range(start: -5m) |> limit(n: 5)' \
     --org my-org --token my-super-secret-auth-token
   ```

### Grafana Issues

**Problem:** Dashboard shows "No data"

**Solutions:**
1. Verify datasource is configured correctly:
   - Go to Configuration → Data Sources → LoRaWAN InfluxDB
   - Click "Test" button
2. Check time range (default is last 1 hour)
3. Wait for nodes to transmit (every ~30 seconds)

**Problem:** Can't login to Grafana

**Solutions:**
1. Default credentials: `admin` / `admin`
2. Reset password:
   ```bash
   docker exec -it grafana grafana-cli admin reset-admin-password newpassword
   ```

### Firmware Issues

**Problem:** `cargo run` fails with "probe not found"

**Solutions:**
1. Check USB connection
2. List available probes:
   ```bash
   probe-rs list
   ```
3. Check permissions (Linux):
   ```bash
   sudo usermod -aG plugdev $USER
   # Logout and login again
   ```

**Problem:** Node won't join LoRaWAN network

**Solutions:**
1. Check credentials match gateway configuration
2. Verify AU915 sub-band 2 is configured
3. Check RSSI/SNR in gateway logs (weak signal?)
4. Ensure node is within gateway range

---

## Quick Reference

### Essential Commands

```bash
# ─────────────────────────────────────────────────────────
# DOCKER SERVICES
# ─────────────────────────────────────────────────────────

# Start all services
./start_services.sh

# Stop all services
./stop_services.sh

# View MQTT bridge logs
docker compose logs -f mqtt-bridge

# Check all containers
docker compose ps

# ─────────────────────────────────────────────────────────
# FIRMWARE
# ─────────────────────────────────────────────────────────

# Flash LoRa-1
cd firmware/lora-1 && cargo run --release

# Flash LoRa-2
cd firmware/lora-2 && cargo run --release

# List probes
probe-rs list

# Attach to running firmware (view logs)
probe-rs attach --chip STM32WL55JCIx

# ─────────────────────────────────────────────────────────
# INFLUXDB
# ─────────────────────────────────────────────────────────

# Query recent data
docker exec influxdb influx query \
  'from(bucket: "lorawan") |> range(start: -5m)' \
  --org my-org --token my-super-secret-auth-token

# List buckets
docker exec influxdb influx bucket list \
  --org my-org --token my-super-secret-auth-token

# ─────────────────────────────────────────────────────────
# MQTT DEBUGGING
# ─────────────────────────────────────────────────────────

# Subscribe to gateway MQTT
docker run --rm -it eclipse-mosquitto \
  mosquitto_sub -h 10.10.10.254 -t "application/#" -v

# ─────────────────────────────────────────────────────────
# URLS
# ─────────────────────────────────────────────────────────

# Grafana:  http://localhost:3000  (admin/admin)
# InfluxDB: http://localhost:8086  (admin/admin123456)
# Gateway:  http://10.10.10.254    (check your gateway docs)
```

### File Locations

| File | Purpose |
|------|---------|
| `firmware/lora-1/src/main.rs` | LoRa-1 firmware source |
| `firmware/lora-2/src/main.rs` | LoRa-2 firmware source |
| `mqtt_to_influx.py` | MQTT→InfluxDB bridge script |
| `docker-compose.yml` | All Docker services |
| `mosquitto/config/mosquitto.conf` | Local MQTT broker config |
| `grafana/lorawan-dashboard.json` | Grafana dashboard definition |
| `start_services.sh` | Start all services |
| `stop_services.sh` | Stop all services |
| `LORAWAN_CREDENTIALS.md` | LoRaWAN keys and EUIs |
| `CLAUDE.md` | Development notes and constraints |

### Default Credentials

| Service | Username | Password |
|---------|----------|----------|
| Grafana | admin | admin |
| InfluxDB | admin | admin123456 |
| InfluxDB Token | - | my-super-secret-auth-token |
| Gateway MQTT | (none) | (none) |

### Network Addresses

| Service | Address |
|---------|---------|
| RAK Gateway | 10.10.10.254 |
| Gateway MQTT | 10.10.10.254:1883 |
| InfluxDB | localhost:8086 |
| Grafana | localhost:3000 |

---

## Recreating on a New Computer

### Quick Start Checklist

1. [ ] Install Docker and Docker Compose
2. [ ] Install Rust and add `thumbv7em-none-eabihf` target
3. [ ] Install probe-rs
4. [ ] Clone/copy this project directory
5. [ ] Run `./start_services.sh`
6. [ ] Connect STM32WL55 boards via USB
7. [ ] Flash firmware to both nodes
8. [ ] Configure gateway with correct credentials
9. [ ] Add Grafana datasource (curl command above)
10. [ ] Import Grafana dashboard (optional)
11. [ ] Verify data in Grafana

### Minimum Files Needed

```
wk10-lorawan/
├── firmware/
│   ├── lora-1/
│   └── lora-2/
├── lora-phy-patched/
├── lorawan-device-patched/
├── mosquitto/
│   └── config/
│       └── mosquitto.conf
├── grafana/
│   └── lorawan-dashboard.json
├── mqtt_to_influx.py
├── docker-compose.yml
├── start_services.sh
├── stop_services.sh
├── LORAWAN_CREDENTIALS.md
└── USERGUIDE.md (this file)
```

---

*Last updated: 2026-01-10*

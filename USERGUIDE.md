# Week 10: LoRaWAN System - User Guide

**Target Audience**: Deployment engineers, system operators
**Prerequisites**: Linux desktop, Docker, basic networking knowledge
**Last Updated**: 2026-01-03

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Hardware Setup](#hardware-setup)
3. [ChirpStack Installation](#chirpstack-installation)
4. [Device Provisioning](#device-provisioning)
5. [Gateway Service Setup](#gateway-service-setup)
6. [Monitoring and Dashboards](#monitoring-and-dashboards)
7. [Troubleshooting](#troubleshooting)
8. [Maintenance](#maintenance)

---

## System Overview

This system combines 4 sensor nodes in a unified monitoring platform:
- **2x Modbus TCP nodes** (Week 9): Ethernet-connected F446RE boards
- **2x LoRaWAN nodes** (Week 10): Wireless STM32WL55 boards

All data flows through MQTT → InfluxDB → Grafana for visualization.

### System Requirements

**Hardware**:
- 2x NUCLEO-WL55JC1 boards (flashed with firmware)
- RAK7268V2 LoRaWAN gateway (configured for AU915)
- Ethernet switch/router (for gateway connectivity)
- Linux desktop (Ubuntu/Pop!_OS recommended)

**Software**:
- Docker & Docker Compose
- Python 3.8+
- Mosquitto MQTT broker
- InfluxDB 2.x
- Grafana 12.x
- ChirpStack network server

---

## Hardware Setup

### RAK7268V2 Gateway Configuration

**Step 1: Physical Setup**
1. Connect RAK7268V2 to power (PoE or DC adapter)
2. Connect Ethernet cable to LAN port
3. Wait for gateway to boot (~60 seconds)

**Step 2: Verify Network Connectivity**
```bash
# Find gateway IP (check your router's DHCP table)
# Or use the default: 192.168.230.1

ping <gateway-ip>
```

**Step 3: Access Gateway Web UI**
1. Open browser: `http://<gateway-ip>`
2. Login with credentials (default: root/root)
3. Verify LoRa configuration:
   - **Region**: AU915
   - **Network Server**: ChirpStack (to be configured)
   - **Packet Forwarder**: UDP (default)

### STM32WL55 Node Setup

**Node 1 (BME688)**:
1. Wire BME688 sensor to I2C1 (PB8/PB9)
2. Wire OLED display to I2C1 (shared bus)
3. Connect ST-Link debugger
4. Flash firmware: `probe-rs run --chip STM32WL55JCIx --release`
5. Verify OLED shows: "WL55-BME688" + "JOINING..."

**Node 2 (SHT41)**:
1. Wire SHT41 sensor to I2C1 (PB8/PB9)
2. Wire OLED display to I2C1 (shared bus)
3. Connect ST-Link debugger
4. Flash firmware: `probe-rs run --chip STM32WL55JCIx --release`
5. Verify OLED shows: "WL55-SHT41" + "JOINING..."

---

## ChirpStack Installation

### Using Docker Compose

**Step 1: Clone ChirpStack Docker Repository**
```bash
cd ~/dev
git clone https://github.com/chirpstack/chirpstack-docker.git
cd chirpstack-docker
```

**Step 2: Configure Region (AU915)**
```bash
# Edit AU915 configuration
nano configuration/chirpstack/region_au915_0.toml
```

Ensure these settings:
```toml
[gateway]
enabled = true

[network]
enabled = true
net_id = "000000"

[[regions]]
id = "au915_0"
common_name = "AU915"
```

**Step 3: Start ChirpStack Services**
```bash
docker-compose up -d
```

**Step 4: Verify Services Running**
```bash
docker-compose ps

# Expected output:
# chirpstack-network-server  (port 8000)
# chirpstack-application-server (port 8080)
# chirpstack-gateway-bridge (port 1700/udp)
# postgresql (port 5432)
# redis (port 6379)
# mosquitto (port 1883)
```

**Step 5: Access Web UI**
1. Open browser: `http://localhost:8080`
2. Login:
   - **Username**: admin
   - **Password**: admin
3. Change password on first login

---

## Device Provisioning

### Gateway Registration

**Step 1: Add Gateway in ChirpStack**
1. Navigate to: **Gateways** → **Add Gateway**
2. Fill details:
   - **Name**: RAK7268V2-AU915
   - **Gateway ID**: (from RAK gateway - 8 byte EUI)
   - **Network Server**: default
   - **Service Profile**: default
3. Click **Add Gateway**

**Step 2: Configure RAK7268V2 to Use ChirpStack**
1. Access RAK gateway web UI
2. Navigate to: **LoRa** → **Packet Forwarder**
3. Set server address:
   - **Server Address**: `<desktop-ip>` (ChirpStack host)
   - **Server Port**: 1700 (UDP)
   - **Protocol**: Semtech UDP
4. Click **Save & Apply**
5. Verify in ChirpStack: Gateway shows "Last Seen" timestamp

### Device Profile Creation

**Step 1: Create Device Profile**
1. Navigate to: **Device Profiles** → **Create**
2. Configure:
   - **Name**: STM32WL55-ClassA-OTAA
   - **LoRaWAN MAC Version**: 1.0.3
   - **Regional Parameters**: A
   - **Max EIRP**: 30 (for AU915)
   - **Uplink Interval**: 60 seconds
3. **Join (OTAA/ABP)**:
   - Select: **Device supports OTAA**
4. **Class-B/C**: Leave disabled (Class A default)
5. Click **Create Device Profile**

### Application Creation

**Step 1: Create Application**
1. Navigate to: **Applications** → **Create**
2. Fill details:
   - **Name**: IIoT-Sensors
   - **Description**: Week 10 LoRaWAN sensor nodes
   - **Service Profile**: default
3. Click **Create Application**

### Device Registration

**Step 2: Add Node 1 (BME688)**
1. Open application: **IIoT-Sensors**
2. Navigate to: **Devices** → **Create**
3. Fill details:
   - **Device Name**: WL55-BME688
   - **Device Description**: Node 1 - Environmental sensor
   - **Device EUI**: (from firmware - check defmt logs or hardcoded value)
   - **Device Profile**: STM32WL55-ClassA-OTAA
4. Click **Create Device**
5. Navigate to: **Keys (OTAA)** tab
6. Set Application Key:
   - **Application Key**: (generate random 128-bit key or use default)
   - Copy this key to firmware configuration
7. Click **Set Device Keys**

**Step 3: Add Node 2 (SHT41)**
Repeat Step 2 with:
- **Device Name**: WL55-SHT41
- **Device Description**: Node 2 - Precision temperature sensor
- **Device EUI**: (different from Node 1)

### Update Firmware with Keys

Edit firmware source files with DevEUI, AppEUI, AppKey from ChirpStack:

**Node 1 (node1-bme688/src/main.rs)**:
```rust
const DEV_EUI: [u8; 8] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01];
const APP_EUI: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const APP_KEY: [u8; 16] = [/* from ChirpStack */];
```

**Node 2 (node2-sht41/src/main.rs)**:
```rust
const DEV_EUI: [u8; 8] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x02];
const APP_EUI: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const APP_KEY: [u8; 16] = [/* from ChirpStack */];
```

Rebuild and reflash both boards.

---

## Gateway Service Setup

### Install Dependencies

```bash
sudo apt update
sudo apt install python3-pip mosquitto-clients

pip3 install asyncua pymodbus paho-mqtt influxdb-client
```

### Configure MQTT Integration in ChirpStack

**Step 1: Enable MQTT Integration**
1. In ChirpStack web UI: **Applications** → **IIoT-Sensors**
2. Navigate to: **Integrations** → **MQTT**
3. Enable MQTT integration
4. Configure:
   - **MQTT Server**: tcp://localhost:1883
   - **Username**: (leave empty for local Mosquitto)
   - **Password**: (leave empty)
5. Click **Save**

**Step 2: Verify MQTT Topics**
```bash
# Subscribe to all ChirpStack events
mosquitto_sub -h localhost -t 'application/#' -v

# Expected topics:
# application/+/device/+/event/up       (uplink data)
# application/+/device/+/event/join     (join events)
# application/+/device/+/event/status   (status updates)
```

### Run Gateway Service

**Start the LoRaWAN MQTT Bridge**:
```bash
cd gateway
python3 lorawan_mqtt_bridge.py
```

**Expected Output**:
```
[INFO] Connecting to MQTT broker: localhost:1883
[INFO] Connected to MQTT broker
[INFO] Subscribing to ChirpStack topics...
[INFO] Connecting to InfluxDB: http://localhost:8086
[INFO] InfluxDB connection successful
[INFO] Waiting for LoRaWAN messages...
```

**Test with Node Join**:
```
[INFO] Join event: WL55-BME688 (DevAddr: 0x260BXX)
[INFO] Uplink from WL55-BME688: RSSI=-45 SNR=10 SF=7
[INFO] Decoded: temp=23.4°C, humidity=45.2%, pressure=1013.2hPa
[INFO] Written to InfluxDB: lorawan_sensors (12 bytes)
```

### Run as Systemd Service (Optional)

**Create service file**:
```bash
sudo nano /etc/systemd/system/lorawan-bridge.service
```

```ini
[Unit]
Description=LoRaWAN MQTT to InfluxDB Bridge
After=network.target docker.service

[Service]
Type=simple
User=tony
WorkingDirectory=/home/tony/dev/4-month-plan/wk10-lorawan/gateway
ExecStart=/usr/bin/python3 lorawan_mqtt_bridge.py
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Enable and start**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable lorawan-bridge
sudo systemctl start lorawan-bridge
sudo systemctl status lorawan-bridge
```

---

## Monitoring and Dashboards

### ChirpStack Monitoring

**Device Status**:
1. Navigate to: **Applications** → **IIoT-Sensors** → **Devices**
2. Check **Last Seen** timestamp for each device
3. Click device name to view:
   - **LoRaWAN Frames**: Raw frame log
   - **Device Data**: Decoded payloads
   - **Activation**: Join history

**Gateway Status**:
1. Navigate to: **Gateways** → **RAK7268V2-AU915**
2. View:
   - **Last Seen**: Connectivity status
   - **Statistics**: RX/TX packets, duty cycle
   - **Live LoRaWAN Frames**: Real-time packet view

### Grafana Dashboard Access

**Open Grafana**:
```bash
# URL: http://localhost:3000
# Username: admin
# Password: (set during initial setup)
```

**Import 4-Node Dashboard**:
1. Navigate to: **Dashboards** → **Import**
2. Upload: `grafana/4-node-unified-dashboard.json`
3. Select InfluxDB data source
4. Click **Import**

**Dashboard Panels**:
- **LoRaWAN Node 1 (BME688)**: Temperature, Humidity, Pressure, Gas, IAQ
- **LoRaWAN Node 2 (SHT41)**: High-precision Temperature, Humidity
- **Modbus Node 1**: Temperature, Humidity (from Week 9)
- **Modbus Node 2**: Temperature, Humidity (from Week 9)
- **Temperature Comparison**: All 4 nodes on one graph
- **LoRaWAN Metrics**: RSSI, SNR, Spreading Factor

---

## Troubleshooting

### Device Won't Join

**Symptom**: OLED shows "JOINING..." for >2 minutes

**Checks**:
1. Verify gateway receives join requests:
   - ChirpStack: **Gateways** → **Live LoRaWAN Frames**
   - Look for "JoinRequest" message type
2. Verify DevEUI/AppEUI/AppKey match ChirpStack:
   - Compare firmware values with ChirpStack device keys
3. Check frequency plan:
   - Gateway: AU915
   - ChirpStack: AU915
   - Firmware: AU915
4. Verify gateway backhaul:
   - Ping ChirpStack host from gateway
   - Check packet forwarder configuration

**Solutions**:
- Reflash firmware with correct keys
- Power cycle device and gateway
- Move device closer to gateway (RSSI > -100 dBm)
- Try manual join trigger (reset button)

### No Uplink Data After Join

**Symptom**: Device shows "JOINED" but no data in ChirpStack

**Checks**:
1. Verify duty cycle compliance:
   - Wait at least 60 seconds between uplinks
   - Check device logs for duty cycle errors
2. Check frame counter:
   - ChirpStack device page shows incrementing FCnt
3. Verify payload encoding:
   - ChirpStack shows raw payload (hex)
   - Payload decoder returns valid JSON

**Solutions**:
- Wait for next scheduled uplink
- Check defmt logs for transmission errors
- Verify sensor is returning valid data
- Test payload decoder with sample data

### Gateway Offline

**Symptom**: ChirpStack shows gateway "Never Seen"

**Checks**:
1. Verify gateway power and network:
   - Ping gateway IP
   - Check gateway web UI accessible
2. Check packet forwarder configuration:
   - Server address = ChirpStack host IP
   - Server port = 1700 (UDP)
3. Verify ChirpStack services running:
   ```bash
   docker-compose ps
   ```

**Solutions**:
- Reconfigure packet forwarder server address
- Restart gateway (power cycle)
- Restart ChirpStack services:
  ```bash
  docker-compose restart
  ```

### Poor RSSI/SNR

**Symptom**: RSSI < -120 dBm, SNR < 0 dB, packet loss

**Checks**:
1. Measure distance to gateway
2. Check for physical obstructions
3. Verify antenna connections
4. Check spreading factor (SF7 = shortest range)

**Solutions**:
- Move device closer to gateway
- Use higher spreading factor (SF10-SF12)
- Improve antenna placement (higher, line-of-sight)
- Check for interference sources

### No Data in Grafana

**Symptom**: Devices joined, uplinks in ChirpStack, but Grafana empty

**Checks**:
1. Verify MQTT bridge running:
   ```bash
   ps aux | grep lorawan_mqtt_bridge
   ```
2. Check MQTT messages:
   ```bash
   mosquitto_sub -h localhost -t 'application/#' -v
   ```
3. Verify InfluxDB write:
   ```bash
   influx query 'from(bucket:"iiot") |> range(start: -1h) |> filter(fn: (r) => r._measurement == "lorawan_sensors")'
   ```
4. Check Grafana data source connection

**Solutions**:
- Restart MQTT bridge service
- Verify InfluxDB bucket name matches
- Check Grafana query syntax
- Verify time range (last 6 hours)

---

## Maintenance

### Regular Tasks

**Daily**:
- Check device "Last Seen" in ChirpStack
- Verify Grafana dashboards updating
- Monitor gateway uptime

**Weekly**:
- Review LoRaWAN frame logs for errors
- Check duty cycle utilization
- Backup ChirpStack database:
  ```bash
  docker exec chirpstack-postgresql pg_dump -U chirpstack > backup.sql
  ```

**Monthly**:
- Update ChirpStack Docker images:
  ```bash
  cd chirpstack-docker
  docker-compose pull
  docker-compose up -d
  ```
- Review InfluxDB storage usage
- Rotate logs

### Firmware Updates

**Update Node Firmware**:
1. Pull latest code from repository
2. Update DevEUI/AppEUI/AppKey if needed
3. Build firmware:
   ```bash
   cd firmware/node1-bme688
   cargo build --release
   ```
4. Flash device:
   ```bash
   probe-rs run --chip STM32WL55JCIx --release
   ```
5. Verify device rejoins network
6. Check first uplink in ChirpStack

---

## Appendix

### Useful Commands

**ChirpStack**:
```bash
# View logs
docker-compose logs -f chirpstack

# Restart services
docker-compose restart

# Stop all services
docker-compose down
```

**MQTT**:
```bash
# Subscribe to uplink data
mosquitto_sub -h localhost -t 'application/+/device/+/event/up' -v

# Test MQTT connection
mosquitto_pub -h localhost -t 'test' -m 'hello'
```

**InfluxDB**:
```bash
# Query recent data
influx query 'from(bucket:"iiot") |> range(start: -1h) |> limit(n:10)'

# Show measurements
influx query 'import "influxdata/influxdb/schema" schema.measurements(bucket: "iiot")'
```

### Contact & Support

**GitHub Issues**: https://github.com/mapfumo/wk10-lorawan/issues
**Email**: [Your contact]
**Documentation**: See README.md, NOTES.md, docs/

---

**Version**: 1.0
**Last Updated**: 2026-01-03

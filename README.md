# Week 10: LoRaWAN Migration with STM32WL55

**Project**: LoRaWAN Migration - Native STM32WL integrated LoRaWAN stack
**Hardware**: 2x NUCLEO-WL55JC1 with RAK7268V2 Gateway
**Network Server**: ChirpStack (AU915)
**Status**: 📋 Starting - Hardware arriving Monday

---

## System Overview

Week 10 migrates from point-to-point LoRa (RYLR998) to production LoRaWAN infrastructure using native STM32WL radio peripherals. This creates a unified 4-node monitoring system combining Week 9's Modbus TCP devices with new LoRaWAN nodes.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         UNIFIED 4-NODE SYSTEM                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ETHERNET NODES (Week 9)          LORAWAN NODES (Week 10)           │
│  ┌──────────────────────┐         ┌──────────────────────┐          │
│  │ MODBUS_1             │         │ WL55 Node 1          │          │
│  │ F446RE + W5500       │         │ STM32WL55JC1         │          │
│  │ 10.10.10.100:502     │         │ BME688 + OLED        │          │
│  │ SHT3x sensor         │         │ DevEUI: xxx...       │          │
│  └──────────────────────┘         └──────────────────────┘          │
│           │                                 │                        │
│           │                                 │ LoRaWAN               │
│           │ Modbus TCP                      │ AU915                 │
│           │                                 ▼                        │
│  ┌──────────────────────┐         ┌──────────────────────┐          │
│  │ MODBUS_2             │         │ RAK7268V2 Gateway    │          │
│  │ F446RE + W5500       │         │ 8-channel AU915      │          │
│  │ 10.10.10.200:502     │         │ ChirpStack Bridge    │          │
│  │ SHT3x sensor         │         └──────────────────────┘          │
│  └──────────────────────┘                   │                        │
│           │                                 │                        │
│           ▼                                 ▼                        │
│  ┌─────────────────────────────────────────────────┐                │
│  │         OPC-UA Server (opcua_modbus_gateway.py) │                │
│  │         ChirpStack MQTT Bridge                  │                │
│  └─────────────────────────────────────────────────┘                │
│                          │                                           │
│                          ▼                                           │
│  ┌─────────────────────────────────────────────────┐                │
│  │  MQTT Broker (Mosquitto)                        │                │
│  └─────────────────────────────────────────────────┘                │
│                          │                                           │
│                          ▼                                           │
│  ┌─────────────────────────────────────────────────┐                │
│  │  InfluxDB (Time-series Database)                │                │
│  └─────────────────────────────────────────────────┘                │
│                          │                                           │
│                          ▼                                           │
│  ┌─────────────────────────────────────────────────┐                │
│  │  Grafana Dashboard (4-node unified view)        │                │
│  │  - 2x Modbus nodes (SHT3x)                      │                │
│  │  - 2x LoRaWAN nodes (BME688 + SHT41)            │                │
│  └─────────────────────────────────────────────────┘                │
│                                                                       │
│  ┌──────────────────────┐                                            │
│  │ WL55 Node 2          │                                            │
│  │ STM32WL55JC1         │                                            │
│  │ SHT41 + OLED         │                                            │
│  │ DevEUI: yyy...       │                                            │
│  └──────────────────────┘                                            │
│           │                                                           │
│           └─────────────► LoRaWAN AU915 ──────────┘                  │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Hardware Configuration

### Node 1 - BME688 Environmental Sensor
- **MCU**: STM32WL55JC1 (NUCLEO-WL55JC1)
  - 256KB Flash, 64KB RAM
  - Integrated SubGHz radio (LoRaWAN)
- **Sensor**: BME688 (I2C 0x76/0x77)
  - Temperature, Humidity, Pressure, Gas Resistance, IAQ
  - Payload: ~12 bytes
- **Display**: SSD1306 OLED 128x64 (I2C 0x3C)
  - Node ID, sensor readings, join status, RSSI/SNR

### Node 2 - SHT41 High-Precision Sensor
- **MCU**: STM32WL55JC1 (NUCLEO-WL55JC1)
- **Sensor**: SHT41 (I2C 0x44)
  - High-precision Temperature & Humidity
  - Payload: ~4 bytes
- **Display**: SSD1306 OLED 128x64 (I2C 0x3C)
  - Node ID, sensor readings, join status, RSSI/SNR

### Gateway
- **Model**: RAK7268V2 WisGate Edge Lite 2
- **Channels**: 8-channel LoRaWAN concentrator
- **Frequency**: AU915 (Australia/New Zealand)
- **Connectivity**: LAN + WiFi
- **Status**: Already configured and operational

---

## Pin Connections

### STM32WL55JC1 - Node 1 (BME688)

| Peripheral | Pin | Function | Device |
|------------|-----|----------|--------|
| I2C1 SDA | PB9 | I2C Data | BME688 (0x76/0x77) + OLED (0x3C) |
| I2C1 SCL | PB8 | I2C Clock | Shared bus |
| SubGHz Radio | Internal | LoRaWAN | 868/915 MHz |
| VDD | 3.3V | Power | All peripherals |
| GND | GND | Ground | All peripherals |

### STM32WL55JC1 - Node 2 (SHT41)

| Peripheral | Pin | Function | Device |
|------------|-----|----------|--------|
| I2C1 SDA | PB9 | I2C Data | SHT41 (0x44) + OLED (0x3C) |
| I2C1 SCL | PB8 | I2C Clock | Shared bus |
| SubGHz Radio | Internal | LoRaWAN | 868/915 MHz |
| VDD | 3.3V | Power | All peripherals |
| GND | GND | Ground | All peripherals |

---

## LoRaWAN Configuration

### Network Parameters
- **Region**: AU915 (sub-band 2 for TTN compatibility)
- **Activation**: OTAA (Over-The-Air Activation)
- **Device Class**: Class A (bi-directional with scheduled RX windows)
- **LoRaWAN Version**: 1.0.3
- **Spreading Factor**: SF7-SF12 (adaptive)
- **Duty Cycle**: Compliant with AU915 regulations

### Device Registration (ChirpStack)
Each node requires:
- **DevEUI**: Unique 64-bit identifier (from STM32WL)
- **AppEUI**: Application identifier (from ChirpStack)
- **AppKey**: 128-bit encryption key (from ChirpStack)

---

## Building and Flashing

### Prerequisites
```bash
# Install probe-rs
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Verify STM32WL55 support
probe-rs chip list | grep STM32WL55
```

### Build Firmware

**Node 1 (BME688):**
```bash
cd firmware/node1-bme688
cargo build --release
```

**Node 2 (SHT41):**
```bash
cd firmware/node2-sht41
cargo build --release
```

### Flash to Hardware

**Node 1:**
```bash
cd firmware/node1-bme688
probe-rs run --chip STM32WL55JCIx --release
```

**Node 2:**
```bash
cd firmware/node2-sht41
probe-rs run --chip STM32WL55JCIx --release
```

### Monitor Logs

**Using defmt-rtt:**
```bash
probe-rs attach --chip STM32WL55JCIx
```

---

## ChirpStack Setup

### Installation (Docker)
```bash
# Clone ChirpStack Docker Compose
git clone https://github.com/chirpstack/chirpstack-docker.git
cd chirpstack-docker

# Edit configuration for AU915
nano configuration/chirpstack/region_au915_0.toml

# Start services
docker-compose up -d
```

### Access Web UI
- **URL**: http://localhost:8080
- **Username**: admin
- **Password**: admin

### Device Registration Steps
1. Create Application: "IIoT-Sensors"
2. Create Device Profile: Class A, OTAA, AU915, LoRaWAN 1.0.3
3. Add Device 1: "WL55-BME688" with DevEUI
4. Add Device 2: "WL55-SHT41" with DevEUI
5. Configure MQTT integration

---

## Gateway Service

### LoRaWAN MQTT Bridge

The Python gateway service subscribes to ChirpStack MQTT topics and writes decoded sensor data to InfluxDB.

**Run the gateway:**
```bash
cd gateway
python3 lorawan_mqtt_bridge.py
```

**MQTT Topics:**
- `application/+/device/+/event/up` - Uplink data
- `application/+/device/+/event/join` - Join events
- `application/+/device/+/event/status` - Status updates

---

## Payload Encoding

### Node 1 - BME688 (~12 bytes)
```
Byte 0-3:   Temperature (f32, IEEE 754)
Byte 4-7:   Humidity (f32, IEEE 754)
Byte 8-9:   Pressure (u16, hPa * 10)
Byte 10-11: Gas Resistance (u16, kΩ)
```

### Node 2 - SHT41 (~4 bytes)
```
Byte 0-1: Temperature (i16, °C * 100)
Byte 2-3: Humidity (u16, % * 100)
```

---

## Testing

### Verify Gateway Connectivity
```bash
# Check RAK7268V2 is reachable
ping <gateway-ip>

# Check ChirpStack logs
docker logs -f chirpstack
```

### Monitor Device Join
1. Power on STM32WL55 node
2. Watch OLED display: "JOINING..." → "JOINED"
3. Check ChirpStack dashboard for join event
4. Verify device appears in "Active Devices"

### Test Uplink Data
1. Wait for first uplink (displayed on OLED)
2. Check ChirpStack "LoRaWAN Frames" tab
3. Verify payload decoding
4. Confirm data appears in InfluxDB
5. View in Grafana dashboard

### Range Testing
```bash
# Indoor baseline: 50m minimum
# Outdoor target: 500m+
# Monitor RSSI/SNR on OLED display
```

---

## Grafana Dashboard

### 4-Node Unified View

**Panels:**
- **Node 1 (BME688)**: Temperature, Humidity, Pressure, Gas Resistance, IAQ
- **Node 2 (SHT41)**: High-Precision Temperature, Humidity
- **Node 3 (MODBUS_1)**: Temperature, Humidity (SHT3x)
- **Node 4 (MODBUS_2)**: Temperature, Humidity (SHT3x)
- **Comparison**: Temperature across all 4 nodes
- **LoRaWAN Metrics**: RSSI, SNR, Spreading Factor, Duty Cycle

**Access**: http://localhost:3000

---

## Troubleshooting

### Device Won't Join
- Check DevEUI/AppEUI/AppKey match ChirpStack
- Verify AU915 frequency plan configured
- Check gateway is receiving packets (ChirpStack logs)
- Try different spreading factors (SF7-SF12)

### No Uplink Data
- Verify device shows "JOINED" status
- Check duty cycle limits (wait 60+ seconds between uplinks)
- Monitor LoRaWAN frame counter
- Check payload encoding matches decoder

### Poor RSSI/SNR
- Test with line-of-sight to gateway
- Try higher spreading factor (SF9, SF10)
- Check antenna connections
- Verify AU915 sub-band configuration

---

## Documentation

- [TODO.md](TODO.md) - Development task tracking
- [NOTES.md](NOTES.md) - Learning insights and design decisions
- [USERGUIDE.md](USERGUIDE.md) - Deployment and operation guide
- [docs/hardware-wiring.md](docs/hardware-wiring.md) - Detailed pin connections
- [docs/chirpstack-setup.md](docs/chirpstack-setup.md) - ChirpStack installation
- [docs/architecture.md](docs/architecture.md) - System architecture design

---

## Key Learning Goals

- STM32WL SubGHz radio peripheral programming
- LoRaWAN OTAA join procedure and session key derivation
- ChirpStack network server administration
- Multi-protocol system integration (Ethernet + LoRaWAN)
- LoRaWAN duty cycle compliance and regulations
- Payload optimization for LPWAN networks
- Embassy async framework for STM32WL

---

## Deliverables

- [x] RAK7268V2 gateway deployed and configured (AU915)
- [ ] ChirpStack network server operational
- [ ] 2x STM32WL55 LoRaWAN nodes (OTAA, Class A)
- [ ] MQTT integration to InfluxDB
- [ ] Unified 4-node Grafana dashboard
- [ ] Complete documentation set
- [ ] Performance metrics report

---

**Status**: 📋 Ready to start - Hardware arriving Monday
**Last Updated**: 2026-01-03

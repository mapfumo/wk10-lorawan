# Week 10: LoRaWAN Migration with STM32WL55

**Project**: LoRaWAN Migration - Native STM32WL integrated LoRaWAN stack
**Hardware**: 2x NUCLEO-WL55JC1 with RAK7268V2 Gateway
**Network Server**: RAK Built-in LoRa Server (AU915) ✅ Operational
**Status**: 🚧 In Progress - Gateway ✅ | NODE_1 Hardware ✅

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

### Node 1 - SHT41 High-Precision Sensor ✅ WORKING
- **MCU**: STM32WL55JC1 (NUCLEO-WL55JC1)
  - Serial: 003E00463234510A33353533
  - 256KB Flash, 64KB RAM
  - Integrated SubGHz radio (LoRaWAN)
- **Sensor**: SHT41 (I2C 0x44)
  - High-precision Temperature & Humidity
  - Current readings: 27°C, 60% RH
  - Payload: ~4 bytes
- **Display**: SH1106 OLED 128x64 (I2C 0x3C)
  - Real-time sensor data updates every 2s
  - Node ID, temperature, humidity, status
- **I2C Bus**: I2C2 (PA12=SCL, PA11=SDA) at 100 kHz
- **Connection**: Breadboard (STEMMA QT cables bypassed)
- **Status**: Hardware integration complete, ready for LoRaWAN stack

### Node 2 - BME688 Environmental Sensor ⏳ PLANNED
- **MCU**: STM32WL55JC1 (NUCLEO-WL55JC1)
  - Serial: 0026003A3234510A33353533
- **Sensor**: BME688 (I2C 0x76/0x77)
  - Temperature, Humidity, Pressure, Gas Resistance, IAQ
  - Payload: ~12 bytes
- **Display**: SH1106 OLED 128x64 (I2C 0x3C)
  - Node ID, sensor readings, join status, RSSI/SNR
- **Status**: To be configured next

### Gateway
- **Model**: RAK7268V2 WisGate Edge Lite 2
- **Gateway EUI**: `ac1f09fffe1bce23`
- **Channels**: 8-channel LoRaWAN concentrator (SX1302)
- **Frequency**: AU915 Sub-band 2 (915.2-916.6 MHz)
- **Network Server**: Built-in LoRa Server (NOT ChirpStack)
- **Connectivity**: LAN + WiFi
- **Status**: ✅ Configured and operational
- **Documentation**: [docs/rak7268v2-config.md](docs/rak7268v2-config.md)

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

### Device Registration (RAK Gateway)
Each node requires:
- **DevEUI**: Unique 64-bit identifier (from STM32WL)
- **AppEUI**: Application identifier (from gateway application "TOT")
- **AppKey**: 128-bit encryption key (from gateway application "TOT")

**Pre-configured Device**:
- DevEUI `23ce1bfeff091fac` already registered as "STM_Nodes" for LoRa-1
- Auto-add enabled: new OTAA devices will be automatically registered
- See [docs/rak7268v2-config.md](docs/rak7268v2-config.md) for credentials

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
cd firmware/lora-1
cargo build --release
```

**Node 2 (SHT41):**
```bash
cd firmware/lora-2
cargo build --release
```

### Flash to Hardware

**Node 1:**
```bash
cd firmware/lora-1
probe-rs run --chip STM32WL55JCIx --release
```

**Node 2:**
```bash
cd firmware/lora-2
probe-rs run --chip STM32WL55JCIx --release
```

### Monitor Logs

**Using defmt-rtt:**
```bash
probe-rs attach --chip STM32WL55JCIx
```

---

## Gateway MQTT Integration

### MQTT Broker

The RAK7268V2 runs a local MQTT broker at `127.0.0.1:1883` (on the gateway itself).

**Subscribe to Device Messages:**
```bash
# Subscribe to all messages from the TOT application
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/#" -v

# Subscribe to specific device uplinks
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/23ce1bfeff091fac/rx" -v

# Monitor gateway statistics
mosquitto_sub -h <gateway-ip> -t "gateway/ac1f09fffe1bce23/stats" -v
```

**MQTT Topics:**
- `application/TOT/device/23ce1bfeff091fac/join` - Join events
- `application/TOT/device/23ce1bfeff091fac/rx` - Uplink data (Base64 encoded)
- `application/TOT/device/23ce1bfeff091fac/ack` - Downlink acknowledgments
- `application/TOT/device/23ce1bfeff091fac/status` - Device status updates

### Python Bridge to InfluxDB

A Python service will subscribe to the gateway's MQTT topics and write decoded sensor data to InfluxDB for visualization in Grafana.

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

# Subscribe to gateway stats via MQTT
mosquitto_sub -h <gateway-ip> -t "gateway/ac1f09fffe1bce23/stats" -v

# SSH into gateway to check LoRa server
ssh root@<gateway-ip>
loraserver_status
```

### Monitor Device Join
1. Power on STM32WL55 node
2. Watch OLED display: "JOINING..." → "JOINED"
3. Subscribe to join events:
   ```bash
   mosquitto_sub -h <gateway-ip> -t "application/TOT/device/+/join" -v
   ```
4. Check gateway web UI for device activity

### Test Uplink Data
1. Wait for first uplink (displayed on OLED)
2. Subscribe to uplink messages:
   ```bash
   mosquitto_sub -h <gateway-ip> -t "application/TOT/device/23ce1bfeff091fac/rx" -v
   ```
3. Verify payload appears (Base64 encoded)
4. Decode payload and confirm sensor data matches OLED display
5. Check data appears in InfluxDB and Grafana dashboard

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
- Check DevEUI/AppEUI/AppKey match gateway configuration (see [docs/rak7268v2-config.md](docs/rak7268v2-config.md))
- Verify AU915 sub-band 2 (915.2-916.6 MHz) configured on device
- Check gateway is receiving packets: `mosquitto_sub -h <gateway-ip> -t "gateway/+/rx" -v`
- SSH to gateway and check logs: `logread | grep lora`
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
- [HARDWARE_CONFIG.md](HARDWARE_CONFIG.md) - Complete hardware configuration (sensors, displays, I2C)
- [docs/rak7268v2-config.md](docs/rak7268v2-config.md) - Gateway configuration and credentials
- [docs/hardware-wiring.md](docs/hardware-wiring.md) - Detailed pin connections
- [docs/architecture.md](docs/architecture.md) - System architecture design
- [USERGUIDE.md](USERGUIDE.md) - Deployment and operation guide (TBD)

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

- [x] **Gateway Configuration Complete**
  - [x] RAK7268V2 deployed and configured (AU915 Sub-band 2)
  - [x] Built-in LoRa Server operational (NOT ChirpStack)
  - [x] Application "TOT" created with OTAA credentials
  - [x] Device "STM_Nodes" pre-registered (DevEUI: 23ce1bfeff091fac)
  - [x] MQTT broker accessible at 127.0.0.1:1883 on gateway
  - [x] Gateway configuration fully documented

- [x] **LoRa-1 Hardware Complete**
  - [x] STM32WL55 board verified (Serial: 003E00463234510A33353533)
  - [x] SHT41 sensor working (31°C, 57% RH)
  - [x] SSD1306 OLED 128x32 displaying real-time data
  - [x] I2C2 bus working (PA12/PA11)
  - [x] Display updating every 2 seconds
  - [x] Firmware renamed to lora-1

- [ ] **LoRa-1 LoRaWAN Implementation** (NEXT STEP)
  - [ ] Add LoRaWAN library dependency
  - [ ] Initialize STM32WL SubGHz radio
  - [ ] Implement OTAA join with documented credentials
  - [ ] Send sensor data uplinks
  - [ ] Display join/TX status on OLED

- [ ] **LoRa-2 Setup**
  - [x] STM32WL55 board verified (Serial: 0026003A3234510A33353533)
  - [ ] BME688 sensor integration
  - [ ] SH1106 OLED 128x64 integration
  - [ ] LoRaWAN implementation

- [ ] **Backend Integration**
  - [ ] Python MQTT bridge to decode payloads
  - [ ] InfluxDB integration
  - [ ] Unified 4-node Grafana dashboard

- [x] **Documentation**
  - [x] Hardware configuration documented (HARDWARE_CONFIG.md)
  - [x] Gateway config documented (docs/rak7268v2-config.md)
  - [x] Development notes with technical learnings (NOTES.md)
  - [ ] Performance metrics report

---

**Status**: 🚧 In Progress - Sensor integration complete, LoRaWAN next
**Last Updated**: 2026-01-08

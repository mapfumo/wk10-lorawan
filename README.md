# Week 10: LoRaWAN Sensor Network with STM32WL55

**Project**: LoRaWAN Sensor Network - Native STM32WL integrated radio
**Hardware**: 2x NUCLEO-WL55JC1 + RAK7268V2 Gateway
**Network Server**: RAK Built-in LoRa Server (AU915)
**Status**: ✅ **COMPLETE** - Both nodes operational, Grafana dashboard live

---

## System Overview

Week 10 implements a production LoRaWAN sensor network using STM32WL55 microcontrollers with native SubGHz radio. The system includes a complete data pipeline from sensors to Grafana visualization.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     LORAWAN SENSOR NETWORK                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────┐         ┌──────────────────────┐              │
│  │ LoRa-1               │         │ LoRa-2               │              │
│  │ STM32WL55JC1         │         │ STM32WL55JC1         │              │
│  │ SHT41 (Temp/Hum)     │         │ BME680 (Env Sensor)  │              │
│  │ SSD1306 OLED 128x32  │         │ SH1106 OLED 128x64   │              │
│  │ DevEUI: 23ce1b...    │         │ DevEUI: 24ce1b...    │              │
│  └──────────┬───────────┘         └──────────┬───────────┘              │
│             │                                 │                          │
│             │         LoRaWAN AU915           │                          │
│             │        (915.2-916.6 MHz)        │                          │
│             └────────────────┬────────────────┘                          │
│                              │                                           │
│                              ▼                                           │
│                    ┌──────────────────────┐                             │
│                    │ RAK7268V2 Gateway    │                             │
│                    │ Built-in LoRa Server │                             │
│                    │ MQTT: 10.10.10.254   │                             │
│                    │ Application: "TOT"   │                             │
│                    └──────────┬───────────┘                             │
│                               │                                          │
│                               │ MQTT (TCP :1883)                        │
│                               ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     Docker Services                              │   │
│  │  ┌─────────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │
│  │  │ wk10-mqtt-bridge│─▶│ wk7-influxdb│─▶│ wk7-grafana         │  │   │
│  │  │ Python decoder  │  │ Time-series │  │ Dashboard           │  │   │
│  │  │                 │  │ :8086       │  │ :3000               │  │   │
│  │  └─────────────────┘  └─────────────┘  └─────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Hardware Configuration

### LoRa-1 - SHT41 Temperature/Humidity Sensor ✅ OPERATIONAL

| Component | Specification |
|-----------|---------------|
| **MCU** | STM32WL55JC1 (Probe: 003E00463234510A33353533) |
| **Sensor** | SHT41 (I2C 0x44) - High-precision temp/humidity |
| **Display** | SSD1306 OLED 128x32 (I2C 0x3C) |
| **I2C Bus** | I2C2: PA12 (SCL), PA11 (SDA) |
| **Radio** | Integrated SubGHz SX126x |
| **DevEUI** | 23ce1bfeff091fac |
| **Payload** | 4 bytes (temp + humidity) |

**Current Readings**: ~31°C, 58% RH
**Uplink Interval**: ~30 seconds

### LoRa-2 - BME680 Environmental Sensor ✅ OPERATIONAL

| Component | Specification |
|-----------|---------------|
| **MCU** | STM32WL55JC1 (Probe: 0026003A3234510A33353533) |
| **Sensor** | BME680 (I2C 0x76) - Temp/Hum/Pressure/Gas |
| **Display** | SH1106 OLED 128x64 (I2C 0x3C) |
| **I2C Bus** | I2C2: PA12 (SCL), PA11 (SDA) |
| **Radio** | Integrated SubGHz SX126x |
| **DevEUI** | 24ce1bfeff091fac |
| **Payload** | 12 bytes (temp + humidity + pressure + gas + padding) |

**Current Readings**: ~28°C, 60% RH, 1020 hPa, 134 kOhm gas resistance
**Uplink Interval**: ~30 seconds

### Gateway - RAK7268V2 WisGate Edge Lite 2 ✅ OPERATIONAL

| Parameter | Value |
|-----------|-------|
| **Gateway EUI** | ac1f09fffe1bce23 |
| **IP Address** | 10.10.10.254 |
| **Region** | AU915 Sub-band 2 (915.2-916.6 MHz) |
| **Network Server** | Built-in LoRa Server (NOT ChirpStack) |
| **MQTT Broker** | 10.10.10.254:1883 (no auth) |
| **Application** | "TOT" |

---

## Data Pipeline

### End-to-End Flow

```
Sensor → STM32WL55 → LoRaWAN → RAK Gateway → MQTT → Python Bridge → InfluxDB → Grafana
```

### Components

1. **MQTT Bridge** ([mqtt_to_influx.py](mqtt_to_influx.py))
   - Subscribes to gateway MQTT (10.10.10.254:1883)
   - Decodes Base64 LoRaWAN payloads
   - Writes to InfluxDB with tags (node, sensor, dev_eui)

2. **InfluxDB** (wk7-influxdb container)
   - Bucket: `lorawan`
   - Measurement: `lorawan_sensor`
   - Fields: temperature, humidity, pressure, gas_resistance, rssi, snr

3. **Grafana Dashboard** (wk7-grafana container)
   - URL: http://localhost:3000/d/lorawan-sensors/lorawan-sensor-network
   - 10 panels: Temperature, Humidity, Pressure, Gas, RSSI, SNR, Stats

---

## Building and Flashing

### Prerequisites

```bash
# Install probe-rs
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Add Rust embedded target
rustup target add thumbv7em-none-eabihf
```

### Build and Flash

**LoRa-1:**
```bash
cd firmware/lora-1
cargo run --release
```

**LoRa-2:**
```bash
cd firmware/lora-2
cargo run --release
```

### Monitor RTT Logs

```bash
probe-rs attach --chip STM32WL55JCIx
```

---

## Running the Data Pipeline

### Start Infrastructure

```bash
# Start Week 7 services (InfluxDB + Grafana)
cd ~/dev/4-month-plan/wk7-mqtt-influx
docker compose up -d

# Start MQTT bridge
cd ~/dev/4-month-plan/wk10-lorawan
docker compose up -d
```

### Verify Data Flow

```bash
# Check MQTT bridge logs
docker logs -f wk10-mqtt-bridge

# Expected output:
# [10:05:15] lora1: Temp=31.0C Hum=58.0% RSSI=-13 SNR=13.2 -> InfluxDB: OK
# [10:05:51] lora2: Temp=28.0C Hum=60.0% Press=1020.0hPa Gas=135kOhm RSSI=-17 SNR=12.0 -> InfluxDB: OK
```

### Access Dashboard

- **Grafana**: http://localhost:3000 (admin/admin)
- **Dashboard**: Dashboards → LoRaWAN Sensor Network

---

## Payload Encoding

### LoRa-1 (SHT41) - 4 bytes

| Bytes | Type | Description | Decode |
|-------|------|-------------|--------|
| 0-1 | i16 BE | Temperature × 100 | `value / 100` = °C |
| 2-3 | u16 BE | Humidity × 100 | `value / 100` = % |

### LoRa-2 (BME680) - 12 bytes

| Bytes | Type | Description | Decode |
|-------|------|-------------|--------|
| 0-1 | i16 BE | Temperature × 100 | `value / 100` = °C |
| 2-3 | u16 BE | Humidity × 100 | `value / 100` = % |
| 4-5 | u16 BE | Pressure × 10 | `value / 10` = hPa |
| 6-7 | u16 BE | Gas Resistance | kOhm |
| 8-11 | - | Padding | (unused) |

### Decode Payloads

```bash
# Decode a Base64 payload
python3 decode_payload.py CowZyA==
# Output: Temp: 27.00°C, Humidity: 66.00%
```

---

## MQTT Topics

Subscribe to gateway MQTT for debugging:

```bash
# All TOT application messages
mosquitto_sub -h 10.10.10.254 -t "application/TOT/device/#" -v

# LoRa-1 uplinks only
mosquitto_sub -h 10.10.10.254 -t "application/TOT/device/23ce1bfeff091fac/rx" -v

# LoRa-2 uplinks only
mosquitto_sub -h 10.10.10.254 -t "application/TOT/device/24ce1bfeff091fac/rx" -v

# Join events
mosquitto_sub -h 10.10.10.254 -t "application/TOT/device/+/join" -v
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [USERGUIDE.md](USERGUIDE.md) | Complete setup and operation guide |
| [HARDWARE_CONFIG.md](HARDWARE_CONFIG.md) | Detailed hardware specifications |
| [LORAWAN_CREDENTIALS.md](LORAWAN_CREDENTIALS.md) | Device credentials and EUIs |
| [docs/rak7268v2-config.md](docs/rak7268v2-config.md) | Gateway configuration |
| [TROUBLESHOOTING_WL55.md](TROUBLESHOOTING_WL55.md) | Common issues and solutions |
| [CLAUDE.md](CLAUDE.md) | Development notes and constraints |
| [NOTES.md](NOTES.md) | Technical learnings |

---

## Deliverables

### Gateway ✅ Complete
- [x] RAK7268V2 deployed and configured (AU915 Sub-band 2)
- [x] Built-in LoRa Server operational
- [x] Application "TOT" created with OTAA credentials
- [x] MQTT broker accessible at 10.10.10.254:1883

### LoRa-1 (SHT41) ✅ Complete
- [x] STM32WL55 hardware integration
- [x] SHT41 sensor reading (temperature, humidity)
- [x] SSD1306 OLED display (TX count, readings, SNR/RSSI)
- [x] LoRaWAN OTAA join
- [x] Uplink transmission (~30 second interval)
- [x] LED flash on transmission

### LoRa-2 (BME680) ✅ Complete
- [x] STM32WL55 hardware integration
- [x] BME680 sensor reading (temp, humidity, pressure, gas)
- [x] SH1106 OLED display (TX count, readings, SNR/RSSI)
- [x] LoRaWAN OTAA join
- [x] Uplink transmission (~30 second interval)
- [x] LED flash on transmission

### Data Pipeline ✅ Complete
- [x] Python MQTT bridge subscribing to gateway
- [x] Payload decoding (both node formats)
- [x] InfluxDB bucket and data writes
- [x] Grafana datasource configured
- [x] Dashboard with 10 panels

### Documentation ✅ Complete
- [x] USERGUIDE.md - Comprehensive deployment guide
- [x] Hardware configuration documented
- [x] Gateway configuration documented
- [x] Troubleshooting guide

---

## Key Learnings

1. **LoRaWAN Byte Order**: EUIs must be reversed (little-endian) in firmware
2. **No FPU**: STM32WL55 requires integer-only math
3. **SHT41 Wake-up**: Sensor needs measurement command before I2C scan
4. **MQTT 3.1 Protocol**: RAK gateway requires older MQTT protocol version
5. **Peripheral Stealing**: Embassy async pattern for I2C sharing

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Join Time | ~7 seconds |
| Uplink Interval | ~30 seconds |
| LoRa-1 Payload | 4 bytes |
| LoRa-2 Payload | 12 bytes |
| Typical RSSI | -15 to -80 dBm |
| Typical SNR | -10 to +14 dB |
| End-to-End Latency | <2 seconds |

---

**Status**: ✅ Week 10 Complete
**Last Updated**: 2026-01-09

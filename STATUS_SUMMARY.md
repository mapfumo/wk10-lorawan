# Week 10 LoRaWAN - Current Status

**Date**: 2026-01-09
**Phase**: ✅ **COMPLETE** - Both nodes operational, Grafana dashboard live

---

## ✅ Completed Work

### 1. Gateway Configuration (100% Complete)

The RAK7268V2 WisGate Edge Lite 2 gateway is fully operational:

- **Gateway EUI**: `ac1f09fffe1bce23`
- **Region**: AU915 Sub-band 2 (915.2-916.6 MHz)
- **Network Server**: Built-in LoRa Server (NOT ChirpStack)
- **MQTT Broker**: 10.10.10.254:1883 (no auth)

**Application "TOT" Configuration**:

- AppEUI: `b130a864c5295356`
- AppKey: `b726739b78ec4b9e9234e5d35ea9681b`
- Auto-add: Enabled (OTAA devices auto-register)
- Data encoding: Base64

**Documentation**: Complete configuration in [docs/rak7268v2-config.md](docs/rak7268v2-config.md)

### 2. LoRa-1 - SHT41 Node (100% Complete)

**Hardware**:

- STM32WL55JC1 (Serial: 003E00463234510A33353533)
- SHT41 temperature/humidity sensor (I2C 0x44)
- SSD1306 OLED 128x32 (I2C 0x3C)
- I2C2 bus: PA12 (SCL), PA11 (SDA)

**Firmware Status**:

- LoRaWAN OTAA join successful
- DevEUI: 23ce1bfeff091fac
- Uplinks every ~30 seconds
- 4-byte payload (temp + humidity)
- Display: TX count, readings, SNR/RSSI
- LED flash on transmission

**Flash Command**:

```bash
cd firmware/lora-1
cargo run --release
```

### 3. LoRa-2 - BME680 Node (100% Complete)

**Hardware**:

- STM32WL55JC1 (Serial: 0026003A3234510A33353533)
- BME680 environmental sensor (I2C 0x76)
- SH1106 OLED 128x64 (I2C 0x3C)
- I2C2 bus: PA12 (SCL), PA11 (SDA)

**Firmware Status**:

- LoRaWAN OTAA join successful
- DevEUI: 24ce1bfeff091fac
- Uplinks every ~30 seconds
- 12-byte payload (temp + hum + pressure + gas)
- Display: TX count, readings, SNR/RSSI
- LED flash on transmission

**Flash Command**:

```bash
cd firmware/lora-2
cargo run --release
```

### 4. Data Pipeline (100% Complete)

**MQTT Bridge** (wk10-mqtt-bridge container):

- Python script subscribing to gateway MQTT
- Decodes Base64 LoRaWAN payloads
- Writes to InfluxDB (bucket: lorawan)
- Handles both 4-byte and 12-byte formats

**InfluxDB** (wk7-influxdb container):

- Bucket: `lorawan`
- Measurement: `lorawan_sensor`
- Fields: temperature, humidity, pressure, gas_resistance, rssi, snr

**Grafana Dashboard** (wk7-grafana container):

- URL: http://localhost:3000/d/lorawan-sensors/lorawan-sensor-network
- 10 panels: Temperature, Humidity, Pressure, Gas, RSSI, SNR, Stats

### 5. Documentation (100% Complete)

- ✅ [README.md](README.md) - System overview with architecture diagram
- ✅ [USERGUIDE.md](USERGUIDE.md) - Comprehensive deployment guide
- ✅ [HARDWARE_CONFIG.md](HARDWARE_CONFIG.md) - Hardware specifications
- ✅ [LORAWAN_CREDENTIALS.md](LORAWAN_CREDENTIALS.md) - Device credentials
- ✅ [docs/rak7268v2-config.md](docs/rak7268v2-config.md) - Gateway configuration
- ✅ [TROUBLESHOOTING_WL55.md](TROUBLESHOOTING_WL55.md) - Common issues
- ✅ [NOTES.md](NOTES.md) - Technical implementation details
- ✅ [CLAUDE.md](CLAUDE.md) - Development guidance

---

## 📊 System Architecture

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

## 📈 Performance Metrics

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

## 🔧 Quick Start Commands

### Start Infrastructure

```bash
# Start Week 7 services (InfluxDB + Grafana)
cd ~/dev/4-month-plan/wk7-mqtt-influx
docker compose up -d

# Start MQTT bridge
cd ~/dev/4-month-plan/wk10-lorawan
docker compose up -d
```

### Flash Firmware

```bash
# LoRa-1
cd ~/dev/4-month-plan/wk10-lorawan/firmware/lora-1
cargo run --release

# LoRa-2
cd ~/dev/4-month-plan/wk10-lorawan/firmware/lora-2
cargo run --release
```

### Monitor Data

```bash
# Check MQTT bridge logs
docker logs -f wk10-mqtt-bridge

# Subscribe to gateway MQTT
mosquitto_sub -h 10.10.10.254 -t "application/TOT/device/#" -v
```

### Access Dashboard

- **Grafana**: http://localhost:3000 (admin/admin)
- **Dashboard**: Dashboards → LoRaWAN Sensor Network

---

## 🎓 Key Learnings

1. **LoRaWAN Byte Order**: EUIs must be reversed (little-endian) in firmware
2. **No FPU**: STM32WL55 requires integer-only math
3. **SHT41 Wake-up**: Sensor needs measurement command before I2C scan
4. **MQTT 3.1 Protocol**: RAK gateway requires older MQTT protocol version
5. **Peripheral Stealing**: Embassy async pattern for I2C sharing

---

**Status**: ✅ Week 10 Complete - Production-ready LoRaWAN sensor network
**Last Updated**: 2026-01-09

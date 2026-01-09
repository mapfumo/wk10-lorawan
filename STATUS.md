# Week 10: Current Status Summary

**Date**: 2026-01-09
**Status**: ✅ **COMPLETE** - Both nodes operational, full data pipeline live

---

## ✅ Completed Milestones

### Gateway Configuration (100% Complete)
- ✅ RAK7268V2 deployed and configured (AU915 Sub-band 2)
- ✅ Built-in LoRa Server operational (NOT ChirpStack)
- ✅ Application "TOT" created with OTAA credentials
- ✅ MQTT broker accessible at 10.10.10.254:1883
- ✅ Both devices registered and auto-joining

### LoRa-1 - SHT41 Node (100% Complete)
- ✅ STM32WL55JC1 (Probe: 003E00463234510A33353533)
- ✅ SHT41 temperature/humidity sensor (I2C 0x44)
- ✅ SSD1306 OLED 128x32 display (I2C 0x3C)
- ✅ LoRaWAN OTAA join successful (DevEUI: 23ce1bfeff091fac)
- ✅ Uplinks transmitting every ~30 seconds
- ✅ 4-byte payload encoding (temp + humidity)
- ✅ Display showing TX count, readings, SNR/RSSI

### LoRa-2 - BME680 Node (100% Complete)
- ✅ STM32WL55JC1 (Probe: 0026003A3234510A33353533)
- ✅ BME680 environmental sensor (I2C 0x76)
- ✅ SH1106 OLED 128x64 display (I2C 0x3C)
- ✅ LoRaWAN OTAA join successful (DevEUI: 24ce1bfeff091fac)
- ✅ Uplinks transmitting every ~30 seconds
- ✅ 12-byte payload encoding (temp + hum + pressure + gas)
- ✅ Display showing TX count, readings, SNR/RSSI

### Data Pipeline (100% Complete)
- ✅ Python MQTT bridge (wk10-mqtt-bridge container)
- ✅ Subscribes to gateway MQTT (10.10.10.254:1883)
- ✅ Decodes Base64 payloads (both node formats)
- ✅ Writes to InfluxDB (bucket: lorawan)
- ✅ Grafana dashboard with 10 panels

### Documentation (100% Complete)
- ✅ README.md - Complete system overview
- ✅ USERGUIDE.md - Comprehensive deployment guide
- ✅ HARDWARE_CONFIG.md - Hardware specifications
- ✅ LORAWAN_CREDENTIALS.md - Device credentials
- ✅ docs/rak7268v2-config.md - Gateway configuration
- ✅ TROUBLESHOOTING_WL55.md - Common issues
- ✅ CLAUDE.md - Development guidance

---

## 🔍 Technical Achievements

### 1. LoRaWAN Byte Order Mastery
**Problem**: Gateway parsed DevEUI as its own EUI
**Root Cause**: LoRaWAN transmits EUIs in little-endian, but displays big-endian
**Solution**: Reverse DevEUI and AppEUI bytes in firmware (NOT AppKey)
```rust
// Gateway shows: 23ce1bfeff091fac
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23]; // REVERSED
```

### 2. No FPU Constraint
**Problem**: STM32WL55 lacks FPU - float operations cause HardFault
**Solution**: Integer-only math for all sensor conversions
```rust
// SHT41: T = -45 + (175 × raw) / 65535
let temp_celsius: i16 = -45 + ((175 * temp_raw as i32) / 65535) as i16;
```

### 3. I2C Peripheral Sharing
**Challenge**: Sensor and display both need I2C2
**Solution**: Embassy async + unsafe peripheral stealing pattern
```rust
loop {
    let i2c = unsafe { I2c::new_blocking(I2C2::steal(), ...) };
    // Read sensor, then pass to display driver
    // Both dropped here, hardware released
}
```

### 4. MQTT Protocol Compatibility
**Problem**: Gateway MQTT broker uses MQTT 3.1 protocol
**Solution**: Use `MQIsdp` protocol name in MQTT client (paho-mqtt handles this)

---

## 📊 Current System Configuration

### Node Status
| Node | Sensor | Display | DevEUI | Status |
|------|--------|---------|--------|--------|
| LoRa-1 | SHT41 | SSD1306 128x32 | 23ce1bfeff091fac | ✅ Active |
| LoRa-2 | BME680 | SH1106 128x64 | 24ce1bfeff091fac | ✅ Active |

### Data Flow
```
LoRa-1/2 → LoRaWAN → RAK7268V2 → MQTT → Python Bridge → InfluxDB → Grafana
```

### Docker Services
| Container | Purpose | Port |
|-----------|---------|------|
| wk7-influxdb | Time-series database | 8086 |
| wk7-grafana | Dashboard | 3000 |
| wk10-mqtt-bridge | MQTT to InfluxDB | - |

### Typical Readings
| Metric | LoRa-1 | LoRa-2 |
|--------|--------|--------|
| Temperature | ~31°C | ~28°C |
| Humidity | ~58% | ~60% |
| Pressure | - | ~1020 hPa |
| Gas Resistance | - | ~135 kΩ |
| RSSI | -13 to -80 dBm | -17 to -80 dBm |
| SNR | 11-13 dB | 10-12 dB |

---

## 📈 Performance Metrics

### LoRaWAN
- **Join Time**: ~7 seconds
- **Uplink Interval**: ~30 seconds
- **Payload Sizes**: 4 bytes (LoRa-1), 12 bytes (LoRa-2)
- **Air Time**: ~1.3s @ SF12/BW125
- **End-to-End Latency**: <2 seconds

### Firmware
- **Flash Usage**: ~28KB (11% of 256KB)
- **RAM Usage**: ~8KB (12.5% of 64KB)
- **Sensor Read**: ~10ms
- **Display Update**: ~50ms

---

## 🔧 Build & Flash Commands

### LoRa-1
```bash
cd firmware/lora-1
cargo run --release
```

### LoRa-2
```bash
cd firmware/lora-2
cargo run --release
```

### Start Data Pipeline
```bash
# Start InfluxDB + Grafana (wk7)
cd ~/dev/4-month-plan/wk7-mqtt-influx
docker compose up -d

# Start MQTT bridge (wk10)
cd ~/dev/4-month-plan/wk10-lorawan
docker compose up -d
```

### Access Dashboard
- **Grafana**: http://localhost:3000 (admin/admin)
- **Dashboard**: Dashboards → LoRaWAN Sensor Network

---

## 📚 Key Documentation

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | System overview |
| [USERGUIDE.md](USERGUIDE.md) | Complete deployment guide |
| [HARDWARE_CONFIG.md](HARDWARE_CONFIG.md) | Hardware specs |
| [LORAWAN_CREDENTIALS.md](LORAWAN_CREDENTIALS.md) | Credentials |
| [TROUBLESHOOTING_WL55.md](TROUBLESHOOTING_WL55.md) | Common issues |

---

## 🎓 Key Learnings

1. **LoRaWAN byte order** - EUIs are little-endian over-the-air but displayed big-endian
2. **No FPU** - STM32WL55 requires integer-only math
3. **SHT41 wake-up** - Sensor needs measurement command before responding to I2C scan
4. **MQTT 3.1** - RAK gateway requires older protocol version
5. **Embassy async** - Peripheral stealing pattern for I2C sharing

---

**Status**: ✅ Week 10 Complete
**Last Updated**: 2026-01-09

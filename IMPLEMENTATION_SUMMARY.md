# Week 10 STM32WL55 LoRaWAN Implementation Summary

**Date**: 2026-01-09
**Status**: ✅ **FULLY OPERATIONAL** - Both nodes + data pipeline complete

---

## What Was Built

A complete 2-node LoRaWAN sensor network using STM32WL55JC microcontrollers with native SubGHz radio, plus a full data pipeline to Grafana visualization.

### System Architecture

```text
┌──────────────┐     ┌──────────────┐
│   LoRa-1     │     │   LoRa-2     │
│   SHT41      │     │   BME680     │
│  (4 bytes)   │     │  (12 bytes)  │
└──────┬───────┘     └──────┬───────┘
       │   LoRaWAN AU915    │
       └─────────┬──────────┘
                 ▼
       ┌─────────────────┐
       │  RAK7268V2      │
       │  Gateway        │
       │  10.10.10.254   │
       └────────┬────────┘
                │ MQTT :1883
                ▼
       ┌─────────────────┐
       │ mqtt_to_influx  │
       │ Python Bridge   │
       └────────┬────────┘
                │
                ▼
       ┌─────────────────┐
       │   InfluxDB      │
       │ bucket:lorawan  │
       └────────┬────────┘
                │
                ▼
       ┌─────────────────┐
       │    Grafana      │
       │  10 panels      │
       └─────────────────┘
```

---

## LoRa-1: SHT41 Temperature/Humidity Node

### Hardware

- **MCU**: STM32WL55JC (Cortex-M4 @ 48 MHz, HSE + PLL clock)
- **Sensor**: SHT41 temperature/humidity (I2C 0x44)
- **Display**: SSD1306 OLED 128x32 (I2C 0x3C)
- **Radio**: Integrated SubGHz SX126x LoRa radio
- **Board**: NUCLEO-WL55JC1 (probe ID: 003E00463234510A33353533)

### Firmware Stack

- **Framework**: Embassy async v0.7.0 (Rust embedded async)
- **LoRa PHY**: `lora-phy` v3.0 (SX126x driver)
- **LoRaWAN MAC**: `lorawan-device` v0.12 (OTAA, Class A)
- **Region**: AU915 Sub-band 2 (915.2-916.6 MHz, channels 8-15)

### Features

- ✅ LoRaWAN OTAA join (~7 seconds)
- ✅ SHT41 sensor reading (integer math only, no FPU)
- ✅ OLED display: TX count, temp/humidity, SNR/RSSI
- ✅ 4-byte uplink payload every ~30 seconds
- ✅ LED flash on transmission
- ✅ DevEUI: `23ce1bfeff091fac`

### Payload Format (4 bytes)

| Bytes | Type | Description | Decode |
|-------|------|-------------|--------|
| 0-1 | i16 BE | Temperature × 100 | `value / 100` = °C |
| 2-3 | u16 BE | Humidity × 100 | `value / 100` = % |

**Example**: 27°C, 66% → `[0x0A, 0x8C, 0x19, 0xC8]` → base64 `CowZyA==`

---

## LoRa-2: BME680 Environmental Sensor Node

### Hardware

- **MCU**: STM32WL55JC (Cortex-M4 @ 48 MHz, HSE + PLL clock)
- **Sensor**: BME680 environmental (I2C 0x76) - temp, humidity, pressure, gas
- **Display**: SH1106 OLED 128x64 (I2C 0x3C)
- **Radio**: Integrated SubGHz SX126x LoRa radio
- **Board**: NUCLEO-WL55JC1 (probe ID: 0026003A3234510A33353533)

### Firmware Stack

- **Framework**: Embassy async v0.7.0 (Rust embedded async)
- **LoRa PHY**: `lora-phy` v3.0 (SX126x driver)
- **LoRaWAN MAC**: `lorawan-device` v0.12 (OTAA, Class A)
- **Region**: AU915 Sub-band 2 (915.2-916.6 MHz, channels 8-15)

### Features

- ✅ LoRaWAN OTAA join (~7 seconds)
- ✅ BME680 sensor reading (integer math only, no FPU)
- ✅ OLED display: TX count, all readings, SNR/RSSI
- ✅ 12-byte uplink payload every ~30 seconds
- ✅ LED flash on transmission
- ✅ DevEUI: `24ce1bfeff091fac`

### Payload Format (12 bytes)

| Bytes | Type | Description | Decode |
|-------|------|-------------|--------|
| 0-1 | i16 BE | Temperature × 100 | `value / 100` = °C |
| 2-3 | u16 BE | Humidity × 100 | `value / 100` = % |
| 4-5 | u16 BE | Pressure × 10 | `value / 10` = hPa |
| 6-7 | u16 BE | Gas Resistance | kOhm |
| 8-11 | - | Padding | (unused) |

---

## Data Pipeline

### MQTT Bridge (mqtt_to_influx.py)

Python script running in Docker container:

- **Subscribes to**: `application/TOT/device/+/rx` on gateway MQTT
- **Decodes**: Base64 LoRaWAN payloads (auto-detects 4 or 12 byte format)
- **Writes to**: InfluxDB `lorawan` bucket

Key configuration:

```python
GATEWAY_MQTT_HOST = "10.10.10.254"
GATEWAY_MQTT_PORT = 1883
INFLUXDB_HOST = "wk7-influxdb"
INFLUXDB_BUCKET = "lorawan"
```

### InfluxDB Storage

- **Bucket**: `lorawan`
- **Measurement**: `lorawan_sensor`
- **Tags**: `node` (lora1/lora2), `sensor` (sht41/bme680), `dev_eui`
- **Fields**: temperature, humidity, pressure, gas_resistance, rssi, snr

### Grafana Dashboard

10 panels displaying:

1. Temperature (both nodes, time series)
2. Humidity (both nodes, time series)
3. Pressure (LoRa-2 only, time series)
4. Gas Resistance (LoRa-2 only, time series)
5. RSSI (both nodes, time series)
6. SNR (both nodes, time series)
7-10. Stat panels (latest values)

---

## Key Technical Challenges Solved

### 1. LoRaWAN Byte Order Shenanigans

**Problem**: Gateway parsed DevEUI as gateway's own EUI (`ac1f09fffe1bce23`)

**Root Cause**: LoRaWAN transmits EUIs in little-endian (LSB first), but `lorawan-device` crate expects arrays in transmission order.

**Solution**: Reverse DevEUI and AppEUI bytes (but NOT AppKey):

```rust
// Gateway shows: 23ce1bfeff091fac
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23]; // REVERSED
```

### 2. No Floating Point Unit (FPU)

**Problem**: STM32WL55 has NO FPU - using `f32`/`f64` causes HardFault

**Solution**: Integer-only math for sensor conversion:

```rust
// SHT41: T = -45 + (175 × raw) / 65535
let temp_celsius: i16 = -45 + ((175 * temp_raw as i32) / 65535) as i16;

// BME680: Similar integer-only conversions
let pressure_hpa: u16 = (raw_pressure / 100) as u16;  // Pa to hPa
```

### 3. I2C Peripheral Sharing

**Challenge**: Both sensor and display need I2C2

**Solution**: Embassy async + unsafe peripheral stealing:

```rust
loop {
    let mut i2c = unsafe { I2c::new_blocking(I2C2::steal(), ...) };
    // Read sensor
    let mut display = Ssd1306::new(I2CDisplayInterface::new(i2c), ...);
    // Update display
    // Both dropped here, hardware released
    Timer::after_secs(2).await;
}
```

### 4. RF Switch Control

**Required**: NUCLEO-WL55JC1 needs GPIO control for antenna routing

**Solution**: Custom `iv.rs` module implementing `InterfaceVariant`:

- PC3, PC4, PC5 control TX/RX paths
- High-power PA enabled for AU915

### 5. MQTT Protocol Compatibility

**Problem**: RAK gateway MQTT broker uses MQTT 3.1 protocol

**Solution**: Use paho-mqtt which handles protocol negotiation automatically

---

## Gateway Configuration

### RAK7268V2 Settings

- **Built-in LoRa Server** (NOT ChirpStack)
- **Application**: "TOT" (ID: 1)
- **Devices**: Auto-add enabled
- **MQTT Broker**: 10.10.10.254:1883 (no auth)

### MQTT Topic Structure

```bash
# Uplink data
application/TOT/device/{DevEUI}/rx

# Join events
application/TOT/device/{DevEUI}/join
```

### Sample MQTT Payload

```json
{
  "applicationID": "1",
  "applicationName": "TOT",
  "devEUI": "23ce1bfeff091fac",
  "deviceName": "STM_Nodes",
  "timestamp": 1767899338,
  "fCnt": 3,
  "fPort": 1,
  "data": "CowZyA==",
  "rxInfo": [{
    "gatewayID": "ac1f09fffe1bce23",
    "loRaSNR": 11.5,
    "rssi": -17
  }],
  "txInfo": {
    "frequency": 916600000,
    "dr": 0
  }
}
```

---

## Build and Flash Commands

### LoRa-1

```bash
cd firmware/lora-1
cargo build --release
cargo run --release  # Build + flash + RTT logs
```

### LoRa-2

```bash
cd firmware/lora-2
cargo build --release
cargo run --release  # Build + flash + RTT logs
```

### Start Data Pipeline

```bash
# Start InfluxDB + Grafana (from wk7)
cd ~/dev/4-month-plan/wk7-mqtt-influx
docker compose up -d

# Start MQTT bridge
cd ~/dev/4-month-plan/wk10-lorawan
docker compose up -d
```

### Decode Payload Manually

```bash
python3 decode_payload.py CowZyA==
# Output:
# Temperature: 27.00°C
# Humidity:    66.00%
```

---

## Performance Metrics

| Metric | LoRa-1 | LoRa-2 |
|--------|--------|--------|
| Join Time | ~7s | ~7s |
| Uplink Interval | ~30s | ~30s |
| Payload Size | 4 bytes | 12 bytes |
| Air Time | ~1.3s | ~1.5s |
| Sensor Read | ~10ms | ~15ms |
| Display Update | ~50ms | ~80ms |

### Link Quality (Typical)

- **RSSI**: -15 to -80 dBm
- **SNR**: 10-14 dB
- **End-to-End Latency**: <2 seconds

### Resource Usage

- **Flash**: ~28KB (11% of 256KB)
- **RAM**: ~8KB (12.5% of 64KB)

---

## Files Created/Modified

### Firmware

- `firmware/lora-1/Cargo.toml` - Dependencies
- `firmware/lora-1/src/main.rs` - LoRaWAN + SHT41 + SSD1306
- `firmware/lora-1/src/iv.rs` - RF switch control
- `firmware/lora-2/Cargo.toml` - Dependencies
- `firmware/lora-2/src/main.rs` - LoRaWAN + BME680 + SH1106
- `firmware/lora-2/src/iv.rs` - RF switch control

### Data Pipeline

- `mqtt_to_influx.py` - MQTT bridge script
- `docker-compose.yml` - Bridge container config
- `grafana/lorawan-dashboard.json` - Dashboard export

### Documentation

- `README.md` - System overview
- `USERGUIDE.md` - Complete deployment guide
- `HARDWARE_CONFIG.md` - Hardware specs
- `LORAWAN_CREDENTIALS.md` - Device credentials
- `CLAUDE.md` - Development guidance
- `TROUBLESHOOTING_WL55.md` - Common issues
- `decode_payload.py` - Payload decoder utility

---

## Lessons Learned

1. **Byte order matters!** LoRaWAN EUI little-endian transmission caught us off-guard
2. **No FPU** - Integer math only, required careful sensor conversion
3. **Embassy async** - Made peripheral sharing elegant with `unsafe` stealing
4. **Working reference invaluable** - Having reference projects saved hours
5. **Baby steps work** - Incremental changes with testing prevented cascading failures
6. **MQTT 3.1** - RAK gateway requires older protocol (handled by paho-mqtt)
7. **Documentation pays off** - CLAUDE.md guided entire implementation

---

**Status**: ✅ Production-ready LoRaWAN sensor network with full data pipeline
**Last Updated**: 2026-01-09

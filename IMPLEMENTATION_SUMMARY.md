# LoRa-1 STM32WL55 LoRaWAN Implementation Summary

**Date**: 2026-01-08
**Status**: ✅ **FULLY OPERATIONAL**

---

## What Was Built

A complete LoRaWAN sensor node using the STM32WL55JC microcontroller with native SubGHz radio peripheral.

### Hardware
- **MCU**: STM32WL55JC (Cortex-M4 @ 48 MHz, HSE + PLL clock)
- **Sensor**: SHT41 temperature/humidity (I2C 0x44)
- **Display**: SSD1306 OLED 128x32 (I2C 0x3C)
- **Radio**: Integrated SubGHz SX126x LoRa radio
- **Board**: NUCLEO-WL55JC1 (probe ID: 0483:374e:003E00463234510A33353533)

### Firmware Stack
- **Framework**: Embassy async v0.7.0 (Rust embedded async)
- **LoRa PHY**: `lora-phy` v3.0 (SX126x driver)
- **LoRaWAN MAC**: `lorawan-device` v0.12 (OTAA, Class A)
- **Region**: AU915 Sub-band 1 (915.2-916.6 MHz, channels 0-7)

---

## Features Implemented

### ✅ LoRaWAN OTAA Join
- Correct little-endian EUI byte order (critical!)
- 5 retry attempts with 10-second delays
- Successful join with device address `02806a22`
- Average join time: ~7 seconds

### ✅ Sensor Reading
- SHT41 high-precision mode (typ 8.3ms measurement)
- Temperature: -45°C to +125°C (integer math only, no FPU!)
- Humidity: 0-100% RH
- Read interval: Every 2 seconds

### ✅ OLED Display (SSD1306 128x32)
**3 lines displayed:**
1. `LoRa-1  TX:3` - Title + uplink counter
2. `27C 66%` - Temperature and humidity
3. `S:11 R:-17` - SNR (dB) and RSSI (dBm)

### ✅ LoRaWAN Uplinks
- **Interval**: Every 60 seconds
- **FPort**: 1 (application data)
- **Payload**: 4 bytes
  - Bytes 0-1: Temperature (°C × 100, signed 16-bit big-endian)
  - Bytes 2-3: Humidity (% × 100, unsigned 16-bit big-endian)
- **Confirmed**: No (unconfirmed uplinks)

**Example**: 27°C, 66% → `[0x0A, 0x8C, 0x19, 0xC8]` → base64 `CowZyA==`

### ✅ Link Quality Tracking
- SNR: 11.5 dB (excellent)
- RSSI: -17 dBm (very strong signal)
- Displayed on OLED after first uplink

---

## Key Technical Challenges Solved

### 1. LoRaWAN Byte Order Shenanigans 🎭
**Problem**: Gateway parsed DevEUI as gateway's own EUI (`ac1f09fffe1bce23`)

**Root Cause**: LoRaWAN transmits EUIs in little-endian (LSB first), but `lorawan-device` crate expects arrays in transmission order.

**Solution**: Reverse DevEUI and AppEUI bytes (but NOT AppKey):
```rust
// Gateway shows: 23ce1bfeff091fac
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23]; // REVERSED
```

See `CLAUDE.md` and `TROUBLESHOOTING_WL55.md` for full documentation.

### 2. No Floating Point Unit (FPU)
**Problem**: STM32WL55 has NO FPU - using `f32`/`f64` causes HardFault

**Solution**: Integer-only math for sensor conversion:
```rust
// SHT41: T = -45 + (175 × raw) / 65535
let temp_celsius: i16 = -45 + ((175 * temp_raw as i32) / 65535) as i16;
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
- Copied from working reference project

---

## Gateway Integration

### RAK7268V2 Configuration
- **Built-in LoRa Server** (NOT ChirpStack)
- **Application**: "TOT" (ID: 1)
- **Device**: "STM_Nodes" (DevEUI: 23ce1bfeff091fac)
- **Auto-add**: Enabled (new OTAA devices register automatically)

### MQTT Topics
```bash
# Join events
application/TOT/device/23ce1bfeff091fac/join

# Uplink data
application/TOT/device/23ce1bfeff091fac/rx

# Subscribe to all
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/#" -v
```

### Verified MQTT Payload
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
  "data_encode": "base64",
  "adr": false,
  "rxInfo": [
    {
      "gatewayID": "ac1f09fffe1bce23",
      "loRaSNR": 11.5,
      "rssi": -17,
      "location": {"latitude": 0.0, "longitude": 0.0, "altitude": 0}
    }
  ],
  "txInfo": {
    "frequency": 916600000,
    "dr": 0
  }
}
```

**Decoded**: Temperature 27.00°C, Humidity 66.00% ✅

---

## Build and Flash Commands

### Build
```bash
cd firmware/lora-1
cargo build --release
```

### Flash and Monitor
```bash
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:003E00463234510A33353533 target/thumbv7em-none-eabihf/release/lora-1
```

### Decode Payload
```bash
./decode_payload.py CowZyA==
# Output:
# Temperature: 27.00°C
# Humidity:    66.00%
```

---

## Performance Metrics

- **Join Time**: ~7 seconds
- **Uplink Interval**: 60 seconds
- **Payload Size**: 4 bytes
- **Air Time**: ~1.3 seconds @ SF12/BW125
- **Sensor Read Time**: ~10 ms
- **Display Update**: ~50 ms
- **Power**: Running from USB (not optimized for battery)

---

## Files Modified/Created

### Core Firmware
- `firmware/lora-1/Cargo.toml` - Updated dependencies to Embassy 0.7.x
- `firmware/lora-1/src/main.rs` - Complete LoRaWAN implementation
- `firmware/lora-1/src/iv.rs` - RF switch control module (copied from reference)

### Documentation
- `CLAUDE.md` - Added "LoRaWAN Byte Order Shenanigans" section
- `TROUBLESHOOTING_WL55.md` - Added "unknow mote" error troubleshooting
- `decode_payload.py` - Python payload decoder script
- `IMPLEMENTATION_SUMMARY.md` - This file

---

## Next Steps (Future Work)

### For LoRa-2 Node
1. Copy working firmware to `firmware/lora-2/`
2. Update to BME688 sensor (I2C 0x76/0x77)
3. Update to SH1106 128x64 display
4. Use different DevEUI (e.g., 24ce1bfeff091fac)
5. Expand payload to 12 bytes (add pressure, gas resistance)

### Optimizations
- [ ] Enable ADR (Adaptive Data Rate) for better power efficiency
- [ ] Implement low-power sleep modes
- [ ] Add confirmed uplinks for critical data
- [ ] Implement downlink handling
- [ ] Add EEPROM for persistent state (join keys, frame counters)

### Integration
- [ ] MQTT → InfluxDB data ingestion
- [ ] Grafana dashboard for visualization
- [ ] Combine with Week 9's Ethernet Modbus nodes (4-node unified system)

---

## Lessons Learned

1. **Byte order matters!** LoRaWAN EUI little-endian transmission caught us off-guard
2. **No FPU** - Integer math only, required careful sensor conversion
3. **Embassy async** - Made peripheral sharing elegant with `unsafe` stealing
4. **Working reference invaluable** - Having `../STM32WL55JC-lorawan/` saved hours
5. **Baby steps work** - Incremental changes with testing prevented cascading failures
6. **Documentation pays off** - CLAUDE.md guided entire implementation

---

**Status**: Production-ready LoRaWAN sensor node! 🚀

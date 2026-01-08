# Week 10 LoRaWAN - Current Status

**Date**: 2026-01-08
**Phase**: Gateway Configured ✅ | Hardware Complete ✅ | Ready for LoRaWAN Implementation

---

## ✅ Completed Work

### 1. Gateway Configuration (100% Complete)

The RAK7268V2 WisGate Edge Lite 2 gateway is fully operational:

- **Gateway EUI**: `ac1f09fffe1bce23`
- **Region**: AU915 Sub-band 2 (915.2-916.6 MHz)
- **Network Server**: Built-in LoRa Server (NOT ChirpStack)
- **MQTT Broker**: 127.0.0.1:1883 (local on gateway)

**Application "TOT" Configuration**:
- AppEUI: `b130a864c5295356`
- AppKey: `b726739b78ec4b9e9234e5d35ea9681b`
- Auto-add: Enabled (OTAA devices auto-register)
- Data encoding: Base64

**Pre-registered Device**:
- DevEUI: `23ce1bfeff091fac`
- Name: "STM_Nodes"
- Class: A (bi-directional)
- Activation: OTAA
- LoRaWAN Version: 1.0.3

**Documentation**: Complete configuration in [docs/rak7268v2-config.md](docs/rak7268v2-config.md)

### 2. LoRa-1 Hardware (100% Complete)

**Hardware**:
- STM32WL55JC1 (Serial: 003E00463234510A33353533)
- SHT41 temperature/humidity sensor (I2C 0x44)
- SSD1306 OLED 128x32 (I2C 0x3C)
- I2C2 bus: PA12 (SCL), PA11 (SDA)

**Firmware Status**:
- Real sensor data: 31°C, 57% RH
- Display updating every 2 seconds
- Compact 2-line layout: "LoRa-1" + "31C 57%"
- Integer-only math (no FPU)
- SHT41 wake-up command working

**Flash Command**:
```bash
cd firmware/lora-1
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:003E00463234510A33353533
```

### 3. LoRa-2 Hardware (Partially Complete)

**Hardware Verified**:
- STM32WL55JC1 (Serial: 0026003A3234510A33353533)
- Board tested and working

**Pending**:
- BME688 sensor connection
- SH1106 OLED 128x64 integration
- Firmware development

### 4. Documentation (Complete)

- ✅ [HARDWARE_CONFIG.md](HARDWARE_CONFIG.md) - Complete hardware specs
- ✅ [docs/rak7268v2-config.md](docs/rak7268v2-config.md) - Gateway config with credentials
- ✅ [NOTES.md](NOTES.md) - Technical implementation details and learnings
- ✅ [README.md](README.md) - Updated with gateway info and correct network server
- ✅ [TODO.md](TODO.md) - Task tracking

---

## 🎯 Next Steps: LoRaWAN Implementation

### Immediate Priority: LoRa-1 LoRaWAN Stack

The hardware is complete and the gateway is operational. The next step is to implement the LoRaWAN stack in the lora-1 firmware.

**Required Tasks**:

1. **Research LoRaWAN Libraries for STM32WL**
   - Investigate `embassy-lora` for async LoRaWAN
   - Check `lora-phy` for low-level radio driver
   - Review STM32WL HAL examples

2. **Add Dependencies to lora-1/Cargo.toml**
   ```toml
   # LoRaWAN stack options:
   # embassy-lora = "0.1"  # If available
   # lora-phy = "0.6"      # Radio PHY layer
   # lorawan-device = "0.11"  # LoRaWAN MAC layer
   ```

3. **Initialize SubGHz Radio**
   - Configure SubGHz peripheral via embassy-stm32
   - Set AU915 sub-band 2 channels (915.2-916.6 MHz)
   - Configure RX2 window (923.3 MHz, DR8)

4. **Implement OTAA Join**
   - Use documented credentials from gateway config:
     ```rust
     const DEV_EUI: [u8; 8] = [0x23, 0xCE, 0x1B, 0xFE, 0xFF, 0x09, 0x1F, 0xAC];
     const APP_EUI: [u8; 8] = [0xB1, 0x30, 0xA8, 0x64, 0xC5, 0x29, 0x53, 0x56];
     const APP_KEY: [u8; 16] = [
         0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
         0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B
     ];
     ```
   - Send Join Request
   - Handle Join Accept
   - Display "JOINING..." → "JOINED" status on OLED

5. **Implement Uplink Messages**
   - Encode SHT41 sensor data (4 bytes):
     ```
     Byte 0-1: Temperature (i16, °C * 100)
     Byte 2-3: Humidity (u16, % * 100)
     ```
   - Send unconfirmed uplinks every 60 seconds
   - Display TX counter on OLED

6. **Test End-to-End**
   - Flash firmware to LoRa-1
   - Monitor join process on OLED
   - Subscribe to MQTT uplink topic:
     ```bash
     mosquitto_sub -h <gateway-ip> -t "application/TOT/device/23ce1bfeff091fac/rx" -v
     ```
   - Verify sensor data appears in MQTT messages

---

## 🔧 Technical Considerations

### Key Facts from Previous Work

1. **No FPU**: STM32WL55 lacks FPU - all math must use integer types (i16, i32)
2. **SHT41 Wake-up**: Sensor requires measurement command (0xFD) to respond
3. **Peripheral Stealing**: Current pattern uses `unsafe { I2c::new(...::steal(), ...) }` each loop
4. **Display Size**: LoRa-1 has compact 128x32 display (2 lines only)
5. **Embassy Async**: All code uses Embassy async framework

### LoRaWAN Constraints

- **Duty Cycle**: AU915 requires compliance (likely 1% or less)
- **Payload Size**: Keep uplinks small (4 bytes for SHT41 is ideal)
- **Join Retry**: Implement exponential backoff if join fails
- **ADR**: Adaptive Data Rate should be enabled
- **Class A**: Use scheduled RX windows after TX

---

## 📊 Testing Plan

### Phase 1: Join Testing
- Verify OTAA join completes successfully
- Monitor join events via MQTT
- Check gateway web UI for device activity

### Phase 2: Uplink Testing
- Confirm uplinks appear in MQTT
- Decode Base64 payload and verify sensor data
- Check RSSI/SNR in gateway reports

### Phase 3: Range Testing
- Indoor baseline: 50m minimum
- Outdoor target: 500m+
- Document RSSI/SNR at various distances

---

## 📚 Reference Documentation

**Gateway Configuration**:
- [docs/rak7268v2-config.md](docs/rak7268v2-config.md) - Full gateway config with credentials

**Hardware Details**:
- [HARDWARE_CONFIG.md](HARDWARE_CONFIG.md) - Complete hardware specs
- [docs/hardware-wiring.md](docs/hardware-wiring.md) - Pin connections

**Technical Learnings**:
- [NOTES.md](NOTES.md) - Implementation notes and troubleshooting

**MQTT Testing Commands**:
```bash
# Subscribe to all TOT application messages
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/#" -v

# Subscribe to join events
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/+/join" -v

# Subscribe to LoRa-1 uplinks
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/23ce1bfeff091fac/rx" -v

# Monitor gateway stats
mosquitto_sub -h <gateway-ip> -t "gateway/ac1f09fffe1bce23/stats" -v
```

---

**Status**: Ready for LoRaWAN implementation. All prerequisites complete.
**Last Updated**: 2026-01-08

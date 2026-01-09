# Week 10: LoRaWAN Sensor Network - TODO

> **Status**: ✅ **COMPLETE** - Both nodes operational, full data pipeline live
> **See also**: `../DOCS/PLAN_WEEKLY.md` for complete week-by-week breakdown

## Project Status

- **Start Date**: Week 10 (Phase 3)
- **Hardware**: 2x NUCLEO-WL55JC1 (STM32WL with integrated LoRaWAN)
- **Gateway**: RAK7268V2 WisGate Edge Lite 2 (AU915, Built-in LoRa Server)
- **Sensors**: SHT41 (LoRa-1), BME680 (LoRa-2)
- **Displays**: SSD1306 128x32 (LoRa-1), SH1106 128x64 (LoRa-2)
- **Status**: ✅ **COMPLETE**

## Hardware Inventory

- [x] RAK7268V2 LoRaWAN gateway (configured, AU915 Sub-band 2)
- [x] 2x NUCLEO-WL55JC1 boards
- [x] 1x BME680 sensor (on LoRa-2)
- [x] 1x SHT41 sensor (on LoRa-1)
- [x] 1x SSD1306 OLED 128x32 (LoRa-1)
- [x] 1x SH1106 OLED 128x64 (LoRa-2)
- [x] Breadboards, jumper wires

## Day 1-2: Gateway & Hardware Setup ✅ COMPLETE

- [x] RAK7268V2 powered and configured (AU915 Sub-band 2)
- [x] Built-in LoRa Server operational (NOT ChirpStack)
- [x] Application "TOT" created with OTAA credentials
- [x] MQTT broker accessible at 10.10.10.254:1883
- [x] Both NUCLEO-WL55JC1 boards tested
- [x] probe-rs flashing and debugging operational
- [x] Embassy async framework verified

## Day 3: LoRa-1 Implementation ✅ COMPLETE

- [x] SHT41 sensor integration (I2C 0x44)
- [x] SSD1306 OLED display (I2C 0x3C)
- [x] LoRaWAN OTAA join successful (DevEUI: 23ce1bfeff091fac)
- [x] Uplinks transmitting every ~30 seconds
- [x] 4-byte payload encoding (temp + humidity)
- [x] Display showing TX count, readings, SNR/RSSI
- [x] LED flash on transmission

## Day 4: LoRa-2 Implementation ✅ COMPLETE

- [x] BME680 sensor integration (I2C 0x76)
- [x] SH1106 OLED display (I2C 0x3C)
- [x] LoRaWAN OTAA join successful (DevEUI: 24ce1bfeff091fac)
- [x] Uplinks transmitting every ~30 seconds
- [x] 12-byte payload encoding (temp + hum + pressure + gas)
- [x] Display showing TX count, readings, SNR/RSSI
- [x] LED flash on transmission

## Day 5: Data Pipeline ✅ COMPLETE

- [x] Python MQTT bridge created (mqtt_to_influx.py)
- [x] Subscribe to gateway MQTT (10.10.10.254:1883)
- [x] Decode Base64 payloads (both 4-byte and 12-byte formats)
- [x] Write to InfluxDB (bucket: lorawan)
- [x] Docker Compose configuration for bridge container
- [x] End-to-end data flow verified

## Day 6: Grafana Dashboard ✅ COMPLETE

- [x] InfluxDB datasource configured
- [x] Dashboard created with 10 panels:
  - [x] Temperature (both nodes)
  - [x] Humidity (both nodes)
  - [x] Pressure (LoRa-2 only)
  - [x] Gas Resistance (LoRa-2 only)
  - [x] RSSI (both nodes)
  - [x] SNR (both nodes)
  - [x] Stat panels (latest values)
- [x] Dashboard JSON exported (grafana/lorawan-dashboard.json)

## Day 7: Documentation ✅ COMPLETE

- [x] README.md - Complete system overview with architecture diagram
- [x] USERGUIDE.md - Comprehensive deployment guide (857 lines)
- [x] HARDWARE_CONFIG.md - Hardware specifications
- [x] LORAWAN_CREDENTIALS.md - Device credentials
- [x] docs/rak7268v2-config.md - Gateway configuration
- [x] TROUBLESHOOTING_WL55.md - Common issues and solutions
- [x] CLAUDE.md - Development guidance
- [x] NOTES.md - Technical learnings
- [x] STATUS.md - Current status summary
- [x] IMPLEMENTATION_SUMMARY.md - Implementation details

## Firmware Implementation ✅ COMPLETE

### LoRa-1 (firmware/lora-1/) ✅

- [x] Embassy async framework
- [x] STM32WL SubGHz radio initialization
- [x] lora-phy v3.0 driver
- [x] lorawan-device v0.12 MAC layer
- [x] OTAA join procedure
- [x] SHT41 sensor driver (integer math only)
- [x] SSD1306 display driver
- [x] Uplink transmission loop
- [x] SNR/RSSI tracking and display

### LoRa-2 (firmware/lora-2/) ✅

- [x] Embassy async framework
- [x] STM32WL SubGHz radio initialization
- [x] lora-phy v3.0 driver
- [x] lorawan-device v0.12 MAC layer
- [x] OTAA join procedure
- [x] BME680 sensor driver (integer math only)
- [x] SH1106 display driver
- [x] Uplink transmission loop (12-byte payload)
- [x] SNR/RSSI tracking and display

## Data Pipeline ✅ COMPLETE

### MQTT Bridge (mqtt_to_influx.py) ✅

- [x] Subscribe to gateway MQTT topics
- [x] Parse JSON payload from gateway
- [x] Decode LoRaWAN payload (4-byte and 12-byte formats)
- [x] Write decoded data to InfluxDB
- [x] Docker container configuration

## Testing ✅ COMPLETE

### LoRaWAN Connectivity ✅

- [x] Both nodes join successfully
- [x] Uplinks received by gateway
- [x] RSSI: -13 to -80 dBm (typical)
- [x] SNR: 10-13 dB (typical)
- [x] End-to-end latency: <2 seconds

### Data Pipeline ✅

- [x] MQTT messages received from gateway
- [x] Payloads decoded correctly
- [x] Data written to InfluxDB
- [x] Grafana displays real-time data

## Deliverables ✅ ALL COMPLETE

- [x] RAK7268V2 LoRaWAN gateway deployed (AU915)
- [x] Built-in LoRa Server operational
- [x] 2x STM32WL55 LoRaWAN devices (OTAA, Class A)
- [x] Python MQTT bridge to InfluxDB
- [x] Grafana dashboard with 10 panels
- [x] Complete documentation set
- [x] USERGUIDE.md deployment guide

## Stretch Goals (Future Work)

- [ ] Implement downlink messages (Class A confirmed messages)
- [ ] Add adaptive data rate (ADR) support
- [ ] Implement low-power sleep mode
- [ ] Add battery voltage monitoring
- [ ] Integrate with Week 9 Modbus nodes (unified 4-node system)
- [ ] Range testing at various distances

## Key Learnings

1. **LoRaWAN byte order** - EUIs must be reversed (little-endian) in firmware
2. **No FPU** - STM32WL55 requires integer-only math
3. **SHT41 wake-up** - Sensor needs measurement command before I2C scan
4. **MQTT 3.1** - RAK gateway requires older protocol version
5. **Embassy async** - Peripheral stealing pattern for I2C sharing

## Notes

- Using native STM32WL LoRaWAN stack via lora-phy + lorawan-device
- AU915 Sub-band 2 frequency plan (915.2-916.6 MHz)
- Class A device (bi-directional with scheduled uplinks)
- OTAA activation (secure key derivation)
- Embassy async/await framework for consistency
- Gateway uses built-in LoRa Server (NOT ChirpStack)

---

**Status**: ✅ Week 10 Complete
**Last Updated**: 2026-01-09

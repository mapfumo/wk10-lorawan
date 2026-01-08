# Week 10: LoRaWAN Migration - TODO

> **See also**: `../DOCS/PLAN_WEEKLY.md` for complete week-by-week breakdown

## Project Status
- **Start Date**: Week 10 (Phase 3)
- **Hardware**: 2x NUCLEO-WL55JC1 (STM32WL with integrated LoRaWAN)
- **Gateway**: RAK7268V2 WisGate Edge Lite 2 (AU915, LAN + WiFi)
- **Sensors**: BME688 (Node 1), SHT41 (Node 2)
- **Display**: SSD1306 OLED 128x64 (I2C) on both boards
- **Status**: 📋 **STARTING - Hardware arriving Monday**

## Hardware Inventory
- [x] RAK7268V2 LoRaWAN gateway (already configured, AU915)
- [ ] 2x NUCLEO-WL55JC1 boards (arriving Monday)
- [ ] 1x BME688 sensor (AI environmental)
- [ ] 1x SHT41 sensor (high-precision temp/humidity)
- [ ] 2x SSD1306 OLED displays (from previous weeks)
- [ ] Breadboards, jumper wires

## Day 1: RAK7268V2 Gateway & ChirpStack Setup
- [x] RAK7268V2 powered and configured (LAN + WiFi, AU915) - COMPLETE
- [ ] Verify RAK7268V2 accessible from desktop via LAN
- [ ] Install ChirpStack via Docker Compose on desktop
- [ ] Configure ChirpStack region (AU915 to match gateway)
- [ ] Register RAK7268V2 in ChirpStack (EUI-64, name, location)
- [ ] Verify packet forwarder connectivity in ChirpStack dashboard
- [ ] Create LoRaWAN device profile (Class A, OTAA, LoRaWAN 1.0.3)
- [ ] Document ChirpStack installation in docs/chirpstack-setup.md

## Day 2: STM32WL55 Development Environment & Sensor Integration
- [x] Unbox and inventory 2x NUCLEO-WL55JC1 boards
- [x] Set up STM32WL55 development environment
  - [x] Install probe-rs support for STM32WL55JC
  - [x] Test probe connection and chip detection
  - [x] Set up .cargo/config.toml for WL55
  - [x] Configure memory.x for STM32WL55JC (256KB Flash, 64KB RAM)
- [x] **Board 1 (NODE_1)**: Wire SHT41 (I2C 0x44) + SH1106 OLED (I2C 0x3C) via breadboard
- [ ] **Board 2 (NODE_2)**: Wire BME688 (I2C 0x76/0x77) + OLED (I2C 0x3C)
- [x] Test sensor communication on NODE_1 (SHT41 working at 27°C, 60% RH)
- [x] Display sensor readings on NODE_1 OLED (real-time updates every 2s)
- [x] Document hardware wiring in docs/hardware-wiring.md

**Note**: NODE_1 configured with SHT41 (not BME688 as originally planned) for initial testing

## Day 3: LoRaWAN OTAA Implementation (Board 1 - BME688)
- [ ] Study STM32WL LoRaWAN examples and HAL
- [ ] Configure LoRaWAN parameters (DevEUI, AppEUI, AppKey)
- [ ] Register device in ChirpStack application
- [ ] Implement OTAA join procedure using STM32WL radio
- [ ] Handle join-accept and derive session keys
- [ ] Verify successful join in ChirpStack device logs
- [ ] Display join status on OLED ("JOINING...", "JOINED")
- [ ] Add LED indicator for join status

## Day 4: LoRaWAN Uplink Messages & Sensor Data (Board 1 - BME688)
- [ ] Implement unconfirmed uplink messages (Class A)
- [ ] Encode BME688 sensor data (temp, humidity, pressure, gas, IAQ) - ~12 bytes
- [ ] Add frame counter management and encryption
- [ ] Configure uplink interval (e.g., 60 seconds)
- [ ] Respect AU915 duty cycle limits
- [ ] Verify uplink messages in ChirpStack (payload decoding)
- [ ] Test sensor data visualization in ChirpStack
- [ ] Create payload decoder function in ChirpStack

## Day 5: MQTT Integration & Second Device (Board 2 - SHT41)
- [ ] Configure ChirpStack MQTT integration
- [ ] Subscribe to LoRaWAN device data on MQTT broker (Mosquitto)
- [ ] Create Python/Rust MQTT subscriber script
- [ ] Parse LoRaWAN JSON payload and write to InfluxDB
- [ ] Adapt firmware for Board 2 (SHT41 sensor, unique DevEUI)
- [ ] Encode SHT41 sensor data (temp, humidity) - ~4 bytes payload
- [ ] Register Board 2 in ChirpStack
- [ ] Test both devices sending data to InfluxDB
- [ ] Verify data flow: WL55 → RAK7268V2 → ChirpStack → MQTT → InfluxDB

## Day 6: Unified Grafana Dashboard (4 Nodes)
- [ ] Create Grafana dashboard for 2x LoRaWAN nodes
- [ ] **Node 1 (BME688)**: Temp, humidity, pressure, gas resistance, IAQ panels
- [ ] **Node 2 (SHT41)**: High-precision temp, humidity panels
- [ ] Integrate with existing Week 9 Modbus nodes (10.10.10.100, 10.10.10.200)
- [ ] Unified view: 2x Modbus (SHT3x) + 2x LoRaWAN (BME688 + SHT41)
- [ ] Create comparison panels (temperature/humidity across all 4 nodes)
- [ ] Add LoRaWAN-specific metrics (RSSI, SNR, spreading factor, duty cycle)
- [ ] Document multi-protocol architecture in docs/architecture.md

## Day 7: Testing, Documentation & Weekly Review
- [ ] End-to-end testing: All 4 nodes reporting to Grafana
- [ ] Performance measurement: latency, RSSI/SNR, uplink success rate
- [ ] Range testing with LoRaWAN (indoor, outdoor)
- [ ] Create RAK7268V2 deployment guide
- [ ] Document STM32WL55 LoRaWAN implementation
- [ ] Write architecture document (Modbus + LoRaWAN + OPC-UA)
- [ ] Update README with complete system architecture
- [ ] Export Grafana dashboards as JSON
- [ ] Weekly review and plan Week 11

## Implementation: STM32WL55 LoRaWAN Stack

### Node 1 (BME688) - firmware/lora-1/
- [ ] Create Cargo project for STM32WL55JC
- [ ] Add dependencies (embassy-stm32, lorawan stack)
- [ ] Initialize STM32WL SubGHz radio peripheral
- [ ] Implement OTAA join sequence
- [ ] Add BME688 sensor driver (I2C)
- [ ] Add OLED display driver (I2C)
- [ ] Implement uplink transmission loop
- [ ] Add error handling and retry logic
- [ ] Test indoor range (target: 50m minimum)

### Node 2 (SHT41) - firmware/lora-2/
- [ ] Clone Node 1 firmware structure
- [ ] Replace BME688 with SHT41 sensor driver
- [ ] Update DevEUI (unique identifier)
- [ ] Adjust payload encoding (4 bytes vs 12 bytes)
- [ ] Test dual-node coexistence
- [ ] Verify both nodes join successfully

## Gateway Service - gateway/lorawan_mqtt_bridge.py
- [ ] Create Python script for MQTT subscription
- [ ] Subscribe to ChirpStack MQTT topics:
  - [ ] application/+/device/+/event/up (uplink data)
  - [ ] application/+/device/+/event/join (join events)
  - [ ] application/+/device/+/event/status (status events)
- [ ] Parse JSON payload from ChirpStack
- [ ] Decode LoRaWAN payload (BME688, SHT41)
- [ ] Write decoded data to InfluxDB (dual-write pattern)
- [ ] Add error handling and reconnection logic
- [ ] Document MQTT topic structure

## Testing: LoRaWAN Connectivity
- [ ] Verify RAK7268V2 receives packets (gateway logs)
- [ ] Check ChirpStack device logs (join, uplinks)
- [ ] Monitor LoRaWAN metrics (RSSI, SNR, SF)
- [ ] Test different spreading factors (SF7, SF9, SF12)
- [ ] Measure latency (sensor → Grafana)
- [ ] Test range at different locations
- [ ] Verify duty cycle compliance (AU915 regulations)

## Testing: Multi-Protocol System
- [ ] All 4 nodes visible in Grafana
- [ ] Temperature comparison across nodes
- [ ] Humidity comparison across nodes
- [ ] Verify data freshness (timestamps)
- [ ] Test system with 1 node offline
- [ ] Test system with gateway offline
- [ ] Recovery testing (reconnection, rejoin)

## Documentation
- [ ] README.md - System overview, quick start
- [ ] docs/hardware-wiring.md - Pin connections for both boards
- [ ] docs/chirpstack-setup.md - ChirpStack installation guide
- [ ] docs/architecture.md - Multi-protocol system design
- [ ] NOTES.md - Development log, learning insights
- [ ] USERGUIDE.md - Deployment and operation guide

## Deliverables
- [x] RAK7268V2 LoRaWAN gateway deployed (AU915) - COMPLETE
- [ ] ChirpStack network server operational
- [ ] 2x STM32WL55 LoRaWAN devices (OTAA, Class A)
- [ ] MQTT integration to InfluxDB
- [ ] Unified 4-node Grafana dashboard
- [ ] Complete documentation set
- [ ] Architecture diagrams
- [ ] Performance metrics report

## Stretch Goals (Optional)
- [ ] Implement downlink messages (Class A confirmed messages)
- [ ] Add adaptive data rate (ADR) support
- [ ] Implement Class C device mode (always listening)
- [ ] Add GPS coordinates to payload (if available)
- [ ] Implement firmware update over LoRaWAN
- [ ] Add battery voltage monitoring
- [ ] Implement low-power sleep mode

## Notes
- Using native STM32WL LoRaWAN stack (not RYLR998 modules)
- AU915 frequency plan (Australia/NZ region)
- Class A device (bi-directional with scheduled uplinks)
- OTAA activation (more secure than ABP)
- One sensor per board for simplicity
- Spare sensors available for future experiments
- Embassy async/await framework for consistency

## Key Learning Goals
- STM32WL native LoRaWAN stack
- LoRaWAN OTAA join procedure
- ChirpStack network server administration
- Multi-protocol system integration (Ethernet + LoRaWAN)
- LoRaWAN duty cycle and regulations
- Payload optimization for LPWAN

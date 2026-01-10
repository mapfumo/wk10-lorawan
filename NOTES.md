# Week 10: LoRaWAN Migration - Development Notes

**Project**: LoRaWAN Migration with STM32WL55
**Start Date**: Week 10 (Phase 3)
**Status**: Starting - Hardware arriving Monday

---

## Overview

Week 10 transitions from point-to-point LoRa (RYLR998 modules) to production LoRaWAN infrastructure using native STM32WL radio peripherals. This integrates with Week 9's Modbus TCP system to create a unified 4-node monitoring platform.

---

## Key Concepts

### LoRaWAN vs LoRa
- **LoRa**: Physical layer modulation (CSS - Chirp Spread Spectrum)
- **LoRaWAN**: MAC layer protocol with network server architecture
- **Benefits**: Security (AES-128), scalability, standardized ecosystem

### OTAA vs ABP
- **OTAA (Over-The-Air Activation)**: Devices join network dynamically
  - Uses DevEUI, AppEUI, AppKey
  - Derives session keys (NwkSKey, AppSKey)
  - More secure (keys rotated on each join)
- **ABP (Activation By Personalization)**: Static session keys
  - Simpler but less secure
  - Used for testing only

### Device Classes
- **Class A**: Bi-directional, scheduled RX windows after TX
  - Most power-efficient
  - Used for sensor nodes
- **Class B**: Scheduled RX windows at fixed intervals
- **Class C**: Continuous listening (highest power consumption)

### AU915 Frequency Plan
- **Region**: Australia/New Zealand
- **Channels**: 64 upstream (915.2-927.8 MHz), 8 downstream
- **Sub-bands**: 8 sub-bands of 8 channels each
- **Duty Cycle**: No strict duty cycle (FHSS compliance instead)

---

## Hardware Architecture

### STM32WL55JC1
- **Dual-core**: Cortex-M4 (application) + Cortex-M0+ (radio stack)
- **Radio**: Integrated SubGHz transceiver (150 MHz - 960 MHz)
- **Memory**: 256KB Flash, 64KB RAM
- **Advantages**: No external LoRa module needed, lower BOM cost

### RAK7268V2 Gateway
- **Concentrator**: Semtech SX1302 (8 channels)
- **Backhaul**: Ethernet + WiFi
- **Packet Forwarder**: Connects to ChirpStack via UDP
- **Already configured**: AU915, operational

---

## Design Decisions

### One Sensor Per Board
**Decision**: Node 1 gets BME688, Node 2 gets SHT41 (not both sensors on each board)

**Rationale**:
- Simpler firmware (no I2C bus management complexity)
- Easier debugging (isolate sensor issues)
- Different payload sizes showcase optimization (12 bytes vs 4 bytes)
- Different data profiles demonstrate variety
- Spare sensors available for backups

### OLED Display on Both Boards
**Display Content**:
- Line 1: Node ID ("WL55-BME688" / "WL55-SHT41")
- Line 2: Join status ("JOINING..." / "JOINED" / "JOIN FAILED")
- Line 3-4: Sensor readings (temp, humidity, etc.)
- Line 5: LoRaWAN metrics (RSSI, SNR, SF, FC)

**Benefits**:
- Real-time debugging without probe-rs
- Visible join status confirmation
- Range testing feedback (RSSI/SNR)

### Embassy Framework
**Why Embassy**:
- Consistency with previous weeks (Week 7-9)
- Async/await for LoRaWAN state machine
- STM32WL HAL support
- Efficient task scheduling

### Code Reuse Strategy - NO common.rs Yet
**Decision**: Wait until Day 4-5 before creating `common.rs`

**Rationale**:
- Unknown LoRaWAN stack choice (lora-phy vs embassy-lora vs direct PAC)
- Learning-focused: see patterns emerge naturally before abstracting
- Week 9 pattern: Board 1 working first, then extract common code
- Easier debugging with everything in `main.rs` initially

**When to Extract to common.rs** (Day 4-5):
- After Node 1 (BME688) fully working
- When building Node 2 (SHT41), extract shared code:
  - `lorawan_join()` - OTAA procedure
  - `lorawan_uplink(payload: &[u8])` - Send data
  - `oled_init()` - Display setup
  - `oled_update_status()` - Status helpers
- Keep sensor-specific code in each `main.rs`

---

## Learning Insights

### To Be Updated Daily

**Day 1**: ChirpStack Setup
- [Date] - Notes on ChirpStack installation
- [Date] - Gateway registration process
- [Date] - Device profile creation

**Day 2**: STM32WL55 Development Environment
- [Date] - probe-rs configuration for WL55
- [Date] - memory.x layout for 256KB Flash
- [Date] - Sensor wiring and I2C testing

**Day 3**: OTAA Implementation
- [Date] - Join procedure implementation
- [Date] - Session key derivation
- [Date] - Challenges encountered

**Day 4**: Uplink Messages
- [Date] - Payload encoding strategies
- [Date] - Frame counter management
- [Date] - Duty cycle compliance

**Day 5**: MQTT Integration
- [Date] - ChirpStack MQTT topics
- [Date] - Payload decoding in Python
- [Date] - InfluxDB dual-write pattern

**Day 6**: Grafana Dashboard
- [Date] - 4-node unified dashboard design
- [Date] - LoRaWAN metrics visualization
- [Date] - Multi-protocol comparison panels

**Day 7**: Testing & Documentation
- [Date] - Range testing results
- [Date] - Performance benchmarks
- [Date] - Week 10 retrospective

---

## Technical Challenges

### 2026-01-10: MQTT Bridge Silent Disconnection

#### Challenge: Grafana Dashboard Stops Populating After ~90 Seconds

**Problem**: The MQTT-to-InfluxDB bridge (`mqtt_to_influx.py`) would work initially, then silently stop forwarding data. Restarting the container fixed it temporarily. No error messages appeared in logs.

**Attempted Solutions**:
- Checked InfluxDB connectivity (was fine)
- Verified MQTT subscription (worked on restart)
- Added logging (no errors shown)

**Final Solution**: Implemented MQTT keep-alive handling. The raw socket MQTT client set `keep_alive=60` but never sent `PINGREQ` packets. After 90 seconds (1.5x keep-alive), the broker disconnected the client. The code didn't detect the dead connection because `recv()` returning empty looked identical to a timeout.

Fix involved:
1. Adding `mqtt_ping()` function to send PINGREQ (0xC0 0x00)
2. Handling PINGRESP (packet type 13) in message reader
3. Sending pings every 30 seconds
4. Tracking last activity and reconnecting after 90s of silence

**Learning**: When implementing raw socket protocols, you must handle ALL protocol requirements - not just the happy path. MQTT keep-alive is mandatory, not optional. Using a proper MQTT library (paho-mqtt) would have handled this automatically, but the minimal raw socket approach for the slim Python container required manual implementation.

---

### To Be Documented

#### Challenge: [Title]

**Problem**: [Description]
**Attempted Solutions**: [What was tried]
**Final Solution**: [What worked]
**Learning**: [Key insight]

---

## References

### LoRaWAN Specifications
- LoRaWAN 1.0.4 Regional Parameters (RP002-1.0.4)
- LoRaWAN L2 1.0.4 Specification
- AU915-928 Regional Parameters

### STM32WL55 Resources
- STM32WL55xx Reference Manual (RM0453)
- STM32WL55xx Datasheet
- AN5406: STM32WL SubGHz radio configuration
- UM2643: STM32CubeWL LoRaWAN package

### ChirpStack Documentation
- ChirpStack Gateway Bridge
- ChirpStack Network Server
- ChirpStack Application Server
- MQTT Integration Guide

### Tools
- UaExpert: OPC-UA client testing
- MQTT Explorer: MQTT debugging
- Grafana: Visualization
- probe-rs: Flashing and debugging

---

## Performance Metrics

### To Be Measured

**Join Performance**:
- [ ] Time to join (from power-on to JOINED)
- [ ] Join success rate
- [ ] RSSI/SNR at join

**Uplink Performance**:
- [ ] Latency (sensor read → Grafana)
- [ ] Packet delivery rate
- [ ] Duty cycle utilization

**Range Testing**:
- [ ] Indoor range (RSSI, SNR, packet loss)
- [ ] Outdoor range (line-of-sight)
- [ ] Spreading factor impact

**System Integration**:
- [ ] 4-node data freshness
- [ ] MQTT broker load
- [ ] InfluxDB write performance

---

## Code Snippets

### To Be Added

**LoRaWAN Join Request**:
```rust
// Example code to be added during implementation
```

**Payload Encoding**:
```rust
// BME688 payload encoding
// SHT41 payload encoding
```

**ChirpStack Payload Decoder**:
```javascript
// JavaScript decoder function for ChirpStack
```

---

## Comparison: Week 9 vs Week 10

| Aspect | Week 9 (Modbus TCP) | Week 10 (LoRaWAN) |
|--------|---------------------|-------------------|
| **MCU** | STM32F446RE | STM32WL55JC1 |
| **Connectivity** | W5500 Ethernet | SubGHz LoRaWAN |
| **Network** | LAN (wired) | LPWAN (wireless) |
| **Protocol** | Modbus TCP | LoRaWAN |
| **IP Addressing** | Static (10.10.10.x) | N/A (uses DevEUI) |
| **Range** | 100m (Ethernet cable) | 500m+ (wireless) |
| **Bandwidth** | 10/100 Mbps | ~50 kbps (SF7) |
| **Latency** | <100ms | 1-5 seconds |
| **Power** | Mains (USB) | Battery-capable |
| **Security** | None (plaintext) | AES-128 encryption |
| **Server** | OPC-UA gateway | ChirpStack |

---

## Next Steps

### Week 11 Preview (Security Architecture)
- STRIDE threat modeling
- TLS/mTLS implementation
- Authentication and authorization
- Security assessment document

### Week 12 Preview (Portfolio Assembly)
- Documentation polish
- Video demonstration
- GitHub optimization
- Portfolio repository

---

## 2026-01-05: STM32WL55JC1 Hardware Verification

### First Test: LED Blink Success! ✅

**Objective**: Verify STM32WL55JC1 boards work before implementing LoRaWAN

**Hardware Tested**: NODE_2 (SHT41) - `0483:374e:0026003A3234510A33353533`

**Firmware**: Simple LED blink using Embassy async framework
- LED1 (Blue): PB15
- LED3 (Red): PB11
- Pattern: Blue 500ms → Red 500ms → Both 1s → Off 1s

**Results**:
- ✅ Embassy STM32WL framework working
- ✅ probe-rs flashing successful (0.71s)
- ✅ defmt-rtt logging working perfectly
- ✅ GPIO control working (both LEDs blinking as expected)
- ✅ Embassy async Timer working

**Troubleshooting Journey**:

1. **Initial Connection Errors** (JtagGetIdcodeError, SwdDpError)
   - Symptoms: probe-rs could not connect to chip
   - ST-Link detected but communication failed
   - ST-Link LED flashing green/orange/red

2. **Root Causes Identified**:
   - **Cause 1**: STM32CubeProgrammer was holding the ST-Link (Device busy error)
   - **Cause 2**: Flash memory was in protected/locked state from previous programming

3. **Solution Steps** (CRITICAL for STM32WL55):
   - Close STM32CubeProgrammer (release ST-Link)
   - In STM32CubeProgrammer:
     1. Select "Connect under reset" mode
     2. Hold RESET button on NUCLEO board
     3. Perform "Full Chip Erase" to clear flash memory
     4. Release RESET button
   - After chip erase, probe-rs connection worked immediately

4. **Final Success**:
   - `cargo run --release` worked on first attempt
   - Firmware flashed in 0.71s
   - LEDs blinking perfectly

**Key Learnings**:
- **STM32WL55 requires chip erase** if previously programmed or locked
- Only ONE tool can access ST-Link at a time (no simultaneous connections)
- "Connect under reset" mode essential for initial programming
- STM32WL55 uses default MSI clock (~4MHz) - sufficient for testing
- Embassy framework setup was correct from the start
- Hardware verified and ready for sensor integration

**Critical Note for Future Programming**:
If connection fails with JtagGetIdcodeError:
1. Check if STM32CubeProgrammer or other tool is running (close it)
2. Use STM32CubeProgrammer: Connect under reset + Full Chip Erase
3. After erase, probe-rs will work normally

**Next Steps**:
1. Test NODE_1 (BME688) with same firmware
2. Add I2C and OLED display support
3. Add BME688 sensor integration
4. Implement LoRaWAN stack (SubGHz radio)

---

## 2026-01-08: NODE_1 SHT41 + OLED Integration Success! ✅

### Real Temperature & Humidity Sensor Data on OLED Display

**Objective**: Get SHT41 sensor working with OLED display on NODE_1

**Hardware**: NODE_1 (0483:374e:003E00463234510A33353533)
- SHT41 temperature & humidity sensor
- SH1106 OLED display 128x64 (I2C)
- Connected via breadboard (bypassing STEMMA QT cables)

**Final Working Configuration**:

**I2C2 Bus** (PA12=SCL, PA11=SDA):
- SHT41 sensor: 0x44
- SH1106 OLED: 0x3C

**Results**:
- ✅ Both devices detected on I2C bus
- ✅ SHT41 reading real sensor data: 27°C, 60% RH
- ✅ OLED displaying sensor readings every 2 seconds
- ✅ LED heartbeat on PB15
- ✅ Integer-only math (no floating point)

**Display Output**:
```
STM32WL55 Node1
Temp: 27 C
Hum:  60 %
SHT41 Active
```

**Technical Implementation**:

1. **I2C Bus Sharing Solution**:
   - Challenge: sh1106 driver consumes I2C bus (no release mechanism)
   - Solution: Use `unsafe` peripheral stealing to recreate I2C each loop iteration
   - Pattern: Read sensor → Create display → Drop both → Repeat

2. **SHT41 Wake-up Requirement** (CRITICAL DISCOVERY):
   - Problem: SHT41 not detected on initial I2C bus scan
   - Solution: Send measurement command (0xFD) BEFORE scanning
   - SHT41 requires an initial command to wake up and respond to I2C
   - After wake-up command, sensor appears at 0x44

3. **Integer Math Implementation** (No FPU on STM32WL55):
   - Temperature: `T = -45 + (175 × raw) / 65535` (°C)
   - Humidity: `RH = -6 + (125 × raw) / 65535` (%RH)
   - Uses i32 intermediate values, casts to i16 for display
   - Avoids floating point operations completely

4. **Code Structure**:
   ```rust
   loop {
       // Step 1: Create I2C and read SHT41
       let mut i2c = unsafe { I2c::new(I2C2::steal(), PA12::steal(), PA11::steal(), ...) };
       i2c.blocking_write(SHT41_ADDR, &[CMD_MEASURE_HIGH_PRECISION])?;
       Timer::after_millis(10).await;  // Wait for measurement
       i2c.blocking_read(SHT41_ADDR, &mut data)?;
       // Convert raw to temperature/humidity (integer math)

       // Step 2: Create OLED display with same I2C
       let mut display = Builder::new().connect_i2c(i2c).into();
       display.init()?;
       // Draw text with sensor readings
       display.flush()?;

       // Both i2c and display dropped here, hardware released
       Timer::after_secs(2).await;
   }
   ```

**Key Learnings**:

1. **SHT41 Wake-up Behavior**:
   - Sensor does NOT respond to simple I2C address probes when idle
   - Must send a measurement command first to "wake" the sensor
   - After wake-up, sensor responds normally to I2C bus scans
   - This is normal behavior per SHT41 datasheet (low power design)

2. **STM32WL55 No FPU**:
   - Cannot use f32/f64 without soft-float library
   - HardFault occurs when trying to format floats
   - Integer math is fast, efficient, and sufficient for display
   - Can show decimal precision by using fixed-point (temp × 10)

3. **Embassy I2C Peripheral Stealing**:
   - Using `steal()` allows recreating peripherals each iteration
   - Safe in this context: single-threaded, exclusive access per iteration
   - Avoids complex lifetime management with display drivers
   - Hardware naturally released when objects drop

4. **SH1106 vs SSD1306**:
   - User has SH1106 OLED (not SSD1306 as originally documented)
   - Different driver needed: `sh1106` crate (not `ssd1306`)
   - Similar API but different initialization sequence
   - Both work on same I2C address (0x3C)

**Troubleshooting Journey**:

1. **Initial Problem**: SHT41 not detected (only OLED at 0x3C found)
2. **Attempted Solution**: Full I2C bus scan (0x00-0x7F)
3. **Discovery**: User confirmed "sht41 address is 0x44"
4. **Breakthrough**: Send wake-up command BEFORE scanning
5. **Success**: Both devices detected and working!

**Firmware Dependencies**:
```toml
embassy-stm32 = "0.1.0"
embassy-time = "0.3"
sh1106 = "0.5"              # OLED display driver
embedded-graphics = "0.8"   # Graphics primitives
heapless = "0.8"           # No-std string formatting
sht4x = "0.2"              # SHT41 sensor (for reference, manual impl used)
```

**Next Steps for NODE_1**:
1. ✅ Hardware working with real sensor data
2. ✅ OLED displaying readings
3. [ ] Add LoRaWAN radio stack (SubGHz peripheral)
4. [ ] Implement OTAA join procedure
5. [ ] Transmit sensor data via LoRaWAN uplink
6. [ ] Connect to ChirpStack network server

**Next Steps for NODE_2 (BME688)**:
1. [ ] Test LED blink on NODE_2
2. [ ] Implement BME688 I2C driver
3. [ ] Add OLED display (same pattern as NODE_1)
4. [ ] Display BME688 environmental data
5. [ ] Clone LoRaWAN stack from NODE_1

---

**Last Updated**: 2026-01-08
**Next Review**: End of Week 10

# Hardware Configuration - Week 10 LoRaWAN

**Last Updated**: 2026-01-08

---

## LoRa-1 (NODE_1)

### Board Information
- **Serial**: 0483:374e:003E00463234510A33353533
- **MCU**: STM32WL55JC1 (NUCLEO-WL55JC1)
- **Status**: ✅ Working - Sensor data displaying

### Sensors
| Component | Model | I2C Address | Pins | Status |
|-----------|-------|-------------|------|--------|
| Temperature/Humidity | SHT41 | 0x44 | PA11 (SDA), PA12 (SCL) | ✅ Working |

### Display
| Component | Model | Resolution | I2C Address | Pins | Driver | Status |
|-----------|-------|------------|-------------|------|--------|--------|
| OLED | SSD1306 Compatible | 128x32 | 0x3C | PA11 (SDA), PA12 (SCL) | ssd1306 v0.8 | ✅ Working |

**Display Notes**:
- White font on black background
- I2C Interface
- 128x32 pixel resolution (2 lines of text with FONT_6X10)
- Layout:
  - Line 1: "LoRa-1"
  - Line 2: "31C 57%"

### I2C Bus Configuration
- **Bus**: I2C2
- **SCL**: PA12
- **SDA**: PA11
- **Speed**: 100 kHz
- **Pull-ups**: Internal (enabled in firmware)
- **Connection**: Breadboard (STEMMA QT cables bypassed)

### Current Readings
- Temperature: 31°C
- Humidity: 57% RH
- Update interval: 2 seconds

### Firmware
- **Directory**: `firmware/lora-1/`
- **Binary**: `lora-1`
- **Flash command**: `probe-rs run --chip STM32WL55JCIx --probe 0483:374e:003E00463234510A33353533`

---

## LoRa-2 (NODE_2)

### Board Information
- **Serial**: 0483:374e:0026003A3234510A33353533
- **MCU**: STM32WL55JC1 (NUCLEO-WL55JC1)
- **Status**: ⏳ Pending - To be configured

### Sensors (Planned)
| Component | Model | I2C Address | Pins | Status |
|-----------|-------|-------------|------|--------|
| Environmental | BME688 | 0x76 or 0x77 | TBD | ⏳ Pending |

**BME688 Capabilities**:
- Temperature
- Humidity
- Pressure
- Gas Resistance
- Indoor Air Quality (IAQ)
- Payload: ~12 bytes

### Display
| Component | Model | Resolution | I2C Address | Pins | Driver | Status |
|-----------|-------|------------|-------------|------|--------|--------|
| OLED | SH1106 | 128x64 | 0x3C | TBD | sh1106 v0.5 | ⏳ Pending |

**Display Notes**:
- SH1106 compatible controller
- 128x64 pixel resolution (6 lines of text with FONT_6X10)
- Larger display than LoRa-1 (64 vs 32 pixels height)
- Can show more sensor data

### I2C Bus Configuration (Planned)
- **Bus**: I2C1 (likely PB8/PB9) or I2C2 (PA12/PA11)
- **Speed**: 100 kHz
- **Pull-ups**: Internal (to be enabled)
- **Connection**: Breadboard

### Firmware
- **Directory**: `firmware/lora-2/`
- **Binary**: `lora-2`
- **Status**: Placeholder code only
- **Flash command**: `probe-rs run --chip STM32WL55JCIx --probe 0483:374e:0026003A3234510A33353533`

---

## Key Differences

### Display Comparison

| Feature | LoRa-1 | LoRa-2 |
|---------|--------|--------|
| **Controller** | SSD1306 | SH1106 |
| **Resolution** | 128x32 | 128x64 |
| **Lines of Text** | 2 lines | 6 lines |
| **Driver Crate** | `ssd1306 = "0.8"` | `sh1106 = "0.5"` |
| **Advantage** | Compact | More info |

### Sensor Comparison

| Feature | LoRa-1 (SHT41) | LoRa-2 (BME688) |
|---------|----------------|-----------------|
| **Temperature** | ✅ High precision | ✅ Standard |
| **Humidity** | ✅ High precision | ✅ Standard |
| **Pressure** | ❌ | ✅ Barometric |
| **Gas** | ❌ | ✅ Resistance |
| **IAQ** | ❌ | ✅ Air Quality |
| **Payload Size** | ~4 bytes | ~12 bytes |
| **Power Draw** | 0.4 mA | 3.7 mA typical |

---

## Common Hardware

### STM32WL55JC1 Specifications
- **Architecture**: Dual-core
  - Cortex-M4 @ 48 MHz (application core)
  - Cortex-M0+ @ 48 MHz (radio stack)
- **Flash**: 256 KB
- **RAM**: 64 KB
- **Radio**: Integrated SubGHz transceiver (150-960 MHz)
- **LoRaWAN**: Hardware accelerated
- **No FPU**: Integer math required for all calculations

### Power Supply
- **Source**: USB (ST-Link V3)
- **Voltage**: 3.3V to all peripherals
- **Current Budget**:
  - STM32WL55: ~5 mA active, 15 mA peak (TX)
  - LoRa-1: ~25-30 mA total
  - LoRa-2: ~30-40 mA total (BME688 draws more)

### LED Indicators
- **PB15**: Blue LED (user LED on NUCLEO board)
- **Usage**: Heartbeat at 500ms intervals
- **Pattern**: Toggle on each sensor read cycle

---

## Critical Implementation Notes

### 1. No Floating Point!
STM32WL55 does NOT have an FPU. All sensor calculations must use integer math:
```rust
// Temperature: T = -45 + (175 × raw) / 65535
temp_int = -45 + ((175 * temp_raw as i32) / 65535) as i16;

// Humidity: RH = -6 + (125 × raw) / 65535
hum_int = -6 + ((125 * hum_raw as i32) / 65535) as i16;
```

### 2. SHT41 Wake-up Requirement
The SHT41 sensor requires a measurement command (0xFD) to be sent before it will respond to I2C bus scans. This is normal low-power behavior.

### 3. I2C Peripheral Stealing
Both firmwares use `unsafe { I2c::new(...::steal(), ...) }` to recreate I2C peripherals each loop iteration. This allows sharing between sensor and display drivers that consume the bus.

### 4. Display Driver Differences
- **SSD1306**: Uses `I2CDisplayInterface` and `into_buffered_graphics_mode()`
- **SH1106**: Uses `Builder::new().connect_i2c().into()`
- Different initialization but similar embedded-graphics API

---

## Flash Commands Reference

### Flash LoRa-1
```bash
cd firmware/lora-1
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:003E00463234510A33353533
```

### Flash LoRa-2
```bash
cd firmware/lora-2
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:0026003A3234510A33353533
```

### List Available Probes
```bash
probe-rs list
```

---

**Status**: LoRa-1 complete, LoRa-2 pending BME688 integration

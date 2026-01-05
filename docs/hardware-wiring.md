# Week 10: Hardware Wiring Guide

**Hardware**: 2x NUCLEO-WL55JC1, BME688, SHT41, 2x OLED displays
**Last Updated**: 2026-01-03

---

## Overview

This document describes the pin connections for both STM32WL55 LoRaWAN nodes.

---

## STM32WL55JC1 Pinout Reference

### I2C1 Pins
- **SDA**: PB9 (Arduino D14)
- **SCL**: PB8 (Arduino D15)
- **Power**: 3.3V, GND

### SubGHz Radio
- **Integrated peripheral** (no external pins needed)
- Frequency: 868/915 MHz
- Controlled via SPI internally

---

## Node 1: STM32WL55 + BME688 + OLED

### BME688 Environmental Sensor (I2C)

| BME688 Pin | STM32WL55 Pin | Function | Notes |
|------------|---------------|----------|-------|
| VCC | 3.3V | Power | Use 3.3V rail |
| GND | GND | Ground | Common ground |
| SDA | PB9 | I2C1 Data | Shared with OLED |
| SCL | PB8 | I2C1 Clock | Shared with OLED |
| SDO | GND or 3.3V | I2C Address Select | GND=0x76, 3.3V=0x77 |
| CSB | 3.3V | Protocol Select | 3.3V = I2C mode |

**I2C Address**: 0x76 (if SDO=GND) or 0x77 (if SDO=3.3V)

### SSD1306 OLED Display (I2C)

| OLED Pin | STM32WL55 Pin | Function | Notes |
|----------|---------------|----------|-------|
| VCC | 3.3V | Power | Use 3.3V rail |
| GND | GND | Ground | Common ground |
| SDA | PB9 | I2C1 Data | Shared with BME688 |
| SCL | PB8 | I2C1 Clock | Shared with BME688 |

**I2C Address**: 0x3C (fixed for most SSD1306 modules)

### Wiring Diagram (Node 1)

```
STM32WL55JC1                BME688                  SSD1306 OLED
┌─────────────┐            ┌────────┐              ┌──────────┐
│             │            │        │              │          │
│ 3.3V ───────┼────────────┤ VCC   │──────────────┤ VCC      │
│ GND  ───────┼────────────┤ GND   │──────────────┤ GND      │
│             │            │        │              │          │
│ PB9 (SDA)───┼────────────┤ SDA   │──────────────┤ SDA      │
│ PB8 (SCL)───┼────────────┤ SCL   │──────────────┤ SCL      │
│             │            │        │              │          │
└─────────────┘            │ SDO───┼─GND          └──────────┘
                           │ CSB───┼─3.3V
                           └────────┘

         I2C Bus (Shared):
         - BME688: 0x76 or 0x77
         - OLED: 0x3C
```

---

## Node 2: STM32WL55 + SHT41 + OLED

### SHT41 High-Precision Sensor (I2C)

| SHT41 Pin | STM32WL55 Pin | Function | Notes |
|-----------|---------------|----------|-------|
| VCC | 3.3V | Power | 2.4V - 3.6V operating range |
| GND | GND | Ground | Common ground |
| SDA | PB9 | I2C1 Data | Shared with OLED |
| SCL | PB8 | I2C1 Clock | Shared with OLED |

**I2C Address**: 0x44 (default, fixed)

### SSD1306 OLED Display (I2C)

| OLED Pin | STM32WL55 Pin | Function | Notes |
|----------|---------------|----------|-------|
| VCC | 3.3V | Power | Use 3.3V rail |
| GND | GND | Ground | Common ground |
| SDA | PB9 | I2C1 Data | Shared with SHT41 |
| SCL | PB8 | I2C1 Clock | Shared with SHT41 |

**I2C Address**: 0x3C (fixed for most SSD1306 modules)

### Wiring Diagram (Node 2)

```
STM32WL55JC1                SHT41                   SSD1306 OLED
┌─────────────┐            ┌────────┐              ┌──────────┐
│             │            │        │              │          │
│ 3.3V ───────┼────────────┤ VCC   │──────────────┤ VCC      │
│ GND  ───────┼────────────┤ GND   │──────────────┤ GND      │
│             │            │        │              │          │
│ PB9 (SDA)───┼────────────┤ SDA   │──────────────┤ SDA      │
│ PB8 (SCL)───┼────────────┤ SCL   │──────────────┤ SCL      │
│             │            │        │              │          │
└─────────────┘            └────────┘              └──────────┘

         I2C Bus (Shared):
         - SHT41: 0x44
         - OLED: 0x3C
```

---

## I2C Bus Considerations

### Pull-up Resistors
- Most modules include on-board 4.7kΩ pull-ups
- Additional pull-ups usually NOT needed
- If bus issues occur, check with multimeter (should read ~3.3V when idle)

### Bus Speed
- Standard: 100 kHz (safe default)
- Fast: 400 kHz (if all devices support)
- STM32WL55 I2C1 configured in firmware

### Address Conflicts
Both nodes use:
- OLED at 0x3C (same on both)
- Different sensor addresses (0x76/0x77 vs 0x44)
- No conflicts on individual boards

---

## Power Considerations

### Current Draw (per node)

| Component | Idle | Active | Peak |
|-----------|------|--------|------|
| STM32WL55 | 1.5 mA | 5 mA | 15 mA (TX) |
| BME688 | 0.15 µA | 3.7 mA | 12 mA (heating) |
| SHT41 | 0.3 µA | 0.4 mA | 0.4 mA |
| OLED | 10 mA | 20 mA | 20 mA |
| **Total (BME688)** | ~12 mA | ~30 mA | ~50 mA |
| **Total (SHT41)** | ~12 mA | ~25 mA | ~35 mA |

### Power Supply
- USB power sufficient for development (500 mA available)
- For battery operation: optimize OLED usage (sleep mode)
- BME688 gas measurement increases power consumption

---

## Testing Checklist

### Node 1 (BME688)
- [ ] 3.3V between VCC and GND on both modules
- [ ] I2C bus idle at 3.3V (SDA, SCL)
- [ ] `i2cdetect` shows 0x76/0x77 (BME688) and 0x3C (OLED)
- [ ] BME688 responds to soft reset command
- [ ] OLED initializes and displays test pattern

### Node 2 (SHT41)
- [ ] 3.3V between VCC and GND on both modules
- [ ] I2C bus idle at 3.3V (SDA, SCL)
- [ ] `i2cdetect` shows 0x44 (SHT41) and 0x3C (OLED)
- [ ] SHT41 responds to measurement command
- [ ] OLED initializes and displays test pattern

---

## Troubleshooting

### I2C Device Not Detected

**Symptoms**: `i2cdetect` doesn't show expected address

**Checks**:
1. Verify power (3.3V on VCC pin)
2. Verify ground connection
3. Check SDA/SCL connections (not swapped)
4. Check pull-up resistors (should be present)
5. Try lower bus speed (100 kHz)

**BME688 Specific**:
- Check SDO pin (GND=0x76, 3.3V=0x77)
- Check CSB pin (must be 3.3V for I2C mode)

### OLED Not Displaying

**Symptoms**: OLED powers on but shows nothing

**Checks**:
1. Verify I2C communication (0x3C detected)
2. Check initialization sequence in code
3. Try different I2C address (some use 0x3D)
4. Verify display buffer flush after drawing

### Sensor Reading Errors

**BME688**:
- Ensure sufficient delay after soft reset (10ms+)
- Check gas heating duration (long measurement time)
- Verify I2C clock stretching support

**SHT41**:
- Use high-precision measurement mode
- Wait for measurement completion (~10ms)
- Check CRC validation in driver

---

## References

- [STM32WL55 Datasheet](https://www.st.com/resource/en/datasheet/stm32wl55jc.pdf)
- [BME688 Datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bme688-ds000.pdf)
- [SHT41 Datasheet](https://www.sensirion.com/fileadmin/user_upload/customers/sensirion/Dokumente/2_Humidity_Sensors/Datasheets/Sensirion_Humidity_Sensors_SHT4x_Datasheet.pdf)
- [SSD1306 Datasheet](https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf)

---

**Last Updated**: 2026-01-03
**Author**: Tony Mapfumo

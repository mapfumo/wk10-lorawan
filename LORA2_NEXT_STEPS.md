# LoRa-2 Implementation - Next Steps

**Status**: Firmware structure copied, dependencies updated, ready for sensor/display code

---

## Completed So Far

### ✅ Project Setup
- Copied lora-1 firmware to `firmware/lora-2/`
- Updated `Cargo.toml`:
  - Package name: `lora-2`
  - Display driver: `sh1106` (128x64) instead of `ssd1306` (128x32)
  - Sensor driver: `bme680` v0.6 crate (works with BME688)
  - Removed `sht4x` dependency

### ✅ LoRaWAN Configuration
- **DevEUI changed**: `24ce1bfeff091fac` (vs LoRa-1's `23ce1bfeff091fac`)
  - Reversed to little-endian: `[0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x24]`
- AppEUI and AppKey remain the same (shared across nodes)
- All LoRaWAN stack code intact from lora-1

### ✅ Display Imports
- Changed from `ssd1306` to `sh1106` imports in main.rs
- Builder pattern: `sh1106::Builder` instead of `Ssd1306::new()`

---

## What Needs To Be Done

### 1. Replace SHT41 Sensor Code with BME680/688

**Current SHT41 code location** (lines ~280-320 in main.rs):
```rust
// Send measurement command
if i2c.blocking_write(SHT41_ADDR, &[CMD_MEASURE_HIGH_PRECISION]).is_ok() {
    Timer::after_millis(10).await;
    let mut data = [0u8; 6];
    if i2c.blocking_read(SHT41_ADDR, &mut data).is_ok() {
        let temp_raw = ((data[0] as u16) << 8) | (data[1] as u16);
        let hum_raw = ((data[3] as u16) << 8) | (data[4] as u16);
        temp_int = -45 + ((175 * temp_raw as i32) / 65535) as i16;
        hum_int = -6 + ((125 * hum_raw as i32) / 65535) as i16;
    }
}
```

**Replace with BME680 crate** (using `bme680` v0.6):
```rust
use bme680::{Bme680, OversamplingSetting, IIRFilterSize, Settings};

// In initialization section:
let mut bme = Bme680::init(i2c, Delay, I2CAddress::Primary)?; // or Secondary for 0x77

let settings = Settings {
    humidity_oversampling: OversamplingSetting::OS2x,
    temperature_oversampling: OversamplingSetting::OS8x,
    pressure_oversampling: OversamplingSetting::OS4x,
    filter: IIRFilterSize::Size3,
    gas_measurement: None, // Can enable later for air quality
};
bme.set_settings(&mut delay, settings)?;

// In sensor reading loop:
let (temp, pressure, humidity, gas_resistance) = bme.measure(&mut delay)?;
temp_int = (temp.celsius() as i16);  // BME680 returns f32, convert carefully!
hum_int = (humidity as i16);
pressure_int = (pressure / 100.0) as i16;  // Convert Pa to hPa
```

**CRITICAL**: The `bme680` crate v0.6 uses `embedded-hal` 0.2.x, NOT 1.0!
Check compatibility or use manual register access if needed.

### 2. Update SSD1306 Display Code to SH1106

**Current SSD1306 code** (lines ~330-370):
```rust
let interface = I2CDisplayInterface::new(i2c);
let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();
display.init()?;
```

**Replace with SH1106** (128x64):
```rust
use sh1106::{prelude::*, Builder};

let mut display: GraphicsMode<_> = Builder::new()
    .connect_i2c(i2c)
    .into();
display.init()?;

// Display now has 8 lines instead of 4 (64 pixels / 8 = 8 lines at 6x10 font)
```

**Updated OLED layout for 128x64**:
```
Line 1 (y=6):  LoRa-2  TX:3
Line 2 (y=16): Temp: 25C  RH: 60%
Line 3 (y=26): Press: 1013 hPa
Line 4 (y=36): Gas: 1234 kOhm
Line 5 (y=46): SNR: 11 dB
Line 6 (y=56): RSSI: -17 dBm
```

### 3. Expand LoRaWAN Payload to 12 Bytes

**Current 4-byte payload** (SHT41):
```rust
let payload: [u8; 4] = [
    (temp_encoded >> 8) as u8,   // Temp MSB
    temp_encoded as u8,           // Temp LSB
    (hum_encoded >> 8) as u8,     // Humidity MSB
    hum_encoded as u8,            // Humidity LSB
];
```

**New 12-byte payload** (BME688):
```rust
// Temperature: signed 16-bit (°C × 100)
let temp_encoded = (temp_int * 100) as i16;
// Humidity: unsigned 16-bit (% × 100)
let hum_encoded = (hum_int * 100) as u16;
// Pressure: unsigned 16-bit (hPa × 10)
let pressure_encoded = (pressure_int * 10) as u16;
// Gas resistance: unsigned 16-bit (kOhm, capped at 65535)
let gas_encoded = gas_resistance.min(65535) as u16;
// Reserved: 4 bytes for future use (air quality index, etc.)

let payload: [u8; 12] = [
    (temp_encoded >> 8) as u8,      // 0: Temp MSB
    temp_encoded as u8,              // 1: Temp LSB
    (hum_encoded >> 8) as u8,        // 2: Humidity MSB
    hum_encoded as u8,               // 3: Humidity LSB
    (pressure_encoded >> 8) as u8,   // 4: Pressure MSB
    pressure_encoded as u8,          // 5: Pressure LSB
    (gas_encoded >> 8) as u8,        // 6: Gas MSB
    gas_encoded as u8,               // 7: Gas LSB
    0, 0, 0, 0,                      // 8-11: Reserved
];
```

### 4. Update Payload Decoder Script

Add BME688 12-byte decoder to `decode_payload.py`:

```python
def decode_bme688_payload(base64_data):
    """Decode LoRa-2 12-byte BME688 payload"""
    payload = base64.b64decode(base64_data)

    if len(payload) != 12:
        print(f"Error: Expected 12 bytes, got {len(payload)}")
        return None

    # Unpack: >h = signed 16-bit BE, >H = unsigned 16-bit BE
    temp_raw, hum_raw, press_raw, gas_raw = struct.unpack('>hHHH', payload[:8])

    return {
        'temperature_c': temp_raw / 100.0,
        'humidity_percent': hum_raw / 100.0,
        'pressure_hpa': press_raw / 10.0,
        'gas_resistance_kohm': gas_raw,
    }
```

---

## Hardware Configuration

### LoRa-2 Node Specs
- **Board**: NUCLEO-WL55JC1 (second board)
- **Probe ID**: `0483:374e:0026003A3234510A33353533`
- **Sensor**: BME688 environmental sensor
  - I2C address: 0x76 or 0x77
  - Measures: Temperature, Humidity, Pressure, Gas Resistance
- **Display**: SH1106 OLED 128x64
  - I2C address: 0x3C
  - Larger than LoRa-1's 128x32
- **I2C Bus**: Same as LoRa-1 (I2C2: PA12=SCL, PA11=SDA)

### Flash Command for LoRa-2
```bash
cd firmware/lora-2
cargo build --release
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:0026003A3234510A33353533 target/thumbv7em-none-eabihf/release/lora-2
```

---

## Testing Checklist

### After Implementation:
1. ☐ Build succeeds without errors
2. ☐ Sensor initializes and reads data
3. ☐ Display shows all 4 sensor values
4. ☐ OTAA join succeeds with DevEUI `24ce1bfeff091fac`
5. ☐ Uplinks transmit every 60 seconds with 12-byte payload
6. ☐ Gateway receives packets from both LoRa-1 and LoRa-2
7. ☐ Payload decoder correctly extracts all 4 sensor values
8. ☐ MQTT messages arrive for both devices

### Expected Gateway MQTT Topics:
```
application/TOT/device/23ce1bfeff091fac/rx  (LoRa-1, 4 bytes)
application/TOT/device/24ce1bfeff091fac/rx  (LoRa-2, 12 bytes)
```

---

## Known Issues & Notes

### BME680 Crate Compatibility
The `bme680` v0.6 crate uses `embedded-hal` 0.2.x (old version).
Embassy 0.2.0 uses `embedded-hal` 1.0 (new version).

**Options:**
1. Use `bme680` v0.6 with compatibility shim
2. Manual register access (simpler but more code)
3. Find a newer BME68x crate that supports `embedded-hal` 1.0

**Recommendation**: Try `bme680` first, fall back to manual registers if needed.

### Integer Math (No FPU!)
The BME680 crate might return `f32` values. Convert carefully:
```rust
// BAD (will HardFault if BME680 uses f32 internally):
let temp_int = temp.celsius() as i16;

// GOOD (if you must use floats, do it quickly):
let temp_f32 = temp.celsius();  // Get f32
let temp_int = (temp_f32 * 100.0) as i16;  // Convert immediately
```

Or better: check if the crate has integer-only mode.

---

## Estimated Effort

- **Sensor replacement**: 30-60 minutes (if `bme680` crate works)
- **Display update**: 15-30 minutes (straightforward)
- **Payload expansion**: 10-15 minutes (copy pattern from lora-1)
- **Testing**: 30-45 minutes (build, flash, verify MQTT)

**Total**: ~2-3 hours for complete LoRa-2 implementation

---

## Next Actions

1. Check `bme680` crate documentation for `embedded-hal` 1.0 support
2. If compatible, implement BME688 sensor reading
3. Update display to SH1106 128x64
4. Expand payload to 12 bytes
5. Test build
6. Flash to second STM32WL55 board
7. Verify both nodes transmitting simultaneously
8. Update `decode_payload.py` with 12-byte decoder
9. Monitor both devices via MQTT

**Start with**: Sensor implementation (biggest unknown)
**Then**: Display (known working pattern)
**Finally**: Payload (trivial extension)

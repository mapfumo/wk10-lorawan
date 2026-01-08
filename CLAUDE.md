# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

This is **Week 10 of a 4-month embedded systems learning project**: LoRaWAN implementation using STM32WL55 microcontrollers with native radio peripherals. The project creates a 2-node LoRaWAN sensor network that integrates with a RAK7268V2 gateway for a unified 4-node monitoring system (combining Week 9's Ethernet Modbus nodes).

**Current Status**: Gateway operational ✅ | LoRa-1 hardware complete ✅ | LoRaWAN stack implementation pending

---

## Hardware Architecture

### Two Independent STM32WL55 Nodes

**LoRa-1 (003E00463234510A33353533)** - WORKING:
- SHT41 temperature/humidity sensor (I2C 0x44)
- SSD1306 OLED 128x32 display (I2C 0x3C)
- I2C2: PA12 (SCL), PA11 (SDA)
- Firmware: `firmware/lora-1/`

**LoRa-2 (0026003A3234510A33353533)** - PLANNED:
- BME688 environmental sensor (I2C 0x76/0x77)
- SH1106 OLED 128x64 display (I2C 0x3C)
- Firmware: `firmware/lora-2/`

### Gateway
- RAK7268V2 with **built-in LoRa Server** (NOT ChirpStack)
- AU915 Sub-band 2 (915.2-916.6 MHz)
- MQTT broker at 127.0.0.1:1883 (on gateway)
- Application "TOT" with pre-registered device credentials

---

## Building and Flashing

### Prerequisites
```bash
# Install probe-rs (required for flashing)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Add Rust embedded target
rustup target add thumbv7em-none-eabihf
```

### Build Commands

**LoRa-1:**
```bash
cd firmware/lora-1
cargo build                    # Debug build
cargo build --release          # Release build (optimized for size)
```

**LoRa-2:**
```bash
cd firmware/lora-2
cargo build
cargo build --release
```

### Flash and Run

**LoRa-1 (auto-detects probe):**
```bash
cd firmware/lora-1
cargo run                      # Debug: builds + flashes + attaches RTT logs
cargo run --release            # Release build
```

**LoRa-1 (explicit probe ID):**
```bash
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:003E00463234510A33353533
```

**LoRa-2 (explicit probe ID):**
```bash
probe-rs run --chip STM32WL55JCIx --probe 0483:374e:0026003A3234510A33353533
```

### Monitor Logs Only
```bash
probe-rs attach --chip STM32WL55JCIx
```

### List Available Probes
```bash
probe-rs list
```

---

## Critical STM32WL55 Constraints

### 1. NO FLOATING POINT UNIT (FPU)
The STM32WL55 does **NOT** have an FPU. Using `f32` or `f64` will cause a **HardFault**.

**WRONG:**
```rust
let temp: f32 = 25.5;
info!("Temp: {:.1}°C", temp);  // HardFault!
```

**CORRECT:**
```rust
let temp_int: i16 = 255;  // 25.5°C * 10
info!("Temp: {}°C", temp_int / 10);  // Works
```

**Sensor Conversion (Integer Math Only):**
```rust
// SHT41 temperature: T = -45 + (175 × raw) / 65535
let temp_raw: u16 = read_sensor();
let temp_celsius: i16 = -45 + ((175 * temp_raw as i32) / 65535) as i16;

// SHT41 humidity: RH = -6 + (125 × raw) / 65535
let hum_raw: u16 = read_sensor();
let humidity: i16 = -6 + ((125 * hum_raw as i32) / 65535) as i16;
```

### 2. SHT41 Sensor Wake-up Requirement
The SHT41 sensor **requires a measurement command** before it will respond to I2C scans:

```rust
const SHT41_ADDR: u8 = 0x44;
const CMD_MEASURE_HIGH_PRECISION: u8 = 0xFD;

// Wake up sensor FIRST
i2c.blocking_write(SHT41_ADDR, &[CMD_MEASURE_HIGH_PRECISION])?;
Timer::after_millis(10).await;

// Now sensor will respond to I2C reads
i2c.blocking_read(SHT41_ADDR, &mut data)?;
```

### 3. I2C Peripheral Stealing Pattern
Both firmware projects use `unsafe` peripheral stealing to share I2C between sensor and display drivers:

```rust
loop {
    // Create I2C for sensor reading
    let mut i2c = unsafe {
        I2c::new(I2C2::steal(), PA12::steal(), PA11::steal(), ...)
    };

    // Read sensor
    i2c.blocking_write(...)?;
    i2c.blocking_read(...)?;

    // Create display with same I2C
    let mut display = Ssd1306::new(...).into_buffered_graphics_mode();
    display.init()?;
    // Draw text
    display.flush()?;

    // Both i2c and display dropped here, hardware released
    Timer::after_secs(2).await;
}
```

This is safe because:
- Single-threaded execution (Embassy async)
- Exclusive access per iteration
- Hardware naturally released on drop

### 4. Display Driver Differences

**LoRa-1 (SSD1306 128x32):**
```rust
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

let interface = I2CDisplayInterface::new(i2c);
let mut display = Ssd1306::new(
    interface,
    DisplaySize128x32,
    DisplayRotation::Rotate0
).into_buffered_graphics_mode();
display.init()?;
```

**LoRa-2 (SH1106 128x64):**
```rust
use sh1106::{prelude::*, Builder};

let mut display: GraphicsMode<_> = Builder::new()
    .connect_i2c(i2c)
    .into();
display.init()?;
```

---

## LoRaWAN Configuration

### Gateway Credentials (Application "TOT")
See `LORAWAN_CREDENTIALS.md` for complete reference.

**CRITICAL: LoRaWAN Byte Order Shenanigans! 🎭**

LoRaWAN transmits EUIs in **little-endian** (LSB first) over-the-air, but displays them in **big-endian** (MSB first) in dashboards/configs. When defining credentials in firmware:

- **DevEUI & AppEUI**: Must be REVERSED (little-endian) for `lorawan-device` crate
- **AppKey**: Stays in big-endian (MSB first)

**Example: Gateway shows DevEUI as `23ce1bfeff091fac`**
- ❌ Wrong: `[0x23, 0xCE, 0x1B, 0xFE, 0xFF, 0x09, 0x1F, 0xAC]` (will be parsed as gateway EUI!)
- ✅ Correct: `[0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23]` (reversed bytes)

**Quick Copy-Paste for Firmware:**
```rust
// LoRa-1 (pre-registered as "STM_Nodes")
// Gateway shows: DevEUI=23ce1bfeff091fac, AppEUI=b130a864c5295356
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23]; // REVERSED
const APP_EUI: [u8; 8] = [0x56, 0x53, 0x29, 0xC5, 0x64, 0xA8, 0x30, 0xB1]; // REVERSED
const APP_KEY: [u8; 16] = [
    0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
    0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B,
]; // NOT reversed (stays MSB first)

// LoRa-2 (suggested DevEUI for when adding)
// Gateway will show: DevEUI=24ce1bfeff091fac (example)
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x24]; // REVERSED
```

**Why This Matters:**
If you get byte order wrong, the gateway will parse your DevEUI as the gateway's own EUI and reject the join request with `nsParseJoinReq: unknow mote` error.

### AU915 Sub-band 2 Configuration
```rust
const CHANNEL_FREQ: [u32; 8] = [
    915200000, 915400000, 915600000, 915800000,
    916000000, 916200000, 916400000, 916600000,
];
const RX2_FREQ: u32 = 923300000;  // 923.3 MHz
const RX2_DR: u8 = 8;             // DR8 (SF12, BW500)
```

### MQTT Topics (for testing)
```bash
# Subscribe to all TOT application messages
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/#" -v

# Subscribe to LoRa-1 uplinks
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/23ce1bfeff091fac/rx" -v

# Monitor join events
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/+/join" -v

# Gateway statistics
mosquitto_sub -h <gateway-ip> -t "gateway/ac1f09fffe1bce23/stats" -v
```

---

## Payload Encoding

### LoRa-1 (SHT41) - 4 bytes
```rust
let temp_encoded: i16 = (temp_celsius * 100) as i16;
let hum_encoded: u16 = (humidity * 100) as u16;

let payload: [u8; 4] = [
    (temp_encoded >> 8) as u8,   // Temp MSB
    temp_encoded as u8,           // Temp LSB
    (hum_encoded >> 8) as u8,     // Humidity MSB
    hum_encoded as u8,            // Humidity LSB
];
```

### LoRa-2 (BME688) - 12 bytes (planned)
```
Bytes 0-3:   Temperature (f32 IEEE 754) - NOTE: Needs integer conversion!
Bytes 4-7:   Humidity (f32 IEEE 754)
Bytes 8-9:   Pressure (u16, hPa * 10)
Bytes 10-11: Gas Resistance (u16, kΩ)
```

---

## Firmware Architecture (Embassy Async)

Both firmware projects use the **Embassy async framework**:

### Main Components
1. **Executor**: `#[embassy_executor::main]` - async runtime
2. **Timers**: `embassy_time::Timer` - non-blocking delays
3. **HAL**: `embassy_stm32` - peripheral drivers
4. **Logging**: `defmt` + `defmt-rtt` - printf-style debugging via RTT

### Typical Main Loop Structure
```rust
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // 1. Initialize peripherals
    let config = Config::default();
    let p = embassy_stm32::init(config);

    // 2. Configure I2C
    let mut i2c_config = I2cConfig::default();
    i2c_config.sda_pullup = true;
    i2c_config.scl_pullup = true;

    // 3. Main loop with async delays
    loop {
        // Read sensor (may block briefly)
        let data = read_sensor(&mut i2c).await;

        // Update display
        update_display(&data).await;

        // Async delay (non-blocking)
        Timer::after_secs(2).await;
    }
}
```

---

## Documentation Structure

### Reference Documents
- **README.md** - Project overview, build instructions, architecture
- **HARDWARE_CONFIG.md** - Complete hardware specs for both nodes
- **LORAWAN_CREDENTIALS.md** - Copy-paste credentials for firmware
- **docs/rak7268v2-config.md** - Full gateway configuration
- **NOTES.md** - Technical implementation notes and lessons learned
- **TODO.md** - Task tracking and progress
- **STATUS_SUMMARY.md** - Current project status

### Quick Reference Files
- **board_id.txt** - Maps node names to probe serial numbers
- **TROUBLESHOOTING_WL55.md** - Common issues and solutions

---

## Common Development Workflows

### Testing Hardware Changes
1. Modify `firmware/lora-1/src/main.rs`
2. Flash: `cd firmware/lora-1 && cargo run`
3. Monitor RTT logs via probe-rs
4. Verify on OLED display

### Adding LoRaWAN Stack (Next Step)
1. Research libraries: `embassy-lora`, `lora-phy`, `lorawan-device`
2. Add dependency to `firmware/lora-1/Cargo.toml`
3. Enable SubGHz peripheral via `embassy_stm32`
4. Configure AU915 sub-band 2 channels
5. Implement OTAA join with credentials from `LORAWAN_CREDENTIALS.md`
6. Test join via MQTT: `mosquitto_sub -h <gateway-ip> -t "application/TOT/device/+/join" -v`

### Switching Between Nodes
```bash
# Flash LoRa-1
cd firmware/lora-1 && cargo run --release

# Flash LoRa-2 (in separate terminal)
cd firmware/lora-2 && cargo run --release
```

### Debugging I2C Issues
1. Check bus scan in RTT logs for device addresses
2. Verify pullup resistors enabled: `i2c_config.sda_pullup = true`
3. For SHT41: ensure wake-up command sent before reading
4. Use oscilloscope/logic analyzer on SCL/SDA if needed

---

## Important Notes

### Network Server is RAK Built-in, NOT ChirpStack
The gateway runs a **built-in LoRa Server**, not ChirpStack. When implementing LoRaWAN:
- Use credentials from `LORAWAN_CREDENTIALS.md`
- MQTT topics follow RAK format: `application/TOT/device/{DevEUI}/rx`
- Gateway MQTT broker is local: `127.0.0.1:1883` (on gateway)
- Auto-add is enabled: new OTAA devices automatically register

### Build Configuration
- **Target**: `thumbv7em-none-eabihf` (Cortex-M4F) - see `firmware/lora-1/.cargo/config.toml`
- **Note**: Despite "hf" (hardware float) in target name, the STM32WL55 lacks FPU. Use integer math.
- **Optimization**: Release builds use `opt-level = "z"` (size optimization)
- **Linker**: Uses LLD with custom `link.x` and `defmt.x` scripts

### Embassy Features
Both projects require these Embassy features:
```toml
embassy-executor = { version = "0.6.3", features = ["arch-cortex-m", "executor-thread", "integrated-timers"] }
embassy-time = { version = "0.3.2", features = ["tick-hz-32_768"] }
embassy-stm32 = { version = "0.1.0", features = [
    "stm32wl55jc-cm4",  # Cortex-M4 core (not M0+ radio core)
    "time-driver-any",
    "memory-x",
    "unstable-pac",
    "exti",
] }
```

### Logging with defmt
```rust
use defmt::*;

info!("Normal log message");
info!("Temperature: {}°C", temp);
warn!("Sensor not responding");
error!("I2C error occurred");

// Format hex: {:02X}
info!("I2C address: 0x{:02X}", addr);
```

---

## Project Goals (Week 10)

**Primary Objective**: Implement LoRaWAN OTAA on STM32WL55 using native SubGHz radio peripheral.

**Learning Focus**:
- STM32WL SubGHz radio programming
- LoRaWAN MAC layer (OTAA join, frame counters, MIC)
- AU915 frequency plan and channel configuration
- Duty cycle compliance
- Payload optimization for LPWAN
- Embassy async embedded framework

**Integration**: Combine with Week 9's Ethernet Modbus nodes for a 4-node unified monitoring system (MQTT → InfluxDB → Grafana).

---

**Last Updated**: 2026-01-08

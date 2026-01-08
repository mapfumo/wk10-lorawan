# LoRaWAN Credentials Reference

**Quick reference for STM32WL55 firmware implementation**

---

## Gateway Information

| Parameter | Value |
|-----------|-------|
| Gateway EUI | `ac1f09fffe1bce23` |
| Region | AU915 |
| Sub-band | 2 (channels 8-15) |
| Uplink Channels | 915.2, 915.4, 915.6, 915.8, 916.0, 916.2, 916.4, 916.6 MHz |
| RX2 Frequency | 923.3 MHz (923300000 Hz) |
| RX2 Data Rate | DR8 (SF12, BW500) |
| MQTT Broker | 127.0.0.1:1883 (on gateway) |

---

## Application "TOT" Credentials

**For use in STM32WL55 firmware:**

```rust
// Application TOT
const APP_EUI: [u8; 8] = [0xB1, 0x30, 0xA8, 0x64, 0xC5, 0x29, 0x53, 0x56];
const APP_KEY: [u8; 16] = [
    0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
    0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B
];
```

**Hex String Format:**
- AppEUI: `b130a864c5295356`
- AppKey: `b726739b78ec4b9e9234e5d35ea9681b`

---

## Device: LoRa-1 (STM_Nodes)

**Pre-registered device for LoRa-1:**

```rust
// Device: STM_Nodes (LoRa-1)
const DEV_EUI: [u8; 8] = [0x23, 0xCE, 0x1B, 0xFE, 0xFF, 0x09, 0x1F, 0xAC];
```

**Hex String Format:**
- DevEUI: `23ce1bfeff091fac`

**Device Configuration:**
- Name: "STM_Nodes"
- Class: A (bi-directional, scheduled RX)
- Activation: OTAA
- LoRaWAN Version: 1.0.3
- Frame Counter Width: 32-bit

---

## Device: LoRa-2 (To Be Added)

**Suggested DevEUI for LoRa-2:**

```rust
// Device: LoRa-2 (not yet registered)
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x24];
```

**Hex String Format:**
- DevEUI: `ac1f09fffe1bce24`

**Note**: This device needs to be registered on the gateway. Auto-add is enabled, so it will be added automatically on first join attempt.

---

## AU915 Channel Configuration

### Uplink Channels (8 channels, 125 kHz BW)

```rust
const CHANNEL_FREQ: [u32; 8] = [
    915200000, // Channel 0: 915.2 MHz
    915400000, // Channel 1: 915.4 MHz
    915600000, // Channel 2: 915.6 MHz
    915800000, // Channel 3: 915.8 MHz
    916000000, // Channel 4: 916.0 MHz
    916200000, // Channel 5: 916.2 MHz
    916400000, // Channel 6: 916.4 MHz
    916600000, // Channel 7: 916.6 MHz
];
```

### RX Windows Configuration

```rust
// RX1: Uses same frequency as uplink, with RX1DrOffset = 0
// RX2: Fixed frequency and data rate
const RX2_FREQ: u32 = 923300000;  // 923.3 MHz
const RX2_DR: u8 = 8;             // DR8 (SF12, BW500)
```

### Timing Parameters

```rust
const RECEIVE_DELAY1: u32 = 1000;  // 1 second (milliseconds)
const RECEIVE_DELAY2: u32 = 2000;  // 2 seconds (RX2 opens 1s after RX1)
```

---

## MQTT Topics

### LoRa-1 (DevEUI: 23ce1bfeff091fac)

```
Join:     application/TOT/device/23ce1bfeff091fac/join
Uplink:   application/TOT/device/23ce1bfeff091fac/rx
Downlink: application/TOT/device/23ce1bfeff091fac/tx
Ack:      application/TOT/device/23ce1bfeff091fac/ack
Status:   application/TOT/device/23ce1bfeff091fac/status
```

### LoRa-2 (DevEUI: ac1f09fffe1bce24)

```
Join:     application/TOT/device/ac1f09fffe1bce24/join
Uplink:   application/TOT/device/ac1f09fffe1bce24/rx
Downlink: application/TOT/device/ac1f09fffe1bce24/tx
Ack:      application/TOT/device/ac1f09fffe1bce24/ack
Status:   application/TOT/device/ac1f09fffe1bce24/status
```

### Wildcard Subscriptions

```bash
# All TOT application messages
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/#" -v

# All join events
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/+/join" -v

# All uplink data
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/+/rx" -v

# Gateway statistics
mosquitto_sub -h <gateway-ip> -t "gateway/ac1f09fffe1bce23/stats" -v
```

---

## Payload Formats

### LoRa-1: SHT41 (4 bytes)

```rust
// Encode
let temp_encoded: i16 = (temp_celsius * 100.0) as i16;  // °C * 100
let hum_encoded: u16 = (humidity * 100.0) as u16;       // % * 100

let payload: [u8; 4] = [
    (temp_encoded >> 8) as u8,   // Temp MSB
    temp_encoded as u8,           // Temp LSB
    (hum_encoded >> 8) as u8,     // Humidity MSB
    hum_encoded as u8,            // Humidity LSB
];

// Decode (Python example)
temp = int.from_bytes(payload[0:2], byteorder='big', signed=True) / 100.0
humidity = int.from_bytes(payload[2:4], byteorder='big', signed=False) / 100.0
```

### LoRa-2: BME688 (12 bytes)

```rust
// Temperature (4 bytes, f32 IEEE 754)
// Humidity (4 bytes, f32 IEEE 754)
// Pressure (2 bytes, u16, hPa * 10)
// Gas Resistance (2 bytes, u16, kΩ)

let payload: [u8; 12] = [
    temp_bytes[0], temp_bytes[1], temp_bytes[2], temp_bytes[3],
    hum_bytes[0], hum_bytes[1], hum_bytes[2], hum_bytes[3],
    (pressure >> 8) as u8, pressure as u8,
    (gas >> 8) as u8, gas as u8,
];
```

---

## Security Notes

⚠️ **IMPORTANT**: These credentials are for DEVELOPMENT/TESTING only!

For production deployment:
1. **Rotate AppKey** - Generate a new secure key
2. **Use unique DevEUIs** - Each device must have a unique identifier
3. **Enable MQTT authentication** - Currently disabled on gateway
4. **Consider TLS** - For production MQTT connections
5. **Disable auto-add** - Manually register each device
6. **Monitor access logs** - Track unauthorized join attempts

---

**Source**: Gateway configuration via `loraserver_config` command
**Last Updated**: 2026-01-08
**Full Documentation**: [docs/rak7268v2-config.md](docs/rak7268v2-config.md)

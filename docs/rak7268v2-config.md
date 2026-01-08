# RAK7268V2 Gateway Configuration

**Gateway Model**: RAK WisGate Edge Lite 2 (RAK7268V2)
**LoRaWAN Region**: AU915
**Network Server**: Built-in LoRa Server (not ChirpStack)
**Date Configured**: 2026-01-08
**Status**: ✅ Operational

---

## Gateway Information

### Hardware Details
- **Gateway EUI**: `ac1f09fffe1bce23`
- **Model**: RAK7268V2
- **Concentrator**: SX1302 (8-channel)
- **Frequency Plan**: AU915 (Australia/New Zealand)
- **Connectivity**: LAN + WiFi

### Network Configuration
- **UDP Port**: 1698 (packet forwarder)
- **MQTT Broker**: 127.0.0.1:1883 (local)
- **Protocol**: Local (built-in network server)

---

## LoRaWAN Configuration

### Region Settings
```json
{
  "region": "AU915",
  "region_variation": "1",
  "network_id": "1",
  "adr_enable": "1",
  "adr_margin_db": "10",
  "dr_min": "0",
  "dr_max": "6"
}
```

### Channel Configuration (AU915 Sub-band 2)

**Uplink Channels** (8 channels, 125 kHz):
- Channel 0: 915.2 MHz
- Channel 1: 915.4 MHz
- Channel 2: 915.6 MHz
- Channel 3: 915.8 MHz
- Channel 4: 916.0 MHz
- Channel 5: 916.2 MHz
- Channel 6: 916.4 MHz
- Channel 7: 916.6 MHz

**Downlink Channel** (500 kHz):
- Channel 8: 915.9 MHz (DR8, BW 500 kHz)

### RX2 Window Configuration
- **Frequency**: 923.3 MHz (923300000 Hz)
- **Data Rate**: DR8
- **Bandwidth**: 500 kHz

### Timing Parameters
- **RECEIVE_DELAY1**: 1 second
- **Rx1DrOffset**: 0
- **Status Interval**: 600 seconds (10 minutes)
- **UpDwellTime**: 1 (enabled)
- **DownDwellTime**: 0 (disabled)

---

## Application Configuration

### Application 1: "TOT" (Testing/IIoT)

**Application Settings**:
```json
{
  "id": 1,
  "name": "TOT",
  "auth_mode": 0,
  "app_key": "b726739b78ec4b9e9234e5d35ea9681b",
  "app_eui": "b130a864c5295356",
  "auto_add": "1",
  "data_encode": "base64",
  "PayloadFormat": "none",
  "OnlyDataObject": "0",
  "ReportRadioInfo": "1"
}
```

**Key Configuration**:
- **AppEUI**: `b130a864c5295356`
- **AppKey**: `b726739b78ec4b9e9234e5d35ea9681b`
- **Auto-add devices**: Enabled (OTAA devices auto-register)
- **Data encoding**: Base64
- **Radio info**: Enabled (includes RSSI, SNR, etc.)

### Registered Device: "STM_Nodes"

**Device Configuration**:
```json
{
  "dev_eui": "23ce1bfeff091fac",
  "name": "STM_Nodes",
  "Class": "A",
  "activation": "OTAA",
  "valid_fcnt": true,
  "fcntwidth": "32",
  "lptp": "0",
  "loRaMacVersion": "1.0.3"
}
```

**Device Details**:
- **DevEUI**: `23ce1bfeff091fac`
- **Device Class**: Class A (bi-directional, scheduled RX)
- **Activation**: OTAA (Over-The-Air Activation)
- **LoRaWAN Version**: 1.0.3
- **Frame Counter**: 32-bit

---

## MQTT Topics

### Gateway Bridge Topics
```
Uplink:   gateway/ac1f09fffe1bce23/rx
Downlink: gateway/ac1f09fffe1bce23/tx
Stats:    gateway/ac1f09fffe1bce23/stats
Ack:      gateway/ac1f09fffe1bce23/ack
```

### Application Topics (Template)
```
Join:     application/{{application_name}}/device/{{device_EUI}}/join
Uplink:   application/{{application_name}}/device/{{device_EUI}}/rx
Downlink: application/{{application_name}}/device/{{device_EUI}}/tx
Ack:      application/{{application_name}}/device/{{device_EUI}}/ack
Status:   application/{{application_name}}/device/{{device_EUI}}/status
```

### Actual Topics for STM_Nodes Device
```
Join:   application/TOT/device/23ce1bfeff091fac/join
Uplink: application/TOT/device/23ce1bfeff091fac/rx
Ack:    application/TOT/device/23ce1bfeff091fac/ack
Status: application/TOT/device/23ce1bfeff091fac/status
```

---

## STM32WL55 Device Configuration

### Required Credentials for OTAA

Copy these values to your STM32WL55 firmware:

```rust
// LoRaWAN Device Credentials
const DEV_EUI: [u8; 8] = [0x23, 0xCE, 0x1B, 0xFE, 0xFF, 0x09, 0x1F, 0xAC]; // Gateway registered
const APP_EUI: [u8; 8] = [0xB1, 0x30, 0xA8, 0x64, 0xC5, 0x29, 0x53, 0x56]; // TOT application
const APP_KEY: [u8; 16] = [
    0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
    0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B
]; // TOT application key
```

**Important Notes**:
1. DevEUI `23ce1bfeff091fac` is already registered as "STM_Nodes"
2. This DevEUI should be used for **one device only** (likely LoRa-1)
3. For LoRa-2, you'll need to add a second device with a different DevEUI
4. Auto-add is enabled, so new OTAA devices will be added automatically

### Generating DevEUI for Additional Devices

For STM32WL55, you can:
1. Use the UID (Unique ID) from the chip
2. Generate from MAC address pattern
3. Use random but unique value

**LoRa-1** (NODE_1):
- Serial: `003E00463234510A33353533`
- Suggested DevEUI: `23ce1bfeff091fac` (already registered)

**LoRa-2** (NODE_2):
- Serial: `0026003A3234510A33353533`
- Suggested DevEUI: `ac1f09fffe1bce24` (needs to be added)

---

## Testing MQTT Connection

### Subscribe to Gateway Stats
```bash
mosquitto_sub -h <gateway-ip> -t "gateway/ac1f09fffe1bce23/stats" -v
```

### Subscribe to All Application Messages
```bash
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/#" -v
```

### Subscribe to Specific Device Uplinks
```bash
mosquitto_sub -h <gateway-ip> -t "application/TOT/device/23ce1bfeff091fac/rx" -v
```

---

## AU915 Sub-band Information

### Why Sub-band 2?
The gateway is configured for **AU915 channels 8-15** (915.2-916.6 MHz), which is:
- **Sub-band 2** in TTN/Helium terminology
- Commonly used in Australia
- Compatible with most AU915 devices

### STM32WL55 Channel Configuration
Your STM32WL55 firmware should use these 8 channels:
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

### Downlink Configuration
```rust
const RX2_FREQ: u32 = 923300000;  // 923.3 MHz
const RX2_DR: u8 = 8;             // DR8 (SF12, BW500)
```

---

## Troubleshooting

### Check Gateway Status
```bash
# SSH into gateway
ssh root@<gateway-ip>

# Check LoRa server status
loraserver_status

# View configuration
loraserver_config

# Check logs
logread | grep lora
```

### Common Issues

1. **Device not joining**:
   - Verify DevEUI/AppEUI/AppKey match exactly
   - Check device is using correct channel frequencies
   - Ensure gateway is receiving packets (check stats)
   - Verify AU915 sub-band 2 configuration

2. **No uplink data**:
   - Check device is in Class A mode
   - Verify frame counter is valid
   - Check duty cycle compliance
   - Monitor MQTT topics for messages

3. **Weak signal (low RSSI)**:
   - Move device closer to gateway
   - Check antenna connections
   - Try higher spreading factor (SF9-SF12)
   - Verify channel configuration

---

## Security Considerations

### Published Information
⚠️ **IMPORTANT**: This configuration contains:
- Gateway EUI (public)
- AppEUI (public)
- AppKey (should be SECRET - rotate for production!)
- DevEUI (public)

### For Production Use:
1. **Rotate AppKey** - Generate new secure key
2. **Use unique DevEUIs** - Never reuse across devices
3. **Enable MQTT authentication** - Currently disabled
4. **Consider TLS** - For production MQTT connections
5. **Disable auto-add** - Manually register devices
6. **Monitor access logs** - Track unauthorized attempts

---

## Quick Reference

| Parameter | Value |
|-----------|-------|
| Gateway EUI | `ac1f09fffe1bce23` |
| Region | AU915 |
| Sub-band | 2 (channels 8-15) |
| AppEUI | `b130a864c5295356` |
| AppKey | `b726739b78ec4b9e9234e5d35ea9681b` |
| Device DevEUI | `23ce1bfeff091fac` |
| Device Class | A |
| LoRaWAN Version | 1.0.3 |
| MQTT Broker | 127.0.0.1:1883 |
| Uplink Topic | `application/TOT/device/23ce1bfeff091fac/rx` |

---

**Last Updated**: 2026-01-08
**Configuration Source**: `loraserver_config` output from RAK7268V2

# STM32WL55 Connection Troubleshooting

## Issue: probe-rs cannot connect to STM32WL55JC

### Symptoms
- ST-Link detected by `lsusb` and `probe-rs list`
- Error: `JtagGetIdcodeError` or `SwdDpError`
- ST-Link LED flashing green/orange/red

### Checklist

#### 1. Hardware Jumpers (CRITICAL for NUCLEO-WL55JC1)
- [ ] Check JP1 jumper (should be present for ST-Link connection)
- [ ] Check CN8 jumpers:
  - JP3 (NRST): Should be ON (connected)
  - JP4 (T_VCP_RX): Optional
  - JP5 (T_VCP_TX): Optional

#### 2. Power
- [ ] Green LD1 (PWR) LED should be solidly ON
- [ ] USB cable fully inserted
- [ ] Try different USB port

#### 3. Reset Sequence
1. Unplug USB
2. Wait 10 seconds
3. Plug back in
4. Press RESET button (black button)
5. Try `probe-rs run` immediately

#### 4. ST-Link Firmware
- ST-Link V3 firmware might need update
- Use STM32CubeProgrammer on Windows/STM32CubeIDE to check

#### 5. Alternative: Try STM32CubeProgrammer
If probe-rs continues failing:
```bash
# Install STM32CubeProgrammer (GUI tool)
# Connect via ST-LINK
# Try to read chip ID
# Flash a simple blink example from STM32Cube
```

#### 6. Dual-Core Issue
STM32WL55 has TWO cores:
- **Cortex-M4** (main application core)
- **Cortex-M0+** (radio stack core)

Make sure firmware targets **CM4** core (we are using `stm32wl55jc-cm4` feature).

### Commands to Try

```bash
# 1. List probes
probe-rs list

# 2. Get chip info
probe-rs info --chip STM32WL55JCIx

# 3. Flash with connect-under-reset
probe-rs run --chip STM32WL55JCIx --connect-under-reset <binary>

# 4. Flash with explicit protocol
probe-rs run --chip STM32WL55JCIx --protocol swd <binary>

# 5. Flash at slower speed
probe-rs run --chip STM32WL55JCIx --speed 1000 <binary>
```

### If Still Failing
- Try the other NUCLEO-WL55 board (NODE_1)
- Check if board is genuine ST hardware
- Verify CN7/CN8 connectors are properly seated
- Try on different computer as sanity check

---

## Issue: LoRaWAN Join Request Rejected with "unknow mote" Error

### Symptoms
- Device transmits join request (visible in gateway packet capture)
- Gateway logs show: `nsParseJoinReq: unknow mote : <wrong-eui>`
- Gateway parses DevEUI as gateway's own EUI (e.g., `ac1f09fffe1bce23` instead of `23ce1bfeff091fac`)
- Join never succeeds despite good RSSI/SNR

### Root Cause
**LoRaWAN byte order shenanigans!** The `lorawan-device` crate expects EUIs in **little-endian** (LSB first) format, but gateway dashboards display them in **big-endian** (MSB first).

### Solution
Reverse the byte order of DevEUI and AppEUI (but NOT AppKey):

```rust
// Gateway shows DevEUI: 23ce1bfeff091fac
// Gateway shows AppEUI: b130a864c5295356

// ❌ WRONG - Direct copy from gateway (will be read backwards!)
const DEV_EUI: [u8; 8] = [0x23, 0xCE, 0x1B, 0xFE, 0xFF, 0x09, 0x1F, 0xAC];
const APP_EUI: [u8; 8] = [0xB1, 0x30, 0xA8, 0x64, 0xC5, 0x29, 0x53, 0x56];

// ✅ CORRECT - Reverse the bytes (little-endian)
const DEV_EUI: [u8; 8] = [0xAC, 0x1F, 0x09, 0xFF, 0xFE, 0x1B, 0xCE, 0x23];
const APP_EUI: [u8; 8] = [0x56, 0x53, 0x29, 0xC5, 0x64, 0xA8, 0x30, 0xB1];

// AppKey stays in big-endian (MSB first)
const APP_KEY: [u8; 16] = [
    0xB7, 0x26, 0x73, 0x9B, 0x78, 0xEC, 0x4B, 0x9E,
    0x92, 0x34, 0xE5, 0xD3, 0x5E, 0xA9, 0x68, 0x1B,
];
```

### How to Verify
Check gateway logs after join attempt:
```bash
ssh root@lora-gw
logread | grep -i "join\|otaa" | tail -20
```

**Before fix:**
```
Join Reqeust from mote [ac1f09fffe1bce23] app [565329c564a830b1]
nsParseJoinReq: unknow mote : ac1f09fffe1bce23
```

**After fix:**
```
Join Reqeust from mote [23ce1bfeff091fac] app [b130a864c5295356]
[Join successful, no "unknow mote" error]
```

### Why This Happens
LoRaWAN specification (LoRaWAN 1.0.3 §6.2.4) defines that EUIs are transmitted **LSB first** over-the-air. The `lorawan-device` crate follows this spec literally, expecting you to provide arrays in transmission order (little-endian). However, humans (and gateway UIs) display EUIs in big-endian for readability.

**TL;DR:** Reverse your EUIs. Yes, all of them. No, not the AppKey. Yes, it's annoying. Welcome to embedded systems. 🎭

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

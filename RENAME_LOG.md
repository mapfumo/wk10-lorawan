# Firmware Directory Rename - 2026-01-08

## Changes Made

### Directory Structure
```
firmware/
├── node1-bme688/  →  lora-1/   ✅ Renamed
└── node2-sht41/   →  lora-2/   ✅ Renamed
```

### Package Names Updated

**lora-1/Cargo.toml**:
- `name = "node1-bme688"` → `name = "lora-1"`

**lora-2/Cargo.toml**:
- `name = "node2-sht41"` → `name = "lora-2"`

### Source Code Updates

**lora-1/src/main.rs**:
- Banner: `"Node 1 - Temperature & Humidity"` → `"STM32WL55 LoRa-1 - SHT41"`
- Display: `"STM32WL55 Node1"` → `"LoRa-1 (SHT41)"`

**lora-2/src/main.rs**:
- Banner: `"Node 2 - SHT41 LoRaWAN Sensor"` → `"STM32WL55 LoRa-2 - BME688"`

### Documentation Updated

All references updated in:
- ✅ README.md
- ✅ TODO.md
- ✅ USERGUIDE.md

### Rationale

1. **Generic naming**: `lora-1` and `lora-2` are hardware-agnostic
2. **Avoid confusion**: Original names didn't match actual sensor configuration
   - NODE_1 has SHT41 (not BME688)
   - NODE_2 will have BME688
3. **Cleaner naming**: Simpler, matches board serials and LoRaWAN node concept

### Verification

Tested lora-1 firmware build and flash:
```bash
cd firmware/lora-1
cargo run --release
```

Result: ✅ Working perfectly
- Sensor reading: 30°C, 57% RH
- OLED displaying: "LoRa-1 (SHT41)"
- No compilation errors

### Current Configuration

| Directory | Hardware | Sensor | Status |
|-----------|----------|--------|--------|
| lora-1 | 003E00463234510A33353533 | SHT41 | ✅ Working |
| lora-2 | 0026003A3234510A33353533 | BME688 (planned) | ⏳ Pending |

---

**Date**: 2026-01-08
**Status**: Complete ✅

#!/usr/bin/env python3
"""
Telegraf execd processor to decode LoRaWAN payloads from STM32WL55 nodes.

Input: InfluxDB line protocol with payload_base64 field
Output: InfluxDB line protocol with decoded sensor fields

Payload formats:
- LoRa-1 (23ce1bfeff091fac): 4 bytes - temp(i16), humidity(u16) - values * 100
- LoRa-2 (24ce1bfeff091fac): 12 bytes - temp(i16), humidity(u16), pressure(u16), gas(u16), padding
"""

import sys
import base64
import struct
import re

LORA1_DEVEUI = "23ce1bfeff091fac"
LORA2_DEVEUI = "24ce1bfeff091fac"

def decode_lora1(payload_bytes):
    """Decode LoRa-1 SHT41 payload: temp(i16), humidity(u16)"""
    if len(payload_bytes) < 4:
        return {}
    temp_raw, hum_raw = struct.unpack('>hH', payload_bytes[:4])
    return {
        'temperature': temp_raw / 100.0,
        'humidity': hum_raw / 100.0,
        'node': 'lora1_sht41'
    }

def decode_lora2(payload_bytes):
    """Decode LoRa-2 BME680 payload: temp(i16), humidity(u16), pressure(u16), gas(u16)"""
    if len(payload_bytes) < 8:
        return {}
    temp_raw, hum_raw, press_raw, gas_raw = struct.unpack('>hHHH', payload_bytes[:8])
    return {
        'temperature': temp_raw / 100.0,
        'humidity': hum_raw / 100.0,
        'pressure': press_raw / 10.0,
        'gas_resistance': float(gas_raw),
        'node': 'lora2_bme680'
    }

def process_line(line):
    """Process a single line of InfluxDB line protocol."""
    line = line.strip()
    if not line:
        return None

    # Parse line protocol: measurement,tags fields timestamp
    # Example: lorawan_uplink,dev_eui=23ce1bfeff091fac payload_base64="...",rssi=-19i 1234567890

    # Find payload_base64 field
    match = re.search(r'payload_base64="([^"]*)"', line)
    if not match:
        return line  # Pass through unchanged

    payload_b64 = match.group(1)

    # Find dev_eui tag
    eui_match = re.search(r'dev_eui=([a-f0-9]+)', line)
    if not eui_match:
        return line

    dev_eui = eui_match.group(1).lower()

    # Decode payload
    try:
        payload_bytes = base64.b64decode(payload_b64)
    except Exception:
        return line

    # Decode based on device
    if dev_eui == LORA1_DEVEUI:
        decoded = decode_lora1(payload_bytes)
    elif dev_eui == LORA2_DEVEUI:
        decoded = decode_lora2(payload_bytes)
    else:
        return line

    if not decoded:
        return line

    # Add decoded fields to line protocol
    # Find the fields section and add new fields
    node_name = decoded.pop('node', 'unknown')

    # Build new fields string
    new_fields = []
    for key, value in decoded.items():
        if isinstance(value, float):
            new_fields.append(f'{key}={value}')
        elif isinstance(value, int):
            new_fields.append(f'{key}={value}i')
        else:
            new_fields.append(f'{key}="{value}"')

    if new_fields:
        # Insert new fields before the timestamp
        # Line format: measurement,tags field1=val1,field2=val2 timestamp
        parts = line.rsplit(' ', 1)  # Split off timestamp
        if len(parts) == 2:
            fields_part, timestamp = parts
            new_line = f'{fields_part},node="{node_name}",{",".join(new_fields)} {timestamp}'
        else:
            new_line = f'{line},node="{node_name}",{",".join(new_fields)}'
        return new_line

    return line

def main():
    """Main loop - read stdin, process, write stdout."""
    for line in sys.stdin:
        result = process_line(line)
        if result:
            print(result, flush=True)

if __name__ == '__main__':
    main()

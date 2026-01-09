#!/usr/bin/env python3
"""
LoRa Payload Decoder for STM32WL55 Nodes

LoRa-1 (SHT41) - 4 bytes:
  Bytes 0-1: Temperature (signed 16-bit big-endian, °C × 100)
  Bytes 2-3: Humidity (unsigned 16-bit big-endian, % × 100)

LoRa-2 (BME688) - 12 bytes:
  Bytes 0-1: Temperature (signed 16-bit big-endian, °C × 100)
  Bytes 2-3: Humidity (unsigned 16-bit big-endian, % × 100)
  Bytes 4-5: Pressure (unsigned 16-bit big-endian, hPa × 10)
  Bytes 6-7: Gas Resistance (unsigned 16-bit big-endian, kOhm)
  Bytes 8-11: Reserved

Usage:
  ./decode_payload.py <base64_payload>
  ./decode_payload.py CigZyA==         # 4-byte LoRa-1 payload
  ./decode_payload.py CowZyAD8AAAAAAA= # 12-byte LoRa-2 payload
"""

import sys
import base64
import struct

def decode_lora1_payload(base64_data):
    """Decode LoRa-1 4-byte SHT41 payload"""
    try:
        # Decode base64
        payload = base64.b64decode(base64_data)

        if len(payload) != 4:
            print(f"Error: Expected 4 bytes for LoRa-1, got {len(payload)}")
            return None

        # Unpack: >h = signed 16-bit big-endian, >H = unsigned 16-bit big-endian
        temp_raw, hum_raw = struct.unpack('>hH', payload)

        # Convert to actual values (divide by 1000 based on observed data)
        temp_celsius = temp_raw / 1000.0
        humidity_percent = hum_raw / 1000.0

        return {
            'node': 'LoRa-1',
            'sensor': 'SHT41',
            'temperature_c': temp_celsius,
            'humidity_percent': humidity_percent,
            'payload_hex': payload.hex(),
            'payload_bytes': [f'0x{b:02X}' for b in payload]
        }
    except Exception as e:
        print(f"Error decoding: {e}")
        return None

def decode_lora2_payload(base64_data):
    """Decode LoRa-2 12-byte BME688 payload"""
    try:
        # Decode base64
        payload = base64.b64decode(base64_data)

        if len(payload) != 12:
            print(f"Error: Expected 12 bytes for LoRa-2, got {len(payload)}")
            return None

        # Unpack: >h = signed 16-bit BE, >H = unsigned 16-bit BE
        temp_raw, hum_raw, press_raw, gas_raw = struct.unpack('>hHHH', payload[:8])

        # Convert to actual values (divide by 1000 based on observed data)
        temp_celsius = temp_raw / 1000.0
        humidity_percent = hum_raw / 1000.0
        pressure_hpa = press_raw / 100.0
        gas_resistance_kohm = gas_raw

        return {
            'node': 'LoRa-2',
            'sensor': 'BME688',
            'temperature_c': temp_celsius,
            'humidity_percent': humidity_percent,
            'pressure_hpa': pressure_hpa,
            'gas_resistance_kohm': gas_resistance_kohm,
            'payload_hex': payload.hex(),
            'payload_bytes': [f'0x{b:02X}' for b in payload]
        }
    except Exception as e:
        print(f"Error decoding: {e}")
        return None

def decode_payload(base64_data):
    """Auto-detect and decode payload based on length"""
    try:
        payload = base64.b64decode(base64_data)

        if len(payload) == 4:
            return decode_lora1_payload(base64_data)
        elif len(payload) == 12:
            return decode_lora2_payload(base64_data)
        else:
            print(f"Error: Unsupported payload length {len(payload)} (expected 4 or 12)")
            return None
    except Exception as e:
        print(f"Error: {e}")
        return None

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(1)

    result = decode_payload(sys.argv[1])

    if result:
        print(f"Node:        {result['node']}")
        print(f"Sensor:      {result['sensor']}")
        print(f"Temperature: {result['temperature_c']:.2f}°C")
        print(f"Humidity:    {result['humidity_percent']:.2f}%")

        # LoRa-2 specific fields
        if 'pressure_hpa' in result:
            print(f"Pressure:    {result['pressure_hpa']:.1f} hPa")
        if 'gas_resistance_kohm' in result:
            print(f"Gas:         {result['gas_resistance_kohm']} kOhm")

        print(f"Hex payload: {result['payload_hex']}")
        print(f"Bytes:       {' '.join(result['payload_bytes'])}")

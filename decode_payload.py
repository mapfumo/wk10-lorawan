#!/usr/bin/env python3
"""
LoRa-1 (SHT41) Payload Decoder
Decodes 4-byte temperature and humidity payload from STM32WL55 LoRa-1 node.

Payload format (4 bytes):
  Bytes 0-1: Temperature (signed 16-bit big-endian, °C × 100)
  Bytes 2-3: Humidity (unsigned 16-bit big-endian, % × 100)

Usage:
  ./decode_payload.py <base64_payload>
  ./decode_payload.py CigZyA==
"""

import sys
import base64
import struct

def decode_payload(base64_data):
    """Decode LoRa-1 4-byte payload"""
    try:
        # Decode base64
        payload = base64.b64decode(base64_data)

        if len(payload) != 4:
            print(f"Error: Expected 4 bytes, got {len(payload)}")
            return None

        # Unpack: >h = signed 16-bit big-endian, >H = unsigned 16-bit big-endian
        temp_raw, hum_raw = struct.unpack('>hH', payload)

        # Convert to actual values
        temp_celsius = temp_raw / 100.0
        humidity_percent = hum_raw / 100.0

        return {
            'temperature_c': temp_celsius,
            'humidity_percent': humidity_percent,
            'payload_hex': payload.hex(),
            'payload_bytes': [f'0x{b:02X}' for b in payload]
        }
    except Exception as e:
        print(f"Error decoding: {e}")
        return None

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(1)

    result = decode_payload(sys.argv[1])

    if result:
        print(f"Temperature: {result['temperature_c']:.2f}°C")
        print(f"Humidity:    {result['humidity_percent']:.2f}%")
        print(f"Hex payload: {result['payload_hex']}")
        print(f"Bytes:       {' '.join(result['payload_bytes'])}")

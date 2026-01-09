#!/usr/bin/env python3
"""
Simple MQTT subscriber for RAK7268V2 gateway
Captures and decodes LoRaWAN uplinks from LoRa-1 and LoRa-2 nodes
"""

import socket
import struct
import base64
import json
import sys

def mqtt_connect(host, port=1883, client_id="python_sub"):
    """Connect to MQTT broker and subscribe to all application topics"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(10)
    sock.connect((host, port))

    # MQTT CONNECT packet
    client_id_bytes = client_id.encode('utf-8')
    remaining_length = 2 + 4 + 1 + 1 + 2 + len(client_id_bytes)

    packet = bytearray()
    packet.append(0x10)  # CONNECT
    packet.append(remaining_length)
    packet.extend(b'\x00\x04MQTT')  # Protocol name
    packet.append(0x04)  # Protocol level (3.1.1)
    packet.append(0x02)  # Connect flags (clean session)
    packet.extend(b'\x00\x3C')  # Keep alive (60s)
    packet.append(0x00)
    packet.append(len(client_id_bytes))
    packet.extend(client_id_bytes)

    sock.send(packet)
    response = sock.recv(4)
    if response[0] != 0x20:
        raise Exception(f"CONNECT failed: {response.hex()}")
    print(f"Connected to MQTT broker at {host}:{port}")

    # SUBSCRIBE to application/# (wildcard for all apps)
    topic = "application/#"
    topic_bytes = topic.encode('utf-8')

    packet = bytearray()
    packet.append(0x82)  # SUBSCRIBE with QoS 1
    remaining = 2 + 2 + len(topic_bytes) + 1
    packet.append(remaining)
    packet.extend(b'\x00\x01')  # Packet ID = 1
    # Topic length (big-endian)
    packet.append((len(topic_bytes) >> 8) & 0xFF)
    packet.append(len(topic_bytes) & 0xFF)
    packet.extend(topic_bytes)
    packet.append(0x00)  # QoS 0

    sock.send(packet)
    response = sock.recv(5)
    print(f"SUBACK response: {response.hex()}")
    if response[0] != 0x90:
        raise Exception(f"SUBSCRIBE failed: {response.hex()}")
    print(f"Subscribed to: {topic}")

    return sock

def decode_lora1(payload_bytes):
    """Decode 4-byte LoRa-1 (SHT41) payload"""
    if len(payload_bytes) != 4:
        return None
    temp_raw, hum_raw = struct.unpack('>hH', payload_bytes)
    return {
        'temperature': temp_raw / 100.0,
        'humidity': hum_raw / 100.0
    }

def decode_lora2(payload_bytes):
    """Decode 12-byte LoRa-2 (BME680) payload"""
    if len(payload_bytes) != 12:
        return None
    temp_raw, hum_raw, press_raw, gas_raw = struct.unpack('>hHHH', payload_bytes[:8])
    return {
        'temperature': temp_raw / 100.0,
        'humidity': hum_raw / 100.0,
        'pressure': press_raw / 10.0,
        'gas_kohm': gas_raw
    }

def read_mqtt_message(sock):
    """Read one MQTT PUBLISH message"""
    sock.settimeout(60)

    header = sock.recv(1)
    if not header:
        return None, None

    if (header[0] >> 4) != 3:  # Not a PUBLISH
        # Read and discard
        remaining = sock.recv(1)[0]
        sock.recv(remaining)
        return None, None

    # Read remaining length (variable length encoding)
    multiplier = 1
    remaining = 0
    while True:
        byte = sock.recv(1)[0]
        remaining += (byte & 0x7F) * multiplier
        if (byte & 0x80) == 0:
            break
        multiplier *= 128

    # Read the rest
    data = b''
    while len(data) < remaining:
        data += sock.recv(remaining - len(data))

    # Parse topic
    topic_len = struct.unpack('>H', data[:2])[0]
    topic = data[2:2+topic_len].decode('utf-8')
    payload = data[2+topic_len:]

    return topic, payload

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "10.10.10.254"

    print(f"Connecting to MQTT broker at {host}...")
    print("Waiting for LoRaWAN uplinks (Ctrl+C to stop)\n")
    print("=" * 60)

    try:
        sock = mqtt_connect(host)

        while True:
            topic, payload = read_mqtt_message(sock)

            if topic and payload:
                # Only process /rx topics (uplinks)
                if '/rx' in topic:
                    try:
                        msg = json.loads(payload.decode('utf-8'))

                        dev_eui = msg.get('devEUI', 'unknown')
                        data_b64 = msg.get('data', '')
                        rssi = msg.get('rxInfo', [{}])[0].get('rssi', 'N/A')
                        snr = msg.get('rxInfo', [{}])[0].get('loRaSNR', 'N/A')

                        if data_b64:
                            payload_bytes = base64.b64decode(data_b64)

                            print(f"\nTopic: {topic}")
                            print(f"DevEUI: {dev_eui}")
                            print(f"RSSI: {rssi} dBm, SNR: {snr} dB")
                            print(f"Payload (hex): {payload_bytes.hex()}")
                            print(f"Payload (b64): {data_b64}")

                            # Auto-detect and decode
                            if len(payload_bytes) == 4:
                                decoded = decode_lora1(payload_bytes)
                                if decoded:
                                    print(f"  [LoRa-1 SHT41] Temp: {decoded['temperature']:.2f}°C, Humidity: {decoded['humidity']:.2f}%")
                            elif len(payload_bytes) == 12:
                                decoded = decode_lora2(payload_bytes)
                                if decoded:
                                    print(f"  [LoRa-2 BME680] Temp: {decoded['temperature']:.2f}°C, Humidity: {decoded['humidity']:.2f}%, Pressure: {decoded['pressure']:.1f} hPa, Gas: {decoded['gas_kohm']} kOhm")
                            else:
                                print(f"  [Unknown] {len(payload_bytes)} bytes")

                            print("-" * 60)

                    except json.JSONDecodeError:
                        print(f"Non-JSON payload on {topic}")
                    except Exception as e:
                        print(f"Error processing message: {e}")

    except KeyboardInterrupt:
        print("\nStopping...")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        try:
            sock.close()
        except:
            pass

if __name__ == '__main__':
    main()

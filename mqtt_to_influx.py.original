#!/usr/bin/env python3
"""
MQTT to InfluxDB bridge for LoRaWAN sensor data.
Subscribes to RAK7268V2 gateway MQTT, decodes payloads, writes to InfluxDB.

Usage: python3 mqtt_to_influx.py
"""

import json
import base64
import struct
import socket
import time
from datetime import datetime

# Configuration
GATEWAY_MQTT_HOST = "10.10.10.254"
GATEWAY_MQTT_PORT = 1883
INFLUXDB_HOST = "wk7-influxdb"
INFLUXDB_PORT = 8086
INFLUXDB_TOKEN = "my-super-secret-auth-token"
INFLUXDB_ORG = "my-org"
INFLUXDB_BUCKET = "lorawan"

# Device EUIs
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
    }


def write_to_influx(measurement, tags, fields, timestamp=None):
    """Write a point to InfluxDB using line protocol over HTTP."""
    import urllib.request
    import urllib.error

    # Build line protocol
    tag_str = ",".join(f"{k}={v}" for k, v in tags.items())
    field_str = ",".join(
        f'{k}={v}' if isinstance(v, (int, float)) else f'{k}="{v}"'
        for k, v in fields.items()
    )

    line = f"{measurement},{tag_str} {field_str}"
    if timestamp:
        line += f" {timestamp}"

    url = f"http://{INFLUXDB_HOST}:{INFLUXDB_PORT}/api/v2/write?org={INFLUXDB_ORG}&bucket={INFLUXDB_BUCKET}&precision=ns"

    req = urllib.request.Request(url, data=line.encode(), method='POST')
    req.add_header('Authorization', f'Token {INFLUXDB_TOKEN}')
    req.add_header('Content-Type', 'text/plain')

    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            return resp.status == 204
    except urllib.error.URLError as e:
        print(f"InfluxDB write error: {e}")
        return False


def mqtt_connect(host, port):
    """Connect to MQTT broker using raw sockets (MQTT 3.1 protocol)."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(30)
    sock.connect((host, port))

    # MQTT CONNECT packet (protocol version 3.1)
    client_id = b"python-lorawan-bridge"
    protocol_name = b"MQIsdp"  # MQTT 3.1

    # Variable header
    var_header = (
        struct.pack(">H", len(protocol_name)) + protocol_name +  # Protocol name
        bytes([3]) +  # Protocol version 3.1
        bytes([0x02]) +  # Connect flags (clean session)
        struct.pack(">H", 60)  # Keep alive (60 seconds)
    )

    # Payload
    payload = struct.pack(">H", len(client_id)) + client_id

    # Fixed header
    remaining_len = len(var_header) + len(payload)
    fixed_header = bytes([0x10]) + bytes([remaining_len])

    sock.send(fixed_header + var_header + payload)

    # Read CONNACK
    response = sock.recv(4)
    if len(response) >= 4 and response[0] == 0x20 and response[3] == 0x00:
        print(f"Connected to MQTT broker at {host}:{port}")
        return sock
    else:
        raise Exception(f"MQTT connection failed: {response.hex()}")


def mqtt_subscribe(sock, topic):
    """Subscribe to an MQTT topic."""
    topic_bytes = topic.encode()
    packet_id = 1

    # Variable header + payload
    var_header = struct.pack(">H", packet_id)
    payload = struct.pack(">H", len(topic_bytes)) + topic_bytes + bytes([0])  # QoS 0

    remaining_len = len(var_header) + len(payload)
    fixed_header = bytes([0x82]) + bytes([remaining_len])

    sock.send(fixed_header + var_header + payload)

    # Read SUBACK
    response = sock.recv(5)
    if len(response) >= 5 and response[0] == 0x90:
        print(f"Subscribed to: {topic}")
        return True
    return False


def mqtt_read_message(sock):
    """Read a single MQTT message."""
    try:
        # Read fixed header
        first_byte = sock.recv(1)
        if not first_byte:
            return None, None

        packet_type = first_byte[0] >> 4

        # Read remaining length (variable length encoding)
        multiplier = 1
        remaining_length = 0
        while True:
            byte = sock.recv(1)
            if not byte:
                return None, None
            remaining_length += (byte[0] & 0x7F) * multiplier
            multiplier *= 128
            if not (byte[0] & 0x80):
                break

        # Read the rest of the packet
        if remaining_length > 0:
            payload = b''
            while len(payload) < remaining_length:
                chunk = sock.recv(remaining_length - len(payload))
                if not chunk:
                    return None, None
                payload += chunk
        else:
            payload = b''

        if packet_type == 3:  # PUBLISH
            # Parse topic
            topic_len = struct.unpack(">H", payload[:2])[0]
            topic = payload[2:2+topic_len].decode()
            message = payload[2+topic_len:]
            return topic, message

        return None, None

    except socket.timeout:
        return None, None


def process_message(topic, message):
    """Process an MQTT message and write to InfluxDB."""
    try:
        data = json.loads(message)
    except json.JSONDecodeError:
        return

    dev_eui = data.get("devEUI", "").lower()
    payload_b64 = data.get("data", "")

    if not dev_eui or not payload_b64:
        return

    # Decode payload
    try:
        payload_bytes = base64.b64decode(payload_b64)
    except Exception:
        return

    # Get radio info
    rx_info = data.get("rxInfo", [{}])[0]
    rssi = rx_info.get("rssi", 0)
    snr = rx_info.get("loRaSNR", 0)
    frame_count = data.get("fCnt", 0)

    # Decode based on device
    if dev_eui == LORA1_DEVEUI:
        fields = decode_lora1(payload_bytes)
        sensor = "SHT41"
        node = "lora1"
    elif dev_eui == LORA2_DEVEUI:
        fields = decode_lora2(payload_bytes)
        sensor = "BME680"
        node = "lora2"
    else:
        print(f"Unknown device: {dev_eui}")
        return

    if not fields:
        return

    # Add radio metadata
    fields["rssi"] = rssi
    fields["snr"] = snr
    fields["frame_count"] = frame_count

    tags = {
        "dev_eui": dev_eui,
        "node": node,
        "sensor": sensor,
    }

    # Write to InfluxDB
    timestamp = int(time.time() * 1e9)
    success = write_to_influx("lorawan_sensor", tags, fields, timestamp)

    # Print summary
    ts = datetime.now().strftime("%H:%M:%S")
    if dev_eui == LORA1_DEVEUI:
        print(f"[{ts}] {node}: Temp={fields['temperature']:.1f}C Hum={fields['humidity']:.1f}% RSSI={rssi} SNR={snr} -> InfluxDB: {'OK' if success else 'FAIL'}")
    else:
        print(f"[{ts}] {node}: Temp={fields['temperature']:.1f}C Hum={fields['humidity']:.1f}% Press={fields['pressure']:.1f}hPa Gas={fields['gas_resistance']:.0f}kOhm RSSI={rssi} SNR={snr} -> InfluxDB: {'OK' if success else 'FAIL'}")


def main():
    print("=" * 60)
    print("LoRaWAN MQTT → InfluxDB Bridge")
    print(f"Gateway: {GATEWAY_MQTT_HOST}:{GATEWAY_MQTT_PORT}")
    print(f"InfluxDB: {INFLUXDB_HOST}:{INFLUXDB_PORT}/{INFLUXDB_BUCKET}")
    print("=" * 60)

    while True:
        try:
            sock = mqtt_connect(GATEWAY_MQTT_HOST, GATEWAY_MQTT_PORT)
            mqtt_subscribe(sock, "application/#")

            print("Waiting for LoRaWAN uplinks...")

            while True:
                topic, message = mqtt_read_message(sock)
                if topic and message:
                    if "/rx" in topic:
                        process_message(topic, message)

        except KeyboardInterrupt:
            print("\nShutting down...")
            break
        except Exception as e:
            print(f"Error: {e}, reconnecting in 5s...")
            time.sleep(5)


if __name__ == "__main__":
    main()

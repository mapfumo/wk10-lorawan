#!/usr/bin/env python3
"""
LoRaWAN MQTT to InfluxDB Bridge

Subscribes to ChirpStack MQTT topics and writes decoded sensor data to InfluxDB.

ChirpStack MQTT Topics:
- application/+/device/+/event/up       (uplink data)
- application/+/device/+/event/join     (join events)
- application/+/device/+/event/status   (status updates)

Author: Tony Mapfumo
Week: 10 - LoRaWAN Migration
"""

import json
import struct
import paho.mqtt.client as mqtt
from influxdb_client import InfluxDBClient, Point
from influxdb_client.client.write_api import SYNCHRONOUS
from datetime import datetime

# Configuration
MQTT_BROKER = "localhost"
MQTT_PORT = 1883
MQTT_TOPIC_UPLINK = "application/+/device/+/event/up"
MQTT_TOPIC_JOIN = "application/+/device/+/event/join"
MQTT_TOPIC_STATUS = "application/+/device/+/event/status"

INFLUXDB_URL = "http://localhost:8086"
INFLUXDB_TOKEN = "your-token-here"  # TODO: Update with actual token
INFLUXDB_ORG = "iiot"
INFLUXDB_BUCKET = "iiot"

# InfluxDB client
influx_client = InfluxDBClient(url=INFLUXDB_URL, token=INFLUXDB_TOKEN, org=INFLUXDB_ORG)
write_api = influx_client.write_api(write_options=SYNCHRONOUS)


def decode_bme688_payload(data: bytes) -> dict:
    """
    Decode BME688 payload (~12 bytes).

    Format:
    Byte 0-3:   Temperature (f32, IEEE 754)
    Byte 4-7:   Humidity (f32, IEEE 754)
    Byte 8-9:   Pressure (u16, hPa * 10)
    Byte 10-11: Gas Resistance (u16, kΩ)
    """
    if len(data) < 12:
        return None

    temperature = struct.unpack('>f', data[0:4])[0]
    humidity = struct.unpack('>f', data[4:8])[0]
    pressure = struct.unpack('>H', data[8:10])[0] / 10.0
    gas_resistance = struct.unpack('>H', data[10:12])[0]

    return {
        "temperature": temperature,
        "humidity": humidity,
        "pressure": pressure,
        "gas_resistance": gas_resistance
    }


def decode_sht41_payload(data: bytes) -> dict:
    """
    Decode SHT41 payload (~4 bytes).

    Format:
    Byte 0-1: Temperature (i16, °C * 100)
    Byte 2-3: Humidity (u16, % * 100)
    """
    if len(data) < 4:
        return None

    temperature = struct.unpack('>h', data[0:2])[0] / 100.0
    humidity = struct.unpack('>H', data[2:4])[0] / 100.0

    return {
        "temperature": temperature,
        "humidity": humidity
    }


def on_connect(client, userdata, flags, rc):
    """MQTT connection callback."""
    if rc == 0:
        print(f"[INFO] Connected to MQTT broker: {MQTT_BROKER}:{MQTT_PORT}")
        client.subscribe(MQTT_TOPIC_UPLINK)
        client.subscribe(MQTT_TOPIC_JOIN)
        client.subscribe(MQTT_TOPIC_STATUS)
        print(f"[INFO] Subscribed to ChirpStack topics")
    else:
        print(f"[ERROR] Failed to connect to MQTT broker: rc={rc}")


def on_message(client, userdata, msg):
    """MQTT message callback."""
    try:
        payload = json.loads(msg.payload.decode())
        topic = msg.topic

        # Handle join events
        if "/event/join" in topic:
            dev_eui = payload.get("devEUI", "unknown")
            dev_addr = payload.get("devAddr", "unknown")
            print(f"[INFO] Join event: DevEUI={dev_eui}, DevAddr={dev_addr}")
            return

        # Handle status events
        if "/event/status" in topic:
            dev_eui = payload.get("devEUI", "unknown")
            margin = payload.get("margin", 0)
            print(f"[INFO] Status event: DevEUI={dev_eui}, Margin={margin}")
            return

        # Handle uplink events
        if "/event/up" in topic:
            dev_eui = payload.get("devEUI", "unknown")
            device_name = payload.get("deviceName", "unknown")

            # Extract LoRaWAN metadata
            rx_info = payload.get("rxInfo", [{}])[0]
            rssi = rx_info.get("rssi", 0)
            snr = rx_info.get("loRaSNR", 0)

            tx_info = payload.get("txInfo", {})
            spreading_factor = tx_info.get("loRaModulationInfo", {}).get("spreadingFactor", 0)

            # Decode base64 payload
            import base64
            data_b64 = payload.get("data", "")
            data_bytes = base64.b64decode(data_b64)

            print(f"[INFO] Uplink from {device_name} ({dev_eui}): RSSI={rssi} SNR={snr} SF={spreading_factor}")
            print(f"[DEBUG] Raw payload: {data_bytes.hex()}")

            # Decode based on device name
            sensor_data = None
            if "BME688" in device_name.upper():
                sensor_data = decode_bme688_payload(data_bytes)
                sensor_type = "bme688"
            elif "SHT41" in device_name.upper():
                sensor_data = decode_sht41_payload(data_bytes)
                sensor_type = "sht41"
            else:
                print(f"[WARN] Unknown device type: {device_name}")
                return

            if sensor_data is None:
                print(f"[ERROR] Failed to decode payload for {device_name}")
                return

            print(f"[INFO] Decoded: {sensor_data}")

            # Write to InfluxDB
            point = Point("lorawan_sensors") \
                .tag("device_name", device_name) \
                .tag("dev_eui", dev_eui) \
                .tag("sensor_type", sensor_type) \
                .field("temperature", sensor_data["temperature"]) \
                .field("humidity", sensor_data["humidity"]) \
                .field("rssi", rssi) \
                .field("snr", snr) \
                .field("spreading_factor", spreading_factor)

            # Add extra fields for BME688
            if sensor_type == "bme688":
                point.field("pressure", sensor_data["pressure"])
                point.field("gas_resistance", sensor_data["gas_resistance"])

            write_api.write(bucket=INFLUXDB_BUCKET, record=point)
            print(f"[INFO] Written to InfluxDB: lorawan_sensors")

    except Exception as e:
        print(f"[ERROR] Failed to process message: {e}")
        print(f"[DEBUG] Topic: {msg.topic}")
        print(f"[DEBUG] Payload: {msg.payload}")


def main():
    """Main entry point."""
    print("=" * 60)
    print("LoRaWAN MQTT to InfluxDB Bridge")
    print("Week 10 - LoRaWAN Migration")
    print("=" * 60)

    # Connect to MQTT broker
    mqtt_client = mqtt.Client()
    mqtt_client.on_connect = on_connect
    mqtt_client.on_message = on_message

    print(f"[INFO] Connecting to MQTT broker: {MQTT_BROKER}:{MQTT_PORT}")
    mqtt_client.connect(MQTT_BROKER, MQTT_PORT, 60)

    print(f"[INFO] Connecting to InfluxDB: {INFLUXDB_URL}")
    print(f"[INFO] Waiting for LoRaWAN messages...")

    # Start MQTT loop
    mqtt_client.loop_forever()


if __name__ == "__main__":
    main()

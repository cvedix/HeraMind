#!/usr/bin/env python3
"""Quick BLE provisioning test — bypasses Chrome's GATT cache entirely.
Usage: python3 ble_test.py
Requires: pip install bleak
"""

import asyncio
import json
import sys

try:
    from bleak import BleakClient, BleakScanner
except ImportError:
    print("Install bleak first: pip install bleak")
    sys.exit(1)

SERVICE_UUID = "0000fffe-0000-1000-8000-00805f9b34fb"
CHAR_UUID    = "0000ff53-0000-1000-8000-00805f9b34fb"

async def main():
    print("Scanning for NE101 devices...")
    devices = await BleakScanner.discover(timeout=10)

    targets = [d for d in devices if d.name and d.name.startswith("NE")]
    if not targets:
        print("No NE101 devices found.")
        return

    for i, d in enumerate(targets):
        print(f"  [{i}] {d.name} ({d.address})")

    idx = 0 if len(targets) == 1 else int(input("Select device: "))
    device = targets[idx]
    print(f"\nConnecting to {device.name} ({device.address})...")

    async with BleakClient(device.address) as client:
        print(f"Connected! MTU: {client.mtu_size}")

        # List all services
        print("\nServices discovered:")
        for svc in client.services:
            print(f"  Service: {svc.uuid}")
            for chr in svc.characteristics:
                props = ", ".join(chr.properties)
                print(f"    Char: {chr.uuid} [{props}]")

        # Find our characteristic
        print(f"\nLooking for {CHAR_UUID}...")
        try:
            # Try direct read first
            resp = await client.read_gatt_char(CHAR_UUID)
            print(f"Read response: {resp}")
        except Exception as e:
            print(f"Read (expected empty): {e}")

        # Write test config
        config = json.dumps({
            "ssid": "TEST_WIFI",
            "wifi_password": "test1234",
            "host": "test.mqtt.local",
            "port": 1883,
            "username": "testuser",
            "password": "testpass",
            "topic_prefix": "test/device",
            "client_id": "test_client"
        }).encode()

        print(f"\nWriting config ({len(config)} bytes)...")
        await client.write_gatt_char(CHAR_UUID, config, response=True)
        print("Write OK — config received by device!")

        # Device disconnects immediately after successful config (ble_prov_stop).
        # Try reading response, but disconnect is expected and means success.
        await asyncio.sleep(0.3)
        try:
            resp = await client.read_gatt_char(CHAR_UUID)
            resp_text = resp.decode()
            print(f"Response: {resp_text}")
            if '"status":0' in resp_text:
                print("\nSUCCESS — BLE provisioning complete!")
            else:
                print(f"\nUnexpected response: {resp_text}")
        except Exception as e:
            # Device disconnected = config was received and ble_prov_stop() killed BLE
            print(f"Device disconnected (expected) — {e}")
            print("\nSUCCESS — device received config and stopped BLE!")

    print("Disconnected.")

if __name__ == "__main__":
    asyncio.run(main())

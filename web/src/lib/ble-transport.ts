// BLE Transport Abstraction
//
// Provides a unified interface for BLE operations in both
// Tauri (native via tauri-plugin-blec) and browser (Web Bluetooth) environments.

import { BLE_PROV_SERVICE_UUID, BLE_CHAR_MQTT } from './ble-protocol'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface BleTransportDevice {
  id: string        // MAC address (Tauri) or device ID (WebBluetooth)
  name: string
  rssi?: number
}

export interface BleTransport {
  scan(): Promise<BleTransportDevice[]>
  connect(deviceId: string, onDisconnect?: () => void): Promise<void>
  read(charUuid: string): Promise<Uint8Array>
  write(charUuid: string, data: Uint8Array): Promise<void>
  writeAndRead(charUuid: string, data: Uint8Array): Promise<Uint8Array>
  disconnect(): void
  getAdapterState?(): Promise<'Unknown' | 'On' | 'Off'>
}

// ---------------------------------------------------------------------------
// Environment detection
// ---------------------------------------------------------------------------

function isTauriEnv(): boolean {
  return !!(window as any).__TAURI_INTERNALS__
}

// ---------------------------------------------------------------------------
// Tauri BLE Transport (via tauri-plugin-blec)
// ---------------------------------------------------------------------------

class TauriBlecTransport implements BleTransport {
  async scan(): Promise<BleTransportDevice[]> {
    const { startScan, stopScan } = await import('@mnlphlp/plugin-blec')

    const devices: BleTransportDevice[] = []
    const seen = new Set<string>()
    const SCAN_DURATION = 10_000

    // startScan resolves immediately — scan runs in background for SCAN_DURATION ms
    await startScan((found) => {
      for (const d of found) {
        if (d.name && d.name.startsWith('NE') && !seen.has(d.address)) {
          seen.add(d.address)
          devices.push({
            id: d.address,
            name: d.name,
            rssi: d.rssi,
          })
        }
      }
    }, SCAN_DURATION)

    // Wait for devices to be discovered
    await new Promise(r => setTimeout(r, SCAN_DURATION))

    await stopScan()

    // Sort by RSSI (strongest signal first)
    devices.sort((a, b) => (b.rssi ?? -100) - (a.rssi ?? -100))
    return devices
  }

  async connect(deviceId: string, onDisconnect?: () => void): Promise<void> {
    const { connect: blecConnect } = await import('@mnlphlp/plugin-blec')
    await blecConnect(deviceId, onDisconnect ?? null)
  }

  async write(charUuid: string, data: Uint8Array): Promise<void> {
    const { send } = await import('@mnlphlp/plugin-blec')
    const dataArray = Array.from(data)
    await send(charUuid, dataArray, 'withResponse', BLE_PROV_SERVICE_UUID)
  }

  async writeAndRead(charUuid: string, data: Uint8Array): Promise<Uint8Array> {
    const { send, read } = await import('@mnlphlp/plugin-blec')

    const dataArray = Array.from(data)
    await send(charUuid, dataArray, 'withResponse', BLE_PROV_SERVICE_UUID)

    // Small delay for device to process the write
    await new Promise(r => setTimeout(r, 300))

    const response = await read(charUuid, BLE_PROV_SERVICE_UUID)
    return new Uint8Array(response)
  }

  async read(charUuid: string): Promise<Uint8Array> {
    const { read } = await import('@mnlphlp/plugin-blec')
    const response = await read(charUuid, BLE_PROV_SERVICE_UUID)
    return new Uint8Array(response)
  }

  disconnect(): void {
    import('@mnlphlp/plugin-blec').then(({ disconnect }) => disconnect()).catch(() => {})
  }

  async getAdapterState(): Promise<'Unknown' | 'On' | 'Off'> {
    const { getAdapterState } = await import('@mnlphlp/plugin-blec')
    return getAdapterState()
  }
}

// ---------------------------------------------------------------------------
// Web Bluetooth Transport
// ---------------------------------------------------------------------------

class WebBluetoothTransport implements BleTransport {
  private bleDevice: BluetoothDevice | null = null
  private characteristic: BluetoothRemoteGATTCharacteristic | null = null

  async scan(): Promise<BleTransportDevice[]> {
    if (!navigator.bluetooth) {
      throw new Error('Web Bluetooth is not supported')
    }

    const selected = await navigator.bluetooth.requestDevice({
      filters: [{ namePrefix: 'NE' }],
      optionalServices: [BLE_PROV_SERVICE_UUID],
    })

    this.bleDevice = selected
    const deviceName = selected.name || 'Unknown'
    return [{
      id: selected.id,
      name: deviceName,
    }]
  }

  async connect(_deviceId: string, onDisconnect?: () => void): Promise<void> {
    if (!this.bleDevice?.gatt) {
      throw new Error('No device selected')
    }

    // Set up disconnect listener
    if (onDisconnect) {
      this.bleDevice.addEventListener('gattserverdisconnected', onDisconnect)
    }

    const gatt = this.bleDevice.gatt
    if (gatt.connected) {
      try { gatt.disconnect() } catch { /* ignore */ }
      await new Promise(r => setTimeout(r, 800))
    }

    const server = await gatt.connect()

    // Direct UUID lookup to bypass Chrome's GATT cache
    const service = await (server as any).getPrimaryService(BLE_PROV_SERVICE_UUID)
    this.characteristic = await (service as any).getCharacteristic(BLE_CHAR_MQTT)

    if (!this.characteristic) {
      throw new Error('Config characteristic not found on device')
    }
  }

  async write(_charUuid: string, data: Uint8Array): Promise<void> {
    if (!this.characteristic) {
      throw new Error('Not connected')
    }
    await this.characteristic.writeValueWithResponse(data)
  }

  async writeAndRead(_charUuid: string, data: Uint8Array): Promise<Uint8Array> {
    if (!this.characteristic) {
      throw new Error('Not connected')
    }

    await this.characteristic.writeValueWithResponse(data)
    await new Promise(r => setTimeout(r, 300))
    const resp = await this.characteristic.readValue()
    return new Uint8Array(resp.buffer, resp.byteOffset, resp.byteLength)
  }

  async read(_charUuid: string): Promise<Uint8Array> {
    if (!this.characteristic) {
      throw new Error('Not connected')
    }
    const resp = await this.characteristic.readValue()
    return new Uint8Array(resp.buffer, resp.byteOffset, resp.byteLength)
  }

  disconnect(): void {
    try {
      this.bleDevice?.gatt?.disconnect()
    } catch { /* ignore */ }
    this.characteristic = null
    // Keep bleDevice reference — it was obtained via requestDevice()
    // which requires user interaction and can't be recreated silently.
    // connect() handles the already-disconnected case by reconnecting.
  }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

export function createBleTransport(): BleTransport {
  return isTauriEnv()
    ? new TauriBlecTransport()
    : new WebBluetoothTransport()
}

export { isTauriEnv }

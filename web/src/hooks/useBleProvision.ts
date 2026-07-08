// useBleProvision Hook
//
// BLE provisioning for HeraMind devices (ESP32-S3 + NimBLE).
// Writes WiFi + MQTT config as JSON to a custom BLE characteristic.
//
// Flow: scan → select device (connects immediately) → fill form → apply (writes config)
// Uses transport abstraction to work in both Tauri (native BLE) and
// browser (Web Bluetooth) environments.

import { useState, useRef, useCallback } from 'react'

import {
  BLE_CHAR_MQTT,
  parseBleDeviceName,
} from '@/lib/ble-protocol'
import type { BleMqttConfig, BleNetmod } from '@/lib/ble-protocol'
import {
  createBleTransport,
  isTauriEnv,
} from '@/lib/ble-transport'
import type { BleTransport, BleTransportDevice } from '@/lib/ble-transport'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Generic BLE device info (works in both Tauri and browser) */
export interface BleDeviceInfo {
  id: string
  name?: string
}

/** Current device config read from BLE characteristic */
export interface BleDeviceConfig {
  device_name: string
  ssid: string
  mac: string
  sn: string
  model: string
  netmod: string
}

export interface UseBleProvisionReturn {
  scanning: boolean
  connecting: boolean
  provisioning: boolean
  connected: boolean
  error: string | null
  device: BleDeviceInfo | null
  deviceModel: string | null
  deviceMac: string | null
  deviceSn: string | null
  deviceConfig: BleDeviceConfig | null
  deviceNetmod: BleNetmod | null
  provisioningStep: string | null
  /** All devices found during last scan */
  scannedDevices: BleTransportDevice[]

  /** Scan for BLE devices. Returns list of found devices. */
  scan: () => Promise<BleTransportDevice[]>
  /** Select a scanned device and connect immediately */
  selectDevice: (device: BleTransportDevice) => Promise<boolean>
  /** Write config to already-connected device */
  writeConfig: (
    ssid: string,
    password: string,
    mqttConfig: BleMqttConfig,
    deviceName?: string,
  ) => Promise<boolean>
  disconnect: () => void
  clearError: () => void
  /** Check if BLE adapter is available (Tauri only) */
  checkAdapter: () => Promise<'ok' | 'off' | 'unavailable'>
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function withTimeout<T>(promise: Promise<T>, ms: number, label: string): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${label} timed out after ${ms}ms`)), ms)
    promise.then(
      (v) => { clearTimeout(timer); resolve(v) },
      (e) => { clearTimeout(timer); reject(e) },
    )
  })
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export function useBleProvision(): UseBleProvisionReturn {
  const [scanning, setScanning] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [provisioning, setProvisioning] = useState(false)
  const [connected, setConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [device, setDevice] = useState<BleDeviceInfo | null>(null)
  const [deviceModel, setDeviceModel] = useState<string | null>(null)
  const [deviceMac, setDeviceMac] = useState<string | null>(null)
  const [deviceSn, setDeviceSn] = useState<string | null>(null)
  const [deviceConfig, setDeviceConfig] = useState<BleDeviceConfig | null>(null)
  const [deviceNetmod, setDeviceNetmod] = useState<BleNetmod | null>(null)
  const [provisioningStep, setProvisioningStep] = useState<string | null>(null)
  const [scannedDevices, setScannedDevices] = useState<BleTransportDevice[]>([])

  const transportRef = useRef<BleTransport | null>(null)

  const clearError = useCallback(() => setError(null), [])

  const getTransport = useCallback((): BleTransport => {
    if (!transportRef.current) {
      transportRef.current = createBleTransport()
    }
    return transportRef.current
  }, [])

  // --- Check BLE adapter ---

  const checkAdapter = useCallback(async (): Promise<'ok' | 'off' | 'unavailable'> => {
    if (!isTauriEnv() && !navigator.bluetooth) {
      return 'unavailable'
    }
    const transport = getTransport()
    if (transport.getAdapterState) {
      const state = await transport.getAdapterState()
      if (state === 'Off') return 'off'
    }
    return 'ok'
  }, [getTransport])

  // --- scan ---

  const scan = useCallback(async (): Promise<BleTransportDevice[]> => {
    const transport = getTransport()
    setScanning(true)
    setError(null)
    setConnected(false)
    setDeviceModel(null)
    setDeviceMac(null)
    setScannedDevices([])

    // Yield to let React render the scanning state before the blocking scan
    await new Promise(resolve => setTimeout(resolve, 0))

    try {
      const devices = await transport.scan()
      setScannedDevices(devices)
      return devices
    } catch (err: unknown) {
      if (err instanceof DOMException && err.name === 'NotFoundError') {
        setScannedDevices([])
        return []
      }
      setError(err instanceof Error ? err.message : String(err))
      return []
    } finally {
      setScanning(false)
    }
  }, [getTransport])

  // --- select device and connect immediately ---

  const selectDevice = useCallback(async (dev: BleTransportDevice): Promise<boolean> => {
    if (dev.name) {
      const parsed = parseBleDeviceName(dev.name)
      if (parsed) {
        setDeviceModel(parsed.model)
        setDeviceMac(parsed.mac)
      }
    }
    setDevice({ id: dev.id, name: dev.name })
    setError(null)

    // Connect immediately so device knows a client is present
    const transport = getTransport()
    setConnecting(true)
    try {
      await withTimeout(transport.connect(dev.id), 30_000, 'BLE connect')
      setConnected(true)

      // Read device info (MAC, SN, model) from BLE characteristic
      try {
        const resp = await transport.read(BLE_CHAR_MQTT)
        const config = JSON.parse(new TextDecoder().decode(resp)) as BleDeviceConfig
        setDeviceConfig(config)
        if (config.mac) setDeviceMac(config.mac)
        if (config.sn) setDeviceSn(config.sn)
        if (config.model) setDeviceModel(config.model)
        if (config.netmod) setDeviceNetmod(config.netmod as BleNetmod)
      } catch {
        // Read is optional — fallback to name-parsed values
      }

      return true
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      setError(msg)
      return false
    } finally {
      setConnecting(false)
    }
  }, [getTransport])

  // --- write config to already-connected device ---

  const writeConfig = useCallback(
    async (
      ssid: string,
      password: string,
      mqttConfig: BleMqttConfig,
      deviceName?: string,
    ): Promise<boolean> => {
      const transport = getTransport()
      setProvisioning(true)
      setError(null)

      try {
        setProvisioningStep('writingMqtt')

        const configJson = JSON.stringify({
          ...mqttConfig,
          ssid,
          wifi_password: password,
          ...(deviceName ? { device_name: deviceName } : {}),
          sleep: true,
        })
        const configData = new TextEncoder().encode(configJson)

        // Write config — catch ALL errors.
        // Device disconnects after receiving config (expected).
        // Device serial confirms config IS received successfully.
        try {
          await withTimeout(
            transport.write(BLE_CHAR_MQTT, configData),
            15_000,
            'BLE write',
          )
        } catch {
          // Device disconnected during/after write — config was likely sent.
          // Also catches timeout — the write may still succeed on the device.
        }

        try { transport.disconnect() } catch { /* ignore */ }

        setProvisioningStep('done')
        return true
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err)
        setError(msg)
        return false
      } finally {
        setProvisioning(false)
        setProvisioningStep(null)
        setConnected(false)
      }
    },
    [getTransport],
  )

  // --- disconnect ---

  const disconnect = useCallback(() => {
    const transport = getTransport()
    transport.disconnect()
    setDevice(null)
    setDeviceModel(null)
    setDeviceMac(null)
    setDeviceSn(null)
    setDeviceConfig(null)
    setDeviceNetmod(null)
    setError(null)
    setConnected(false)
    setProvisioningStep(null)
    setScannedDevices([])
  }, [getTransport])

  return {
    scanning, connecting, provisioning, connected, error, device,
    deviceModel, deviceMac, deviceSn, deviceNetmod, deviceConfig, provisioningStep, scannedDevices,
    scan, selectDevice, writeConfig, disconnect, clearError, checkAdapter,
  }
}

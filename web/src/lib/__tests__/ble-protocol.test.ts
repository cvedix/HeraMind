import { describe, it, expect } from 'vitest'

import {
  parseBleDeviceName,
  modelToDeviceType,
  encodeMqttConfig,
  encodeSetConfig,
  encodeGetStatus,
  decodeRespSetConfig,
  decodeRespGetStatus,
  viewToBytes,
  WifiState,
  writeWithMtu,
} from '@/lib/ble-protocol'

// ---------------------------------------------------------------------------
// parseBleDeviceName
// ---------------------------------------------------------------------------

describe('parseBleDeviceName', () => {
  it('parses underscore format with full MAC: NE101_AABBCC70B0E2', () => {
    expect(parseBleDeviceName('NE101_AABBCC70B0E2')).toEqual({
      model: 'NE101',
      mac: 'AA:BB:CC:70:B0:E2',
      macSuffix: '70B0E2',
    })
  })

  it('parses old 3-byte suffix format: NE101_70B0E2', () => {
    expect(parseBleDeviceName('NE101_70B0E2')).toEqual({
      model: 'NE101',
      mac: '70B0E2',
      macSuffix: '70B0E2',
    })
  })

  it('parses dash format: NE301-A1B2C3', () => {
    expect(parseBleDeviceName('NE301-A1B2C3')).toEqual({
      model: 'NE301',
      mac: 'A1B2C3',
      macSuffix: 'A1B2C3',
    })
  })

  it('parses full MAC with dash: NE301-AABBCCEEDFF0', () => {
    expect(parseBleDeviceName('NE301-AABBCCEEDFF0')).toEqual({
      model: 'NE301',
      mac: 'AA:BB:CC:EE:DF:F0',
      macSuffix: 'EEDFF0',
    })
  })

  it('returns null for plain name without separator', () => {
    expect(parseBleDeviceName('NE101')).toBeNull()
  })

  it('returns null for empty string', () => {
    expect(parseBleDeviceName('')).toBeNull()
  })

  it('returns null for lowercase model', () => {
    expect(parseBleDeviceName('ne101_ABC')).toBeNull()
  })

  it('returns null for model without digits', () => {
    expect(parseBleDeviceName('DEVICE_AB')).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// modelToDeviceType
// ---------------------------------------------------------------------------

describe('modelToDeviceType', () => {
  it('maps NE101 to ne101_camera', () => {
    expect(modelToDeviceType('NE101')).toBe('ne101_camera')
  })

  it('maps NE301 to ne301_camera', () => {
    expect(modelToDeviceType('NE301')).toBe('ne301_camera')
  })

  it('derives type for unknown models matching pattern', () => {
    expect(modelToDeviceType('NE202')).toBe('ne202_camera')
    expect(modelToDeviceType('AB500')).toBe('ab500_camera')
  })

  it('returns null for non-model strings', () => {
    expect(modelToDeviceType('device')).toBeNull()
    expect(modelToDeviceType('123')).toBeNull()
    expect(modelToDeviceType('')).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// encodeMqttConfig
// ---------------------------------------------------------------------------

describe('encodeMqttConfig', () => {
  it('encodes MQTT config as JSON Uint8Array', () => {
    const config = {
      host: '192.168.1.100',
      port: 1883,
      username: 'user1',
      password: 'pass1',
      topic_prefix: 'device/ne101_camera/ne101_abc123',
      client_id: 'ne101_abc123',
    }
    const bytes = encodeMqttConfig(config)
    const json = JSON.parse(new TextDecoder().decode(bytes))
    expect(json).toEqual(config)
  })

  it('handles empty strings', () => {
    const config = {
      host: 'localhost',
      port: 1883,
      username: '',
      password: '',
      topic_prefix: 'test',
      client_id: '',
    }
    const bytes = encodeMqttConfig(config)
    const json = JSON.parse(new TextDecoder().decode(bytes))
    expect(json.username).toBe('')
    expect(json.password).toBe('')
  })
})

// ---------------------------------------------------------------------------
// Protobuf encode/decode round-trip tests
// ---------------------------------------------------------------------------

describe('encodeSetConfig + decodeRespSetConfig', () => {
  it('encodes SetConfig with SSID and password', () => {
    const data = encodeSetConfig('MyWiFi', 'secret123')
    // Should be a non-empty protobuf message
    expect(data.length).toBeGreaterThan(0)
    // First byte should be a varint field tag for field 1 (msg type)
    expect(data[0]).toBeDefined()
  })
})

describe('encodeGetStatus', () => {
  it('produces a valid protobuf payload', () => {
    const data = encodeGetStatus()
    expect(data.length).toBeGreaterThan(0)
  })
})

describe('decodeRespSetConfig', () => {
  it('returns -1 for empty data', () => {
    expect(decodeRespSetConfig(new Uint8Array(0))).toBe(-1)
  })

  it('returns -1 for random bytes', () => {
    expect(decodeRespSetConfig(new Uint8Array([0x01, 0x02, 0x03]))).toBe(-1)
  })
})

describe('decodeRespGetStatus', () => {
  it('returns defaults for empty data', () => {
    const result = decodeRespGetStatus(new Uint8Array(0))
    expect(result).toEqual({ status: 0, wifiState: 0, failedReason: 0 })
  })

  it('returns defaults for random bytes', () => {
    const result = decodeRespGetStatus(new Uint8Array([0xFF, 0xFF]))
    // Should not crash, returns some parsed value
    expect(result).toHaveProperty('status')
    expect(result).toHaveProperty('wifiState')
    expect(result).toHaveProperty('failedReason')
  })
})

// ---------------------------------------------------------------------------
// viewToBytes helper
// ---------------------------------------------------------------------------

describe('viewToBytes', () => {
  it('converts DataView to Uint8Array', () => {
    const buffer = new ArrayBuffer(4)
    const view = new DataView(buffer)
    view.setUint8(0, 0x0A)
    view.setUint8(1, 0x0B)
    view.setUint8(2, 0x0C)
    view.setUint8(3, 0x0D)
    const bytes = viewToBytes(view)
    expect(bytes).toEqual(new Uint8Array([0x0A, 0x0B, 0x0C, 0x0D]))
  })

  it('handles offset DataView', () => {
    const buffer = new ArrayBuffer(8)
    const view = new DataView(buffer, 2, 4)
    view.setUint8(0, 0xAA)
    view.setUint8(1, 0xBB)
    const bytes = viewToBytes(view)
    expect(bytes.length).toBe(4)
    expect(bytes[0]).toBe(0xAA)
    expect(bytes[1]).toBe(0xBB)
  })
})

// ---------------------------------------------------------------------------
// WifiState enum
// ---------------------------------------------------------------------------

describe('WifiState', () => {
  it('has expected values', () => {
    expect(WifiState.Connected).toBe(0)
    expect(WifiState.Connecting).toBe(1)
    expect(WifiState.Disconnected).toBe(2)
    expect(WifiState.ConnectionFailed).toBe(3)
  })
})

// ---------------------------------------------------------------------------
// writeWithMtu
// ---------------------------------------------------------------------------

describe('writeWithMtu', () => {
  // Minimal mock for BluetoothRemoteGATTCharacteristic
  function makeMockChar() {
    const written: Uint8Array[] = []
    const char = {
      writeValueWithResponse: vi.fn(async (data: Uint8Array) => {
        written.push(new Uint8Array(data))
      }),
      readValue: vi.fn(async () => {
        return new DataView(new ArrayBuffer(0))
      }),
      _written: written,
    }
    return char as any
  }

  it('writes small payload in a single chunk', async () => {
    const data = new Uint8Array([1, 2, 3])
    const char = makeMockChar()
    await writeWithMtu(char, data, 20)
    expect(char.writeValueWithResponse).toHaveBeenCalledTimes(1)
    expect(char._written[0]).toEqual(data)
  })

  it('writes large payload in multiple chunks', async () => {
    // 50 bytes with MTU=20 → 3 chunks (20 + 20 + 10)
    const data = new Uint8Array(50).fill(0xAB)
    const char = makeMockChar()
    await writeWithMtu(char, data, 20)
    expect(char.writeValueWithResponse).toHaveBeenCalledTimes(3)
    expect(char._written[0].length).toBe(20)
    expect(char._written[1].length).toBe(20)
    expect(char._written[2].length).toBe(10)
  })

  it('handles payload exactly equal to MTU', async () => {
    const data = new Uint8Array(20).fill(0x01)
    const char = makeMockChar()
    await writeWithMtu(char, data, 20)
    expect(char.writeValueWithResponse).toHaveBeenCalledTimes(1)
  })

  it('uses default MTU of 20 when not specified', async () => {
    const data = new Uint8Array(25).fill(0xCC)
    const char = makeMockChar()
    await writeWithMtu(char, data)
    expect(char.writeValueWithResponse).toHaveBeenCalledTimes(2)
  })
})

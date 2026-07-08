// useExtensionStream Hook for React Components
//
// Provides extension streaming capabilities to React components.
// Supports stateless and stateful streaming modes.

import { useEffect, useState, useCallback, useRef } from 'react'
import type {
  StreamCapability,
  ExtensionStreamConnectionState,
  ExtensionStreamResult,
  ExtensionSessionStats,
  UseExtensionStreamOptions,
} from '@/types'
import { getExtensionStreamClient, closeExtensionStreamClient } from '@/lib/extension-stream'

export interface UseExtensionStreamResult {
  /**
   * Current connection state
   */
  state: ExtensionStreamConnectionState

  /**
   * Stream capability (available after connection)
   */
  capability: StreamCapability | null

  /**
   * Current session ID (for stateful streams)
   */
  sessionId: string | null

  /**
   * Whether connected
   */
  isConnected: boolean

  /**
   * Whether connecting
   */
  isConnecting: boolean

  /**
   * Last error
   */
  error: Error | null

  /**
   * Connect to the extension stream
   */
  connect: () => void

  /**
   * Disconnect from the stream
   */
  disconnect: () => void

  /**
   * Send binary data chunk
   */
  sendChunk: (data: ArrayBuffer | Uint8Array, sequence: number) => void

  /**
   * Initialize a stateful session
   */
  initSession: (config?: Record<string, unknown>) => void

  /**
   * Send acknowledgment (flow control)
   */
  sendAck: (sequence: number) => void

  /**
   * Request close
   */
  close: () => void

  /**
   * Recent results
   */
  results: ExtensionStreamResult[]
}

/**
 * useExtensionStream - Hook for extension streaming
 *
 * @example
 * ```tsx
 * function ImageAnalyzer({ extensionId }: { extensionId: string }) {
 *   const { isConnected, capability, sendChunk, results } = useExtensionStream({
 *     extensionId,
 *     onResult: (result) => {
 *       console.log('Processed:', result.dataType, result.data.length)
 *     }
 *   })
 *
 *   const handleFileSelect = async (file: File) => {
 *     const data = await file.arrayBuffer()
 *     sendChunk(data, 0)
 *   }
 *
 *   return (
 *     <div>
 *       <p>Status: {isConnected ? 'Connected' : 'Disconnected'}</p>
 *       <input type="file" onChange={(e) => handleFileSelect(e.target.files[0])} />
 *     </div>
 *   )
 * }
 * ```
 */
export function useExtensionStream(options: UseExtensionStreamOptions): UseExtensionStreamResult {
  const {
    extensionId,
    config,
    enabled = true,
    onResult,
    onError,
    onConnectionChange,
    onSessionClosed,
  } = options

  const [state, setState] = useState<ExtensionStreamConnectionState>({
    status: 'disconnected',
    sessionId: null,
    capability: null,
  })
  const [results, setResults] = useState<ExtensionStreamResult[]>([])
  const [error, setError] = useState<Error | null>(null)

  // Use refs to store latest callbacks without causing re-renders
  const onResultRef = useRef(onResult)
  const onErrorRef = useRef(onError)
  const onConnectionChangeRef = useRef(onConnectionChange)
  const onSessionClosedRef = useRef(onSessionClosed)
  const configRef = useRef(config)

  // Update refs when callbacks/config change
  useEffect(() => {
    onResultRef.current = onResult
  }, [onResult])

  useEffect(() => {
    onErrorRef.current = onError
  }, [onError])

  useEffect(() => {
    onConnectionChangeRef.current = onConnectionChange
  }, [onConnectionChange])

  useEffect(() => {
    onSessionClosedRef.current = onSessionClosed
  }, [onSessionClosed])

  useEffect(() => {
    configRef.current = config
  }, [config])

  // Get or create stream client
  const clientRef = useRef<ReturnType<typeof getExtensionStreamClient> | null>(null)
  const unsubscribesRef = useRef<Array<() => void>>([])

  // Connection methods
  const connect = useCallback(() => {
    // Clean up previous handlers if any
    unsubscribesRef.current.forEach(unsub => unsub?.())
    unsubscribesRef.current = []

    const client = getExtensionStreamClient(extensionId)
    clientRef.current = client

    // Set up handlers
    const unsubscribeConnection = client.onConnection((newState) => {
      setState(newState)
      if (newState.status === 'error' && newState.errorMessage) {
        setError(new Error(newState.errorMessage))
      } else {
        setError(null)
      }
      onConnectionChangeRef.current?.(newState)
    })

    const unsubscribeResult = client.onResult((data, dataType, sequence) => {
      const result: ExtensionStreamResult = {
        inputSequence: null, // Could be tracked if needed
        outputSequence: sequence,
        data,
        dataType,
        processingMs: 0, // Could be included in server message
      }
      setResults(prev => [...prev, result].slice(-100)) // Keep last 100
      onResultRef.current?.(result)
    })

    const unsubscribeError = client.onError((err) => {
      setError(err)
      onErrorRef.current?.(err)
    })

    const unsubscribeSessionClosed = client.onSessionClosed((stats) => {
      onSessionClosedRef.current?.(stats)
    })

    // Store unsubscribe functions for cleanup
    unsubscribesRef.current = [
      unsubscribeConnection,
      unsubscribeResult,
      unsubscribeError,
      unsubscribeSessionClosed,
    ]

    // Connect with config
    client.connect(configRef.current)
  }, [extensionId])

  const disconnect = useCallback(() => {
    // Call unsubscribe functions
    unsubscribesRef.current.forEach(unsub => unsub?.())
    unsubscribesRef.current = []

    if (clientRef.current) {
      clientRef.current.disconnect()
      clientRef.current = null
    }
  }, [])

  const sendChunk = useCallback((data: ArrayBuffer | Uint8Array, sequence: number) => {
    clientRef.current?.sendChunk(data, sequence)
  }, [])

  const initSession = useCallback((sessionConfig?: Record<string, unknown>) => {
    clientRef.current?.initSession(sessionConfig)
  }, [])

  const sendAck = useCallback((sequence: number) => {
    clientRef.current?.sendAck(sequence)
  }, [])

  const close = useCallback(() => {
    clientRef.current?.close()
  }, [])

  // Auto-connect when enabled
  useEffect(() => {
    if (enabled) {
      connect()
    }

    return () => {
      disconnect()
    }
  }, [enabled, connect])

  // Clean up client on unmount
  useEffect(() => {
    return () => {
      if (clientRef.current) {
        closeExtensionStreamClient(extensionId)
      }
    }
  }, [extensionId])

  return {
    state,
    capability: state.capability,
    sessionId: state.sessionId,
    isConnected: state.status === 'connected',
    isConnecting: state.status === 'connecting',
    error,
    connect,
    disconnect,
    sendChunk,
    initSession,
    sendAck,
    close,
    results,
  }
}

/**
 * useImageAnalyzer - Hook for image analysis extensions
 *
 * Convenience hook for stateless image analysis extensions.
 */
export function useImageAnalyzer(options: {
  extensionId: string
  onResult: (result: {
    data: Uint8Array
    dataType: string
    analysis?: unknown
  }) => void
  onError?: (error: Error) => void
}) {
  const { extensionId, onResult, onError } = options

  return useExtensionStream({
    extensionId,
    onResult: (result) => {
      // Try to parse JSON result for analysis data
      let analysis: unknown
      if (result.dataType === 'application/json') {
        try {
          const text = new TextDecoder().decode(result.data)
          analysis = JSON.parse(text)
        } catch {
          // Not JSON, keep as is
        }
      }
      onResult({ data: result.data, dataType: result.dataType, analysis })
    },
    onError,
  })
}

/**
 * useVideoProcessor - Hook for video processing extensions
 *
 * Convenience hook for stateful video stream processing.
 */
export function useVideoProcessor(options: {
  extensionId: string
  config?: Record<string, unknown>
  onFrameResult: (result: {
    frameNumber: number
    data: Uint8Array
    dataType: string
    detections?: unknown
  }) => void
  onSessionClosed?: (stats: ExtensionSessionStats) => void
  onError?: (error: Error) => void
}) {
  const { extensionId, config, onFrameResult, onSessionClosed, onError } = options
  const frameCountRef = useRef(0)

  return useExtensionStream({
    extensionId,
    config,
    onResult: (result) => {
      frameCountRef.current++
      // Try to parse detection results
      let detections: unknown
      if (result.dataType === 'application/json') {
        try {
          const text = new TextDecoder().decode(result.data)
          detections = JSON.parse(text)
        } catch {
          // Not JSON
        }
      }
      onFrameResult({
        frameNumber: result.outputSequence,
        data: result.data,
        dataType: result.dataType,
        detections,
      })
    },
    onSessionClosed: (stats) => {
      frameCountRef.current = 0
      onSessionClosed?.(stats)
    },
    onError,
  })
}

/**
 * useSensorStream - Hook for sensor data streaming extensions
 *
 * Convenience hook for push-mode sensor streams.
 */
export function useSensorStream(options: {
  extensionId: string
  config?: Record<string, unknown>
  onDataPoint: (data: {
    timestamp: number
    value: number
    unit?: string
    metadata?: Record<string, unknown>
  }) => void
  onError?: (error: Error) => void
}) {
  const { extensionId, config, onDataPoint, onError } = options

  return useExtensionStream({
    extensionId,
    config,
    onResult: (result) => {
      // Try to parse sensor data
      try {
        const text = new TextDecoder().decode(result.data)
        const data = JSON.parse(text) as {
          timestamp?: number
          value?: number
          unit?: string
          metadata?: Record<string, unknown>
        }
        onDataPoint({
          timestamp: data.timestamp ?? Date.now(),
          value: data.value ?? 0,
          unit: data.unit,
          metadata: data.metadata,
        })
      } catch (err) {
        console.error('[useSensorStream] Failed to parse sensor data:', err)
      }
    },
    onError,
  })
}

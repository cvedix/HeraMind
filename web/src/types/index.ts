// ============================================================================
// HeraMind TypeScript Types — Barrel
// ============================================================================
//
// This file re-exports all type domains so existing `import { X } from '@/types'`
// calls continue to work without modification. Domain files live alongside
// this barrel and own their respective definitions.
//
// Do NOT add definitions directly to this file — add them to the relevant
// domain module below and let the barrel pick them up via `export *`.

// Authentication & users
export * from './auth'

// Devices, telemetry, commands, auto-onboarding
export * from './device'

// Unified messaging / notification system
export * from './message'

// Common API types (ApiResponse, JsonSchema, JSONSchema, PluginConfigSchema)
export * from './api'

// Chat sessions, WebSocket messages, streaming
export * from './chat'

// MQTT broker & external broker connections
export * from './mqtt'

// Rules engine
export * from './rule'

// Agent memory & system memory config
export * from './memory'

// Events & search
export * from './event'

// Extensions (capabilities, commands, metrics, queries)
export * from './extension'

// Tool definitions & execution metrics
export * from './tool'

// LLM backend management & device adapters
export * from './llm-backend'

// Automations (transforms, templates)
export * from './automation'

// AI Agents (schedules, resources, executions, decision process)
export * from './agent'

// Dashboard API DTOs (response/request shapes for dashboard CRUD)
export * from './dashboard-api'

// Extension streaming (WebSocket protocol, capabilities, hooks)
export * from './extension-streaming'

// Dashboard components provided by extensions (widgets catalog)
export * from './extension-component'

// Data push targets & delivery logs
export * from './data-push'

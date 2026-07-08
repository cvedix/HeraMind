/**
 * Seed initial values for a command's parameters.
 *
 * Consolidates the parameter-default initialization logic that previously
 * lived inline in both `CommandButton.tsx` and `DeviceDetail.tsx`. Beyond
 * deduplication, this module also recognizes a small set of "ID-shaped"
 * parameter names (`request_id`, `trace_id`, …) and pre-fills them with a
 * generated UUID so the user doesn't have to hand-craft a tracking ID
 * every time they invoke a command like NE301's `capture`.
 *
 * The detection is intentionally conservative — name-based only, opt-out
 * by renaming the parameter. Template authors who want explicit control
 * can still set `default_value` on the parameter, which always wins.
 */

import { generateId } from '@/lib/id'
import type { ParameterDefinition } from '@/types/device'

/**
 * Parameter names that should be auto-populated with a generated id.
 * Matched case-insensitively against the bare parameter name, with or
 * without underscores. Covers the common conventions:
 *   request_id, requestId, trace_id, traceId, correlation_id, msg_id, message_id
 */
const AUTO_ID_PATTERNS = [
  /^request_?id$/i,
  /^trace_?id$/i,
  /^correlation_?id$/i,
  /^msg_?id$/i,
  /^message_?id$/i,
]

/** Returns true if this parameter should be auto-filled with a generated id. */
export function isAutoIdParam(param: ParameterDefinition): boolean {
  // Explicit defaults win — never override a template-declared default.
  if (param.default_value !== undefined) return false
  // Only string-typed parameters are candidates. Numbers/booleans named
  // "request_id" would be nonsensical and we shouldn't second-guess that.
  if (param.data_type !== 'string') return false
  return AUTO_ID_PATTERNS.some((re) => re.test(param.name))
}

/**
 * Build the initial `values` map for a command's parameters.
 *
 * Resolution order for each parameter:
 *   1. `default_value` from schema (always wins)
 *   2. Auto-generated UUID for recognized ID parameters
 *   3. Type-appropriate empty value (0 / false / [] / first enum item / '')
 */
export function seedCommandDefaults(
  parameters: ParameterDefinition[] | undefined | null,
): Record<string, unknown> {
  if (!parameters || parameters.length === 0) return {}

  const values: Record<string, unknown> = {}

  for (const param of parameters) {
    // 1. Explicit default
    if (param.default_value !== undefined) {
      values[param.name] = param.default_value
      continue
    }

    // 2. Auto-id (only for string params matching known patterns)
    if (isAutoIdParam(param)) {
      values[param.name] = generateId()
      continue
    }

    // 3. Type-appropriate empty value
    values[param.name] = emptyValueFor(param)
  }

  return values
}

/** Return a sensible "empty" value based on the parameter's declared type. */
function emptyValueFor(param: ParameterDefinition): unknown {
  // Object-form enum on data_type → first option
  if (
    param.data_type &&
    typeof param.data_type === 'object' &&
    'enum' in param.data_type &&
    param.data_type.enum.length > 0
  ) {
    return param.data_type.enum[0]
  }

  // Explicit allowed_values array → first option (preserves original type)
  if (param.allowed_values && param.allowed_values.length > 0) {
    return param.allowed_values[0]
  }

  switch (param.data_type) {
    case 'integer':
    case 'float':
      return 0
    case 'boolean':
      return false
    case 'array':
      return []
    case 'string':
    case 'binary':
    case null:
    case undefined:
    default:
      return ''
  }
}

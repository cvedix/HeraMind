/**
 * Data Mapping Utilities
 *
 * Provides unified data transformation and mapping capabilities
 * for dashboard components with auto-detection and configurable options.
 */

// ============================================================================
// Types
// ============================================================================

/**
 * Field detection candidates for auto-detection
 */
const VALUE_FIELD_CANDIDATES = ['value', 'v', 'val', 'count', 'amount', 'avg', 'min', 'max', 'data'] as const
const LABEL_FIELD_CANDIDATES = ['name', 'label', 'category', 'key', 'title', 'id', 'deviceId'] as const
const TIMESTAMP_FIELD_CANDIDATES = ['timestamp', 'time', 't', 'date', 'createdAt', 'updatedAt'] as const

/**
 * Common data mapping configuration
 */
export interface DataMappingConfig {
  /** Field to extract value from */
  valueField?: string
  /** Field to extract label from */
  labelField?: string
  /** Field to extract timestamp from */
  timestampField?: string

  /** Aggregation method for multiple values */
  aggregate?: 'none' | 'sum' | 'avg' | 'count' | 'min' | 'max' | 'latest' | 'first'

  /** Data filtering */
  filter?: {
    field?: string
    operator?: 'eq' | 'gt' | 'lt' | 'gte' | 'lte' | 'contains' | 'ne'
    value?: unknown
  }

  /** Data sorting */
  sortBy?: 'value' | 'label' | 'timestamp' | 'index'
  sortOrder?: 'asc' | 'desc'

  /** Data limiting */
  limit?: number
  offset?: number

  /** Custom transform function name (for predefined transforms) */
  transform?: string
}

/**
 * Single value display specific configuration
 */
export interface SingleValueMappingConfig extends DataMappingConfig {
  /** Format type */
  format?: 'number' | 'currency' | 'percent' | 'bytes' | 'duration' | 'auto'
  /** Custom format string */
  formatString?: string

  /** Thresholds for status determination */
  thresholds?: {
    warning?: { operator: string; value: number }
    error?: { operator: string; value: number }
    success?: { operator: string; value: number }
  }

  /** LED-specific state thresholds for indicator components */
  stateThresholds?: {
    on?: { operator: string; value: number }
    off?: { operator: string; value: number }
    error?: { operator: string; value: number }
    warning?: { operator: string; value: number }
  }

  /** Unit display */
  unit?: string
  unitPosition?: 'prefix' | 'suffix'
}

/**
 * Control component specific configuration
 */
export interface ControlMappingConfig extends DataMappingConfig {
  /** Value mapping for toggle-like controls */
  valueMapping?: {
    onValues: unknown[]
    offValues: unknown[]
  }

  /** Options for selection controls */
  options?: Array<{
    label: string
    value: unknown
    icon?: string
    color?: string
  }>

  /** Write configuration for commands */
  writeConfig?: {
    command?: string
    valueTemplate?: string
  }
}

/**
 * Time series chart specific configuration
 */
export interface TimeSeriesMappingConfig extends DataMappingConfig {
  /** Time aggregation level */
  timeAggregate?: 'raw' | '1m' | '5m' | '15m' | '1h' | '1d' | '1w'

  /** Chart styling */
  smooth?: boolean
  fillArea?: boolean
  showPoints?: boolean

  /** Multi-series handling */
  multiSeries?: {
    groupBy?: string
    stack?: boolean
  }
}

/**
 * Categorical chart specific configuration
 */
export interface CategoricalMappingConfig extends DataMappingConfig {
  /** How to aggregate categories */
  aggregateMethod?: 'none' | 'sum' | 'avg' | 'count'

  /** Show only top N items */
  showTop?: number
  /** Label for "others" category */
  othersLabel?: string

  /** Label handling */
  labelTruncate?: number
  labelRotate?: boolean

  /** Pie specific */
  showPercentage?: boolean
  minSlicePercent?: number
}

/**
 * Normalized data point with value and label
 */
export interface MappedDataPoint {
  value: number
  label: string
  timestamp?: number
  original?: unknown
  color?: string
}

/**
 * Time series data point
 */
export interface TimeSeriesPoint {
  timestamp: number
  value: number
  label?: string
  series?: string
}

/**
 * Categorical data point
 */
export interface CategoricalDataPoint {
  name: string
  value: number
  percentage?: number
  color?: string
}

// ============================================================================
// DataMapper Class
// ============================================================================

export class DataMapper {
  /**
   * Auto-detect the value field from an object
   */
  static detectValueField(item: Record<string, unknown>): string {
    for (const field of VALUE_FIELD_CANDIDATES) {
      if (field in item && typeof item[field] === 'number') {
        return field
      }
    }
    // Fallback to first numeric field
    for (const [key, value] of Object.entries(item)) {
      if (typeof value === 'number') {
        return key
      }
    }
    return 'value'
  }

  /**
   * Auto-detect the label field from an object
   */
  static detectLabelField(item: Record<string, unknown>): string {
    for (const field of LABEL_FIELD_CANDIDATES) {
      if (field in item && typeof item[field] === 'string') {
        return field
      }
    }
    // Fallback to first string field
    for (const [key, value] of Object.entries(item)) {
      if (typeof value === 'string') {
        return key
      }
    }
    return 'name'
  }

  /**
   * Auto-detect the timestamp field from an object
   */
  static detectTimestampField(item: Record<string, unknown>): string | undefined {
    for (const field of TIMESTAMP_FIELD_CANDIDATES) {
      if (field in item) {
        const value = item[field]
        if (typeof value === 'number' || typeof value === 'string') {
          return field
        }
      }
    }
    return undefined
  }

  /**
   * Extract value from data item using config or auto-detection
   */
  static extractValue(item: unknown, config?: DataMappingConfig): number {
    if (item === null || item === undefined) {
      return 0
    }

    // Direct number
    if (typeof item === 'number') {
      return item
    }

    // String that can be parsed to number
    if (typeof item === 'string') {
      const parsed = parseFloat(item)
      return isNaN(parsed) ? 0 : parsed
    }

    // Boolean to number
    if (typeof item === 'boolean') {
      return item ? 1 : 0
    }

    // Array - take last element (most recent for telemetry data)
    if (Array.isArray(item)) {
      if (item.length > 0) {
        return this.extractValue(item[item.length - 1], config)
      }
      return 0
    }

    // Object - extract field
    if (typeof item === 'object') {
      const obj = item as Record<string, unknown>
      const field = config?.valueField || this.detectValueField(obj)
      const value = obj[field]

      if (typeof value === 'number') {
        return value
      }
      if (typeof value === 'string') {
        const parsed = parseFloat(value)
        return isNaN(parsed) ? 0 : parsed
      }
      if (typeof value === 'boolean') {
        return value ? 1 : 0
      }
    }

    return 0
  }

  /**
   * Extract label from data item using config or auto-detection
   */
  static extractLabel(item: unknown, index: number, config?: DataMappingConfig): string {
    if (item === null || item === undefined) {
      return `Item ${index + 1}`
    }

    // Direct string
    if (typeof item === 'string') {
      return item
    }

    // Direct number
    if (typeof item === 'number') {
      return `Value ${item}`
    }

    // Object - extract field
    if (typeof item === 'object') {
      const obj = item as Record<string, unknown>
      const field = config?.labelField || this.detectLabelField(obj)
      const label = obj[field]

      if (typeof label === 'string') {
        return label
      }
      if (typeof label === 'number') {
        return label.toString()
      }
    }

    return `Item ${index + 1}`
  }

  /**
   * Extract timestamp from data item
   */
  static extractTimestamp(item: unknown, config?: DataMappingConfig): number | undefined {
    if (item === null || item === undefined) {
      return undefined
    }

    if (typeof item === 'object') {
      const obj = item as Record<string, unknown>
      const field = config?.timestampField || this.detectTimestampField(obj)

      if (field) {
        const value = obj[field]
        if (typeof value === 'number') {
          return value
        }
        if (typeof value === 'string') {
          const parsed = new Date(value).getTime()
          return isNaN(parsed) ? undefined : parsed / 1000
        }
      }
    }

    return undefined
  }

  /**
   * Apply filter to data array
   */
  static filterData<T>(data: T[], config?: DataMappingConfig): T[] {
    if (!config?.filter || !config.filter.field) {
      return data
    }

    const { field, operator = 'eq', value } = config.filter

    return data.filter((item) => {
      if (typeof item !== 'object' || item === null) {
        return true
      }

      const obj = item as Record<string, unknown>
      const itemValue = obj[field]

      switch (operator) {
        case 'eq':
          return itemValue === value
        case 'ne':
          return itemValue !== value
        case 'gt':
          return typeof itemValue === 'number' && typeof value === 'number' && itemValue > value
        case 'lt':
          return typeof itemValue === 'number' && typeof value === 'number' && itemValue < value
        case 'gte':
          return typeof itemValue === 'number' && typeof value === 'number' && itemValue >= value
        case 'lte':
          return typeof itemValue === 'number' && typeof value === 'number' && itemValue <= value
        case 'contains':
          return typeof itemValue === 'string' && typeof value === 'string' && itemValue.includes(value)
        default:
          return true
      }
    })
  }

  /**
   * Sort data array
   */
  static sortData<T>(data: T[], config?: DataMappingConfig): T[] {
    if (!config?.sortBy) {
      return data
    }

    const { sortBy, sortOrder = 'asc' } = config
    const multiplier = sortOrder === 'asc' ? 1 : -1

    return [...data].sort((a, b) => {
      let aVal: unknown
      let bVal: unknown

      if (typeof a === 'object' && a !== null) {
        const objA = a as Record<string, unknown>
        if (sortBy === 'value') {
          aVal = this.extractValue(a, config)
        } else if (sortBy === 'label') {
          aVal = this.extractLabel(a, 0, config)
        } else if (sortBy === 'timestamp') {
          aVal = this.extractTimestamp(a, config) ?? 0
        } else {
          aVal = objA[sortBy]
        }
      } else {
        aVal = a
      }

      if (typeof b === 'object' && b !== null) {
        const objB = b as Record<string, unknown>
        if (sortBy === 'value') {
          bVal = this.extractValue(b, config)
        } else if (sortBy === 'label') {
          bVal = this.extractLabel(b, 0, config)
        } else if (sortBy === 'timestamp') {
          bVal = this.extractTimestamp(b, config) ?? 0
        } else {
          bVal = objB[sortBy]
        }
      } else {
        bVal = b
      }

      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return (aVal - bVal) * multiplier
      }
      if (typeof aVal === 'string' && typeof bVal === 'string') {
        return aVal.localeCompare(bVal) * multiplier
      }
      return 0
    })
  }

  /**
   * Apply limit and offset to data array
   */
  static limitData<T>(data: T[], config?: DataMappingConfig): T[] {
    let result = data

    if (config?.offset) {
      result = result.slice(config.offset)
    }

    if (config?.limit) {
      result = result.slice(0, config.limit)
    }

    return result
  }

  /**
   * Apply aggregation to array of values
   */
  static aggregate(values: number[], method: DataMappingConfig['aggregate'] = 'none'): number {
    if (values.length === 0) {
      return 0
    }

    // Filter NaN to prevent poisoning aggregation results
    const cleanValues = values.filter((v: number) => !isNaN(v))
    if (cleanValues.length === 0) return 0

    switch (method) {
      case 'sum':
        return cleanValues.reduce((a, b) => a + b, 0)
      case 'avg':
        return cleanValues.reduce((a, b) => a + b, 0) / cleanValues.length
      case 'count':
        return cleanValues.length
      case 'min':
        return cleanValues.reduce((a, b) => Math.min(a, b), cleanValues[0])
      case 'max':
        return cleanValues.reduce((a, b) => Math.max(a, b), cleanValues[0])
      case 'latest':
        return cleanValues[cleanValues.length - 1]
      case 'first':
        return cleanValues[0]
      case 'none':
      default:
        return cleanValues[0]
    }
  }

  /**
   * Map array of raw data to normalized data points
   */
  static mapToDataPoints(data: unknown[], config?: DataMappingConfig): MappedDataPoint[] {
    // Apply filter
    const filtered = this.filterData(data, config)

    // Apply sort
    const sorted = this.sortData(filtered, config)

    // Apply limit
    const limited = this.limitData(sorted, config)

    return limited.map((item, index) => ({
      value: this.extractValue(item, config),
      label: this.extractLabel(item, index, config),
      timestamp: this.extractTimestamp(item, config),
      original: item,
    }))
  }

  /**
   * Map array to time series points
   */
  static mapToTimeSeries(data: unknown[], config?: DataMappingConfig): TimeSeriesPoint[] {
    const points = this.mapToDataPoints(data, config)

    // Sort by timestamp if available
    if (points.some(p => p.timestamp !== undefined)) {
      return points
        .filter(p => p.timestamp !== undefined)
        .sort((a, b) => (a.timestamp ?? 0) - (b.timestamp ?? 0))
        .map(p => ({
          timestamp: p.timestamp ?? 0,
          value: p.value,
          label: p.label,
        }))
    }

    // No timestamps - use index as time
    return points.map((p, i) => ({
      timestamp: i,
      value: p.value,
      label: p.label,
    }))
  }

  /**
   * Map array to categorical data points
   */
  static mapToCategorical(data: unknown[], config?: CategoricalMappingConfig): CategoricalDataPoint[] {
    let points = this.mapToDataPoints(data, config)

    // Apply aggregation for same labels
    if (config?.aggregateMethod && config.aggregateMethod !== 'none') {
      const grouped = new Map<string, number[]>()

      for (const point of points) {
        const key = point.label
        if (!grouped.has(key)) {
          grouped.set(key, [])
        }
        grouped.get(key)!.push(point.value)
      }

      points = Array.from(grouped.entries()).map(([label, values]) => ({
        value: this.aggregate(values, config.aggregateMethod),
        label,
      }))
    }

    // Sort by value descending
    points.sort((a, b) => b.value - a.value)

    // Apply showTop and merge others
    if (config?.showTop && config.showTop < points.length) {
      const top = points.slice(0, config.showTop)
      const others = points.slice(config.showTop)
      const othersValue = others.reduce((sum, p) => sum + p.value, 0)

      if (othersValue > 0) {
        top.push({
          label: config.othersLabel || 'Others',
          value: othersValue,
        })
      }

      points = top
    }

    // Calculate percentages
    const total = points.reduce((sum, p) => sum + p.value, 0)
    return points.map(p => ({
      name: p.label,
      value: p.value,
      percentage: total > 0 ? (p.value / total) * 100 : 0,
      color: p.color,
    }))
  }

  /**
   * Format a value according to format config
   */
  static formatValue(value: number, config?: SingleValueMappingConfig): string {
    const format = config?.format || 'auto'
    const formatString = config?.formatString

    if (formatString) {
      // Use custom format string (simple implementation)
      if (formatString.includes('#%')) {
        return `${(value * 100).toFixed(formatString.includes('.') ? formatString.split('.')[1].length : 0)}%`
      }
      if (formatString.includes(',')) {
        return value.toLocaleString()
      }
      // For more complex formatting, would use a library like numeral
      return value.toString()
    }

    switch (format) {
      case 'number':
        return value.toLocaleString(undefined, { maximumFractionDigits: 2 })
      case 'currency':
        return new Intl.NumberFormat('zh-CN', { style: 'currency', currency: 'CNY' }).format(value)
      case 'percent':
        return `${(value * 100).toFixed(1)}%`
      case 'bytes':
        return this.formatBytes(value)
      case 'duration':
        return this.formatDuration(value)
      case 'auto':
      default:
        return this.autoFormatValue(value)
    }
  }

  /**
   * Auto-format a value based on its magnitude
   */
  static autoFormatValue(value: number): string {
    if (value === 0) return '0'
    if (Math.abs(value) >= 1000000) {
      return `${(value / 1000000).toFixed(1)}M`
    }
    if (Math.abs(value) >= 1000) {
      return `${(value / 1000).toFixed(1)}K`
    }
    if (Number.isInteger(value)) {
      return value.toString()
    }
    return value.toFixed(2)
  }

  /**
   * Format bytes to human readable string
   */
  static formatBytes(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let size = bytes
    let unitIndex = 0

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024
      unitIndex++
    }

    return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
  }

  /**
   * Format seconds to human readable duration
   */
  static formatDuration(seconds: number): string {
    if (seconds < 60) {
      return `${seconds}s`
    }
    if (seconds < 3600) {
      const mins = Math.floor(seconds / 60)
      const secs = seconds % 60
      return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`
    }
    const hours = Math.floor(seconds / 3600)
    const mins = Math.floor((seconds % 3600) / 60)
    return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`
  }

  /**
   * Determine status based on thresholds
   */
  static getStatus(value: number, thresholds?: SingleValueMappingConfig['thresholds']): 'success' | 'warning' | 'error' | 'normal' {
    if (!thresholds) {
      return 'normal'
    }

    // Check error threshold
    if (thresholds.error) {
      const { operator, value: threshold } = thresholds.error
      if (this.evaluateThreshold(value, operator, threshold)) {
        return 'error'
      }
    }

    // Check warning threshold
    if (thresholds.warning) {
      const { operator, value: threshold } = thresholds.warning
      if (this.evaluateThreshold(value, operator, threshold)) {
        return 'warning'
      }
    }

    // Check success threshold
    if (thresholds.success) {
      const { operator, value: threshold } = thresholds.success
      if (this.evaluateThreshold(value, operator, threshold)) {
        return 'success'
      }
    }

    return 'normal'
  }

  /**
   * Evaluate threshold comparison
   */
  static evaluateThreshold(value: number, operator: string, threshold: number): boolean {
    switch (operator.toLowerCase()) {
      case '>':
      case 'gt':
        return value > threshold
      case '<':
      case 'lt':
        return value < threshold
      case '>=':
      case 'gte':
        return value >= threshold
      case '<=':
      case 'lte':
        return value <= threshold
      case '==':
      case 'eq':
        return value === threshold
      case '!=':
      case 'ne':
        return value !== threshold
      default:
        return false
    }
  }

  /**
   * Map a value to boolean for toggle controls
   */
  static mapToBoolean(value: unknown, config?: ControlMappingConfig): boolean {
    // Direct boolean
    if (typeof value === 'boolean') {
      return value
    }

    // Use mapping config
    if (config?.valueMapping?.onValues) {
      return config.valueMapping.onValues.some(v => v === value)
    }

    // Auto-detect truthy values
    if (typeof value === 'string') {
      return ['true', 'on', '1', 'yes', 'enabled', 'active'].includes(value.toLowerCase())
    }
    if (typeof value === 'number') {
      return value !== 0
    }

    return Boolean(value)
  }

  /**
   * Format value with unit
   */
  static formatWithUnit(value: number, config?: SingleValueMappingConfig): string {
    const formatted = this.formatValue(value, config)
    const unit = config?.unit
    const position = config?.unitPosition || 'suffix'

    if (!unit) {
      return formatted
    }

    return position === 'prefix' ? `${unit}${formatted}` : `${formatted}${unit}`
  }

  /**
   * Truncate label to max length
   */
  static truncateLabel(label: string, maxLength?: number): string {
    if (!maxLength || label.length <= maxLength) {
      return label
    }
    return `${label.slice(0, maxLength - 1)}…`
  }
}

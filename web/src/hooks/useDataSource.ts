/**
 * useDataSource — re-export barrel.
 * Public API: useDataSource, useNumberArrayDataSource, fetchHistoricalTelemetry, clearGlobalCacheIntervals.
 */
export { useDataSource, useNumberArrayDataSource } from './useDataSource/useDataSource'
export type { UseDataSourceResult } from './useDataSource/useDataSource'
export { fetchHistoricalTelemetry, clearGlobalCacheIntervals } from './useDataSource/fetch'

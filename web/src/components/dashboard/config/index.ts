/**
 * Dashboard Config Components
 *
 * Unified configuration system for dashboard components.
 * Provides abstracted, reusable configuration sections.
 */

// New unified data source configuration
export { UnifiedDataSourceConfig } from './UnifiedDataSourceConfig'
export type { UnifiedDataSourceConfigProps } from './UnifiedDataSourceConfig'

// Dual-mode source field (manual input + data source binding)
export { DualModeSourceField } from './DualModeSourceField'
export type { DualModeSourceFieldProps, DualModeInputType } from './DualModeSourceField'

export { ConfigSection } from './ConfigSection'
export { ConfigRenderer } from './ConfigRenderer'
export { ComponentConfigDialog } from './ComponentConfigDialog'
export type { ComponentConfigDialogProps } from './ComponentConfigDialog'

// Value Map Editor for LED Indicator
export { ValueMapEditor } from './ValueMapEditor'
export type { ValueStateMapping, LEDState } from './ValueMapEditor'

export type {
  ComponentConfigSchema,
  DataSourceSection,
  ValueSection,
  RangeSection,
  SizeSection,
  ColorSection,
  MultiColorSection,
  LabelSection,
  BooleanSection,
  SelectSection,
  TextSection,
  OrientationSection,
  AnimationSection,
  CustomSection,
} from './ComponentConfigBuilder'

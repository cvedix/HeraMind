/**
 * Device Type Dialogs - Modular entry point
 * This file exports all device type dialog components
 *
 * Components are organized as:
 * - types.ts: Shared TypeScript interfaces
 * - BasicInfoStep.tsx: Step 1 - Basic information
 * - DataDefinitionStep.tsx: Step 2 - Data/metrics definition
 * - CommandsStep.tsx: Step 3 - Commands definition
 * - ReviewStep.tsx: Step 4 - Review before save
 * - FinishStep.tsx: Step 5 - Completion screen
 * - AddDeviceTypeDialog.tsx: Main add/edit dialog
 * - ViewDeviceTypeDialog.tsx: View-only dialog
 * - EditDeviceTypeDialog.tsx: Edit mode wrapper
 */

// Re-export from original file during migration
export * from '../DeviceTypeDialogs'

// Re-export new modular components
export { BasicInfoStep } from './BasicInfoStep'

// Re-export types
export type {
  ValidationResult,
  FormErrors,
  Step,
  AddDeviceTypeDialogProps,
  ViewDeviceTypeDialogProps,
  EditDeviceTypeDialogProps,
} from './types'

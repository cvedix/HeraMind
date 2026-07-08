import type { DeviceType } from "@/types"

// Validation result type
export interface ValidationResult {
  valid: boolean
  errors?: string[]
  warnings?: string[]
  message: string
}

// Form errors type
export interface FormErrors {
  device_type?: string
  name?: string
  metrics?: Record<number, string>
  commands?: Record<number, string>
  [key: string]: string | Record<number, string> | undefined
}

// Step type
export type Step = 'basic' | 'data' | 'commands' | 'review' | 'finish'

// ============================================================================
// MAIN DIALOG PROPS
// ============================================================================

export interface AddDeviceTypeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onAdd: (definition: DeviceType) => Promise<boolean>
  onValidate: (definition: DeviceType) => Promise<ValidationResult>
  adding: boolean
  validating: boolean
  editDeviceType?: DeviceType | null
}

export interface ViewDeviceTypeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  deviceType: DeviceType | null
}

export interface EditDeviceTypeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  deviceType: DeviceType | null
  onEdit: (updated: DeviceType) => void
  editing: boolean
}

// ============================================================================
// STEP PROPS
// ============================================================================

export interface BasicInfoStepProps {
  data: DeviceType
  errors: FormErrors
  onDataChange: (data: DeviceType) => void
  onErrorsChange: (errors: FormErrors) => void
}

export interface DataDefinitionStepProps {
  data: DeviceType
  errors: FormErrors
  onDataChange: (data: DeviceType) => void
  onErrorsChange: (errors: FormErrors) => void
}

export interface CommandsStepProps {
  data: DeviceType
  errors: FormErrors
  onDataChange: (data: DeviceType) => void
  onErrorsChange: (errors: FormErrors) => void
}

export interface ReviewStepProps {
  data: DeviceType
  onEdit: () => void
}

export interface FinishStepProps {
  deviceType: string
  isEditMode: boolean
  onAddAnother: () => void
  onClose: () => void
}

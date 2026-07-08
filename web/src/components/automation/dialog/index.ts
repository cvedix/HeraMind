// Main dialog container
export {
  FullScreenDialog,
  FullScreenDialogHeader,
  FullScreenDialogContent,
  FullScreenDialogFooter,
  FullScreenDialogSidebar,
  FullScreenDialogMain,
} from './FullScreenDialog'
export type {
  FullScreenDialogProps,
  FullScreenDialogHeaderProps,
  FullScreenDialogContentProps,
  FullScreenDialogFooterProps,
  FullScreenDialogSidebarProps,
  FullScreenDialogMainProps,
} from './FullScreenDialog'

// Step progress
export {
  ProgressStepper,
  VerticalStepper,
  HorizontalStepper,
} from './ProgressStepper'
export type {
  Step,
  StepStatus,
  ProgressStepperProps,
  VerticalStepperProps,
  HorizontalStepperProps,
} from './ProgressStepper'

// Legacy exports (kept for backwards compatibility)
export { FullScreenHeader } from './FullScreenHeader'
export type { FullScreenHeaderProps } from './FullScreenHeader'

export { BaseInfoSection } from './BaseInfoSection'
export type { BaseInfoSectionProps, ScopeType, ScopeOption } from './BaseInfoSection'

export { ValidationBanner, ValidationBadge } from './ValidationBanner'
export type { ValidationBannerProps, ValidationBadgeProps } from './ValidationBanner'

// Re-export commonly used icons for convenience
export { X, Check, ArrowLeft, Save, Loader2, Play } from 'lucide-react'

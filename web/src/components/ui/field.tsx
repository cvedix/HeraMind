import * as React from "react"
import { cn } from "@/lib/utils"
import { Label } from "@/components/ui/label"
import { AlertCircle, CheckCircle2, AlertTriangle, Info } from "lucide-react"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

export interface FieldProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
}

export const Field = React.forwardRef<HTMLDivElement, FieldProps>(
  ({ className, children, ...props }, ref) => (
    <div ref={ref} className={cn("flex flex-col gap-1.5", className)} {...props}>
      {children}
    </div>
  )
)
Field.displayName = "Field"

export const FieldGroup = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex flex-col gap-4", className)}
    {...props}
  />
))
FieldGroup.displayName = "FieldGroup"

export const FieldLabel = React.forwardRef<
  HTMLLabelElement,
  React.LabelHTMLAttributes<HTMLLabelElement>
>(({ className, ...props }, ref) => (
  <Label
    ref={ref}
    className={cn("text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70", className)}
    {...props}
  />
))
FieldLabel.displayName = "FieldLabel"

export const FieldDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
FieldDescription.displayName = "FieldDescription"

export const FieldMessage = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    className={cn("text-sm font-medium text-error", className)}
    {...props}
  />
))
FieldMessage.displayName = "FieldMessage"

export const FieldSeparator = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, children, ...props }, ref) => (
  <div
    ref={ref}
    role="separator"
    className={cn("relative flex py-1 items-center", className)}
    {...props}
  >
    <div className="flex-grow border-t border-border"></div>
    {children ? (
      <>
        <span className="mx-4 text-xs text-muted-foreground uppercase">
          {children}
        </span>
        <div className="flex-grow border-t border-border"></div>
      </>
    ) : null}
  </div>
))
FieldSeparator.displayName = "FieldSeparator"

/**
 * FormField - Enhanced field with validation states
 *
 * Provides a consistent form field layout with:
 * - Label with required indicator
 * - Validation states (error, warning, success)
 * - Help text and tooltip support
 * - Proper accessibility attributes
 */

export type ValidationState = 'error' | 'warning' | 'success' | undefined

export interface FormFieldProps extends Omit<React.HTMLAttributes<HTMLDivElement>, 'title'> {
  /** Field label */
  label: string
  /** Whether the field is required */
  required?: boolean
  /** Field ID (used for label association) */
  id?: string
  /** Error message to display */
  error?: string
  /** Warning message to display */
  warning?: string
  /** Success message to display */
  success?: string
  /** Help text below the field */
  helpText?: string
  /** Tooltip content on hover */
  tooltip?: string
  /** Field content (input, select, etc.) */
  children: React.ReactNode
  /** Whether the field is in a horizontal layout */
  horizontal?: boolean
  /** Additional label class name */
  labelClassName?: string
  /** Size variant */
  size?: 'sm' | 'md' | 'lg'
}

const validationIcons = {
  error: AlertCircle,
  warning: AlertTriangle,
  success: CheckCircle2,
}

const validationColors = {
  error: 'text-error',
  warning: 'text-warning',
  success: 'text-success',
}

const validationBorderColors = {
  error: 'border-error focus-visible:ring-error',
  warning: 'border-warning focus-visible:ring-warning',
  success: 'border-success focus-visible:ring-success',
}

/**
 * Get the current validation state based on messages
 */
function getValidationState(
  error?: string,
  warning?: string,
  success?: string
): ValidationState {
  if (error) return 'error'
  if (warning) return 'warning'
  if (success) return 'success'
  return undefined
}

/**
 * FormField Component
 *
 * @example
 * // Basic usage
 * <FormField label="Name" required>
 *   <Input />
 * </FormField>
 *
 * // With validation
 * <FormField
 *   label="Email"
 *   required
 *   error={errors.email}
 *   helpText="We'll never share your email"
 * >
 *   <Input type="email" />
 * </FormField>
 *
 * // With tooltip
 * <FormField
 *   label="API Key"
 *   tooltip="Find your API key in settings"
 * >
 *   <Input type="password" />
 * </FormField>
 */
export const FormField = React.forwardRef<HTMLDivElement, FormFieldProps>(({
  label,
  required = false,
  id,
  error,
  warning,
  success,
  helpText,
  tooltip,
  children,
  horizontal = false,
  className,
  labelClassName,
  size = 'md',
  ...props
}, ref) => {
  const generatedId = React.useId()
  const fieldId = id || generatedId
  const validationState = getValidationState(error, warning, success)
  const ValidationIcon = validationState ? validationIcons[validationState] : null

  // Get validation message to display
  const validationMessage = error || warning || success

  // Size classes
  const sizeClasses = {
    sm: {
      label: 'text-xs',
      message: 'text-xs',
      gap: 'gap-1',
    },
    md: {
      label: 'text-sm',
      message: 'text-sm',
      gap: 'gap-1.5',
    },
    lg: {
      label: 'text-base',
      message: 'text-sm',
      gap: 'gap-2',
    },
  }

  const sizes = sizeClasses[size]

  return (
    <div
      ref={ref}
      className={cn(
        "flex flex-col",
        sizes.gap,
        horizontal && "flex-row items-start gap-4",
        className
      )}
      {...props}
    >
      {/* Label */}
      <div className={cn(
        "flex items-center gap-1.5",
        horizontal && "min-w-[120px] pt-0.5"
      )}>
        <Label
          htmlFor={fieldId}
          className={cn(
            "font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
            sizes.label,
            labelClassName
          )}
        >
          {label}
          {required && (
            <span className="text-error ml-0.5" aria-hidden="true">*</span>
          )}
        </Label>
        {tooltip && (
          <TooltipProvider delayDuration={300}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="h-3.5 w-3.5 text-muted-foreground cursor-help" />
              </TooltipTrigger>
              <TooltipContent side="top" className="max-w-xs">
                <p>{tooltip}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </div>

      {/* Input area */}
      <div className={cn("flex-1 flex flex-col", sizes.gap)}>
        {/* Clone child to add validation styling and id */}
        {React.Children.map(children, (child) => {
          if (!React.isValidElement(child)) return child

          // Add validation border classes and id to the child
          const childClassName = cn(
            (child.props as any).className,
            validationState && validationBorderColors[validationState]
          )

          return React.cloneElement(child as React.ReactElement<any>, {
            id: fieldId,
            className: childClassName,
            'aria-invalid': validationState === 'error' ? true : undefined,
            'aria-describedby': validationMessage || helpText
              ? `${fieldId}-description`
              : undefined,
          })
        })}

        {/* Validation message or help text */}
        {(validationMessage || helpText) && (
          <div
            id={`${fieldId}-description`}
            className={cn(
              "flex items-center gap-1.5",
              sizes.message,
              validationState ? validationColors[validationState] : "text-muted-foreground"
            )}
          >
            {ValidationIcon && <ValidationIcon className="h-3.5 w-3.5 shrink-0" />}
            <span>{validationMessage || helpText}</span>
          </div>
        )}
      </div>
    </div>
  )
})
FormField.displayName = "FormField"

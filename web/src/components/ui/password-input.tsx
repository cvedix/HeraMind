import * as React from "react"
import { useTranslation } from "react-i18next"
import { Eye, EyeOff } from "lucide-react"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"

/**
 * Password input with show/hide toggle.
 *
 * Wraps the base `Input` primitive. When showing, switches to type="text"
 * so the IME-safe password path no longer applies (intentional — the user
 * is verifying what they typed, not composing).
 *
 * Callers can pass `startAdornment`/`endAdornment` (in addition to the eye)
 * via className padding (`pl-9`, `pr-10` etc.) just like the login page.
 * The toggle button reserves `pr-10` by default; pass `passwordClassName`
 * to override padding when extra end adornments are needed.
 */
export interface PasswordInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type"> {
  /** Override the toggle button's position classes (rare). */
  toggleClassName?: string
}

export const PasswordInput = React.forwardRef<HTMLInputElement, PasswordInputProps>(
  ({ className, toggleClassName, ...props }, ref) => {
    const { t } = useTranslation("auth")
    const [show, setShow] = React.useState(false)
    return (
      <div className="relative">
        <Input
          ref={ref}
          type={show ? "text" : "password"}
          className={cn("pr-10", className)}
          {...props}
        />
        <button
          type="button"
          onClick={() => setShow((s) => !s)}
          className={cn(
            "absolute right-2 top-1/2 -translate-y-1/2 h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors",
            toggleClassName
          )}
          aria-label={show ? t("hidePassword") : t("showPassword")}
          title={show ? t("hidePassword") : t("showPassword")}
          tabIndex={-1}
        >
          {show ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
        </button>
      </div>
    )
  }
)
PasswordInput.displayName = "PasswordInput"

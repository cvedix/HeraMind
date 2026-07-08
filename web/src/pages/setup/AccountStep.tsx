/**
 * AccountStep - Create admin account with optional timezone selection
 *
 * Combined step 1 of setup: account creation + timezone (auto-detected)
 */
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Lock, User, Shield, Check, ArrowRight, Globe } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { PasswordInput } from '@/components/ui/password-input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { cn } from '@/lib/utils'
import { textNano } from '@/design-system/tokens/typography'
import { SetupBackground } from './SetupBackground'
import { SetupHeader } from './SetupHeader'
import { getLocalizedTimezones, getBrowserTimezone, COMMON_TIMEZONE_IDS } from '@/lib/time/format'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

// Mailchimp subscription function
function mcSubscribe(email: string, username?: string): Promise<{ result: string; msg: string }> {
  const base = "https://camthink.us2.list-manage.com/subscribe/post-json"
  const cb = "mc_cb_" + Date.now() + "_" + Math.random().toString(16).slice(2)

  const params = new URLSearchParams({
    u: "4ecc400d85930178fb49aa9de",
    id: "466fcc3b55",
    f_id: "00e60ae1f0",
    EMAIL: email,
    b_4ecc400d85930178fb49aa9de_466fcc3b55: "",
    c: cb,
    _: Date.now().toString(),
  })

  if (username) params.append("FNAME", username)

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup()
      reject(new Error("Mailchimp JSONP timeout"))
    }, 8000)

    function cleanup() {
      clearTimeout(timeout)
      try { delete (window as any)[cb] } catch (_) { /* ignore */ }
      if (script && script.parentNode) script.parentNode.removeChild(script)
    }

    ;(window as any)[cb] = function (data: { result: string; msg: string }) {
      cleanup()
      resolve(data)
    }

    const script = document.createElement("script")
    script.src = base + "?" + params.toString()
    script.onerror = () => { cleanup(); reject(new Error("Mailchimp JSONP network error")) }
    document.head.appendChild(script)
  })
}

// Error translation helper
function translateError(error: string, t: (key: string, params?: Record<string, unknown>) => string): string {
  const lowerError = error.toLowerCase()
  if (lowerError.includes("password must be at least")) return t('minPasswordLength', { ns: 'validation' })
  if (lowerError.includes("username must be at least")) return t('minUsernameLength', { ns: 'validation' })
  if (lowerError.includes("password must contain")) return t('passwordComplexity')
  if (lowerError.includes("setup already completed")) return t('setupAlreadyCompleted')
  return error || t("setupFailed")
}

interface AccountStepProps {
  getApiUrl: (path: string) => string
  onAccountCreated: (username: string, password: string, token: string) => void
}

export function AccountStep({ getApiUrl, onAccountCreated }: AccountStepProps) {
  const { t } = useTranslation(['common', 'auth', 'setup', 'validation'])
  const timezoneOptions = getLocalizedTimezones(t)

  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState("")

  // Account form
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [email, setEmail] = useState("")
  const [subscribeToNewsletter, setSubscribeToNewsletter] = useState(false)

  // Timezone (auto-detect browser timezone, fallback to Asia/Shanghai)
  const browserTz = getBrowserTimezone()
  const defaultTz = COMMON_TIMEZONE_IDS.includes(browserTz as any) ? browserTz : "Asia/Shanghai"
  const [selectedTimezone, setSelectedTimezone] = useState(defaultTz)

  // Password validation
  const getPasswordErrors = (pwd: string): string[] => {
    const errors: string[] = []
    if (pwd.length < 8) errors.push(t('minPasswordLength', { ns: 'validation' }))
    if (!pwd.match(/[a-zA-Z]/)) errors.push(t('passwordNeedsLetter', { ns: 'validation' }))
    if (!pwd.match(/[0-9]/)) errors.push(t('passwordNeedsNumber', { ns: 'validation' }))
    return errors
  }
  const passwordErrors = getPasswordErrors(password)

  const formatTimeInTimezone = (tz: string) => {
    try {
      return new Intl.DateTimeFormat('zh-CN', {
        hour: '2-digit', minute: '2-digit', second: '2-digit',
        timeZone: tz, hour12: false,
      }).format(new Date())
    } catch {
      return '--:--:--'
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError("")

    if (password !== confirmPassword) {
      setError(t('passwordsDoNotMatch', { ns: 'validation' }))
      return
    }
    const pwdErrors = getPasswordErrors(password)
    if (pwdErrors.length > 0) {
      setError(pwdErrors[0])
      return
    }

    setIsLoading(true)
    try {
      // Create admin account
      const response = await fetch(getApiUrl('/setup/initialize'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password, email: email || undefined }),
      })

      const data = await response.json()
      if (!response.ok) {
        throw new Error(data.message || data.error || 'Failed to create admin account')
      }

      localStorage.setItem('heramind_token', data.token)
      localStorage.setItem('heramind_user', JSON.stringify(data.user))

      // Save timezone
      try {
        await fetch(getApiUrl('/settings/timezone'), {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${data.token}`,
          },
          body: JSON.stringify({ timezone: selectedTimezone }),
        })
      } catch (tzError) {
        console.warn('Failed to save timezone, continuing:', tzError)
      }

      // Newsletter subscription (non-blocking)
      if (subscribeToNewsletter && email?.trim()) {
        mcSubscribe(email, username).catch(() => {})
      }

      onAccountCreated(username, password, data.token)
    } catch (err) {
      setError(translateError(err instanceof Error ? err.message : String(err), t))
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="h-screen flex flex-col bg-background">
      <SetupBackground />

      <div className="absolute top-0 left-0 right-0 z-20">
        <SetupHeader />
      </div>

      <main className="relative z-10 flex-1 overflow-y-auto px-4 py-8 sm:px-6 sm:py-12 flex items-center justify-center">
        <div className="w-full max-w-md my-auto">
          <div className="bg-bg-50 backdrop-blur-md rounded-lg p-6 sm:p-8">
            {/* Icon */}
            <div className="flex justify-center mb-5">
              <div className="flex size-14 items-center justify-center rounded-full bg-muted text-primary">
                <User className="size-6" />
              </div>
            </div>

            {/* Title */}
            <h2 className="text-2xl font-semibold mb-2 text-center">{t('setup:createAccount')}</h2>
            <p className="text-muted-foreground text-center mb-6 text-sm">{t('setup:accountDescription')}</p>

            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              {/* Username */}
              <div>
                <Label htmlFor="username" className="text-sm">{t('auth:username')}</Label>
                <div className="relative mt-1.5">
                  <User className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
                  <Input
                    id="username"
                    type="text"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    placeholder={t('setup:usernamePlaceholder')}
                    autoComplete="username"
                    required
                    minLength={3}
                    className="pl-9 h-10 bg-bg-70 border-border"
                  />
                </div>
              </div>

              {/* Email */}
              <div>
                <Label htmlFor="email" className="text-sm">{t('setup:email')} <span className="text-muted-foreground">({t('optional')})</span></Label>
                <Input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder={t('setup:emailPlaceholder')}
                  autoComplete="email"
                  className="h-10 bg-bg-70 border-border mt-1.5"
                />
                {email?.trim() && (
                  <div className="flex items-center gap-2 mt-2">
                    <Checkbox
                      id="subscribe"
                      checked={subscribeToNewsletter}
                      onCheckedChange={(checked) => setSubscribeToNewsletter(!!checked)}
                    />
                    <label htmlFor="subscribe" className="text-xs text-muted-foreground cursor-pointer leading-tight">
                      {t('setup:subscribeNewsletter')}
                    </label>
                  </div>
                )}
              </div>

              {/* Password */}
              <div>
                <Label htmlFor="password" className="text-sm">{t('auth:password')}</Label>
                <div className="relative mt-1.5">
                  <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
                  <PasswordInput
                    id="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder={t('setup:passwordPlaceholder')}
                    autoComplete="new-password"
                    required
                    minLength={8}
                    className="pl-9 h-10 bg-bg-70 border-border"
                  />
                </div>
              </div>

              {/* Confirm Password */}
              <div>
                <Label htmlFor="confirmPassword" className="text-sm">{t('setup:confirmPassword')}</Label>
                <div className="relative mt-1.5">
                  <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
                  <PasswordInput
                    id="confirmPassword"
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder={t('setup:confirmPasswordPlaceholder')}
                    autoComplete="new-password"
                    required
                    className="pl-9 h-10 bg-bg-70 border-border"
                  />
                </div>
              </div>

              {/* Password Strength */}
              {password && (
                <div className="space-y-1.5">
                  <div className="text-xs text-muted-foreground">{t('setup:passwordStrength')}</div>
                  <div className="flex gap-1">
                    {passwordErrors.length === 0 ? (
                      <>{[1,2,3,4].map(i => <div key={i} className="h-1 flex-1 rounded-full bg-success" />)}</>
                    ) : password.length >= 8 ? (
                      <>
                        <div className="h-1 flex-1 rounded-full bg-success" />
                        <div className="h-1 flex-1 rounded-full bg-warning" />
                        <div className="h-1 flex-1 rounded-full bg-border" />
                        <div className="h-1 flex-1 rounded-full bg-border" />
                      </>
                    ) : (
                      <>
                        <div className="h-1 flex-1 rounded-full bg-error" />
                        <div className="h-1 flex-1 rounded-full bg-border" />
                        <div className="h-1 flex-1 rounded-full bg-border" />
                        <div className="h-1 flex-1 rounded-full bg-border" />
                      </>
                    )}
                  </div>
                  {passwordErrors.length > 0 && (
                    <div className="text-xs text-error">{passwordErrors[0]}</div>
                  )}
                </div>
              )}

              {/* Timezone Section */}
              <div className="pt-2 border-t border-border">
                <div className="flex items-center gap-2 mb-2">
                  <Globe className="h-4 w-4 text-muted-foreground" />
                  <Label className="text-sm font-medium">{t('setup:selectTimezone')}</Label>
                  <span className={cn(textNano, "text-muted-foreground ml-auto")}>
                    {formatTimeInTimezone(selectedTimezone)}
                  </span>
                </div>
                <Select value={selectedTimezone} onValueChange={setSelectedTimezone}>
                  <SelectTrigger className="h-10 bg-bg-70 border-border">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="max-h-60">
                    {timezoneOptions.map((tz) => (
                      <SelectItem key={tz.id} value={tz.id}>{tz.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Error */}
              {error && (
                <div className="flex items-start gap-2 text-sm text-error bg-muted rounded-md p-3">
                  <Shield className="h-4 w-4 mt-0.5 flex-shrink-0" />
                  <span>{error}</span>
                </div>
              )}

              {/* Submit */}
              <Button
                type="submit"
                disabled={isLoading || !username || !password || !confirmPassword || passwordErrors.length > 0}
                className="h-10 w-full mt-1"
                size="default"
              >
                {isLoading ? t('setup:creating') : t('setup:getStarted')}
                {!isLoading && <ArrowRight className="ml-2 h-4 w-4" />}
              </Button>
            </form>
          </div>
        </div>
      </main>
    </div>
  )
}

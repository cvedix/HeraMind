/**
 * CompleteStep - Setup completion screen with quick-start guide
 */
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { Check, MessageSquare, Settings, ChevronRight, Cpu, Zap } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { SetupBackground } from './SetupBackground'
import { SetupHeader } from './SetupHeader'

interface CompleteStepProps {
  username: string
  onComplete: () => void
}

export function CompleteStep({ username, onComplete }: CompleteStepProps) {
  const { t } = useTranslation(['common', 'setup'])
  const navigate = useNavigate()

  const quickActions = [
    {
      icon: MessageSquare,
      title: t('setup:quickChat'),
      description: t('setup:quickChatDesc'),
      action: () => { onComplete() },
    },
    {
      icon: Cpu,
      title: t('setup:quickLlm'),
      description: t('setup:quickLlmDesc'),
      action: () => { onComplete(); setTimeout(() => navigate('/settings'), 100) },
    },
    {
      icon: Zap,
      title: t('setup:quickExplore'),
      description: t('setup:quickExploreDesc'),
      action: () => { onComplete() },
    },
  ]

  return (
    <div className="min-h-screen flex flex-col bg-background overflow-hidden">
      <SetupBackground />

      <div className="absolute top-0 left-0 right-0 z-20">
        <SetupHeader />
      </div>

      <main className="relative z-10 flex-1 px-4 py-6 sm:px-6 sm:py-12 flex items-center justify-center">
        <div className="w-full max-w-md text-center">
          <div className="bg-bg-50 backdrop-blur-md rounded-lg p-6 sm:p-8">
            {/* Success Icon */}
            <div className="flex justify-center mb-5">
              <div className="flex size-14 items-center justify-center rounded-full bg-success-light text-success">
                <Check className="size-7" />
              </div>
            </div>

            {/* Success Message */}
            <h2 className="text-2xl font-semibold mb-2">{t('setup:completeTitle')}</h2>
            <p className="text-muted-foreground mb-2 text-sm">{t('setup:completeMessage')}</p>

            {/* Account Info */}
            <div className="inline-flex items-center gap-2 bg-muted-30 rounded-full px-4 py-1.5 mb-6">
              <span className="text-xs text-muted-foreground">{t('setup:accountCreated')}:</span>
              <span className="text-sm font-mono font-medium">{username}</span>
            </div>

            {/* Quick Start Guide */}
            <div className="text-left space-y-2 mb-6">
              <div className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">
                {t('setup:nextSteps')}
              </div>
              {quickActions.map((action, i) => (
                <button
                  key={i}
                  onClick={action.action}
                  className="w-full flex items-center gap-3 p-3 rounded-lg border border-border hover:bg-muted-50 transition-colors text-left group"
                >
                  <div className="flex size-9 items-center justify-center rounded-lg bg-muted text-primary shrink-0">
                    <action.icon className="size-4" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium">{action.title}</div>
                    <div className="text-xs text-muted-foreground">{action.description}</div>
                  </div>
                  <ChevronRight className="size-4 text-muted-foreground group-hover:text-foreground transition-colors shrink-0" />
                </button>
              ))}
            </div>

            {/* Main CTA */}
            <Button
              onClick={onComplete}
              className="w-full h-10"
              size="default"
            >
              {t('setup:goToDashboard')}
              <ChevronRight className="ml-2 h-4 w-4" />
            </Button>
          </div>
        </div>
      </main>
    </div>
  )
}

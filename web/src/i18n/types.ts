declare module 'react-i18next' {
  interface CustomTypeOptions {
    resources: {
      common: typeof import('./locales/en/common.json')
      devices: typeof import('./locales/en/devices.json')
      alerts: typeof import('./locales/en/alerts.json')
      automation: typeof import('./locales/en/automation.json')
      plugins: typeof import('./locales/en/plugins.json')
      extensions: typeof import('./locales/en/extensions.json')
      settings: typeof import('./locales/en/settings.json')
      auth: typeof import('./locales/en/auth.json')
      validation: typeof import('./locales/en/validation.json')
      dashboard: typeof import('./locales/en/dashboard.json')
      agents: typeof import('./locales/en/agents.json')
      dashboardComponents: typeof import('./locales/en/dashboard-components.json')
      chat: typeof import('./locales/en/chat.json')
      setup: typeof import('./locales/en/setup.json')
      data: typeof import('./locales/en/data.json')
      instances: typeof import('./locales/en/instances.json')
      ui: typeof import('./locales/en/ui.json')
    }
    defaultNS: 'common'
    returnNull: false
  }
}

/// <reference types="vite/client" />

// Tauri global type declaration
declare global {
  interface Window {
    __TAURI__?: unknown
  }
}

interface ImportMetaEnv {
  readonly DEV: boolean
  readonly MODE: string
  readonly BASE_URL: string
  readonly PROD: boolean
  readonly SSR: boolean
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

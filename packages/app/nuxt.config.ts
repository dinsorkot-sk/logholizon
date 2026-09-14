// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  modules: ['@nuxt/ui', './modules/desktop'],
  css: ['~/assets/css/main.css'],
  // Desktop (Tauri) builds a static SPA: no Nitro server, the in-process
  // Rust sidecar serves /v1 directly. Enabled with LOGHOLIZON_DESKTOP=1.
  // Web builds keep the default SSR + Nitro gateway behavior.
  ssr: process.env.LOGHOLIZON_DESKTOP === '1' ? false : true,
  runtimeConfig: {
    coreUrl: process.env.CORE_URL || 'http://127.0.0.1:8787',
    public: {
      desktop: process.env.LOGHOLIZON_DESKTOP === '1'
    }
  },
  devtools: { enabled: process.env.NODE_ENV !== 'production' }
})

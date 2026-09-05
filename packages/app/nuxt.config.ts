// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  runtimeConfig: {
    coreUrl: process.env.CORE_URL || 'http://127.0.0.1:8787'
  },
  devtools: { enabled: process.env.NODE_ENV !== 'production' }
})

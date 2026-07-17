import { defineStore } from 'pinia'
import { ref } from 'vue'

export type AppLocale = 'zh-CN' | 'en-US'
const SETTINGS_STORAGE_KEY = 'modlab_settings'

const isAppLocale = (value: unknown): value is AppLocale => value === 'zh-CN' || value === 'en-US'

const loadLocale = (): AppLocale => {
  if (typeof window === 'undefined') return 'zh-CN'

  try {
    const saved = JSON.parse(window.localStorage.getItem(SETTINGS_STORAGE_KEY) || 'null') as { locale?: unknown } | null
    return isAppLocale(saved?.locale) ? saved.locale : 'zh-CN'
  } catch {
    return 'zh-CN'
  }
}

const persistLocale = (locale: AppLocale) => {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify({ locale }))
}

export const useSettingsStore = defineStore('settings', () => {
  const locale = ref<AppLocale>(loadLocale())

  const setLocale = (nextLocale: AppLocale) => {
    locale.value = nextLocale
    persistLocale(nextLocale)
  }

  return { locale, setLocale }
})

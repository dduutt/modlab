import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSettingsStore } from './settingsStore'

const storage = new Map<string, string>()

const localStorageMock: Storage = {
  getItem: (key) => storage.get(key) ?? null,
  setItem: (key, value) => storage.set(key, value),
  removeItem: (key) => storage.delete(key),
  clear: () => storage.clear(),
  key: (index) => Array.from(storage.keys())[index] ?? null,
  get length() {
    return storage.size
  },
}

const createConfiguredPinia = () => {
  const pinia = createPinia()
  setActivePinia(pinia)
  return pinia
}

describe('settingsStore', () => {
  beforeEach(() => {
    storage.clear()
    vi.stubGlobal('localStorage', localStorageMock)
    vi.stubGlobal('window', { localStorage: localStorageMock })
  })

  it('defaults to Chinese', () => {
    createConfiguredPinia()

    const store = useSettingsStore()

    expect(store.locale).toBe('zh-CN')
  })

  it('persists a selected language and restores it in a new Pinia instance', () => {
    createConfiguredPinia()
    useSettingsStore().setLocale('en-US')

    createConfiguredPinia()
    const restoredStore = useSettingsStore()

    expect(restoredStore.locale).toBe('en-US')
  })

  it('falls back to Chinese for an unsupported persisted language', () => {
    storage.set('modlab_settings', JSON.stringify({ locale: 'ja-JP' }))
    createConfiguredPinia()

    const store = useSettingsStore()

    expect(store.locale).toBe('zh-CN')
  })
})

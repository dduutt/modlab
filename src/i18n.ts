import { createI18n } from 'vue-i18n';
import en from './locales/en';
import zh from './locales/zh';

export const i18n = createI18n({
  legacy: false, // Set to false to use Composition API
  locale: 'en', // Default locale
  fallbackLocale: 'en',
  messages: {
    en,
    zh
  }
});

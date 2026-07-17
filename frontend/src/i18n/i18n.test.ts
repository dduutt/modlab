import { describe, expect, it } from 'vitest'
import i18n from './index'

describe('application translations', () => {
  it('renders dynamic status messages in Chinese and English', () => {
    i18n.global.locale.value = 'zh-CN'
    expect(i18n.global.t('status.autoReadSuccess', { name: '主站 1', count: 2 })).toContain('主站 1')

    i18n.global.locale.value = 'en-US'
    expect(i18n.global.t('status.autoReadSuccess', { name: 'Master 1', count: 2 })).toContain('Master 1')
    expect(i18n.global.t('app.copyright')).toContain('dote27@163.com')
  })
})

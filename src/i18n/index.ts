import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import en from './locales/en'

export type MessageSchema = typeof zhCN

// 获取存储的语言偏好或使用系统语言
function getDefaultLocale(): string {
  const savedLocale = localStorage.getItem('locale')
  if (savedLocale) {
    return savedLocale
  }
  
  // 检测操作系统语言设置
  const systemLang = navigator.language.toLowerCase()
  if (systemLang.startsWith('zh')) {
    return 'zh-CN'
  }
  return 'en'
}

const i18n = createI18n<[MessageSchema], 'zh-CN' | 'en'>({
  legacy: false, // 使用 Composition API 模式
  locale: getDefaultLocale(),
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    'en': en
  }
})

export default i18n

// 切换语言的辅助函数
export function setLocale(locale: 'zh-CN' | 'en') {
  ;(i18n.global.locale as any).value = locale
  localStorage.setItem('locale', locale)
  document.documentElement.lang = locale === 'zh-CN' ? 'zh-CN' : 'en'
}

// 获取当前语言
export function getLocale(): 'zh-CN' | 'en' {
  return (i18n.global.locale as any).value as 'zh-CN' | 'en'
}

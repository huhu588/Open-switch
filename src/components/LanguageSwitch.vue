<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, getLocale } from '@/i18n'

const { t } = useI18n()

const currentLocale = computed(() => getLocale())

function toggleLanguage() {
  const newLocale = currentLocale.value === 'zh-CN' ? 'en' : 'zh-CN'
  setLocale(newLocale)
}
</script>

<template>
  <button
    @click="toggleLanguage"
    class="flex w-full items-center justify-between rounded-md border border-border bg-background/50 px-3 py-2 text-xs font-medium transition-all hover:border-accent/40 hover:bg-surface-hover active:scale-[0.98]"
    :title="t('language.switch')"
  >
    <div class="flex items-center gap-2">
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="2" y1="12" x2="22" y2="12"></line>
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
      </svg>
      <span class="text-muted-foreground group-hover:text-primary transition-colors">
        {{ currentLocale === 'zh-CN' ? t('language.zh') : t('language.en') }}
      </span>
    </div>
    <div class="flex items-center gap-1 text-[10px] text-muted-foreground font-mono">
      <span :class="{ 'text-accent font-medium': currentLocale === 'zh-CN' }">中</span>
      <span>/</span>
      <span :class="{ 'text-accent font-medium': currentLocale === 'en' }">EN</span>
    </div>
  </button>
</template>

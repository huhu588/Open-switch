<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ModelItem } from '@/stores/providers'

const { t } = useI18n()

interface Props {
  models: ModelItem[]
  selected: string | null
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
})

const emit = defineEmits<{
  select: [id: string]
  add: []
  delete: [id: string]
  fetch: []
}>()

function formatTokens(count: number | null): string {
  if (count === null) return '-'
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`
  return String(count)
}
</script>

<template>
  <div class="flex h-full flex-col rounded-lg border border-border bg-surface/30 backdrop-blur-sm overflow-hidden shadow-sm transition-opacity duration-300"
       :class="{ 'opacity-60 grayscale pointer-events-none': disabled }">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-surface/50">
      <h3 class="font-bold text-xs uppercase tracking-wider text-muted-foreground">{{ t('model.title') }}</h3>
      <span class="text-[10px] font-mono text-muted-foreground bg-surface border border-border px-1.5 py-0.5 rounded shadow-sm">
        {{ models.length }}
      </span>
    </div>

    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-border/50 bg-background/50 backdrop-blur-sm">
      <button
        @click="emit('add')"
        :disabled="disabled"
        class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm active:scale-95"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
        <span>{{ t('model.addModel') }}</span>
      </button>
      <button
        @click="emit('fetch')"
        :disabled="disabled"
        class="flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md border border-border bg-surface hover:bg-surface-hover hover:border-accent/40 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 group"
        title="Fetch models from provider"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="group-hover:text-accent transition-colors"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
        <span>{{ t('model.sync') }}</span>
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 overflow-auto p-2 scrollbar-thin">
      <div v-if="disabled" class="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
        <p class="text-xs">{{ t('model.selectProviderFirst') }}</p>
      </div>
      <div v-else-if="models.length === 0" class="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="opacity-20"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line></svg>
        <p class="text-xs">{{ t('model.noModels') }}</p>
      </div>
      <ul v-else class="space-y-1">
        <li
          v-for="model in models"
          :key="model.id"
          @click="emit('select', model.id)"
          class="group relative flex cursor-pointer flex-col gap-1 rounded-md border px-3 py-2.5 transition-all duration-200"
          :class="[
            model.id === selected
              ? 'bg-accent/5 border-accent/40 shadow-[0_0_10px_-4px_rgba(245,158,11,0.2)]'
              : 'bg-transparent border-transparent hover:bg-surface-hover hover:border-border'
          ]"
        >
          <!-- Selection Marker -->
          <div v-if="model.id === selected" class="absolute left-0 top-2 bottom-2 w-0.5 bg-accent rounded-r-full"></div>

          <div class="flex items-center justify-between">
            <span class="font-mono text-xs font-medium truncate" :class="{ 'text-accent': model.id === selected }">
              {{ model.id }}
            </span>
            <button
              @click.stop="emit('delete', model.id)"
              class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive opacity-0 group-hover:opacity-100 transition-all"
              :title="t('model.delete')"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
            </button>
          </div>
          
          <div v-if="model.context_limit || model.output_limit" class="flex items-center gap-3 text-[10px] text-muted-foreground font-mono">
            <span v-if="model.context_limit" class="flex items-center gap-1">
              CTX: {{ formatTokens(model.context_limit) }}
            </span>
            <span v-if="model.output_limit" class="flex items-center gap-1">
              OUT: {{ formatTokens(model.output_limit) }}
            </span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

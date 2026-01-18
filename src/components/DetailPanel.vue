<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ProviderItem, ModelItem } from '@/stores/providers'

const { t } = useI18n()

interface Props {
  provider?: ProviderItem
  model?: ModelItem
}

const props = defineProps<Props>()

function formatTokens(count: number | null): string {
  if (count === null) return '-'
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`
  return String(count)
}
</script>

<template>
  <div class="h-full rounded-lg border border-border bg-surface/30 backdrop-blur-sm overflow-hidden flex flex-col transition-all duration-300 shadow-sm">
    <!-- Header -->
    <div class="px-5 py-4 border-b border-border bg-surface/50">
      <h3 class="font-bold text-sm text-primary tracking-tight">{{ t('detail.title') }}</h3>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto p-6 scrollbar-thin">
      <div v-if="!provider" class="flex h-full flex-col items-center justify-center gap-4 text-muted-foreground opacity-50">
        <div class="p-4 rounded-full bg-surface border border-border">
          <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line></svg>
        </div>
        <p class="text-sm font-medium">{{ t('detail.selectProvider') }}</p>
      </div>

      <div v-else class="space-y-8 animate-fade-in">
        <!-- Provider Info -->
        <section class="space-y-4">
          <div class="flex items-center gap-2 pb-2 border-b border-border/50">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
            <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              {{ t('detail.providerSpec') }}
            </h4>
          </div>
          
          <div class="grid grid-cols-[100px_1fr] gap-y-4 gap-x-4 text-sm">
            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.name') }}</span>
            <span class="font-medium text-primary">{{ provider.name }}</span>

            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.endpoint') }}</span>
            <div class="flex items-center gap-2">
               <span class="font-mono text-xs text-accent bg-accent/5 px-2 py-1 rounded border border-accent/20 break-all select-all">
                 {{ provider.base_url }}
               </span>
            </div>

            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.models') }}</span>
            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-secondary text-secondary-foreground border border-border w-fit">
              {{ provider.model_count }} {{ t('detail.available') }}
            </span>

            <template v-if="provider.description">
              <span class="text-muted-foreground font-mono text-xs">{{ t('detail.desc') }}</span>
              <span class="text-muted-foreground">{{ provider.description }}</span>
            </template>
          </div>
        </section>

        <!-- Model Info -->
        <section v-if="model" class="space-y-4 animate-slide-up">
          <div class="flex items-center gap-2 pb-2 border-b border-border/50">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
            <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              {{ t('detail.selectedModel') }}
            </h4>
          </div>
          
          <div class="grid grid-cols-[100px_1fr] gap-y-4 gap-x-4 text-sm">
            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.id') }}</span>
            <span class="font-mono text-xs text-primary select-all bg-surface border border-border px-1.5 py-0.5 rounded w-fit">{{ model.id }}</span>

            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.name') }}</span>
            <span class="text-primary">{{ model.name }}</span>

            <template v-if="model.context_limit">
              <span class="text-muted-foreground font-mono text-xs">{{ t('detail.context') }}</span>
              <span class="font-mono text-xs">{{ formatTokens(model.context_limit) }} {{ t('detail.tokens') }}</span>
            </template>

            <template v-if="model.output_limit">
              <span class="text-muted-foreground font-mono text-xs">{{ t('detail.output') }}</span>
              <span class="font-mono text-xs">{{ formatTokens(model.output_limit) }} {{ t('detail.tokens') }}</span>
            </template>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ProviderItem } from '@/stores/providers'

const { t } = useI18n()

interface Props {
  providers: ProviderItem[]
  selected: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [name: string]
  add: []
  edit: [name: string]
  delete: [name: string]
  apply: []
}>()
</script>

<template>
  <div class="flex h-full flex-col rounded-lg border border-border bg-surface/30 backdrop-blur-sm overflow-hidden shadow-sm">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-surface/50">
      <h3 class="font-bold text-xs uppercase tracking-wider text-muted-foreground">{{ t('provider.title') }}</h3>
      <span class="text-[10px] font-mono text-muted-foreground bg-surface border border-border px-1.5 py-0.5 rounded shadow-sm">
        {{ providers.length }}
      </span>
    </div>

    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-border/50 bg-background/50 backdrop-blur-sm">
      <button
        @click="emit('add')"
        class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-all shadow-sm active:scale-95"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
        <span>{{ t('provider.new') }}</span>
      </button>
      <button
        @click="emit('apply')"
        :disabled="providers.length === 0"
        class="flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md border border-border bg-surface hover:bg-surface-hover hover:border-accent/40 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 group"
        :title="`应用全部 ${providers.length} 个服务商配置`"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="group-hover:text-accent transition-colors"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
        <span>{{ t('provider.apply') }}</span>
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 overflow-auto p-2 scrollbar-thin">
      <div v-if="providers.length === 0" class="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="opacity-20"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
        <p class="text-xs">{{ t('provider.noProviders') }}</p>
      </div>
      <ul v-else class="space-y-1">
        <li
          v-for="provider in providers"
          :key="provider.name"
          @click="emit('select', provider.name)"
          class="group relative flex cursor-pointer flex-col gap-1 rounded-md border px-3 py-2.5 transition-all duration-200"
          :class="[
            provider.name === selected
              ? 'bg-accent/5 border-accent/40 shadow-[0_0_10px_-4px_rgba(245,158,11,0.2)]'
              : 'bg-transparent border-transparent hover:bg-surface-hover hover:border-border'
          ]"
        >
          <!-- Selection Marker -->
          <div v-if="provider.name === selected" class="absolute left-0 top-2 bottom-2 w-0.5 bg-accent rounded-r-full"></div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm tracking-tight transition-colors" :class="{ 'text-accent': provider.name === selected }">
                {{ provider.name }}
              </span>
            </div>
            
            <!-- Actions -->
            <div class="flex items-center gap-1 opacity-0 transition-opacity duration-200 group-hover:opacity-100">
              <button
                @click.stop="emit('edit', provider.name)"
                class="rounded p-1 text-muted-foreground hover:bg-background hover:text-primary transition-colors"
                :title="t('provider.edit')"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path></svg>
              </button>
              <button
                @click.stop="emit('delete', provider.name)"
                class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
                :title="t('provider.delete')"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
              </button>
            </div>
          </div>
          
          <div class="flex items-center gap-2 text-[10px] text-muted-foreground font-mono">
             <span class="flex items-center gap-1">
               <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-50"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
               {{ provider.model_count }} {{ t('provider.models') }}
             </span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

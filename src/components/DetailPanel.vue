<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ProviderItem, ModelItem } from '@/stores/providers'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface Props {
  provider?: ProviderItem
  model?: ModelItem
}

const props = defineProps<Props>()

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
          <SvgIcon name="monitor" :size="32" />
        </div>
        <p class="text-sm font-medium">{{ t('detail.selectProvider') }}</p>
      </div>

      <div v-else class="space-y-8 animate-fade-in">
        <!-- Provider Info -->
        <section class="space-y-4">
          <div class="flex items-center gap-2 pb-2 border-b border-border/50">
            <SvgIcon name="cube" :size="16" class="text-accent" />
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
            <SvgIcon name="server" :size="16" class="text-accent" />
            <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              {{ t('detail.selectedModel') }}
            </h4>
          </div>
          
          <div class="grid grid-cols-[100px_1fr] gap-y-4 gap-x-4 text-sm">
            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.id') }}</span>
            <span class="font-mono text-xs text-primary select-all bg-surface border border-border px-1.5 py-0.5 rounded w-fit">{{ model.id }}</span>

            <span class="text-muted-foreground font-mono text-xs">{{ t('detail.name') }}</span>
            <span class="text-primary">{{ model.name }}</span>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

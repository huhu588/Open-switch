<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ProviderItem } from '@/stores/providers'
import SvgIcon from '@/components/SvgIcon.vue'

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
  toggle: [name: string, enabled: boolean]
  viewDeployed: []
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
        <SvgIcon name="plus" :size="12" />
        <span>{{ t('provider.new') }}</span>
      </button>
      <button
        @click="emit('apply')"
        :disabled="providers.length === 0"
        class="flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md border border-border bg-surface hover:bg-surface-hover hover:border-accent/40 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 group"
        :title="`应用全部 ${providers.length} 个服务商配置`"
      >
        <SvgIcon name="check" :size="12" class="group-hover:text-accent transition-colors" />
        <span>{{ t('provider.apply') }}</span>
      </button>
      <button
        @click="emit('viewDeployed')"
        class="flex items-center justify-center gap-1.5 px-2 py-1.5 text-xs font-medium rounded-md border border-border bg-surface hover:bg-surface-hover hover:border-accent/40 transition-all active:scale-95 group"
        :title="t('deployed.manageTitle')"
      >
        <SvgIcon name="refresh" :size="12" class="group-hover:text-accent transition-colors" />
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 overflow-auto p-2 scrollbar-thin">
      <div v-if="providers.length === 0" class="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
        <SvgIcon name="server" :size="24" class="opacity-20" />
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
              ? 'bg-emerald-50 dark:bg-emerald-950/20 border-emerald-400 dark:border-emerald-600 shadow-sm'
              : 'bg-transparent border-border hover:bg-surface-hover hover:border-border'
          ]"
        >
          <!-- Selection Marker -->
          <div v-if="provider.name === selected" class="absolute left-0 top-2 bottom-2 w-0.5 bg-emerald-500 rounded-r-full"></div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span 
                class="font-medium text-sm tracking-tight transition-colors" 
                :class="[
                  provider.name === selected ? 'text-emerald-700 dark:text-emerald-400' : '',
                  !provider.enabled ? 'line-through opacity-50' : ''
                ]"
              >
                {{ provider.name }}
              </span>
              <span v-if="!provider.enabled" class="text-[10px] text-muted-foreground bg-surface px-1 rounded">
                已禁用
              </span>
            </div>
            
            <!-- Actions -->
            <div class="flex items-center gap-1 opacity-0 transition-opacity duration-200 group-hover:opacity-100">
              <!-- 启用/禁用开关 -->
              <button
                @click.stop="emit('toggle', provider.name, !provider.enabled)"
                class="rounded p-1 transition-colors"
                :class="provider.enabled ? 'text-green-500 hover:bg-green-500/10' : 'text-muted-foreground hover:bg-surface-hover'"
                :title="provider.enabled ? '点击禁用' : '点击启用'"
              >
                <SvgIcon :name="provider.enabled ? 'eye' : 'eye-off'" :size="12" />
              </button>
              <button
                @click.stop="emit('edit', provider.name)"
                class="rounded p-1 text-muted-foreground hover:bg-background hover:text-primary transition-colors"
                :title="t('provider.edit')"
              >
                <SvgIcon name="edit" :size="12" />
              </button>
              <button
                @click.stop="emit('delete', provider.name)"
                class="rounded p-1 text-red-400 hover:bg-red-500/20 hover:text-red-500 transition-colors"
                :title="t('provider.delete')"
              >
                <SvgIcon name="trash" :size="12" />
              </button>
            </div>
          </div>
          
          <div class="flex items-center gap-2 text-[10px] text-muted-foreground">
             <span class="flex items-center gap-1 font-mono">
               <SvgIcon name="cube" :size="10" class="opacity-50" />
               {{ provider.model_count }} {{ t('provider.models') }}
             </span>
             <span v-if="provider.description" class="truncate max-w-[150px]" :title="provider.description">
               · {{ provider.description }}
             </span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

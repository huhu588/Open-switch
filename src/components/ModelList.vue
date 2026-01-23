<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ModelItem } from '@/stores/providers'
import SvgIcon from '@/components/SvgIcon.vue'

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
  edit: [model: { id: string, name: string }]
  delete: [id: string]
  fetch: []
}>()

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
        <SvgIcon name="plus" :size="12" />
        <span>{{ t('model.addModel') }}</span>
      </button>
      <button
        @click="emit('fetch')"
        :disabled="disabled"
        class="flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md border border-border bg-surface hover:bg-surface-hover hover:border-accent/40 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 group"
        title="Fetch models from provider"
      >
        <SvgIcon name="refresh" :size="12" class="group-hover:text-accent transition-colors" />
        <span>{{ t('model.sync') }}</span>
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 overflow-auto p-2 scrollbar-thin">
      <div v-if="disabled" class="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
        <p class="text-xs">{{ t('model.selectProviderFirst') }}</p>
      </div>
      <div v-else-if="models.length === 0" class="flex h-32 flex-col items-center justify-center gap-2 text-muted-foreground">
        <SvgIcon name="monitor" :size="24" class="opacity-20" />
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
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all">
              <button
                @click.stop="emit('edit', { id: model.id, name: model.name })"
                class="rounded p-1 text-blue-400 hover:bg-blue-500/20 hover:text-blue-500 transition-all"
                :title="t('model.edit')"
              >
                <SvgIcon name="edit" :size="12" />
              </button>
              <button
                @click.stop="emit('delete', model.id)"
                class="rounded p-1 text-red-400 hover:bg-red-500/20 hover:text-red-500 transition-all"
                :title="t('model.delete')"
              >
                <SvgIcon name="trash" :size="12" />
              </button>
            </div>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

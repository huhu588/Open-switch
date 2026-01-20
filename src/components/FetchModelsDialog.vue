<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()

interface Props {
  visible: boolean
  providerName: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  added: []
}>()

const models = ref<string[]>([])
const selectedModels = ref<Set<string>>(new Set())
const loading = ref(false)
const adding = ref(false)
const error = ref<string | null>(null)

watch(() => props.visible, async (visible) => {
  if (visible && props.providerName) {
    await fetchModels()
  } else {
    models.value = []
    selectedModels.value.clear()
    error.value = null
  }
})

async function fetchModels() {
  if (!props.providerName) return
  
  loading.value = true
  error.value = null
  
  try {
    models.value = await invoke<string[]>('fetch_site_models', {
      providerName: props.providerName
    })
    // 默认全选
    selectedModels.value = new Set(models.value)
  } catch (e) {
    error.value = String(e)
    models.value = []
  } finally {
    loading.value = false
  }
}

function toggleModel(id: string) {
  const newSet = new Set(selectedModels.value)
  if (newSet.has(id)) {
    newSet.delete(id)
  } else {
    newSet.add(id)
  }
  selectedModels.value = newSet
}

function selectAll() {
  selectedModels.value = new Set(models.value)
}

function clearAll() {
  selectedModels.value = new Set()
}

function close() {
  emit('update:visible', false)
}

async function addSelected() {
  if (selectedModels.value.size === 0 || !props.providerName) return
  
  adding.value = true
  
  try {
    await invoke('add_models_batch', {
      providerName: props.providerName,
      modelIds: Array.from(selectedModels.value)
    })
    emit('added')
    close()
  } catch (e) {
    error.value = String(e)
  } finally {
    adding.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="close">
        <div class="w-full max-w-lg rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <div class="px-5 py-4 border-b border-border">
            <h3 class="font-semibold text-lg">{{ t('fetchModels.title') }}</h3>
          </div>

          <div class="px-5 py-4">
            <div v-if="error" class="mb-4 px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <div v-if="loading" class="py-8 text-center text-muted-foreground">
              {{ t('fetchModels.fetching') }}
            </div>

            <div v-else-if="models.length === 0" class="py-8 text-center text-muted-foreground">
              {{ t('fetchModels.noModels') }}
            </div>

            <div v-else>
              <div class="flex items-center justify-between mb-3">
                <span class="text-sm text-muted-foreground">
                  {{ t('fetchModels.totalModels', { total: models.length, selected: selectedModels.size }) }}
                </span>
                <div class="flex gap-2">
                  <button @click="selectAll" class="text-xs text-accent-500 hover:text-accent-600">{{ t('common.selectAll') }}</button>
                  <button @click="clearAll" class="text-xs text-accent-500 hover:text-accent-600">{{ t('common.clearAll') }}</button>
                </div>
              </div>

              <div class="max-h-64 overflow-auto border border-border rounded-lg">
                <label
                  v-for="model in models"
                  :key="model"
                  class="flex items-center gap-3 px-3 py-2 cursor-pointer hover:bg-surface-hover"
                >
                  <input
                    type="checkbox"
                    :checked="selectedModels.has(model)"
                    @change="toggleModel(model)"
                    class="w-4 h-4 rounded"
                  />
                  <span class="text-sm font-mono truncate">{{ model }}</span>
                </label>
              </div>
            </div>
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-border">
            <button @click="close" :disabled="adding" class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors">
              {{ t('common.cancel') }}
            </button>
            <button
              @click="addSelected"
              :disabled="adding || selectedModels.size === 0"
              class="px-4 py-2 text-sm font-medium rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm"
            >
              {{ adding ? t('model.adding') : t('fetchModels.addModels', { count: selectedModels.size }) }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>

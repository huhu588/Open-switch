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
  saved: []
}>()

// 推理强度选项
const reasoningEffortOptions = [
  { value: '', label: '无' },
  { value: 'low', label: 'Low (低)' },
  { value: 'medium', label: 'Medium (中)' },
  { value: 'high', label: 'High (高)' }
]

const form = ref({
  id: '',
  name: '',
  reasoning_effort: ''
})

const loading = ref(false)
const error = ref<string | null>(null)

watch(() => props.visible, (visible) => {
  if (visible) {
    form.value = { id: '', name: '', reasoning_effort: '' }
    error.value = null
  }
})

function close() {
  emit('update:visible', false)
}

async function save() {
  if (!form.value.id.trim()) {
    error.value = t('model.modelIdRequired')
    return
  }
  if (!props.providerName) {
    error.value = t('model.providerNotSelected')
    return
  }

  loading.value = true
  error.value = null

  try {
    await invoke('add_model', {
      providerName: props.providerName,
      input: {
        id: form.value.id,
        name: form.value.name || form.value.id,
        reasoning_effort: form.value.reasoning_effort || null
      }
    })
    emit('saved')
    close()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="close">
        <div class="w-full max-w-sm rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <div class="px-5 py-4 border-b border-border">
            <h3 class="font-semibold text-lg">{{ t('model.addModel') }}</h3>
          </div>

          <div class="px-5 py-4 space-y-4">
            <div v-if="error" class="px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('model.modelId') }} *</label>
              <input
                v-model="form.id"
                type="text"
                :placeholder="t('model.placeholder.modelId')"
                class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary font-mono"
              />
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('model.displayName') }}</label>
              <input
                v-model="form.name"
                type="text"
                :placeholder="t('model.placeholder.displayName')"
                class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary"
              />
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('model.reasoningEffort') }}</label>
              <select
                v-model="form.reasoning_effort"
                class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary"
              >
                <option v-for="opt in reasoningEffortOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <p class="mt-1 text-xs text-secondary">{{ t('model.reasoningEffortHint') }}</p>
            </div>
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-border">
            <button @click="close" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors">
              {{ t('common.cancel') }}
            </button>
            <button @click="save" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm">
              {{ loading ? t('model.adding') : t('common.add') }}
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

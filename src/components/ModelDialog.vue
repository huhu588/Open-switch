<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()

// 编辑模式的模型信息
interface EditingModel {
  id: string
  name: string
}

interface Props {
  visible: boolean
  providerName: string | null
  editing?: EditingModel | null  // 编辑模式: 传入现有模型信息
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: []
}>()

const form = ref({
  id: '',
  name: ''
})

const loading = ref(false)
const error = ref<string | null>(null)

// 是否为编辑模式
const isEditing = computed(() => !!props.editing)

// 弹窗标题
const dialogTitle = computed(() => isEditing.value ? t('model.editModel') : t('model.addModel'))

watch(() => props.visible, (visible) => {
  if (visible) {
    if (props.editing) {
      // 编辑模式: 回显现有数据
      form.value = {
        id: props.editing.id,
        name: props.editing.name
      }
    } else {
      // 新增模式: 清空表单
      form.value = { id: '', name: '' }
    }
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
    if (isEditing.value) {
      // 编辑模式: 调用 update_model
      await invoke('update_model', {
        providerName: props.providerName,
        modelId: props.editing!.id,
        input: {
          id: form.value.id,
          name: form.value.name || form.value.id
        }
      })
    } else {
      // 新增模式: 调用 add_model
      await invoke('add_model', {
        providerName: props.providerName,
        input: {
          id: form.value.id,
          name: form.value.name || form.value.id
        }
      })
    }
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
            <h3 class="font-semibold text-lg">{{ dialogTitle }}</h3>
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
                :disabled="isEditing"
                :class="[
                  'w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary font-mono',
                  isEditing ? 'opacity-60 cursor-not-allowed' : ''
                ]"
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
          </div>

          <div class="px-5 py-4 flex justify-end gap-3 border-t border-border">
            <button @click="close" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors">
              {{ t('common.cancel') }}
            </button>
            <button @click="save" :disabled="loading" class="px-4 py-2 text-sm font-medium rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm">
              {{ loading ? (isEditing ? t('common.saving') : t('model.adding')) : (isEditing ? t('common.save') : t('common.add')) }}
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

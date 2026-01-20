<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface Props {
  visible: boolean
  title?: string
  message?: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  danger: false
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: []
  cancel: []
}>()

function close() {
  emit('update:visible', false)
}

function confirm() {
  emit('confirm')
  close()
}

function cancel() {
  emit('cancel')
  close()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="cancel">
        <div class="w-full max-w-sm rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <!-- 标题 -->
          <div class="px-5 py-4 border-b border-border">
            <h3 class="font-semibold text-lg">{{ title || t('confirm.title') }}</h3>
          </div>

          <!-- 内容 -->
          <div class="px-5 py-4">
            <p class="text-sm text-muted-foreground">{{ message || t('confirm.defaultMessage') }}</p>
          </div>

          <!-- 按钮 -->
          <div class="px-5 py-4 flex justify-end gap-3 border-t border-border">
            <button
              @click="cancel"
              class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover transition-colors"
            >
              {{ cancelText || t('common.cancel') }}
            </button>
            <button
              @click="confirm"
              class="px-4 py-2 text-sm font-medium rounded-lg text-white transition-colors"
              :class="danger ? 'bg-red-500 hover:bg-red-600' : 'bg-accent hover:bg-accent/90'"
            >
              {{ confirmText || t('common.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

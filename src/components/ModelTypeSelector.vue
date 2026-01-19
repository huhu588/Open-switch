<script setup lang="ts">
import { computed } from 'vue'
import { MODEL_TYPES, type ModelType } from '@/config/modelTypes'
import SvgIcon from '@/components/SvgIcon.vue'

interface Props {
  modelValue: ModelType
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: ModelType]
}>()

const selected = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})
</script>

<template>
  <div class="inline-flex items-center gap-1 p-1 rounded-full bg-surface border border-border">
    <button
      v-for="type in MODEL_TYPES"
      :key="type.id"
      @click="selected = type.id"
      :class="[
        'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium transition-all',
        selected === type.id
          ? 'bg-background text-primary shadow-sm'
          : 'text-muted-foreground hover:text-primary'
      ]"
    >
      <SvgIcon :name="type.icon" :size="18" />
      <span>{{ type.name }}</span>
    </button>
  </div>
</template>

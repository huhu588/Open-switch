<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { 
  PROVIDER_PRESETS, 
  getModelsByType,
  type ApiProtocol
} from '@/config/providerPresets'
import { MODEL_TYPES, type ModelType } from '@/config/modelTypes'

const { t } = useI18n()

interface Props {
  visible: boolean
  editing?: string | null
  defaultModelType?: ModelType
}

const props = withDefaults(defineProps<Props>(), {
  editing: null,
  defaultModelType: 'claude'
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: []
}>()

// 表单数据
const form = ref({
  name: '',
  api_key: '',
  base_url: '',
  description: '',
  protocol: 'anthropic' as ApiProtocol,
  model_type: 'claude' as ModelType,
})

// 预设和模型相关
const selectedPreset = ref<string>('自定义')
const autoAddModels = ref(true)
const selectedModels = ref<string[]>([])

const loading = ref(false)
const error = ref<string | null>(null)
const showApiKey = ref(false)

// 扁平化预设列表（排除自定义）
const flatPresets = computed(() => {
  return PROVIDER_PRESETS.filter(p => p.category !== 'custom')
})

// 当前选中的预设
const currentPreset = computed(() => {
  return PROVIDER_PRESETS.find(p => p.name === selectedPreset.value)
})

// 当前预设支持的协议
const supportedProtocols = computed(() => {
  return currentPreset.value?.supportedProtocols || ['anthropic', 'openai']
})

// 根据模型厂家获取模型列表
const presetModels = computed(() => {
  return getModelsByType(form.value.model_type)
})

// 选择预设时自动填充
function onPresetChange(presetName: string) {
  const preset = PROVIDER_PRESETS.find(p => p.name === presetName)
  if (preset) {
    form.value.name = preset.category === 'custom' ? '' : preset.name
    form.value.base_url = preset.baseUrl
    form.value.protocol = preset.defaultProtocol
    form.value.description = preset.description || ''
    // 根据当前模型厂家选中所有模型
    selectedModels.value = getModelsByType(form.value.model_type).map(m => m.id)
  }
}

// 切换全选/取消全选模型
function toggleAllModels() {
  if (selectedModels.value.length === presetModels.value.length) {
    selectedModels.value = []
  } else {
    selectedModels.value = presetModels.value.map(m => m.id)
  }
}

// 监听模型厂家变化，更新选中的模型
watch(() => form.value.model_type, () => {
  selectedModels.value = presetModels.value.map(m => m.id)
})

// 监听 editing 变化，加载数据
watch(() => props.visible, async (visible) => {
  if (visible && props.editing) {
    try {
      const provider = await invoke<any>('get_provider', { name: props.editing })
      if (provider) {
        // 根据 npm 包推断协议
        const npm = provider.npm || ''
        const inferredProtocol = npm.includes('openai') ? 'openai' : 'anthropic'
        
        form.value = {
          name: props.editing,
          api_key: provider.options.api_key || '',
          base_url: provider.options.base_url || '',
          description: provider.metadata?.description || '',
          protocol: inferredProtocol as ApiProtocol,
          model_type: provider.model_type || provider.metadata?.model_type || 'claude',
        }
        selectedPreset.value = '自定义'
        autoAddModels.value = false
      }
    } catch (e) {
      console.error('加载 Provider 失败:', e)
    }
  } else if (visible) {
    // 添加模式，默认自定义配置
    selectedPreset.value = '自定义'
    onPresetChange('自定义')
    form.value.api_key = ''
    form.value.model_type = props.defaultModelType
    autoAddModels.value = true
  }
  error.value = null
})

function close() {
  emit('update:visible', false)
}

async function save() {
  if (!form.value.name.trim()) {
    error.value = t('provider.nameRequired')
    return
  }
  if (!form.value.api_key.trim()) {
    error.value = t('provider.apiKeyRequired')
    return
  }

  loading.value = true
  error.value = null

  try {
    const baseUrl = form.value.base_url || 'https://api.anthropic.com'
    
    if (props.editing) {
      // 根据协议选择 npm 包
      const npm = form.value.protocol === 'openai' ? '@ai-sdk/openai' : '@ai-sdk/anthropic'
      
      await invoke('update_provider', {
        name: props.editing,
        input: {
          name: form.value.name,
          api_key: form.value.api_key,
          base_url: baseUrl,
          description: form.value.description || null,
          npm: npm
        }
      })
    } else {
      // 添加 Provider
      await invoke('add_provider', {
        input: {
          name: form.value.name,
          api_key: form.value.api_key,
          base_url: baseUrl,
          description: form.value.description || null,
          model_type: form.value.model_type
        }
      })
      
      // 自动添加选中的模型
      if (autoAddModels.value && selectedModels.value.length > 0) {
        for (const modelId of selectedModels.value) {
          const modelDef = presetModels.value.find(m => m.id === modelId)
          if (modelDef) {
            try {
              await invoke('add_model', {
                providerName: form.value.name,
                input: {
                  id: modelDef.id,
                  name: modelDef.name,
                  context_limit: modelDef.contextLimit || null,
                  output_limit: modelDef.outputLimit || null,
                }
              })
            } catch (e) {
              console.warn(`添加模型 ${modelId} 失败:`, e)
            }
          }
        }
      }
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
        <div class="w-full max-w-md rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <!-- 标题 -->
          <div class="px-5 py-4 border-b border-border">
            <h3 class="font-semibold text-lg">{{ editing ? t('provider.editProvider') : t('provider.addProvider') }}</h3>
          </div>

          <!-- 表单 -->
          <div class="px-5 py-4 space-y-4 max-h-[60vh] overflow-y-auto">
            <!-- 错误提示 -->
            <div v-if="error" class="px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <!-- 预设选择 (仅新增时显示) -->
            <div v-if="!editing">
              <label class="block text-sm font-medium mb-2">{{ t('provider.preset') || '预设供应商' }}</label>
              <div class="flex flex-wrap gap-2">
                <!-- 自定义配置按钮 -->
                <button
                  type="button"
                  @click="onPresetChange('自定义')"
                  :class="[
                    'px-3 py-1.5 text-sm rounded-full border-2 transition-all font-medium',
                    selectedPreset === '自定义'
                      ? 'bg-purple-600 text-white border-purple-600 shadow-sm'
                      : 'border-border text-primary hover:border-purple-400 hover:bg-surface-hover'
                  ]"
                >
                  自定义配置
                </button>
                <!-- 预设供应商按钮 -->
                <button
                  v-for="preset in flatPresets"
                  :key="preset.name"
                  type="button"
                  @click="onPresetChange(preset.name)"
                  :class="[
                    'px-3 py-1.5 text-sm rounded-full border-2 transition-all inline-flex items-center gap-1 font-medium',
                    selectedPreset === preset.name
                      ? 'bg-purple-600 text-white border-purple-600 shadow-sm'
                      : 'border-border text-primary hover:border-purple-400 hover:bg-surface-hover'
                  ]"
                >
                  {{ preset.name }}
                  <span v-if="preset.category === 'aggregator'" class="text-yellow-300 text-xs">★</span>
                </button>
              </div>
              <p class="mt-2 text-xs text-muted-foreground">
                💡 自定义配置需手动填写所有必要字段
              </p>
              <p v-if="currentPreset?.apiKeyUrl" class="mt-1 text-xs text-muted-foreground">
                <a :href="currentPreset.apiKeyUrl" target="_blank" class="text-accent-500 hover:underline">
                  获取 API Key →
                </a>
              </p>
            </div>

            <!-- 模型厂家选择 (仅新增时显示) -->
            <div v-if="!editing">
              <label class="block text-sm font-medium mb-2">模型厂家</label>
              <div class="flex gap-2">
                <button
                  v-for="type in MODEL_TYPES"
                  :key="type.id"
                  type="button"
                  @click="form.model_type = type.id"
                  :class="[
                    'flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-full border-2 transition-all font-medium',
                    form.model_type === type.id
                      ? 'bg-purple-600 text-white border-purple-600 shadow-sm'
                      : 'border-border text-primary hover:border-purple-400 hover:bg-surface-hover'
                  ]"
                >
                  <span>{{ type.icon }}</span>
                  <span>{{ type.name }}</span>
                </button>
              </div>
            </div>

            <!-- 名称 -->
            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('provider.name') }} *</label>
              <input
                v-model="form.name"
                type="text"
                :placeholder="t('provider.placeholder.name')"
                :disabled="!!editing"
                class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary disabled:opacity-60"
              />
            </div>

            <!-- API Key -->
            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('provider.apiKey') }} *</label>
              <div class="relative">
                <input
                  v-model="form.api_key"
                  :type="showApiKey ? 'text' : 'password'"
                  :placeholder="t('provider.placeholder.apiKey')"
                  class="w-full px-3 py-2 pr-16 rounded-lg border border-border bg-surface text-primary font-mono"
                />
                <button
                  type="button"
                  @click="showApiKey = !showApiKey"
                  class="absolute right-2 top-1/2 -translate-y-1/2 px-2 py-1 text-xs text-muted-foreground hover:text-primary transition-colors"
                >
                  {{ showApiKey ? t('provider.hideApiKey') : t('provider.showApiKey') }}
                </button>
              </div>
            </div>

            <!-- Base URL -->
            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('provider.baseUrl') }}</label>
              <input
                v-model="form.base_url"
                type="text"
                :placeholder="t('provider.placeholder.baseUrl')"
                class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary font-mono text-sm"
              />
            </div>

            <!-- 协议选择 -->
            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('provider.protocol') || 'API 协议' }}</label>
              <div class="flex gap-4">
                <label 
                  v-for="protocol in supportedProtocols" 
                  :key="protocol"
                  class="flex items-center gap-2 cursor-pointer"
                >
                  <input
                    type="radio"
                    :value="protocol"
                    v-model="form.protocol"
                    class="w-4 h-4 text-accent-500"
                  />
                  <span class="text-sm">
                    {{ protocol === 'anthropic' ? 'Anthropic 协议' : 'OpenAI 协议' }}
                  </span>
                </label>
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ form.protocol === 'anthropic' ? '使用 Anthropic 原生 API 格式' : '使用 OpenAI 兼容 API 格式' }}
              </p>
            </div>

            <!-- 描述 -->
            <div>
              <label class="block text-sm font-medium mb-1.5">{{ t('provider.description') }}</label>
              <input
                v-model="form.description"
                type="text"
                :placeholder="t('provider.placeholder.description')"
                class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary"
              />
            </div>

            <!-- 自动添加模型 (仅新增时显示) -->
            <div v-if="!editing" class="border-t border-border pt-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="autoAddModels"
                  class="w-4 h-4 rounded text-accent-500"
                />
                <span class="text-sm font-medium">自动添加预设模型</span>
              </label>
              
              <!-- 模型选择列表 -->
              <div v-if="autoAddModels" class="mt-3 space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs text-muted-foreground">
                    已选择 {{ selectedModels.length }} / {{ presetModels.length }} 个模型
                  </span>
                  <button 
                    type="button"
                    @click="toggleAllModels"
                    class="text-xs text-accent-500 hover:underline"
                  >
                    {{ selectedModels.length === presetModels.length ? '取消全选' : '全选' }}
                  </button>
                </div>
                <div class="grid grid-cols-2 gap-2 max-h-32 overflow-y-auto p-2 rounded-lg bg-surface">
                  <label 
                    v-for="model in presetModels" 
                    :key="model.id"
                    class="flex items-center gap-2 cursor-pointer text-sm"
                  >
                    <input
                      type="checkbox"
                      :value="model.id"
                      v-model="selectedModels"
                      class="w-3.5 h-3.5 rounded text-accent-500"
                    />
                    <span class="truncate" :title="model.name">{{ model.name }}</span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          <!-- 按钮 -->
          <div class="px-5 py-4 flex justify-end gap-3 border-t border-border">
            <button
              @click="close"
              :disabled="loading"
              class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="save"
              :disabled="loading"
              class="px-4 py-2 text-sm font-medium rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors"
            >
              {{ loading ? t('common.saving') : t('common.save') }}
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

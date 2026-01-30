<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import { type ModelItem, type BaseUrlItem, type UrlTestResult } from '@/stores/providers'
import { 
  PROVIDER_PRESETS, 
  getModelsByType,
  getNpmPackageByProtocol,
  type ApiProtocol
} from '@/config/providerPresets'
import { MODEL_TYPES, type ModelType } from '@/config/modelTypes'
import SvgIcon from '@/components/SvgIcon.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

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

// ============================================================================
// 表单数据
// ============================================================================

const form = ref({
  name: '',
  api_key: '',
  description: '',
  protocol: 'anthropic' as ApiProtocol,
  model_type: 'claude' as ModelType,
})

// 多 URL 管理
const baseUrls = ref<{ url: string; latency_ms: number | null; quality: string }[]>([])
const activeBaseUrl = ref('')
const newUrlInput = ref('')

// URL 后缀控制
const autoAddV1Suffix = ref(true)

// 预设和模型相关
const selectedPreset = ref<string>('自定义')
const autoAddModels = ref(true)
const selectedModels = ref<string[]>([])

// 自定义模型相关
const customModelInput = ref('')
const customModels = ref<string[]>([])

// 编辑模式下的模型列表
const existingModels = ref<ModelItem[]>([])

// 状态
const loading = ref(false)
const error = ref<string | null>(null)
const showApiKey = ref(false)
const testing = ref(false)
const testingUrl = ref<string | null>(null)

// 自动选择最快 URL 开关
const autoSelectFastestEnabled = ref(true)

// 应用目标选择
const applyTargets = ref<string[]>(['opencode'])

// 保存到 Open Switch 统一配置
const saveToOpenSwitch = ref(true)

// 删除确认对话框
const showDeleteModelDialog = ref(false)
const deleteModelTarget = ref<string | null>(null)

// 根据 model_type 获取可选的应用目标
const availableTargets = computed(() => {
  const targets = [
    { id: 'opencode', label: 'OpenCode', icon: 'code' }
  ]
  
  switch (form.value.model_type) {
    case 'claude':
      targets.unshift({ id: 'claude_code', label: 'Claude Code', icon: 'claude' })
      break
    case 'codex':
      targets.unshift({ id: 'codex', label: 'Codex CLI', icon: 'openai' })
      break
    case 'gemini':
      targets.unshift({ id: 'gemini', label: 'Gemini CLI', icon: 'gemini' })
      break
  }
  
  return targets
})

// 根据选择的应用目标，判断是否需要添加模型
const needsModels = computed(() => {
  return applyTargets.value.includes('opencode')
})

// 获取模型提示信息
const modelGuidance = computed(() => {
  const targets = applyTargets.value
  const hasOpencode = targets.includes('opencode')
  const hasCli = targets.some(t => ['claude_code', 'codex', 'gemini'].includes(t))
  
  if (hasOpencode && hasCli) {
    return {
      type: 'info',
      message: '需要添加模型。OpenCode 使用模型列表，CLI 工具将使用第一个模型作为默认模型。'
    }
  } else if (hasOpencode) {
    return {
      type: 'required',
      message: 'OpenCode 需要模型列表才能正常工作，请添加至少一个模型。'
    }
  } else if (hasCli) {
    return {
      type: 'optional',
      message: 'CLI 工具只需要 API Key 和 Base URL，模型可选（将使用工具默认模型）。'
    }
  }
  return null
})

// ============================================================================
// 计算属性
// ============================================================================

const flatPresets = computed(() => {
  return PROVIDER_PRESETS.filter(p => p.category !== 'custom')
})

const currentPreset = computed(() => {
  return PROVIDER_PRESETS.find(p => p.name === selectedPreset.value)
})

const supportedProtocols = computed(() => {
  return currentPreset.value?.supportedProtocols || ['anthropic', 'openai']
})

const presetModels = computed(() => {
  if (currentPreset.value?.name === '智谱 AI' && currentPreset.value.models.length > 0) {
    return currentPreset.value.models
  }
  return getModelsByType(form.value.model_type)
})

// ============================================================================
// 预设处理
// ============================================================================

function onPresetChange(presetName: string) {
  selectedPreset.value = presetName
  const preset = PROVIDER_PRESETS.find(p => p.name === presetName)
  if (preset) {
    form.value.name = preset.category === 'custom' ? '' : preset.name
    form.value.protocol = preset.defaultProtocol
    form.value.description = preset.description || ''
    
    // 设置 base URL
    if (preset.baseUrl) {
      baseUrls.value = [{ url: preset.baseUrl, latency_ms: null, quality: 'untested' }]
      activeBaseUrl.value = preset.baseUrl
    }
    
    if (preset.name === '智谱 AI' && preset.models.length > 0) {
      selectedModels.value = preset.models.map(m => m.id)
    } else {
      selectedModels.value = getModelsByType(form.value.model_type).map(m => m.id)
    }
    autoAddV1Suffix.value = preset.name !== '智谱 AI'
  }
}

function onModelTypeChange(typeId: ModelType) {
  form.value.model_type = typeId
  const protocolMap: Record<ModelType, ApiProtocol> = {
    'claude': 'anthropic',
    'codex': 'openai',
    'gemini': 'openai',
  }
  form.value.protocol = protocolMap[typeId] || 'anthropic'
}

watch(() => form.value.model_type, (newType) => {
  if (currentPreset.value?.name === '智谱 AI') return
  selectedModels.value = presetModels.value.map(m => m.id)
  
  // 更新默认应用目标
  const defaultTargets = ['opencode']
  if (newType === 'claude') defaultTargets.unshift('claude_code')
  else if (newType === 'codex') defaultTargets.unshift('codex')
  else if (newType === 'gemini') defaultTargets.unshift('gemini')
  applyTargets.value = defaultTargets
})

// ============================================================================
// URL 管理
// ============================================================================

function addUrl() {
  const url = newUrlInput.value.trim()
  if (url && !baseUrls.value.some(u => u.url === url)) {
    baseUrls.value.push({ url, latency_ms: null, quality: 'untested' })
    if (!activeBaseUrl.value) {
      activeBaseUrl.value = url
    }
    newUrlInput.value = ''
  }
}

function removeUrl(url: string) {
  if (baseUrls.value.length <= 1) return
  baseUrls.value = baseUrls.value.filter(u => u.url !== url)
  if (activeBaseUrl.value === url && baseUrls.value.length > 0) {
    activeBaseUrl.value = baseUrls.value[0].url
  }
}

function setActiveUrl(url: string) {
  activeBaseUrl.value = url
}

function onUrlKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    addUrl()
  }
}

// ============================================================================
// 延迟测试
// ============================================================================

async function testSingleUrl(url: string) {
  if (!form.value.api_key) {
    error.value = '请先填写 API Key'
    return
  }
  
  testingUrl.value = url
  
  try {
    const results = await invoke<{ results: UrlTestResult[] }>('test_provider_urls', {
      providerName: form.value.name || 'test',
      urls: [url],
      apiKey: form.value.api_key,
      modelType: form.value.model_type,
      testCount: 3
    })
    
    if (results.results.length > 0) {
      const result = results.results[0]
      const urlConfig = baseUrls.value.find(u => u.url === url)
      if (urlConfig) {
        urlConfig.latency_ms = result.latency_ms
        urlConfig.quality = result.quality
      }
    }
  } catch (e) {
    console.error('测试失败:', e)
    error.value = `测试失败: ${String(e)}`
  } finally {
    testingUrl.value = null
  }
}

async function testAllUrls() {
  if (!form.value.api_key) {
    error.value = '请先填写 API Key'
    return
  }
  
  if (baseUrls.value.length === 0) {
    error.value = '请先添加 URL'
    return
  }
  
  testing.value = true
  error.value = null
  
  try {
    const urls = baseUrls.value.map(u => u.url)
    const results = await invoke<{ results: UrlTestResult[]; fastest_url: string | null }>('test_provider_urls', {
      providerName: form.value.name || 'test',
      urls,
      apiKey: form.value.api_key,
      modelType: form.value.model_type,
      testCount: 3
    })
    
    // 更新测试结果
    for (const result of results.results) {
      const urlConfig = baseUrls.value.find(u => u.url === result.url)
      if (urlConfig) {
        urlConfig.latency_ms = result.latency_ms
        urlConfig.quality = result.quality
      }
    }
    
    // 如果启用了自动选择最快，则自动选择
    if (autoSelectFastestEnabled.value) {
      autoSelectFastest()
    }
  } catch (e) {
    console.error('测试失败:', e)
    error.value = `测试失败: ${String(e)}`
  } finally {
    testing.value = false
  }
}

function autoSelectFastest() {
  const tested = baseUrls.value.filter(u => u.latency_ms !== null)
  if (tested.length === 0) {
    error.value = '没有可用的测试结果'
    return
  }
  
  const fastest = tested.reduce((a, b) => 
    (a.latency_ms || Infinity) < (b.latency_ms || Infinity) ? a : b
  )
  activeBaseUrl.value = fastest.url
}

function getQualityColor(quality: string) {
  switch (quality) {
    case 'excellent': return 'text-green-500'
    case 'good': return 'text-blue-500'
    case 'fair': return 'text-yellow-500'
    case 'poor': return 'text-orange-500'
    case 'failed': return 'text-red-500'
    default: return 'text-gray-400'
  }
}

function getQualityLabel(quality: string) {
  switch (quality) {
    case 'excellent': return '优秀'
    case 'good': return '良好'
    case 'fair': return '一般'
    case 'poor': return '较差'
    case 'failed': return '失败'
    default: return '未测试'
  }
}

// ============================================================================
// 模型管理
// ============================================================================

function toggleAllModels() {
  if (selectedModels.value.length === presetModels.value.length) {
    selectedModels.value = []
  } else {
    selectedModels.value = presetModels.value.map(m => m.id)
  }
}

function addCustomModel() {
  const modelName = customModelInput.value.trim()
  if (modelName && !customModels.value.includes(modelName)) {
    customModels.value.push(modelName)
    customModelInput.value = ''
  }
}

function removeCustomModel(modelName: string) {
  customModels.value = customModels.value.filter(m => m !== modelName)
}

function onCustomModelKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    addCustomModel()
  }
}

// 编辑模式下删除模型
function openDeleteModel(modelId: string) {
  deleteModelTarget.value = modelId
  showDeleteModelDialog.value = true
}

async function confirmDeleteModel() {
  if (!deleteModelTarget.value || !props.editing) return
  
  try {
    await invoke('delete_model', {
      providerName: props.editing,
      modelId: deleteModelTarget.value
    })
    existingModels.value = existingModels.value.filter(m => m.id !== deleteModelTarget.value)
  } catch (e) {
    console.error('删除模型失败:', e)
    error.value = `删除模型失败: ${String(e)}`
  }
  
  showDeleteModelDialog.value = false
  deleteModelTarget.value = null
}

// 编辑模式下添加模型
async function addModelToExisting() {
  const modelName = customModelInput.value.trim()
  if (!modelName || !props.editing) return
  
  try {
    await invoke('add_model', {
      providerName: props.editing,
      input: { id: modelName, name: modelName }
    })
    existingModels.value.push({ id: modelName, name: modelName })
    customModelInput.value = ''
  } catch (e) {
    console.error('添加模型失败:', e)
    error.value = `添加模型失败: ${String(e)}`
  }
}

// ============================================================================
// 数据加载
// ============================================================================

watch(() => props.visible, async (visible) => {
  if (visible && props.editing) {
    // 编辑模式
    try {
      const provider = await invoke<any>('get_provider', { name: props.editing })
      if (provider) {
        const npm = provider.npm || ''
        let inferredProtocol: ApiProtocol = 'anthropic'
        let inferredModelType: ModelType = 'claude'
        
        if (npm.includes('openai-compatible')) {
          inferredProtocol = 'openai-compatible'
          inferredModelType = 'codex'
        } else if (npm.includes('openai')) {
          inferredProtocol = 'openai'
          inferredModelType = 'codex'
        } else if (npm.includes('anthropic')) {
          inferredProtocol = 'anthropic'
          if (props.editing.toLowerCase().includes('gemini')) {
            inferredModelType = 'gemini'
          } else {
            inferredModelType = 'claude'
          }
        }
        
        form.value = {
          name: props.editing,
          api_key: provider.options.api_key || '',
          description: provider.description || '',
          protocol: inferredProtocol,
          model_type: provider.model_type || inferredModelType,
        }
        
        // 加载 base_urls
        if (provider.options.base_urls && provider.options.base_urls.length > 0) {
          baseUrls.value = provider.options.base_urls.map((u: BaseUrlItem) => ({
            url: u.url,
            latency_ms: u.latency_ms,
            quality: u.quality || 'untested'
          }))
        } else {
          baseUrls.value = [{ url: provider.options.base_url, latency_ms: null, quality: 'untested' }]
        }
        activeBaseUrl.value = provider.options.base_url
        
        selectedPreset.value = '自定义'
        autoAddModels.value = false
        
        // 加载已有模型
        const models = await invoke<ModelItem[]>('get_models', { providerName: props.editing })
        existingModels.value = models
        
        // 设置默认应用目标（编辑模式下默认只应用到 opencode）
        const modelType = provider.model_type || inferredModelType
        const defaultTargets = ['opencode']
        if (modelType === 'claude') defaultTargets.unshift('claude_code')
        else if (modelType === 'codex') defaultTargets.unshift('codex')
        else if (modelType === 'gemini') defaultTargets.unshift('gemini')
        applyTargets.value = defaultTargets
      }
    } catch (e) {
      console.error('加载 Provider 失败:', e)
    }
  } else if (visible) {
    // 新增模式
    selectedPreset.value = '自定义'
    onPresetChange('自定义')
    form.value.api_key = ''
    form.value.model_type = props.defaultModelType
    autoAddModels.value = true
    customModels.value = []
    customModelInput.value = ''
    baseUrls.value = []
    activeBaseUrl.value = ''
    existingModels.value = []
    
    // 设置默认应用目标
    const defaultTargets = ['opencode']
    if (props.defaultModelType === 'claude') defaultTargets.unshift('claude_code')
    else if (props.defaultModelType === 'codex') defaultTargets.unshift('codex')
    else if (props.defaultModelType === 'gemini') defaultTargets.unshift('gemini')
    applyTargets.value = defaultTargets
  }
  error.value = null
})

// ============================================================================
// 保存和关闭
// ============================================================================

function close() {
  emit('update:visible', false)
}

async function openApiKeyUrl() {
  if (currentPreset.value?.apiKeyUrl) {
    try {
      await openUrl(currentPreset.value.apiKeyUrl)
    } catch (e) {
      console.error('打开链接失败:', e)
      if (typeof window !== 'undefined') {
        window.open(currentPreset.value.apiKeyUrl, '_blank')
      }
    }
  }
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
  if (baseUrls.value.length === 0) {
    error.value = '请至少添加一个 Base URL'
    return
  }
  
  // 检查是否需要模型
  const hasModels = (autoAddModels.value && selectedModels.value.length > 0) || 
                    customModels.value.length > 0 ||
                    existingModels.value.length > 0
  
  if (applyTargets.value.includes('opencode') && !hasModels && !props.editing) {
    error.value = 'OpenCode 需要至少一个模型，请添加模型或取消勾选 OpenCode'
    return
  }

  loading.value = true
  error.value = null

  try {
    const baseUrl = activeBaseUrl.value || baseUrls.value[0]?.url || ''
    const npm = getNpmPackageByProtocol(form.value.protocol)
    
    // 如果有多个 URL，自动测试并选择最快的
    if (baseUrls.value.length > 1) {
      const untested = baseUrls.value.filter(u => u.latency_ms === null)
      if (untested.length > 0) {
        await testAllUrls()
        autoSelectFastest()
      }
    }
    
    if (props.editing) {
      const nameChanged = props.editing !== form.value.name.trim()
      
      if (nameChanged) {
        // 名称改变了，需要删除旧的并创建新的
        // 先获取旧 provider 的模型列表
        const oldModels = existingModels.value.map(m => ({ id: m.id, name: m.name }))
        
        // 删除旧的 provider
        await invoke('delete_provider', { name: props.editing })
        
        // 创建新的 provider
        await invoke('add_provider', {
          input: {
            name: form.value.name,
            api_key: form.value.api_key,
            base_url: activeBaseUrl.value || baseUrl,
            base_urls: baseUrls.value.map(u => u.url),
            description: form.value.description || null,
            model_type: form.value.model_type,
            npm,
            auto_add_v1_suffix: autoAddV1Suffix.value
          }
        })
        
        // 重新添加模型
        if (oldModels.length > 0) {
          await invoke('add_models_batch_detailed', {
            providerName: form.value.name,
            inputs: oldModels,
          })
        }
      } else {
        // 名称没变，正常更新
        await invoke('update_provider', {
          name: props.editing,
          input: {
            name: form.value.name,
            api_key: form.value.api_key,
            base_url: activeBaseUrl.value || baseUrl,
            base_urls: baseUrls.value.map(u => u.url),
            description: form.value.description || null,
            npm,
            model_type: form.value.model_type,
            auto_add_v1_suffix: autoAddV1Suffix.value
          }
        })
      }
      
      // 编辑模式下添加新模型
      if (customModels.value.length > 0) {
        const newModels = customModels.value.map(modelName => ({ id: modelName, name: modelName }))
        await invoke('add_models_batch_detailed', {
          providerName: form.value.name,
          inputs: newModels,
        })
      }
    } else {
      await invoke('add_provider', {
        input: {
          name: form.value.name,
          api_key: form.value.api_key,
          base_url: activeBaseUrl.value || baseUrl,
          base_urls: baseUrls.value.map(u => u.url),
          description: form.value.description || null,
          model_type: form.value.model_type,
          npm,
          auto_add_v1_suffix: autoAddV1Suffix.value
        }
      })
      
      // 添加模型
      const batchInputs: Array<{ id: string; name?: string | null }> = []

      if (autoAddModels.value && selectedModels.value.length > 0) {
        for (const modelId of selectedModels.value) {
          const modelDef = presetModels.value.find(m => m.id === modelId)
          if (modelDef) {
            batchInputs.push({ id: modelDef.id, name: modelDef.name })
          }
        }
      }

      if (customModels.value.length > 0) {
        for (const modelName of customModels.value) {
          batchInputs.push({ id: modelName, name: modelName })
        }
      }

      if (batchInputs.length > 0) {
        await invoke('add_models_batch_detailed', {
          providerName: form.value.name,
          inputs: batchInputs,
        })
      }
    }
    
    // 应用到选中的目标
    const activeUrl = activeBaseUrl.value || baseUrls.value[0]?.url || ''
    const firstModelId = selectedModels.value[0] || customModels.value[0] || existingModels.value[0]?.id
    
    for (const target of applyTargets.value) {
      try {
        if (target === 'claude_code' && form.value.model_type === 'claude') {
          await invoke('apply_provider_to_claude_code', {
            provider: {
              name: form.value.name,
              api_key: form.value.api_key,
              base_url: activeUrl || null,
              model: firstModelId || null,
              enabled: true,
              description: form.value.description || null
            }
          })
        } else if (target === 'codex' && form.value.model_type === 'codex') {
          await invoke('apply_provider_to_codex', {
            provider: {
              name: form.value.name,
              api_key: form.value.api_key,
              base_url: activeUrl,
              env_key: 'OPENAI_API_KEY',
              enabled: true,
              description: form.value.description || null
            }
          })
        } else if (target === 'gemini' && form.value.model_type === 'gemini') {
          await invoke('apply_provider_to_gemini', {
            provider: {
              name: form.value.name,
              api_key: form.value.api_key,
              base_url: activeUrl || null,
              model: firstModelId || null,
              enabled: true,
              description: form.value.description || null
            }
          })
        }
        // 'opencode' 目标已经在上面的 add_provider/update_provider 中处理了
      } catch (e) {
        console.warn(`应用到 ${target} 失败:`, e)
        // 不中断整个流程，只是警告
      }
    }
    
    // 保存到 Open Switch 统一配置
    if (saveToOpenSwitch.value) {
      try {
        // 构建 OpenCode 模型配置
        const opencodeModels: Record<string, { name: string }> = {}
        const allModels = [
          ...(autoAddModels.value ? selectedModels.value : []),
          ...customModels.value,
          ...existingModels.value.map(m => m.id)
        ]
        for (const modelId of allModels) {
          const modelDef = presetModels.value.find(m => m.id === modelId)
          opencodeModels[modelId] = { name: modelDef?.name || modelId }
        }
        
        await invoke('add_open_switch_provider', {
          input: {
            name: form.value.name,
            base_url: activeUrl,
            api_key: form.value.api_key,
            apps: {
              opencode: applyTargets.value.includes('opencode'),
              claude: applyTargets.value.includes('claude_code'),
              codex: applyTargets.value.includes('codex'),
              gemini: applyTargets.value.includes('gemini'),
            },
            models: {
              opencode: applyTargets.value.includes('opencode') ? {
                npm: getNpmPackageByProtocol(form.value.protocol),
                models: opencodeModels,
              } : undefined,
              claude: applyTargets.value.includes('claude_code') ? {
                model: firstModelId,
              } : undefined,
              codex: applyTargets.value.includes('codex') ? {
                model: firstModelId,
                reasoning_effort: 'high',
              } : undefined,
              gemini: applyTargets.value.includes('gemini') ? {
                model: firstModelId,
              } : undefined,
            },
            notes: form.value.description || undefined,
          }
        })
      } catch (e) {
        console.warn('保存到 Open Switch 统一配置失败:', e)
        // 不中断整个流程
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
      <div v-if="visible" class="fixed inset-0 z-50 bg-background">
        <!-- 全屏容器 -->
        <div class="h-full flex flex-col">
          <!-- 标题栏 -->
          <div class="flex-shrink-0 px-6 py-4 border-b border-border flex items-center justify-between">
            <h3 class="font-semibold text-xl">
              {{ editing ? t('provider.editProvider') : t('provider.addProvider') }}
              <span v-if="editing" class="text-muted-foreground font-normal">: {{ editing }}</span>
            </h3>
            <button @click="close" class="p-2 hover:bg-surface-hover rounded-lg transition-colors">
              <SvgIcon name="close" :size="20" />
            </button>
          </div>

          <!-- 错误提示 -->
          <div v-if="error" class="flex-shrink-0 mx-6 mt-4 px-4 py-3 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
            {{ error }}
          </div>

          <!-- 主内容区 - 两栏布局 -->
          <div class="flex-1 flex gap-6 p-6 min-h-0 overflow-hidden">
            <!-- 左侧：基本信息和 URL 管理 -->
            <div class="w-1/2 flex flex-col gap-4 overflow-y-auto pr-2">
              <!-- 预设选择 (仅新增时显示) -->
              <div v-if="!editing" class="space-y-2">
                <label class="block text-sm font-medium">{{ t('provider.preset') || '预设供应商' }}</label>
                <div class="flex flex-wrap gap-2">
                  <button
                    type="button"
                    @click="onPresetChange('自定义')"
                    :class="['px-3 py-1.5 text-sm rounded-full border-2 transition-all font-medium',
                      selectedPreset === '自定义'
                        ? 'bg-purple-600 text-white border-purple-600 shadow-sm'
                        : 'border-border text-primary hover:border-purple-400 hover:bg-surface-hover']"
                  >
                    自定义配置
                  </button>
                  <button
                    v-for="preset in flatPresets"
                    :key="preset.name"
                    type="button"
                    @click="onPresetChange(preset.name)"
                    :class="['px-3 py-1.5 text-sm rounded-full border-2 transition-all inline-flex items-center gap-1 font-medium',
                      selectedPreset === preset.name
                        ? 'bg-purple-600 text-white border-purple-600 shadow-sm'
                        : 'border-border text-primary hover:border-purple-400 hover:bg-surface-hover']"
                  >
                    {{ preset.name }}
                    <span v-if="preset.category === 'aggregator'" class="text-yellow-300 text-xs">★</span>
                  </button>
                </div>
                <div v-if="currentPreset?.apiKeyUrl" class="mt-2">
                  <button type="button" @click="openApiKeyUrl"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-gradient-to-r from-emerald-500 to-teal-500 text-white hover:from-emerald-600 hover:to-teal-600 transition-all shadow-md hover:shadow-lg">
                    <SvgIcon name="key" :size="16" />
                    <span>获取 API Key</span>
                  </button>
                </div>
              </div>

              <!-- 模型提供商 -->
              <div class="space-y-2">
                <label class="block text-sm font-medium">模型提供商</label>
                <div class="flex flex-wrap gap-2">
                  <button
                    v-for="type in MODEL_TYPES"
                    :key="type.id"
                    type="button"
                    @click="onModelTypeChange(type.id)"
                    :class="['flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-full border-2 transition-all font-medium',
                      form.model_type === type.id
                        ? 'bg-purple-600 text-white border-purple-600 shadow-sm'
                        : 'border-border text-primary hover:border-purple-400 hover:bg-surface-hover']"
                  >
                    <SvgIcon :name="type.icon" :size="18" />
                    <span>{{ type.name }}</span>
                  </button>
                </div>
              </div>

              <!-- 应用目标选择 -->
              <div class="space-y-2">
                <label class="block text-sm font-medium">应用到</label>
                <div class="flex flex-wrap gap-3">
                  <label
                    v-for="target in availableTargets"
                    :key="target.id"
                    :class="['flex items-center gap-2 px-3 py-2 rounded-lg border-2 cursor-pointer transition-all',
                      applyTargets.includes(target.id)
                        ? 'border-green-500 bg-green-500/10'
                        : 'border-border hover:border-green-400 hover:bg-surface-hover']"
                  >
                    <input
                      type="checkbox"
                      :value="target.id"
                      v-model="applyTargets"
                      class="w-4 h-4 rounded text-green-500"
                    />
                    <SvgIcon :name="target.icon" :size="18" />
                    <span class="text-sm font-medium">{{ target.label }}</span>
                  </label>
                </div>
                <p class="text-xs text-muted-foreground">选择要将此服务商配置应用到的工具</p>
              </div>

              <!-- 保存到 Open Switch 统一配置 -->
              <div class="flex items-center gap-3 p-3 rounded-lg bg-accent/5 border border-accent/20">
                <label class="flex items-center gap-2 cursor-pointer flex-1">
                  <input
                    type="checkbox"
                    v-model="saveToOpenSwitch"
                    class="w-4 h-4 rounded text-accent"
                  />
                  <div>
                    <span class="text-sm font-medium">{{ t('provider.saveToOpenSwitch') }}</span>
                    <p class="text-xs text-muted-foreground">{{ t('provider.saveToOpenSwitchDesc') }}</p>
                  </div>
                </label>
              </div>

              <!-- 名称 -->
              <div class="space-y-1.5">
                <label class="block text-sm font-medium">{{ t('provider.name') }} *</label>
                <input v-model="form.name" type="text" :placeholder="t('provider.placeholder.name')"
                  class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary" />
              </div>

              <!-- API Key -->
              <div class="space-y-1.5">
                <label class="block text-sm font-medium">{{ t('provider.apiKey') }} *</label>
                <div class="relative">
                  <input v-model="form.api_key" :type="showApiKey ? 'text' : 'password'" :placeholder="t('provider.placeholder.apiKey')"
                    class="w-full px-3 py-2 pr-16 rounded-lg border border-border bg-surface text-primary font-mono" />
                  <button type="button" @click="showApiKey = !showApiKey"
                    class="absolute right-2 top-1/2 -translate-y-1/2 px-2 py-1 text-xs text-muted-foreground hover:text-primary transition-colors">
                    {{ showApiKey ? t('provider.hideApiKey') : t('provider.showApiKey') }}
                  </button>
                </div>
              </div>

              <!-- Base URL 管理 -->
              <div class="space-y-2 border-t border-border pt-4">
                <div class="flex items-center justify-between">
                  <label class="text-sm font-medium">Base URL 列表</label>
                  <label class="flex items-center gap-1.5 cursor-pointer">
                    <input type="checkbox" v-model="autoAddV1Suffix" class="w-3.5 h-3.5 rounded text-accent-500" />
                    <span class="text-xs text-muted-foreground">自动添加 /v1 后缀</span>
                  </label>
                </div>
                
                <!-- 添加 URL -->
                <div class="flex gap-2">
                  <input v-model="newUrlInput" type="text" placeholder="输入 Base URL，如 https://api.example.com"
                    @keydown="onUrlKeydown"
                    class="flex-1 px-3 py-2 rounded-lg border border-border bg-surface text-primary text-sm font-mono" />
                  <button type="button" @click="addUrl" :disabled="!newUrlInput.trim()"
                    class="flex-shrink-0 px-4 py-2 rounded-lg bg-purple-600 text-white hover:bg-purple-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors flex items-center justify-center">
                    <SvgIcon name="plus" :size="18" />
                  </button>
                </div>

                <!-- URL 列表 -->
                <div class="space-y-2 max-h-48 overflow-y-auto">
                  <div v-for="urlConfig in baseUrls" :key="urlConfig.url"
                    :class="['p-3 rounded-lg border transition-all cursor-pointer',
                      activeBaseUrl === urlConfig.url
                        ? 'border-green-500 bg-green-500/10'
                        : 'border-border hover:border-accent-500/50']"
                    @click="setActiveUrl(urlConfig.url)">
                    <div class="flex items-center justify-between gap-2">
                      <div class="flex items-center gap-2 flex-1 min-w-0">
                        <div v-if="activeBaseUrl === urlConfig.url" class="w-2 h-2 rounded-full bg-green-500 flex-shrink-0"></div>
                        <span class="text-sm font-mono truncate">{{ urlConfig.url }}</span>
                      </div>
                      <div class="flex items-center gap-2 flex-shrink-0">
                        <span v-if="urlConfig.latency_ms !== null" :class="['text-xs font-medium', getQualityColor(urlConfig.quality)]">
                          {{ urlConfig.latency_ms }}ms · {{ getQualityLabel(urlConfig.quality) }}
                        </span>
                        <span v-else class="text-xs text-gray-400">未测试</span>
                        <button type="button" @click.stop="testSingleUrl(urlConfig.url)" :disabled="testingUrl === urlConfig.url"
                          class="p-1 hover:bg-surface-hover rounded transition-colors">
<SvgIcon v-if="testingUrl === urlConfig.url" name="loading" :size="14" class="animate-spin" />
                        <SvgIcon v-else name="activity" :size="14" />
                        </button>
                        <button v-if="baseUrls.length > 1" type="button" @click.stop="removeUrl(urlConfig.url)"
                          class="p-1 hover:bg-red-500/20 text-red-500 rounded transition-colors">
                          <SvgIcon name="close" :size="14" />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 测试按钮和自动选择开关 -->
                <div class="space-y-2">
                  <div class="flex gap-2">
                    <button type="button" @click="testAllUrls" :disabled="testing || baseUrls.length === 0"
                      class="flex-1 px-3 py-2 text-sm rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors">
                      <span v-if="testing" class="flex items-center justify-center gap-2">
                        <SvgIcon name="loading" :size="14" class="animate-spin" />
                        测试中...
                      </span>
                      <span v-else>测试所有 URL</span>
                    </button>
                  </div>
                  <!-- 自动选择最快 URL 开关 -->
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      v-model="autoSelectFastestEnabled"
                      class="w-4 h-4 rounded text-blue-500"
                    />
                    <span class="text-sm">自动选择最快 URL</span>
                  </label>
                </div>
              </div>

              <!-- 协议选择 -->
              <div class="space-y-1.5">
                <label class="block text-sm font-medium">{{ t('provider.protocol') || 'API 协议' }}</label>
                <div class="flex gap-4">
                  <label v-for="protocol in supportedProtocols" :key="protocol" class="flex items-center gap-2 cursor-pointer">
                    <input type="radio" :value="protocol" v-model="form.protocol" class="w-4 h-4 text-accent-500" />
                    <span class="text-sm">{{ protocol === 'anthropic' ? 'Anthropic 协议' : 'OpenAI 协议' }}</span>
                  </label>
                </div>
              </div>

              <!-- 描述 -->
              <div class="space-y-1.5">
                <label class="block text-sm font-medium">{{ t('provider.description') }}</label>
                <input v-model="form.description" type="text" :placeholder="t('provider.placeholder.description')"
                  class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-primary" />
              </div>
            </div>

            <!-- 右侧：模型管理 -->
            <div class="w-1/2 flex flex-col border-l border-border pl-6">
              <h4 class="text-lg font-semibold mb-2">模型管理</h4>
              
              <!-- 模型提示信息 -->
              <div v-if="modelGuidance && !editing" 
                class="mb-4 px-3 py-2 rounded-lg text-sm"
                :class="{
                  'bg-blue-500/10 text-blue-600 border border-blue-500/30': modelGuidance.type === 'info',
                  'bg-amber-500/10 text-amber-600 border border-amber-500/30': modelGuidance.type === 'required',
                  'bg-gray-500/10 text-gray-500 border border-gray-500/30': modelGuidance.type === 'optional'
                }">
                {{ modelGuidance.message }}
              </div>
              
              <!-- 编辑模式：显示已有模型 -->
              <div v-if="editing" class="flex-1 flex flex-col gap-4 min-h-0">
                <div class="flex-1 overflow-y-auto space-y-2">
                  <div v-if="existingModels.length === 0" class="text-center py-8 text-muted-foreground">
                    暂无模型
                  </div>
                  <div v-for="model in existingModels" :key="model.id"
                    class="p-3 rounded-lg border border-border hover:border-accent-500/50 transition-all flex items-center justify-between">
                    <div>
                      <div class="font-medium text-sm">{{ model.name }}</div>
                      <div class="text-xs text-muted-foreground font-mono">{{ model.id }}</div>
                    </div>
                    <button type="button" @click="openDeleteModel(model.id)"
                      class="p-1.5 hover:bg-red-500/20 text-red-500 rounded transition-colors">
                      <SvgIcon name="delete" :size="16" />
                    </button>
                  </div>
                </div>
                
                <!-- 添加模型 -->
                <div class="flex-shrink-0 space-y-2 border-t border-border pt-4">
                  <label class="block text-sm font-medium">添加模型</label>
                  <div class="flex gap-2">
                    <input v-model="customModelInput" type="text" placeholder="输入模型 ID"
                      @keydown="e => e.key === 'Enter' && addModelToExisting()"
                      class="flex-1 px-3 py-2 rounded-lg border border-border bg-surface text-primary text-sm font-mono" />
                    <button type="button" @click="addModelToExisting" :disabled="!customModelInput.trim()"
                      class="px-4 py-2 rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors">
                      添加
                    </button>
                  </div>
                </div>
              </div>

              <!-- 新增模式：选择预设模型 -->
              <div v-else class="flex-1 flex flex-col gap-4 min-h-0 overflow-y-auto">
                <!-- 自动添加预设模型 -->
                <div>
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input type="checkbox" v-model="autoAddModels" class="w-4 h-4 rounded text-accent-500" />
                    <span class="text-sm font-medium">自动添加预设模型</span>
                    <span v-if="needsModels && !autoAddModels && customModels.length === 0" 
                      class="text-xs text-amber-500">(推荐)</span>
                  </label>
                  
                  <div v-if="autoAddModels" class="mt-3 space-y-2">
                    <div class="flex items-center justify-between">
                      <span class="text-xs text-muted-foreground">
                        已选择 {{ selectedModels.length }} / {{ presetModels.length }} 个预设模型
                      </span>
                      <button type="button" @click="toggleAllModels" class="text-xs text-accent-500 hover:underline">
                        {{ selectedModels.length === presetModels.length ? '取消全选' : '全选' }}
                      </button>
                    </div>
                    <div class="grid grid-cols-2 gap-2 max-h-40 overflow-y-auto p-2 rounded-lg bg-surface">
                      <label v-for="model in presetModels" :key="model.id" class="flex items-center gap-2 cursor-pointer text-sm">
                        <input type="checkbox" :value="model.id" v-model="selectedModels" class="w-3.5 h-3.5 rounded text-accent-500" />
                        <span class="truncate" :title="model.name">{{ model.name }}</span>
                      </label>
                    </div>
                  </div>
                </div>
                
                <!-- 自定义模型添加 -->
                <div class="border-t border-border pt-4">
                  <label class="block text-sm font-medium mb-2">添加自定义模型</label>
                  <div class="flex gap-2">
                    <input v-model="customModelInput" type="text" placeholder="输入模型名称，如 gpt-4o-mini"
                      @keydown="onCustomModelKeydown"
                      class="flex-1 px-3 py-2 rounded-lg border border-border bg-surface text-primary text-sm font-mono" />
                    <button type="button" @click="addCustomModel" :disabled="!customModelInput.trim()"
                      class="px-3 py-2 rounded-lg bg-accent-500 text-white hover:bg-accent-600 disabled:opacity-50 transition-colors">
                      <SvgIcon name="plus" :size="16" />
                    </button>
                  </div>
                  
                  <div v-if="customModels.length > 0" class="mt-3">
                    <div class="flex items-center justify-between mb-2">
                      <span class="text-xs text-muted-foreground">
                        已添加 {{ customModels.length }} 个自定义模型
                      </span>
                      <button type="button" @click="customModels = []" class="text-xs text-red-500 hover:underline">
                        清空全部
                      </button>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <span v-for="model in customModels" :key="model"
                        class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-violet-500/20 text-violet-400 text-xs font-mono">
                        {{ model }}
                        <button type="button" @click="removeCustomModel(model)" class="hover:text-red-400 transition-colors">
                          <SvgIcon name="close" :size="12" />
                        </button>
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="flex-shrink-0 px-6 py-4 flex justify-end gap-3 border-t border-border">
            <button @click="close" :disabled="loading"
              class="px-6 py-2.5 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover disabled:opacity-50 transition-colors">
              {{ t('common.cancel') }}
            </button>
            <button @click="save" :disabled="loading"
              class="px-6 py-2.5 text-sm font-medium rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 disabled:opacity-50 transition-all shadow-sm">
              {{ loading ? t('common.saving') : t('common.save') }}
            </button>
          </div>
        </div>

        <!-- 删除模型确认对话框 -->
        <ConfirmDialog
          v-model:visible="showDeleteModelDialog"
          :title="t('confirm.deleteTitle')"
          :message="t('confirm.deleteModel', { name: deleteModelTarget })"
          :confirm-text="t('common.delete')"
          danger
          @confirm="confirmDeleteModel"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

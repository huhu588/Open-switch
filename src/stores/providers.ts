import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 类型定义
export interface ProviderItem {
  name: string
  base_url: string
  model_count: number
  description: string | null
  model_type: string
  enabled: boolean
}

export interface ModelItem {
  id: string
  name: string
  reasoning_effort?: string
  thinking_budget?: number | null
}

// 已部署的服务商信息
export interface DeployedProviderItem {
  name: string
  base_url: string
  model_count: number
  source: string // "global" 或 "project"
  inferred_model_type?: string // 推断的模型类型
}

export const useProvidersStore = defineStore('providers', () => {
  // 状态
  const providers = ref<ProviderItem[]>([])
  const selectedProvider = ref<string | null>(null)
  const models = ref<ModelItem[]>([])
  const selectedModel = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const currentProvider = computed(() => 
    providers.value.find(p => p.name === selectedProvider.value)
  )

  // 加载 Provider 列表
  async function loadProviders() {
    loading.value = true
    error.value = null
    try {
      providers.value = await invoke<ProviderItem[]>('get_providers')
      // 如果有 provider 且没有选中的，选中第一个
      if (providers.value.length > 0 && !selectedProvider.value) {
        selectedProvider.value = providers.value[0].name
        await loadModels()
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  // 加载 Model 列表
  async function loadModels() {
    if (!selectedProvider.value) {
      models.value = []
      return
    }
    
    try {
      models.value = await invoke<ModelItem[]>('get_models', {
        providerName: selectedProvider.value
      })
    } catch (e) {
      console.error('加载模型失败:', e)
      models.value = []
    }
  }

  // 选择 Provider
  async function selectProvider(name: string) {
    selectedProvider.value = name
    selectedModel.value = null
    await loadModels()
  }

  // 添加 Provider
  async function addProvider(input: {
    name: string
    api_key: string
    base_url: string
    npm?: string
    description?: string
  }) {
    await invoke('add_provider', { input })
    await loadProviders()
    selectedProvider.value = input.name
    await loadModels()
  }

  // 更新 Provider
  async function updateProvider(name: string, input: {
    name: string
    api_key: string
    base_url: string
    npm?: string
    description?: string
  }) {
    await invoke('update_provider', { name, input })
    await loadProviders()
  }

  // 删除 Provider
  async function deleteProvider(name: string) {
    await invoke('delete_provider', { name })
    if (selectedProvider.value === name) {
      selectedProvider.value = null
      models.value = []
    }
    await loadProviders()
  }

  // 添加 Model
  async function addModel(input: {
    id: string
    name?: string
  }) {
    if (!selectedProvider.value) return
    await invoke('add_model', {
      providerName: selectedProvider.value,
      input
    })
    await loadModels()
    await loadProviders() // 更新 model_count
  }

  // 删除 Model
  async function deleteModel(modelId: string) {
    if (!selectedProvider.value) return
    await invoke('delete_model', {
      providerName: selectedProvider.value,
      modelId: modelId
    })
    if (selectedModel.value === modelId) {
      selectedModel.value = null
    }
    await loadModels()
    await loadProviders() // 更新 model_count
  }

  // 获取站点模型
  async function fetchSiteModels(): Promise<string[]> {
    if (!selectedProvider.value) return []
    return await invoke<string[]>('fetch_site_models', {
      providerName: selectedProvider.value
    })
  }

  // 批量添加模型
  async function addModelsBatch(modelIds: string[]) {
    if (!selectedProvider.value) return
    await invoke('add_models_batch', {
      providerName: selectedProvider.value,
      modelIds: modelIds
    })
    await loadModels()
    await loadProviders()
  }

  // 应用配置
  async function applyConfig(providerNames: string[], toGlobal: boolean, toProject: boolean) {
    await invoke('apply_config', {
      input: {
        provider_names: providerNames,
        apply_to_global: toGlobal,
        apply_to_project: toProject
      }
    })
  }

  // 切换 Provider 启用状态
  async function toggleProvider(name: string, enabled: boolean) {
    await invoke('toggle_provider', { name, enabled })
    await loadProviders()
  }

  // 获取已部署到 opencode 的 Provider 列表
  async function loadDeployedProviders(): Promise<DeployedProviderItem[]> {
    return await invoke<DeployedProviderItem[]>('get_deployed_providers')
  }

  // 从已部署的 opencode 配置中删除 Provider
  async function removeDeployedProvider(name: string, fromGlobal: boolean, fromProject: boolean) {
    await invoke('remove_deployed_provider', {
      input: {
        name,
        from_global: fromGlobal,
        from_project: fromProject
      }
    })
  }

  // 导入已部署的 Provider 到管理界面
  async function importDeployedProvider(name: string, modelType: string) {
    await invoke('import_deployed_provider', {
      input: {
        name,
        model_type: modelType
      }
    })
  }

  return {
    // 状态
    providers,
    selectedProvider,
    models,
    selectedModel,
    loading,
    error,
    // 计算属性
    currentProvider,
    // 方法
    loadProviders,
    loadModels,
    selectProvider,
    addProvider,
    updateProvider,
    deleteProvider,
    addModel,
    deleteModel,
    fetchSiteModels,
    addModelsBatch,
    applyConfig,
    toggleProvider,
    loadDeployedProviders,
    removeDeployedProvider,
    importDeployedProvider,
  }
})

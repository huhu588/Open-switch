import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ============================================================================
// 类型定义
// ============================================================================

// Base URL 配置
export interface BaseUrlItem {
  url: string
  latency_ms: number | null
  last_tested: string | null
  quality: 'excellent' | 'good' | 'fair' | 'poor' | 'failed' | 'untested'
}

// Provider 列表项
export interface ProviderItem {
  name: string
  base_url: string           // 当前激活的 URL（向后兼容）
  base_urls: BaseUrlItem[]   // 所有 URL 列表
  model_count: number
  description: string | null
  model_type: string
  enabled: boolean
}

// Model 列表项
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
  api_key?: string // API Key（用于导入时自动填充）
  model_count: number // -1 表示已配置但不是模型列表（如 Claude Code/Codex/Gemini）
  source: string // "global", "project", "claude_code", "codex", "gemini"
  inferred_model_type?: string // 推断的模型类型
  tool?: string // 所属工具: "opencode", "claude_code", "codex", "gemini"
  current_model?: string // 当前使用的模型（适用于非 OpenCode 工具）
}

// URL 测试结果
export interface UrlTestResult {
  url: string
  latency_ms: number | null
  success: boolean
  quality: string
  error_message: string | null
}

// Provider URLs 测试结果
export interface ProviderUrlsTestResult {
  provider_name: string
  results: UrlTestResult[]
  fastest_url: string | null
  fastest_latency_ms: number | null
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

  // 刷新托盘菜单
  async function refreshTrayMenu() {
    try {
      await invoke('refresh_tray_menu')
    } catch (e) {
      console.warn('刷新托盘菜单失败:', e)
    }
  }

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
      // 刷新托盘菜单
      await refreshTrayMenu()
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
    model_type?: string
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
    // 刷新托盘菜单
    await refreshTrayMenu()
  }

  // 获取已部署到 opencode 的 Provider 列表
  async function loadDeployedProviders(): Promise<DeployedProviderItem[]> {
    return await invoke<DeployedProviderItem[]>('get_deployed_providers')
  }

  // 获取所有工具的已配置服务商（OpenCode + Claude Code + Codex + Gemini + cc-switch）
  async function loadAllDeployedProviders(): Promise<DeployedProviderItem[]> {
    const allProviders: DeployedProviderItem[] = []
    
    // 0. cc-switch 配置 (~/.cc-switch/config.json)
    // cc-switch 存储的服务商列表，这是主要的配置来源
    try {
      const ccSwitchProviders = await invoke<DeployedProviderItem[]>('get_cc_switch_providers')
      allProviders.push(...ccSwitchProviders)
    } catch (e) {
      console.warn('加载 cc-switch 配置失败:', e)
    }
    
    // 1. OpenCode 配置
    try {
      const opencodeProviders = await invoke<DeployedProviderItem[]>('get_deployed_providers')
      allProviders.push(...opencodeProviders.map(p => ({ ...p, tool: 'opencode' })))
    } catch (e) {
      console.warn('加载 OpenCode 配置失败:', e)
    }
    
    // 2. Claude Code 配置
    // cc-switch 存储在 ~/.claude/settings.json，env.ANTHROPIC_BASE_URL 和 model 字段
    try {
      const status = await invoke<{ is_configured: boolean; has_api_key: boolean; api_key_masked?: string }>('get_claude_code_status')
      if (status.is_configured && status.has_api_key) {
        const settings = await invoke<{ env?: Record<string, string>; model?: string }>('get_claude_code_settings')
        const baseUrl = settings.env?.['ANTHROPIC_BASE_URL'] || 'https://api.anthropic.com'
        // Claude Code 只存储当前选择的模型，不是模型列表
        // model_count 用 -1 表示"已配置"但不是模型列表
        allProviders.push({
          name: 'Claude Code',
          base_url: baseUrl,
          model_count: -1, // 特殊值表示已配置
          source: 'claude_code',
          tool: 'claude_code',
          inferred_model_type: 'claude',
          current_model: settings.model // 添加当前模型信息
        } as DeployedProviderItem & { current_model?: string })
      }
    } catch (e) {
      console.warn('加载 Claude Code 配置失败:', e)
    }
    
    // 3. Codex CLI 配置
    // cc-switch 存储在 ~/.codex/config.toml 的 [model_providers.xxx] 段
    try {
      const status = await invoke<{ is_configured: boolean; has_auth: boolean; provider_count: number }>('get_codex_status')
      if (status.is_configured && status.provider_count > 0) {
        const providers = await invoke<Record<string, { name?: string; base_url?: string }>>('get_codex_providers')
        for (const [name, provider] of Object.entries(providers)) {
          allProviders.push({
            name: name || 'Codex Provider',
            base_url: provider.base_url || 'https://api.openai.com/v1',
            model_count: -1, // 已配置
            source: 'codex',
            tool: 'codex',
            inferred_model_type: 'codex'
          })
        }
      }
    } catch (e) {
      console.warn('加载 Codex 配置失败:', e)
    }
    
    // 4. Gemini CLI 配置
    // cc-switch 存储在 ~/.gemini/.env 和 ~/.gemini/settings.json
    try {
      const status = await invoke<{ is_configured: boolean; has_api_key: boolean; base_url?: string }>('get_gemini_status')
      if (status.is_configured && status.has_api_key) {
        const settings = await invoke<{ model?: string }>('get_gemini_settings')
        allProviders.push({
          name: 'Gemini CLI',
          base_url: status.base_url || 'https://generativelanguage.googleapis.com',
          model_count: -1, // 已配置
          source: 'gemini',
          tool: 'gemini',
          inferred_model_type: 'gemini',
          current_model: settings.model
        } as DeployedProviderItem & { current_model?: string })
      }
    } catch (e) {
      console.warn('加载 Gemini 配置失败:', e)
    }
    
    return allProviders
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

  // ============================================================================
  // 多 Base URL 管理
  // ============================================================================

  // 添加 Base URL
  async function addBaseUrl(providerName: string, url: string) {
    await invoke('add_provider_base_url', {
      input: {
        provider_name: providerName,
        url
      }
    })
    await loadProviders()
  }

  // 删除 Base URL
  async function removeBaseUrl(providerName: string, url: string) {
    await invoke('remove_provider_base_url', {
      input: {
        provider_name: providerName,
        url
      }
    })
    await loadProviders()
  }

  // 设置激活的 Base URL
  async function setActiveBaseUrl(providerName: string, url: string) {
    await invoke('set_active_base_url', {
      input: {
        provider_name: providerName,
        url
      }
    })
    await loadProviders()
  }

  // 更新 URL 延迟测试结果
  async function updateUrlLatency(providerName: string, url: string, latencyMs: number | null) {
    await invoke('update_url_latency', {
      input: {
        provider_name: providerName,
        url,
        latency_ms: latencyMs
      }
    })
    await loadProviders()
  }

  // 自动选择最快的 Base URL
  async function autoSelectFastestUrl(providerName: string): Promise<string> {
    const result = await invoke<string>('auto_select_fastest_base_url', {
      providerName
    })
    await loadProviders()
    return result
  }

  // ============================================================================
  // 延迟测试
  // ============================================================================

  // 测试 Provider 的所有 URL
  async function testProviderUrls(
    providerName: string,
    urls: string[],
    apiKey: string | null,
    modelType: string
  ): Promise<ProviderUrlsTestResult> {
    return await invoke<ProviderUrlsTestResult>('test_provider_urls', {
      providerName,
      urls,
      apiKey,
      modelType,
      testCount: 3
    })
  }

  // 测试并自动选择最快的 URL
  async function testAndAutoSelectFastest(
    providerName: string,
    urls: string[],
    apiKey: string | null,
    modelType: string
  ): Promise<ProviderUrlsTestResult> {
    const result = await invoke<ProviderUrlsTestResult>('test_and_auto_select_fastest', {
      providerName,
      urls,
      apiKey,
      modelType
    })
    await loadProviders()
    return result
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
    loadAllDeployedProviders,
    removeDeployedProvider,
    importDeployedProvider,
    // 多 URL 管理
    addBaseUrl,
    removeBaseUrl,
    setActiveBaseUrl,
    updateUrlLatency,
    autoSelectFastestUrl,
    // 延迟测试
    testProviderUrls,
    testAndAutoSelectFastest,
    // 托盘菜单
    refreshTrayMenu,
  }
})

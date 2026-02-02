<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface UsageSummary {
  totalRequests: number
  totalCost: string
  totalInputTokens: number
  totalOutputTokens: number
  totalCacheCreationTokens: number
  totalCacheReadTokens: number
  successRate: number
}

interface UsageTrend {
  period: string
  requestCount: number
  totalCost: number
  inputTokens: number
  outputTokens: number
  topModel?: string | null
}

interface ModelUsage {
  model: string
  inputTokens: number
  outputTokens: number
  totalTokens: number
  totalCost: number
  requestCount: number
}

interface ModelTrendData {
  period: string
  models: ModelUsage[]
  totalTokens: number
  totalCost: number
}

interface ProxyStatus {
  running: boolean
  address: string
  port: number
  totalRequests: number
  successRequests: number
  failedRequests: number
  uptimeSeconds: number
}

interface TakeoverStatus {
  claude: boolean
  codex: boolean
  gemini: boolean
}

interface ProviderStats {
  providerId: string
  providerName: string
  requestCount: number
  totalTokens: number
  totalCost: string
  successRate: number
}

interface ScanResult {
  claudeFiles: number
  claudeEntries: number
  claudePath: string | null
  codexFiles: number
  codexEntries: number
  codexPath: string | null
  geminiFiles: number
  geminiEntries: number
  geminiPath: string | null
  opencodeFiles: number
  opencodeEntries: number
  opencodePath: string | null
  existingRecords: number
}

interface LocalLogImportResult {
  imported: number
  skipped: number
  failed: number
  total: number
}

interface ModelPricing {
  modelId: string
  displayName: string
  inputCostPerMillion: string
  outputCostPerMillion: string
  cacheReadCostPerMillion: string
  cacheCreationCostPerMillion: string
}

interface ProviderModelPricing {
  id?: number
  providerId: string
  modelId: string
  inputCostPerMillion: string
  outputCostPerMillion: string
  cacheReadCostPerMillion: string
  cacheCreationCostPerMillion: string
}

const loading = ref(false)
const period = ref<'24h' | '7d' | '30d' | 'all'>('30d')
const summary = ref<UsageSummary | null>(null)
const trend = ref<UsageTrend[]>([])
const modelTrend = ref<ModelTrendData[]>([])
const proxyStatus = ref<ProxyStatus | null>(null)
const takeoverStatus = ref<TakeoverStatus>({ claude: false, codex: false, gemini: false })
const providerStats = ref<ProviderStats[]>([])
const proxyInitialized = ref(false)

// 趋势图显示模式
const trendChartMode = ref<'daily' | 'cumulative'>('daily')

// 本地日志导入相关状态
const showImportDialog = ref(false)
const scanning = ref(false)
const importing = ref(false)
const scanResult = ref<ScanResult | null>(null)
const importResult = ref<LocalLogImportResult | null>(null)
const importClaude = ref(true)
const importCodex = ref(true)
const importGemini = ref(true)
const importOpencode = ref(true)

// 日志保留设置
const logRetention = ref<'permanent' | 'days30'>('permanent')

// 服务商筛选
const selectedProvider = ref<'all' | 'claude' | 'codex' | 'gemini' | 'opencode'>('all')

// 模型定价相关状态
const showPricingDialog = ref(false)
const pricingList = ref<ModelPricing[]>([])
const editingPricing = ref<ModelPricing | null>(null)
const loadingPricing = ref(false)

// 服务商定价相关状态
const pricingProviders = ref<string[]>([])
const selectedPricingProvider = ref<string>('')
const providerPricingList = ref<ProviderModelPricing[]>([])
const editingProviderPricing = ref<ProviderModelPricing | null>(null)
const newProviderPricing = ref<ProviderModelPricing | null>(null)

let statusInterval: number | null = null

// 预定义的模型颜色（按常见模型分配）
const modelColors: Record<string, string> = {
  // Claude 系列 - 橙色/黄色系
  'claude 4.5 opus (thinking)': '#f59e0b',
  'claude-opus-4-5-20251101': '#f59e0b',
  'Claude Opus 4.5 (thinking)': '#3b82f6',
  'claude 4.5 opus': '#ef4444',
  'claude 4.5 sonnet': '#8b5cf6',
  'Claude Sonnet 4.5': '#06b6d4',
  'claude-sonnet-4-5-20250929': '#8b5cf6',
  'claude-haiku-4-5-20251001': '#10b981',
  'claude-opus-4-20250514': '#f97316',
  'Claude Opus 4.5': '#22c55e',
  // GPT 系列 - 绿色/青色系
  'gpt-5.2 (xhigh reasoning)': '#10b981',
  'GPT-5.2 (high reasoning)': '#14b8a6',
  'gpt-5 (high reasoning)': '#f97316',
  'gpt-5.1 codex max (xhig...)': '#3b82f6',
  'gpt-5.2': '#22c55e',
  'gpt-5.2-codex': '#06b6d4',
  'gpt-5.1': '#0ea5e9',
  'gpt-5.1-codex': '#38bdf8',
  'gpt-5': '#6366f1',
  // Gemini 系列 - 蓝色系
  'gemini-3-pro-preview': '#6366f1',
  'gemini-3-flash-preview': '#818cf8',
  'gemini-2.5-pro': '#a78bfa',
  'gemini-2.5-flash': '#c4b5fd',
  // DeepSeek/Kimi 系列
  'deepseek-v3.2': '#ec4899',
  'deepseek-v3': '#f472b6',
  'kimi-k2-thinking': '#a855f7',
  'kimi-k2-0905': '#d946ef',
}

// 动态生成颜色的函数
const colorPalette = [
  '#f59e0b', '#3b82f6', '#ef4444', '#10b981', '#8b5cf6',
  '#06b6d4', '#f97316', '#22c55e', '#14b8a6', '#6366f1',
  '#ec4899', '#a855f7', '#0ea5e9', '#84cc16', '#f43f5e',
  '#8b5cf6', '#fbbf24', '#34d399', '#60a5fa', '#f472b6',
]

function getModelColor(model: string, index: number): string {
  // 先尝试精确匹配
  if (modelColors[model]) return modelColors[model]
  // 尝试部分匹配
  const lowerModel = model.toLowerCase()
  for (const [key, color] of Object.entries(modelColors)) {
    if (lowerModel.includes(key.toLowerCase()) || key.toLowerCase().includes(lowerModel)) {
      return color
    }
  }
  // 使用调色板循环
  return colorPalette[index % colorPalette.length]
}

// 获取所有出现的模型列表（按总使用量排序）
const allModels = computed(() => {
  const modelMap = new Map<string, number>()
  for (const item of modelTrend.value) {
    for (const m of item.models) {
      modelMap.set(m.model, (modelMap.get(m.model) || 0) + m.totalTokens)
    }
  }
  return Array.from(modelMap.entries())
    .sort((a, b) => b[1] - a[1])
    .map(([model]) => model)
})

// 模型颜色映射（确保同一模型在所有图表中颜色一致）
const modelColorMap = computed(() => {
  const map: Record<string, string> = {}
  allModels.value.forEach((model, index) => {
    map[model] = getModelColor(model, index)
  })
  return map
})

// 计算每日增量数据的最大值
const maxDailyTokens = computed(() => {
  if (modelTrend.value.length === 0) return 1
  return Math.max(...modelTrend.value.map(d => d.totalTokens), 1)
})

// 计算累计数据
const cumulativeTrend = computed(() => {
  const result: { period: string; models: Record<string, number>; total: number }[] = []
  const runningTotals: Record<string, number> = {}
  
  for (const item of modelTrend.value) {
    for (const m of item.models) {
      runningTotals[m.model] = (runningTotals[m.model] || 0) + m.totalTokens
    }
    const total = Object.values(runningTotals).reduce((a, b) => a + b, 0)
    result.push({
      period: item.period,
      models: { ...runningTotals },
      total,
    })
  }
  return result
})

// 累计数据的最大值
const maxCumulativeTokens = computed(() => {
  if (cumulativeTrend.value.length === 0) return 1
  return Math.max(...cumulativeTrend.value.map(d => d.total), 1)
})

// 显示的前 N 个模型（用于图例）
const topModels = computed(() => allModels.value.slice(0, 10))
const otherModelsCount = computed(() => Math.max(0, allModels.value.length - 10))
const otherModelsTokens = computed(() => {
  const topSet = new Set(topModels.value)
  let total = 0
  for (const item of modelTrend.value) {
    for (const m of item.models) {
      if (!topSet.has(m.model)) {
        total += m.totalTokens
      }
    }
  }
  return total
})

// 根据服务商筛选统计数据
const filteredProviderStats = computed(() => {
  if (selectedProvider.value === 'all') {
    return providerStats.value
  }
  const providerMap: Record<string, string[]> = {
    'claude': ['claude_local', 'Claude Code (Local)'],
    'codex': ['codex_local', 'Codex CLI (Local)'],
    'gemini': ['gemini_local', 'Gemini CLI (Local)'],
    'opencode': ['opencode_local', 'Opencode (Local)'],
  }
  const targetIds = providerMap[selectedProvider.value] || []
  return providerStats.value.filter(s => 
    targetIds.includes(s.providerId) || targetIds.includes(s.providerName)
  )
})

// 供趋势查询使用的 provider_id
function getTrendProviderId(): string | null {
  if (selectedProvider.value === 'all') return null
  const map: Record<string, string> = {
    'claude': 'claude_local',
    'codex': 'codex_local',
    'gemini': 'gemini_local',
    'opencode': 'opencode_local',
  }
  return map[selectedProvider.value] || null
}

// 格式化成本
function formatCost(cost: number | string): string {
  const num = typeof cost === 'string' ? parseFloat(cost) : cost
  return `$${num.toFixed(4)}`
}

// 格式化 token 数
function formatTokens(tokens: number): string {
  if (tokens >= 1000000) {
    return `${(tokens / 1000000).toFixed(1)}M`
  }
  if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(1)}k`
  }
  return tokens.toString()
}

// 格式化运行时间
function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  if (minutes > 0) {
    return `${minutes}m ${secs}s`
  }
  return `${secs}s`
}

// 初始化代理服务
async function initProxy() {
  try {
    await invoke('init_proxy_service')
    proxyInitialized.value = true
    await loadProxyStatus()
  } catch (e) {
    console.error('初始化代理服务失败:', e)
  }
}

// 加载代理状态
async function loadProxyStatus() {
  try {
    proxyStatus.value = await invoke<ProxyStatus>('get_proxy_status')
    takeoverStatus.value = await invoke<TakeoverStatus>('get_takeover_status')
  } catch (e) {
    console.error('加载代理状态失败:', e)
  }
}

// 加载统计数据
async function loadData() {
  loading.value = true
  try {
    const [summaryData, trendData, modelTrendData, statsData] = await Promise.all([
      invoke<UsageSummary>('get_proxy_usage_summary', { period: period.value }),
      invoke<UsageTrend[]>('get_proxy_usage_trend', { 
        period: period.value, 
        providerId: getTrendProviderId(),
      }),
      invoke<ModelTrendData[]>('get_proxy_usage_trend_by_model', {
        period: period.value,
        providerId: getTrendProviderId(),
      }),
      invoke<ProviderStats[]>('get_provider_stats', { period: period.value }),
    ])
    summary.value = summaryData
    trend.value = trendData
    modelTrend.value = modelTrendData
    providerStats.value = statsData
  } catch (e) {
    console.error('加载使用统计失败:', e)
  } finally {
    loading.value = false
  }
}

// 启动代理
async function startProxy() {
  try {
    // 获取选中的应用列表
    const apps: string[] = []
    if (takeoverStatus.value.claude) apps.push('claude')
    if (takeoverStatus.value.codex) apps.push('codex')
    if (takeoverStatus.value.gemini) apps.push('gemini')
    
    if (apps.length > 0) {
      await invoke('start_proxy_with_takeover', { apps })
    } else {
      await invoke('start_proxy')
    }
    await loadProxyStatus()
  } catch (e) {
    console.error('启动代理失败:', e)
    alert(`启动代理失败: ${e}`)
  }
}

// 停止代理
async function stopProxy() {
  try {
    await invoke('stop_proxy_with_restore')
    await loadProxyStatus()
  } catch (e) {
    console.error('停止代理失败:', e)
    alert(`停止代理失败: ${e}`)
  }
}

// 切换应用接管
async function toggleTakeover(app: 'claude' | 'codex' | 'gemini') {
  const newValue = !takeoverStatus.value[app]
  try {
    await invoke('set_takeover_for_app', { appType: app, enabled: newValue })
    await loadProxyStatus()
  } catch (e) {
    console.error('切换接管失败:', e)
    alert(`切换接管失败: ${e}`)
  }
}

// 清除统计
async function clearStats() {
  if (!confirm(t('usage.confirmClear'))) return
  try {
    await invoke('clear_proxy_usage_stats')
    await loadData()
  } catch (e) {
    console.error('清除统计失败:', e)
  }
}

// 打开导入对话框并扫描
async function openImportDialog() {
  showImportDialog.value = true
  importResult.value = null
  await scanLocalLogs()
}

// 扫描本地日志
async function scanLocalLogs() {
  scanning.value = true
  try {
    scanResult.value = await invoke<ScanResult>('scan_local_logs')
  } catch (e) {
    console.error('扫描本地日志失败:', e)
  } finally {
    scanning.value = false
  }
}

// 导入本地日志
async function importLocalLogs() {
  const sources: string[] = []
  if (importClaude.value && scanResult.value?.claudeFiles) sources.push('claude')
  if (importCodex.value && scanResult.value?.codexFiles) sources.push('codex')
  if (importGemini.value && scanResult.value?.geminiFiles) sources.push('gemini')
  if (importOpencode.value && scanResult.value?.opencodeFiles) sources.push('opencode')
  
  if (sources.length === 0) return
  
  importing.value = true
  try {
    importResult.value = await invoke<LocalLogImportResult>('import_local_logs', { sources })
    // 刷新统计数据
    await loadData()
  } catch (e) {
    console.error('导入本地日志失败:', e)
    alert(`${t('usage.importFailed')}: ${e}`)
  } finally {
    importing.value = false
  }
}

// 清除本地导入的日志
async function clearLocalLogs() {
  if (!confirm(t('usage.confirmClearLocal'))) return
  try {
    const deleted = await invoke<number>('clear_local_logs')
    alert(`${t('usage.clearedLocalLogs')}: ${deleted}`)
    await scanLocalLogs()
    await loadData()
  } catch (e) {
    console.error('清除本地日志失败:', e)
  }
}

// 关闭导入对话框
function closeImportDialog() {
  showImportDialog.value = false
  scanResult.value = null
  importResult.value = null
}

// 加载日志保留设置
async function loadLogRetention() {
  try {
    const retention = await invoke<string>('get_log_retention')
    logRetention.value = retention as 'permanent' | 'days30'
  } catch (e) {
    console.error('加载日志保留设置失败:', e)
  }
}

// 设置日志保留策略
async function setLogRetention(retention: 'permanent' | 'days30') {
  try {
    await invoke('set_log_retention', { retention })
    logRetention.value = retention
    
    // 如果设置为 30 天，立即清理过期日志
    if (retention === 'days30') {
      const cleaned = await invoke<number>('cleanup_old_logs')
      if (cleaned > 0) {
        await loadData()
      }
    }
  } catch (e) {
    console.error('设置日志保留失败:', e)
  }
}

// 打开模型定价对话框
async function openPricingDialog() {
  showPricingDialog.value = true
  selectedPricingProvider.value = ''
  providerPricingList.value = []
  await Promise.all([
    loadPricingList(),
    loadPricingProviders(),
  ])
}

// 加载模型定价列表
async function loadPricingList() {
  loadingPricing.value = true
  try {
    pricingList.value = await invoke<ModelPricing[]>('get_model_pricing_list')
  } catch (e) {
    console.error('加载模型定价失败:', e)
  } finally {
    loadingPricing.value = false
  }
}

// 编辑模型定价
function editPricing(pricing: ModelPricing) {
  editingPricing.value = { ...pricing }
}

// 保存模型定价
async function savePricing() {
  if (!editingPricing.value) return
  
  try {
    await invoke('update_model_pricing', {
      modelId: editingPricing.value.modelId,
      inputCost: editingPricing.value.inputCostPerMillion,
      outputCost: editingPricing.value.outputCostPerMillion,
      cacheReadCost: editingPricing.value.cacheReadCostPerMillion,
      cacheCreationCost: editingPricing.value.cacheCreationCostPerMillion,
    })
    editingPricing.value = null
    await loadPricingList()
  } catch (e) {
    console.error('保存模型定价失败:', e)
    alert(`保存失败: ${e}`)
  }
}

// 取消编辑
function cancelEditPricing() {
  editingPricing.value = null
}

// 重置模型定价为默认值
async function resetPricing() {
  if (!confirm(t('usage.confirmResetPricing'))) return
  
  try {
    await invoke('reset_model_pricing')
    await loadPricingList()
  } catch (e) {
    console.error('重置模型定价失败:', e)
    alert(`重置失败: ${e}`)
  }
}

// 关闭模型定价对话框
function closePricingDialog() {
  showPricingDialog.value = false
  editingPricing.value = null
  selectedPricingProvider.value = ''
  providerPricingList.value = []
  editingProviderPricing.value = null
  newProviderPricing.value = null
}

// 加载服务商列表
async function loadPricingProviders() {
  try {
    pricingProviders.value = await invoke<string[]>('get_pricing_providers')
  } catch (e) {
    console.error('加载服务商列表失败:', e)
  }
}

// 选择服务商
async function selectPricingProvider(providerId: string) {
  selectedPricingProvider.value = providerId
  editingProviderPricing.value = null
  newProviderPricing.value = null
  await loadProviderPricingList(providerId)
}

// 加载服务商定价列表
async function loadProviderPricingList(providerId: string) {
  try {
    providerPricingList.value = await invoke<ProviderModelPricing[]>('get_provider_model_pricing', { providerId })
  } catch (e) {
    console.error('加载服务商定价失败:', e)
  }
}

// 开始添加服务商定价
function startAddProviderPricing() {
  newProviderPricing.value = {
    providerId: selectedPricingProvider.value,
    modelId: '',
    inputCostPerMillion: '0',
    outputCostPerMillion: '0',
    cacheReadCostPerMillion: '0',
    cacheCreationCostPerMillion: '0',
  }
}

// 编辑服务商定价
function editProviderPricing(pricing: ProviderModelPricing) {
  editingProviderPricing.value = { ...pricing }
}

// 保存服务商定价
async function saveProviderPricing() {
  const pricing = editingProviderPricing.value || newProviderPricing.value
  if (!pricing) return
  
  try {
    await invoke('set_provider_model_pricing', {
      providerId: pricing.providerId,
      modelId: pricing.modelId,
      inputCost: pricing.inputCostPerMillion,
      outputCost: pricing.outputCostPerMillion,
      cacheReadCost: pricing.cacheReadCostPerMillion,
      cacheCreationCost: pricing.cacheCreationCostPerMillion,
    })
    editingProviderPricing.value = null
    newProviderPricing.value = null
    await loadProviderPricingList(selectedPricingProvider.value)
    await loadPricingProviders()
  } catch (e) {
    console.error('保存服务商定价失败:', e)
    alert(`保存失败: ${e}`)
  }
}

// 取消编辑服务商定价
function cancelEditProviderPricing() {
  editingProviderPricing.value = null
  newProviderPricing.value = null
}

// 删除服务商定价
async function deleteProviderPricing(pricing: ProviderModelPricing) {
  if (!confirm(t('usage.confirmDeletePricing'))) return
  
  try {
    await invoke('delete_provider_model_pricing', {
      providerId: pricing.providerId,
      modelId: pricing.modelId,
    })
    await loadProviderPricingList(selectedPricingProvider.value)
    await loadPricingProviders()
  } catch (e) {
    console.error('删除服务商定价失败:', e)
    alert(`删除失败: ${e}`)
  }
}

// 获取服务商显示名称
function getProviderDisplayName(providerId: string): string {
  const map: Record<string, string> = {
    'claude_local': 'Claude Code (Local)',
    'codex_local': 'Codex CLI (Local)',
    'gemini_local': 'Gemini CLI (Local)',
    'opencode_local': 'Opencode (Local)',
  }
  return map[providerId] || providerId
}

// 监听周期变化
watch(period, () => {
  loadData()
})

// 监听服务商筛选变化（同步趋势）
watch(selectedProvider, () => {
  loadData()
})

onMounted(async () => {
  await initProxy()
  await loadData()
  await loadLogRetention()
  
  // 启动时清理过期日志
  try {
    await invoke('cleanup_old_logs')
  } catch (e) {
    console.error('清理过期日志失败:', e)
  }
  
  // 定期刷新代理状态
  statusInterval = window.setInterval(() => {
    if (proxyStatus.value?.running) {
      loadProxyStatus()
    }
  }, 5000)
})

onUnmounted(() => {
  if (statusInterval) {
    clearInterval(statusInterval)
  }
})
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
          <SvgIcon name="activity" class="w-6 h-6 text-blue-500" />
        </div>
        <div>
          <h1 class="text-xl font-semibold">{{ t('usage.title') }}</h1>
          <p class="text-sm text-gray-500">{{ t('usage.description') }}</p>
        </div>
      </div>
      
      <!-- 时间周期选择 -->
      <div class="flex items-center gap-2">
        <div class="flex rounded-lg bg-surface border border-border overflow-hidden">
          <button
            v-for="p in (['24h', '7d', '30d', 'all'] as const)"
            :key="p"
            @click="period = p"
            :class="[
              'px-4 py-1.5 text-sm font-medium transition-colors',
              period === p 
                ? 'bg-blue-500 text-white' 
                : 'text-muted-foreground hover:bg-surface-hover'
            ]"
          >
            {{ p === '24h' ? t('usage.period24h') : p === '7d' ? t('usage.period7d') : p === '30d' ? t('usage.period30d') : t('usage.periodAll') }}
          </button>
        </div>
      </div>
    </div>

    <!-- 代理控制面板 -->
    <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
          <div :class="[
            'w-3 h-3 rounded-full',
            proxyStatus?.running ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
          ]"></div>
          <h3 class="font-semibold">{{ t('usage.proxyControl') }}</h3>
          <span v-if="proxyStatus?.running" class="text-xs text-gray-500">
            {{ proxyStatus.address }}:{{ proxyStatus.port }} | 
            {{ t('usage.uptime') }}: {{ formatUptime(proxyStatus.uptimeSeconds) }}
          </span>
        </div>
        
        <button
          @click="proxyStatus?.running ? stopProxy() : startProxy()"
          :class="[
            'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
            proxyStatus?.running 
              ? 'bg-red-500 hover:bg-red-600 text-white' 
              : 'bg-green-500 hover:bg-green-600 text-white'
          ]"
        >
          {{ proxyStatus?.running ? t('usage.stopProxy') : t('usage.startProxy') }}
        </button>
      </div>
      
      <!-- 接管设置 -->
      <div class="flex items-center gap-4">
        <span class="text-sm text-gray-500">{{ t('usage.takeover') }}:</span>
        
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            :checked="takeoverStatus.claude"
            @change="toggleTakeover('claude')"
            :disabled="proxyStatus?.running"
            class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
          />
          <span class="text-sm">Claude Code</span>
        </label>
        
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            :checked="takeoverStatus.codex"
            @change="toggleTakeover('codex')"
            :disabled="proxyStatus?.running"
            class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
          />
          <span class="text-sm">Codex</span>
        </label>
        
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            :checked="takeoverStatus.gemini"
            @change="toggleTakeover('gemini')"
            :disabled="proxyStatus?.running"
            class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
          />
          <span class="text-sm">Gemini CLI</span>
        </label>
      </div>
      
      <!-- 实时统计 -->
      <div v-if="proxyStatus?.running" class="flex items-center gap-6 mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <div class="text-center">
          <p class="text-2xl font-bold text-blue-500">{{ proxyStatus.totalRequests }}</p>
          <p class="text-xs text-gray-500">{{ t('usage.totalRequests') }}</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-green-500">{{ proxyStatus.successRequests }}</p>
          <p class="text-xs text-gray-500">{{ t('usage.success') }}</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-red-500">{{ proxyStatus.failedRequests }}</p>
          <p class="text-xs text-gray-500">{{ t('usage.failed') }}</p>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- 总请求数 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.totalRequests') }}</span>
          <div class="w-8 h-8 rounded-lg bg-violet-500/10 flex items-center justify-center">
            <SvgIcon name="activity" class="w-4 h-4 text-violet-500" />
          </div>
        </div>
        <p class="text-2xl font-bold">{{ summary?.totalRequests || 0 }}</p>
        <p class="text-xs text-gray-500 mt-1">
          {{ t('usage.successRate') }}: {{ (summary?.successRate || 0).toFixed(1) }}%
        </p>
      </div>

      <!-- 总成本 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.totalCost') }}</span>
          <div class="w-8 h-8 rounded-lg bg-green-500/10 flex items-center justify-center">
            <span class="text-green-500 font-bold">$</span>
          </div>
        </div>
        <p class="text-2xl font-bold">{{ formatCost(summary?.totalCost || 0) }}</p>
      </div>

      <!-- 总 Token 数 -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.totalTokens') }}</span>
          <div class="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center">
            <SvgIcon name="layers" class="w-4 h-4 text-blue-500" />
          </div>
        </div>
        <p class="text-2xl font-bold">{{ formatTokens((summary?.totalInputTokens || 0) + (summary?.totalOutputTokens || 0)) }}</p>
        <div class="flex gap-4 mt-1 text-xs text-gray-500">
          <span>Input: {{ formatTokens(summary?.totalInputTokens || 0) }}</span>
          <span>Output: {{ formatTokens(summary?.totalOutputTokens || 0) }}</span>
        </div>
      </div>

      <!-- 缓存 Token -->
      <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-gray-500">{{ t('usage.cacheTokens') }}</span>
          <div class="w-8 h-8 rounded-lg bg-orange-500/10 flex items-center justify-center">
            <SvgIcon name="save" class="w-4 h-4 text-orange-500" />
          </div>
        </div>
        <p class="text-2xl font-bold">{{ formatTokens((summary?.totalCacheCreationTokens || 0) + (summary?.totalCacheReadTokens || 0)) }}</p>
        <div class="flex gap-4 mt-1 text-xs text-gray-500">
          <span>{{ t('usage.cacheCreation') }}: {{ formatTokens(summary?.totalCacheCreationTokens || 0) }}</span>
          <span>{{ t('usage.cacheHit') }}: {{ formatTokens(summary?.totalCacheReadTokens || 0) }}</span>
        </div>
      </div>
    </div>

    <!-- 服务商统计 -->
    <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-semibold">{{ t('usage.byProvider') }}</h3>
        <!-- 服务商筛选器 -->
        <div class="flex rounded-lg bg-gray-100 dark:bg-gray-700 overflow-hidden text-sm">
          <button
            v-for="provider in [
              { id: 'all', label: t('usage.allProviders') },
              { id: 'claude', label: 'Claude' },
              { id: 'codex', label: 'Codex' },
              { id: 'gemini', label: 'Gemini' },
              { id: 'opencode', label: 'Opencode' },
            ]"
            :key="provider.id"
            @click="selectedProvider = provider.id as any"
            :class="[
              'px-3 py-1 transition-colors',
              selectedProvider === provider.id
                ? 'bg-blue-500 text-white'
                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
            ]"
          >
            {{ provider.label }}
          </button>
        </div>
      </div>
      <div v-if="filteredProviderStats.length > 0" class="space-y-3">
        <div 
          v-for="stat in filteredProviderStats" 
          :key="stat.providerId"
          class="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50"
        >
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
              <span class="text-blue-500 font-bold text-sm">{{ stat.providerName.charAt(0).toUpperCase() }}</span>
            </div>
            <div>
              <p class="font-medium">{{ stat.providerName }}</p>
              <p class="text-xs text-gray-500">{{ stat.requestCount }} requests</p>
            </div>
          </div>
          <div class="text-right">
            <p class="font-bold">{{ formatCost(stat.totalCost) }}</p>
            <p class="text-xs text-gray-500">{{ formatTokens(stat.totalTokens) }} tokens</p>
          </div>
        </div>
      </div>
      <div v-else class="py-8 text-center text-gray-400">
        {{ t('usage.noProviderData') }}
      </div>
    </div>

    <!-- 模型 Token 用量趋势图 -->
    <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-semibold">{{ t('usage.modelTokenUsage') }}</h3>
        <div class="flex items-center gap-4">
          <!-- 图表模式切换 -->
          <div class="flex rounded-lg bg-gray-100 dark:bg-gray-700 overflow-hidden text-sm">
            <button
              @click="trendChartMode = 'daily'"
              :class="[
                'px-3 py-1 transition-colors',
                trendChartMode === 'daily'
                  ? 'bg-blue-500 text-white'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
              ]"
            >
              {{ t('usage.dailyUsage') }}
            </button>
            <button
              @click="trendChartMode = 'cumulative'"
              :class="[
                'px-3 py-1 transition-colors',
                trendChartMode === 'cumulative'
                  ? 'bg-blue-500 text-white'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
              ]"
            >
              {{ t('usage.cumulativeUsage') }}
            </button>
          </div>
          <span class="text-sm text-gray-500">
            {{ period === '24h' ? t('usage.past24h') : period === '7d' ? t('usage.past7d') : period === '30d' ? t('usage.past30d') : t('usage.pastAll') }}
          </span>
        </div>
      </div>
      
      <!-- 堆叠柱状图 - 每日增量 -->
      <div v-if="modelTrend.length > 0 && trendChartMode === 'daily'" class="h-64">
        <div class="h-full flex items-end gap-1">
          <div
            v-for="(item, index) in modelTrend"
            :key="index"
            class="flex-1 min-w-0 h-full flex flex-col items-center justify-end"
          >
            <!-- 堆叠柱子 -->
            <div 
              class="w-full flex flex-col-reverse relative group cursor-pointer"
              :style="{ height: `${Math.max((item.totalTokens / maxDailyTokens) * 100, 2)}%` }"
            >
              <!-- 每个模型的柱子段 -->
              <div
                v-for="(modelUsage, mIndex) in item.models"
                :key="modelUsage.model"
                class="w-full transition-opacity hover:opacity-80"
                :style="{
                  height: item.totalTokens > 0 ? `${(modelUsage.totalTokens / item.totalTokens) * 100}%` : '0%',
                  backgroundColor: modelColorMap[modelUsage.model] || colorPalette[mIndex % colorPalette.length],
                  minHeight: modelUsage.totalTokens > 0 ? '2px' : '0',
                }"
              ></div>
              
              <!-- Tooltip -->
              <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 bg-gray-900 text-white text-xs rounded-lg opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-20 pointer-events-none shadow-xl max-w-xs">
                <div class="font-semibold mb-1 border-b border-gray-700 pb-1">{{ item.period.split('-').slice(1).join('/') }}</div>
                <div v-for="m in item.models.slice(0, 5)" :key="m.model" class="flex items-center gap-2 py-0.5">
                  <span 
                    class="w-2 h-2 rounded-full flex-shrink-0" 
                    :style="{ backgroundColor: modelColorMap[m.model] }"
                  ></span>
                  <span class="truncate max-w-[120px]">{{ m.model }}</span>
                  <span class="font-medium ml-auto">{{ formatTokens(m.totalTokens) }}</span>
                </div>
                <div v-if="item.models.length > 5" class="text-gray-400 pt-1 border-t border-gray-700 mt-1">
                  + {{ item.models.length - 5 }} {{ t('usage.moreModels') }}
                </div>
                <div class="font-semibold pt-1 border-t border-gray-700 mt-1">
                  {{ t('usage.total') }}: {{ formatTokens(item.totalTokens) }}
                </div>
              </div>
            </div>
            <!-- 日期标签 -->
            <span 
              v-if="index % Math.ceil(modelTrend.length / 12) === 0 || index === modelTrend.length - 1"
              class="text-[10px] text-gray-500 mt-1 truncate w-full text-center"
            >
              {{ item.period.split('-').slice(1).join('/') }}
            </span>
          </div>
        </div>
        <!-- Y 轴标签 -->
        <div class="flex justify-between text-xs text-gray-400 mt-2">
          <span>0</span>
          <span>{{ formatTokens(maxDailyTokens) }}</span>
        </div>
      </div>

      <!-- 堆叠柱状图 - 累计 -->
      <div v-else-if="cumulativeTrend.length > 0 && trendChartMode === 'cumulative'" class="h-64">
        <div class="h-full flex items-end gap-1">
          <div
            v-for="(item, index) in cumulativeTrend"
            :key="index"
            class="flex-1 min-w-0 h-full flex flex-col items-center justify-end"
          >
            <!-- 堆叠柱子 -->
            <div 
              class="w-full flex flex-col-reverse relative group cursor-pointer"
              :style="{ height: `${Math.max((item.total / maxCumulativeTokens) * 100, 2)}%` }"
            >
              <!-- 每个模型的柱子段 -->
              <div
                v-for="(model, mIndex) in topModels"
                :key="model"
                class="w-full transition-opacity hover:opacity-80"
                :style="{
                  height: item.total > 0 && item.models[model] ? `${(item.models[model] / item.total) * 100}%` : '0%',
                  backgroundColor: modelColorMap[model] || colorPalette[mIndex % colorPalette.length],
                  minHeight: item.models[model] ? '2px' : '0',
                }"
              ></div>
              
              <!-- Tooltip -->
              <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 bg-gray-900 text-white text-xs rounded-lg opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-20 pointer-events-none shadow-xl max-w-xs">
                <div class="font-semibold mb-1 border-b border-gray-700 pb-1">{{ item.period.split('-').slice(1).join('/') }}</div>
                <div v-for="model in topModels.filter(m => item.models[m])" :key="model" class="flex items-center gap-2 py-0.5">
                  <span 
                    class="w-2 h-2 rounded-full flex-shrink-0" 
                    :style="{ backgroundColor: modelColorMap[model] }"
                  ></span>
                  <span class="truncate max-w-[120px]">{{ model }}</span>
                  <span class="font-medium ml-auto">{{ formatTokens(item.models[model]) }}</span>
                </div>
                <div class="font-semibold pt-1 border-t border-gray-700 mt-1">
                  {{ t('usage.total') }}: {{ formatTokens(item.total) }}
                </div>
              </div>
            </div>
            <!-- 日期标签 -->
            <span 
              v-if="index % Math.ceil(cumulativeTrend.length / 12) === 0 || index === cumulativeTrend.length - 1"
              class="text-[10px] text-gray-500 mt-1 truncate w-full text-center"
            >
              {{ item.period.split('-').slice(1).join('/') }}
            </span>
          </div>
        </div>
        <!-- Y 轴标签 -->
        <div class="flex justify-between text-xs text-gray-400 mt-2">
          <span>0</span>
          <span>{{ formatTokens(maxCumulativeTokens) }}</span>
        </div>
      </div>
      
      <!-- 空状态 -->
      <div v-else class="h-64 flex items-center justify-center text-gray-400">
        {{ t('usage.noData') }}
      </div>

      <!-- 模型图例 -->
      <div v-if="topModels.length > 0" class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <div class="flex flex-wrap gap-x-4 gap-y-2 text-xs">
          <div 
            v-for="(model, index) in topModels" 
            :key="model"
            class="flex items-center gap-1.5"
          >
            <span class="text-gray-500">{{ index + 1 }}.</span>
            <span 
              class="w-3 h-3 rounded-sm flex-shrink-0" 
              :style="{ backgroundColor: modelColorMap[model] }"
            ></span>
            <span class="text-gray-700 dark:text-gray-300 truncate max-w-[150px]" :title="model">{{ model }}</span>
          </div>
          <div v-if="otherModelsCount > 0" class="flex items-center gap-1.5 text-gray-400">
            <span>+ {{ otherModelsCount }} {{ t('usage.otherModels') }} ({{ formatTokens(otherModelsTokens) }})</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 设置区域 -->
    <div class="p-4 rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 space-y-4">
      <!-- 日志保留设置 -->
      <div class="flex items-center justify-between">
        <div>
          <h3 class="font-medium text-sm">{{ t('usage.logRetention') }}</h3>
          <p class="text-xs text-gray-500 mt-1">{{ t('usage.logRetentionDesc') }}</p>
        </div>
        <div class="flex rounded-lg bg-surface border border-border overflow-hidden">
          <button
            @click="setLogRetention('permanent')"
            :class="[
              'px-3 py-1.5 text-sm font-medium transition-colors',
              logRetention === 'permanent' 
                ? 'bg-blue-500 text-white' 
                : 'text-muted-foreground hover:bg-surface-hover'
            ]"
          >
            {{ t('usage.retentionPermanent') }}
          </button>
          <button
            @click="setLogRetention('days30')"
            :class="[
              'px-3 py-1.5 text-sm font-medium transition-colors',
              logRetention === 'days30' 
                ? 'bg-blue-500 text-white' 
                : 'text-muted-foreground hover:bg-surface-hover'
            ]"
          >
            {{ t('usage.retention30Days') }}
          </button>
        </div>
      </div>
      
      <!-- 模型定价设置 -->
      <div class="flex items-center justify-between border-t border-gray-200 dark:border-gray-700 pt-4">
        <div>
          <h3 class="font-medium text-sm">{{ t('usage.modelPricing') }}</h3>
          <p class="text-xs text-gray-500 mt-1">{{ t('usage.modelPricingDesc') }}</p>
        </div>
        <button
          @click="openPricingDialog"
          class="px-4 py-1.5 text-sm font-medium bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
        >
          {{ t('usage.editPricing') }}
        </button>
      </div>
    </div>

    <!-- 底部操作 -->
    <div class="flex justify-between">
      <button
        @click="openImportDialog"
        class="px-4 py-2 text-sm text-blue-500 hover:bg-blue-500/10 rounded-lg transition-colors flex items-center gap-2"
      >
        <SvgIcon name="download" class="w-4 h-4" />
        {{ t('usage.importLocalLogs') }}
      </button>
      <button
        @click="clearStats"
        class="px-4 py-2 text-sm text-red-500 hover:bg-red-500/10 rounded-lg transition-colors"
      >
        {{ t('usage.clearStats') }}
      </button>
    </div>

    <!-- 本地日志导入对话框 -->
    <div v-if="showImportDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-xl p-6 w-full max-w-md mx-4 shadow-xl">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold">{{ t('usage.importLocalLogs') }}</h2>
          <button @click="closeImportDialog" class="text-gray-400 hover:text-gray-600">
            <SvgIcon name="x" class="w-5 h-5" />
          </button>
        </div>

        <!-- 扫描中 -->
        <div v-if="scanning" class="py-8 text-center text-gray-500">
          <div class="animate-spin w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full mx-auto mb-3"></div>
          {{ t('usage.scanning') }}
        </div>

        <!-- 扫描结果 -->
        <div v-else-if="scanResult && !importResult" class="space-y-4">
          <!-- Claude Code -->
          <div class="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50">
            <label class="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                v-model="importClaude"
                :disabled="!scanResult.claudeFiles || importing"
                class="mt-1 w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
              />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">Claude Code</span>
                  <span v-if="scanResult.claudeFiles" class="text-xs px-2 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300 rounded">
                    {{ scanResult.claudeFiles }} {{ t('usage.files') }}
                  </span>
                  <span v-else class="text-xs text-gray-400">{{ t('usage.notFound') }}</span>
                </div>
                <p v-if="scanResult.claudePath" class="text-xs text-gray-500 mt-1 break-all">
                  {{ scanResult.claudePath }}
                </p>
                <p v-if="scanResult.claudeEntries" class="text-xs text-gray-500">
                  ~{{ scanResult.claudeEntries }} {{ t('usage.entries') }}
                </p>
              </div>
            </label>
          </div>

          <!-- Codex CLI -->
          <div class="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50">
            <label class="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                v-model="importCodex"
                :disabled="!scanResult.codexFiles || importing"
                class="mt-1 w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
              />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">Codex CLI</span>
                  <span v-if="scanResult.codexFiles" class="text-xs px-2 py-0.5 bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300 rounded">
                    {{ scanResult.codexFiles }} {{ t('usage.files') }}
                  </span>
                  <span v-else class="text-xs text-gray-400">{{ t('usage.notFound') }}</span>
                </div>
                <p v-if="scanResult.codexPath" class="text-xs text-gray-500 mt-1 break-all">
                  {{ scanResult.codexPath }}
                </p>
                <p v-if="scanResult.codexEntries" class="text-xs text-gray-500">
                  ~{{ scanResult.codexEntries }} {{ t('usage.entries') }}
                </p>
              </div>
            </label>
          </div>

          <!-- Gemini CLI -->
          <div class="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50">
            <label class="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                v-model="importGemini"
                :disabled="!scanResult.geminiFiles || importing"
                class="mt-1 w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
              />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">Gemini CLI</span>
                  <span v-if="scanResult.geminiFiles" class="text-xs px-2 py-0.5 bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300 rounded">
                    {{ scanResult.geminiFiles }} {{ t('usage.files') }}
                  </span>
                  <span v-else class="text-xs text-gray-400">{{ t('usage.notFound') }}</span>
                </div>
                <p v-if="scanResult.geminiPath" class="text-xs text-gray-500 mt-1 break-all">
                  {{ scanResult.geminiPath }}
                </p>
                <p v-if="scanResult.geminiEntries" class="text-xs text-gray-500">
                  ~{{ scanResult.geminiEntries }} {{ t('usage.entries') }}
                </p>
              </div>
            </label>
          </div>

          <!-- Opencode -->
          <div class="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50">
            <label class="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                v-model="importOpencode"
                :disabled="!scanResult.opencodeFiles || importing"
                class="mt-1 w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
              />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">Opencode</span>
                  <span v-if="scanResult.opencodeFiles" class="text-xs px-2 py-0.5 bg-teal-100 dark:bg-teal-900 text-teal-600 dark:text-teal-300 rounded">
                    {{ scanResult.opencodeFiles }} {{ t('usage.files') }}
                  </span>
                  <span v-else class="text-xs text-gray-400">{{ t('usage.notFound') }}</span>
                </div>
                <p v-if="scanResult.opencodePath" class="text-xs text-gray-500 mt-1 break-all">
                  {{ scanResult.opencodePath }}
                </p>
                <p v-if="scanResult.opencodeEntries" class="text-xs text-gray-500">
                  ~{{ scanResult.opencodeEntries }} {{ t('usage.entries') }}
                </p>
              </div>
            </label>
          </div>

          <!-- 已导入记录提示 -->
          <div v-if="scanResult.existingRecords > 0" class="text-xs text-gray-500 px-1">
            {{ t('usage.existingRecords') }}: {{ scanResult.existingRecords }}
          </div>

          <!-- 无可导入数据 -->
          <div v-if="!scanResult.claudeFiles && !scanResult.codexFiles && !scanResult.geminiFiles && !scanResult.opencodeFiles" class="text-center py-4 text-gray-400">
            {{ t('usage.noLogsFound') }}
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-3 pt-2">
            <button
              v-if="scanResult.existingRecords > 0"
              @click="clearLocalLogs"
              :disabled="importing"
              class="px-4 py-2 text-sm text-red-500 hover:bg-red-500/10 rounded-lg transition-colors"
            >
              {{ t('usage.clearLocalLogs') }}
            </button>
            <div class="flex-1"></div>
            <button
              @click="closeImportDialog"
              :disabled="importing"
              class="px-4 py-2 text-sm text-gray-500 hover:bg-gray-500/10 rounded-lg transition-colors"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="importLocalLogs"
              :disabled="importing || (!importClaude && !importCodex && !importGemini && !importOpencode) || (!scanResult.claudeFiles && !scanResult.codexFiles && !scanResult.geminiFiles && !scanResult.opencodeFiles)"
              class="px-4 py-2 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              <div v-if="importing" class="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full"></div>
              {{ importing ? t('usage.importing') : t('usage.import') }}
            </button>
          </div>
        </div>

        <!-- 导入结果 -->
        <div v-else-if="importResult" class="space-y-4">
          <div class="text-center py-4">
            <div class="w-16 h-16 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
              <SvgIcon name="check" class="w-8 h-8 text-green-500" />
            </div>
            <h3 class="text-lg font-semibold mb-2">{{ t('usage.importComplete') }}</h3>
          </div>

          <div class="grid grid-cols-2 gap-3 text-center">
            <div class="p-3 rounded-lg bg-green-50 dark:bg-green-900/20">
              <p class="text-2xl font-bold text-green-500">{{ importResult.imported }}</p>
              <p class="text-xs text-gray-500">{{ t('usage.imported') }}</p>
            </div>
            <div class="p-3 rounded-lg bg-yellow-50 dark:bg-yellow-900/20">
              <p class="text-2xl font-bold text-yellow-500">{{ importResult.skipped }}</p>
              <p class="text-xs text-gray-500">{{ t('usage.skipped') }}</p>
            </div>
          </div>

          <div v-if="importResult.failed > 0" class="text-center text-xs text-red-500">
            {{ t('usage.failedEntries') }}: {{ importResult.failed }}
          </div>

          <button
            @click="closeImportDialog"
            class="w-full px-4 py-2 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
          >
            {{ t('common.done') }}
          </button>
        </div>
      </div>
    </div>

    <!-- 模型定价对话框 -->
    <div v-if="showPricingDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-xl p-6 w-full max-w-3xl mx-4 shadow-xl max-h-[85vh] flex flex-col">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold">{{ t('usage.modelPricing') }}</h2>
          <button @click="closePricingDialog" class="text-gray-400 hover:text-gray-600">
            <SvgIcon name="x" class="w-5 h-5" />
          </button>
        </div>

        <!-- 加载中 -->
        <div v-if="loadingPricing" class="py-8 text-center text-gray-500">
          <div class="animate-spin w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full mx-auto mb-3"></div>
          {{ t('common.loading') }}
        </div>

        <div v-else class="flex-1 flex flex-col overflow-hidden">
          <!-- 服务商选择区域 -->
          <div class="mb-4">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 block">{{ t('usage.selectProvider') }}</label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="provider in pricingProviders"
                :key="provider"
                @click="selectPricingProvider(provider)"
                :class="[
                  'px-3 py-1.5 text-sm rounded-lg transition-colors',
                  selectedPricingProvider === provider 
                    ? 'bg-blue-500 text-white' 
                    : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                ]"
              >
                {{ getProviderDisplayName(provider) }}
              </button>
            </div>
          </div>

          <!-- 服务商特定定价 -->
          <div v-if="selectedPricingProvider" class="flex-1 overflow-y-auto border-t border-gray-200 dark:border-gray-700 pt-4">
            <div class="flex items-center justify-between mb-3">
              <h3 class="font-medium text-sm">{{ getProviderDisplayName(selectedPricingProvider) }} {{ t('usage.customPricing') }}</h3>
              <button
                v-if="!newProviderPricing"
                @click="startAddProviderPricing"
                class="px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded"
              >
                {{ t('common.add') }}
              </button>
            </div>
            
            <!-- 新增定价表单 -->
            <div v-if="newProviderPricing" class="p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 mb-3">
              <div class="space-y-3">
                <div>
                  <label class="text-xs text-gray-500">{{ t('usage.modelId') }}</label>
                  <select
                    v-model="newProviderPricing.modelId"
                    class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600"
                  >
                    <option value="">{{ t('usage.selectModel') }}</option>
                    <option v-for="model in pricingList" :key="model.modelId" :value="model.modelId">
                      {{ model.displayName }} ({{ model.modelId }})
                    </option>
                  </select>
                </div>
                <div class="grid grid-cols-2 gap-2">
                  <div>
                    <label class="text-xs text-gray-500">{{ t('usage.inputCost') }}</label>
                    <input v-model="newProviderPricing.inputCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                  </div>
                  <div>
                    <label class="text-xs text-gray-500">{{ t('usage.outputCost') }}</label>
                    <input v-model="newProviderPricing.outputCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                  </div>
                  <div>
                    <label class="text-xs text-gray-500">{{ t('usage.cacheReadCost') }}</label>
                    <input v-model="newProviderPricing.cacheReadCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                  </div>
                  <div>
                    <label class="text-xs text-gray-500">{{ t('usage.cacheCreationCost') }}</label>
                    <input v-model="newProviderPricing.cacheCreationCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                  </div>
                </div>
                <div class="flex justify-end gap-2">
                  <button @click="cancelEditProviderPricing" class="px-3 py-1 text-sm text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-600 rounded">{{ t('common.cancel') }}</button>
                  <button @click="saveProviderPricing" :disabled="!newProviderPricing.modelId" class="px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded disabled:opacity-50">{{ t('common.save') }}</button>
                </div>
              </div>
            </div>

            <!-- 已有定价列表 -->
            <div class="space-y-2">
              <div v-if="providerPricingList.length === 0" class="text-center py-4 text-gray-400 text-sm">
                {{ t('usage.noCustomPricing') }}
              </div>
              <div v-for="pricing in providerPricingList" :key="`${pricing.providerId}-${pricing.modelId}`" class="p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50">
                <!-- 编辑模式 -->
                <div v-if="editingProviderPricing?.modelId === pricing.modelId" class="space-y-3">
                  <div class="font-medium text-sm">{{ pricing.modelId }}</div>
                  <div class="grid grid-cols-2 gap-2">
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.inputCost') }}</label>
                      <input v-model="editingProviderPricing.inputCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.outputCost') }}</label>
                      <input v-model="editingProviderPricing.outputCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.cacheReadCost') }}</label>
                      <input v-model="editingProviderPricing.cacheReadCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.cacheCreationCost') }}</label>
                      <input v-model="editingProviderPricing.cacheCreationCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                  </div>
                  <div class="flex justify-end gap-2">
                    <button @click="cancelEditProviderPricing" class="px-3 py-1 text-sm text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-600 rounded">{{ t('common.cancel') }}</button>
                    <button @click="saveProviderPricing" class="px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded">{{ t('common.save') }}</button>
                  </div>
                </div>
                <!-- 显示模式 -->
                <div v-else class="flex items-center justify-between">
                  <div>
                    <div class="font-medium text-sm">{{ pricing.modelId }}</div>
                    <div class="text-xs text-gray-500 mt-1">
                      {{ t('usage.input') }}: ${{ pricing.inputCostPerMillion }}/M | {{ t('usage.output') }}: ${{ pricing.outputCostPerMillion }}/M
                    </div>
                  </div>
                  <div class="flex gap-2">
                    <button @click="editProviderPricing(pricing)" class="px-3 py-1 text-sm text-blue-500 hover:bg-blue-500/10 rounded">{{ t('common.edit') }}</button>
                    <button @click="deleteProviderPricing(pricing)" class="px-3 py-1 text-sm text-red-500 hover:bg-red-500/10 rounded">{{ t('common.delete') }}</button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 默认定价说明 -->
          <div v-else class="flex-1 overflow-y-auto border-t border-gray-200 dark:border-gray-700 pt-4">
            <p class="text-sm text-gray-500 mb-4">{{ t('usage.selectProviderHint') }}</p>
            
            <!-- 默认定价列表（只读显示） -->
            <h3 class="font-medium text-sm mb-3">{{ t('usage.defaultPricing') }}</h3>
            <div class="space-y-2">
              <div v-for="pricing in pricingList" :key="pricing.modelId" class="p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50">
                <!-- 编辑模式 -->
                <div v-if="editingPricing?.modelId === pricing.modelId" class="space-y-3">
                  <div class="font-medium text-sm">{{ pricing.displayName }}</div>
                  <div class="grid grid-cols-2 gap-2">
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.inputCost') }}</label>
                      <input v-model="editingPricing.inputCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.outputCost') }}</label>
                      <input v-model="editingPricing.outputCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.cacheReadCost') }}</label>
                      <input v-model="editingPricing.cacheReadCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                    <div>
                      <label class="text-xs text-gray-500">{{ t('usage.cacheCreationCost') }}</label>
                      <input v-model="editingPricing.cacheCreationCostPerMillion" type="text" class="w-full px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600" />
                    </div>
                  </div>
                  <div class="flex justify-end gap-2">
                    <button @click="cancelEditPricing" class="px-3 py-1 text-sm text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-600 rounded">{{ t('common.cancel') }}</button>
                    <button @click="savePricing" class="px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded">{{ t('common.save') }}</button>
                  </div>
                </div>
                <!-- 显示模式 -->
                <div v-else class="flex items-center justify-between">
                  <div>
                    <div class="font-medium text-sm">{{ pricing.displayName }}</div>
                    <div class="text-xs text-gray-500 mt-1">
                      {{ t('usage.input') }}: ${{ pricing.inputCostPerMillion }}/M | {{ t('usage.output') }}: ${{ pricing.outputCostPerMillion }}/M
                    </div>
                  </div>
                  <button @click="editPricing(pricing)" class="px-3 py-1 text-sm text-blue-500 hover:bg-blue-500/10 rounded">{{ t('common.edit') }}</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 底部操作 -->
        <div class="flex justify-between pt-4 border-t border-gray-200 dark:border-gray-700 mt-4">
          <button
            @click="resetPricing"
            class="px-4 py-2 text-sm text-red-500 hover:bg-red-500/10 rounded-lg transition-colors"
          >
            {{ t('usage.resetPricing') }}
          </button>
          <button
            @click="closePricingDialog"
            class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded-lg transition-colors"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

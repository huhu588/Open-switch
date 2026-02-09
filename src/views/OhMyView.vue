<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/SvgIcon.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const { t } = useI18n()

// 类型定义
interface OhMyVersionInfo {
  current_version: string | null
  latest_version: string | null
  has_update: boolean
}

interface OhMyStatus {
  bun_installed: boolean
  bun_version: string | null
  npm_installed: boolean
  ohmy_installed: boolean
  config: OhMyConfig | null
  version_info: OhMyVersionInfo | null
}

interface OhMyConfig {
  agents: Record<string, { model: string }>
}

interface AvailableModel {
  provider_name: string
  model_id: string
  display_name: string
}

interface AgentInfo {
  key: string
  name: string
  description: string
  usage: string | null
}

// 状态
const loading = ref(true)
const installing = ref(false)
const uninstalling = ref(false)
const updating = ref(false)
const installLog = ref('')
const status = ref<OhMyStatus | null>(null)
const availableModels = ref<AvailableModel[]>([])
const agentInfos = ref<AgentInfo[]>([])

// Agent 模型选择（key -> model display_name）
const agentModels = ref<Record<string, string>>({})

// 消息提示
const message = ref('')
const messageType = ref<'success' | 'error'>('success')

// 卸载确认对话框
const showUninstallConfirm = ref(false)

// Agent 图标映射
const agentIcons: Record<string, string> = {
  'Sisyphus': 'robot',
  'oracle': 'puzzle',
  'librarian': 'book',
  'explore': 'search',
  'frontend-ui-ux-engineer': 'layers',
  'document-writer': 'file-text',
  'multimodal-looker': 'eye'
}

// 检查是否有用户配置的模型（非 OpenCode Zen 内置模型）
const hasUserConfiguredModels = computed(() => {
  return availableModels.value.some(m => m.provider_name !== 'OpenCode Zen')
})

// 分组模型：免费模型 vs 用户配置的模型
const freeModels = computed(() => {
  return availableModels.value.filter(m => m.provider_name === 'OpenCode Zen')
})

const userModels = computed(() => {
  return availableModels.value.filter(m => m.provider_name !== 'OpenCode Zen')
})

// 默认模型选择逻辑
const defaultModel = computed(() => {
  // 如果有用户配置的模型，优先使用用户配置的
  if (hasUserConfiguredModels.value) {
    // 优先查找 i7 开头的供应商的 claude-4.5-opus
    const i7Model = availableModels.value.find(m => 
      m.provider_name.toLowerCase().includes('i7') && 
      m.model_id.toLowerCase().includes('claude-4.5-opus')
    )
    if (i7Model) return i7Model.display_name
    
    // 次优先：任何供应商的 claude-4.5-opus
    const opusModel = availableModels.value.find(m => 
      m.provider_name !== 'OpenCode Zen' &&
      m.model_id.toLowerCase().includes('claude-4.5-opus')
    )
    if (opusModel) return opusModel.display_name
    
    // 再次：任何用户配置的 claude 模型
    const claudeModel = availableModels.value.find(m => 
      m.provider_name !== 'OpenCode Zen' &&
      m.model_id.toLowerCase().includes('claude')
    )
    if (claudeModel) return claudeModel.display_name
    
    // 再次：任何用户配置的模型
    const userModel = availableModels.value.find(m => m.provider_name !== 'OpenCode Zen')
    if (userModel) return userModel.display_name
  }
  
  // 没有用户配置时，默认使用 GLM-4.7（OpenCode Zen 免费模型）
  const glmModel = availableModels.value.find(m => 
    m.provider_name === 'OpenCode Zen' && 
    m.model_id === 'glm-4.7'
  )
  if (glmModel) return glmModel.display_name
  
  // 兜底：第一个模型
  return availableModels.value[0]?.display_name || ''
})

// 初始化 Agent 模型选择
function initAgentModels() {
  const models: Record<string, string> = {}
  
  for (const agent of agentInfos.value) {
    // 如果已有配置，使用已有配置
    if (status.value?.config?.agents?.[agent.key]?.model) {
      models[agent.key] = status.value.config.agents[agent.key].model
    } else {
      // 否则使用默认模型
      models[agent.key] = defaultModel.value
    }
  }
  
  agentModels.value = models
}

// 加载状态
async function loadStatus() {
  loading.value = true
  try {
    // 并行加载所有数据
    const [statusResult, modelsResult, agentsResult] = await Promise.all([
      invoke<OhMyStatus>('check_ohmy_status'),
      invoke<AvailableModel[]>('get_available_models'),
      invoke<AgentInfo[]>('get_agent_infos')
    ])
    
    status.value = statusResult
    availableModels.value = modelsResult
    agentInfos.value = agentsResult
    
    // 加载成功后清空日志（仅失败时保留）
    if (!installLog.value.includes('❌')) {
      installLog.value = ''
    }
    
    // 初始化模型选择
    initAgentModels()
  } catch (e) {
    console.error('加载状态失败:', e)
    showMessage(t('ohmy.loadFailed'), 'error')
  } finally {
    loading.value = false
  }
}

// 一键安装并配置
async function installAndConfigure() {
  installing.value = true
  installLog.value = t('ohmy.startingInstall') + '\n'
  
  try {
    await invoke<string>('install_and_configure', {
      agents: agentModels.value
    })
    // 成功时清空日志，不需要显示
    installLog.value = ''
    showMessage(t('ohmy.installSuccess'), 'success')
    
    // 重新加载状态
    await loadStatus()
  } catch (e) {
    console.error('安装失败:', e)
    // 失败时保留日志并附加错误信息
    installLog.value = installLog.value + '\n❌ ' + String(e)
    showMessage(t('ohmy.installFailed'), 'error')
  } finally {
    installing.value = false
  }
}

// 保存配置（已安装时使用）
async function saveConfig() {
  try {
    await invoke('save_ohmy_config', {
      agents: agentModels.value
    })
    showMessage(t('ohmy.saved'), 'success')
  } catch (e) {
    console.error('保存失败:', e)
    showMessage(t('ohmy.saveFailed'), 'error')
  }
}

// 显示消息
function showMessage(msg: string, type: 'success' | 'error') {
  message.value = msg
  messageType.value = type
  setTimeout(() => { message.value = '' }, 3000)
}

// 为所有 Agent 设置相同的模型
function setAllAgentsModel(model: string) {
  for (const agent of agentInfos.value) {
    agentModels.value[agent.key] = model
  }
}

// 卸载 oh-my-opencode
async function doUninstall() {
  uninstalling.value = true
  installLog.value = ''
  
  try {
    await invoke<string>('uninstall_ohmy')
    // 成功时清空日志
    installLog.value = ''
    showMessage(t('ohmy.uninstallSuccess'), 'success')
    
    // 重新加载状态
    await loadStatus()
  } catch (e) {
    console.error('卸载失败:', e)
    // 失败时显示错误
    installLog.value = '❌ ' + String(e)
    showMessage(t('ohmy.uninstallFailed'), 'error')
  } finally {
    uninstalling.value = false
  }
}

// 更新 oh-my-opencode
async function updateOhmy() {
  updating.value = true
  installLog.value = t('ohmy.startingUpdate') + '\n'
  
  try {
    await invoke<string>('update_ohmy')
    // 成功时清空日志
    installLog.value = ''
    showMessage(t('ohmy.updateSuccess'), 'success')
    
    // 重新加载状态
    await loadStatus()
  } catch (e) {
    console.error('更新失败:', e)
    // 失败时保留日志
    installLog.value = installLog.value + '\n❌ ' + String(e)
    showMessage(t('ohmy.updateFailed'), 'error')
  } finally {
    updating.value = false
  }
}

onMounted(() => {
  loadStatus()
})
</script>

<template>
  <ConfirmDialog
    v-model:visible="showUninstallConfirm"
    :title="t('ohmy.uninstallTitle', t('confirm.title'))"
    :message="t('ohmy.confirmUninstall')"
    :confirm-text="t('ohmy.uninstall', t('common.confirm'))"
    danger
    @confirm="doUninstall"
  />
  <div class="h-full flex flex-col gap-4 overflow-auto">
    <!-- 标题区域 -->
    <div class="flex items-center justify-between flex-shrink-0">
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 text-white shadow-lg">
          <SvgIcon name="robot" :size="24" />
        </div>
        <div>
          <h1 class="text-xl font-bold">{{ t('ohmy.title') }}</h1>
          <p class="text-xs text-muted-foreground">{{ t('ohmy.subtitle') }}</p>
        </div>
      </div>
      
      <!-- 右侧：操作按钮和消息提示 -->
      <div class="flex items-center gap-3">
        <!-- 消息提示 -->
        <div 
          v-if="message" 
          class="text-sm px-4 py-2 rounded-lg animate-pulse"
          :class="messageType === 'success' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'"
        >
          {{ message }}
        </div>
        
        <!-- 刷新按钮 -->
        <button
          v-if="!loading"
          @click="loadStatus"
          class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm font-medium transition-all flex items-center gap-2"
        >
          <SvgIcon name="refresh" :size="16" />
          {{ t('ohmy.refresh') }}
        </button>
        
        <!-- 安装/保存按钮 -->
        <button
          v-if="!loading && status?.ohmy_installed"
          @click="saveConfig"
          class="px-6 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all flex items-center gap-2"
        >
          <SvgIcon name="save" :size="16" />
          {{ t('ohmy.saveConfig') }}
        </button>
        
        <button
          v-if="!loading && !status?.ohmy_installed"
          @click="installAndConfigure"
          :disabled="installing || availableModels.length === 0"
          class="px-6 py-2 rounded-lg bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white text-sm font-medium transition-all flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <SvgIcon v-if="installing" name="refresh" :size="16" class="animate-spin" />
          <SvgIcon v-else name="download" :size="16" />
          {{ installing ? t('ohmy.installing') : t('ohmy.installAndConfigure') }}
        </button>
      </div>
    </div>
    
    <!-- 加载状态 -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-2 border-accent border-t-transparent mb-3"></div>
        <p class="text-muted-foreground">{{ t('common.loading') }}</p>
      </div>
    </div>
    
    <!-- 主内容 -->
    <div v-else class="flex-1 space-y-6">
      <!-- 状态提示 -->
      <div class="rounded-xl bg-surface/50 border border-border p-4">
        <div class="flex items-center gap-4 flex-wrap">
          <!-- Bun 状态 -->
          <div class="flex items-center gap-2">
            <div 
              class="w-3 h-3 rounded-full"
              :class="status?.bun_installed ? 'bg-emerald-500' : 'bg-yellow-500'"
            ></div>
            <span class="text-sm">
              Bun: {{ status?.bun_installed ? (status?.bun_version || t('ohmy.installed')) : t('ohmy.notInstalled') }}
            </span>
          </div>
          
          <!-- npm 状态 -->
          <div class="flex items-center gap-2">
            <div 
              class="w-3 h-3 rounded-full"
              :class="status?.npm_installed ? 'bg-emerald-500' : 'bg-yellow-500'"
            ></div>
            <span class="text-sm">
              npm: {{ status?.npm_installed ? t('ohmy.installed') : t('ohmy.notInstalled') }}
            </span>
          </div>
          
          <!-- oh-my-opencode 状态 -->
          <div class="flex items-center gap-2">
            <div 
              class="w-3 h-3 rounded-full"
              :class="status?.ohmy_installed ? 'bg-emerald-500' : 'bg-yellow-500'"
            ></div>
            <span class="text-sm">
              oh-my-opencode: {{ status?.ohmy_installed ? t('ohmy.installed') : t('ohmy.notInstalled') }}
            </span>
          </div>
          
          <!-- 版本信息 -->
          <div v-if="status?.version_info?.current_version" class="flex items-center gap-2">
            <span class="text-sm text-muted-foreground">
              v{{ status.version_info.current_version }}
            </span>
            <!-- 有更新时显示更新提示 -->
            <template v-if="status.version_info.has_update && status.version_info.latest_version">
              <span class="text-xs text-yellow-500">→ v{{ status.version_info.latest_version }}</span>
              <button
                @click="updateOhmy"
                :disabled="updating"
                class="px-2 py-1 rounded text-xs bg-accent hover:bg-accent/90 text-white font-medium transition-all flex items-center gap-1 disabled:opacity-50"
              >
                <SvgIcon v-if="updating" name="refresh" :size="12" class="animate-spin" />
                <SvgIcon v-else name="download" :size="12" />
                {{ updating ? t('ohmy.updating') : t('ohmy.update') }}
              </button>
            </template>
            <!-- 已是最新版本 -->
            <span v-else-if="status.version_info.latest_version && !status.version_info.has_update" class="text-xs text-emerald-500">
              ✓ {{ t('ohmy.latestVersion') }}
            </span>
          </div>
        </div>
      </div>
      
      <!-- 安装日志（移到状态下方，更显眼） -->
      <div v-if="installLog" class="rounded-xl border p-4" :class="installLog.includes('❌') ? 'bg-red-500/10 border-red-500/30' : 'bg-background border-border'">
        <h4 class="text-sm font-medium mb-2 flex items-center gap-2">
          <SvgIcon :name="installLog.includes('❌') ? 'warning' : 'file-text'" :size="16" />
          {{ t('ohmy.installLog') }}
        </h4>
        <pre class="text-xs font-mono whitespace-pre-wrap max-h-60 overflow-auto" :class="installLog.includes('❌') ? 'text-red-400' : 'text-muted-foreground'">{{ installLog }}</pre>
      </div>
      
      <!-- 快速设置（全部使用同一个模型） -->
      <div v-if="availableModels.length > 0" class="rounded-xl bg-surface/30 border border-border p-4">
        <div class="flex items-center justify-between gap-4">
          <div>
            <h3 class="font-medium">{{ t('ohmy.quickSet') }}</h3>
            <p class="text-xs text-muted-foreground">{{ t('ohmy.quickSetDesc') }}</p>
          </div>
          <select
            @change="(e) => setAllAgentsModel((e.target as HTMLSelectElement).value)"
            class="px-4 py-2 rounded-lg bg-background border border-border text-sm focus:outline-none focus:border-accent min-w-[280px]"
          >
            <option value="" disabled selected>{{ t('ohmy.selectModelForAll') }}</option>
            <!-- 用户配置的模型（优先显示） -->
            <optgroup v-if="userModels.length > 0" :label="t('ohmy.yourModels')">
              <option v-for="model in userModels" :key="model.display_name" :value="model.display_name">
                {{ model.display_name }}
              </option>
            </optgroup>
            <!-- OpenCode Zen 免费模型 -->
            <optgroup :label="t('ohmy.freeModels')">
              <option v-for="model in freeModels" :key="model.display_name" :value="model.display_name">
                {{ model.display_name }}
              </option>
            </optgroup>
          </select>
        </div>
      </div>
      
      <!-- Agent 配置卡片网格 -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div
          v-for="agent in agentInfos"
          :key="agent.key"
          class="rounded-xl bg-surface/30 border border-border p-5 hover:border-accent/50 transition-all"
        >
          <!-- Agent 头部 -->
          <div class="flex items-start gap-3 mb-4">
            <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/10 text-accent">
              <SvgIcon :name="agentIcons[agent.key] || 'robot'" :size="20" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-semibold">{{ agent.name }}</h3>
              <p class="text-sm text-muted-foreground">{{ agent.description }}</p>
            </div>
          </div>
          
          <!-- 用法示例 -->
          <div v-if="agent.usage" class="mb-4 p-3 rounded-lg bg-background/50 border border-border/50">
            <p class="text-xs text-muted-foreground">
              <span class="text-accent font-medium">{{ t('ohmy.usage') }}:</span>
              {{ agent.usage }}
            </p>
          </div>
          
          <!-- 模型选择 -->
          <div>
            <label class="text-xs text-muted-foreground mb-1.5 block">{{ t('ohmy.selectModel') }}</label>
            <select
              v-model="agentModels[agent.key]"
              class="w-full px-3 py-2 rounded-lg bg-background border border-border text-sm focus:outline-none focus:border-accent"
            >
              <!-- 用户配置的模型（优先显示） -->
              <optgroup v-if="userModels.length > 0" :label="t('ohmy.yourModels')">
                <option v-for="model in userModels" :key="model.display_name" :value="model.display_name">
                  {{ model.display_name }}
                </option>
              </optgroup>
              <!-- OpenCode Zen 免费模型 -->
              <optgroup :label="t('ohmy.freeModels')">
                <option v-for="model in freeModels" :key="model.display_name" :value="model.display_name">
                  {{ model.display_name }}
                </option>
              </optgroup>
            </select>
          </div>
        </div>
      </div>
      
      <!-- 无可用模型提示 -->
      <div v-if="availableModels.length === 0" class="rounded-xl bg-yellow-500/10 border border-yellow-500/30 p-4">
        <div class="flex items-start gap-3">
          <SvgIcon name="warning" :size="20" class="text-yellow-500 flex-shrink-0 mt-0.5" />
          <div>
            <h4 class="font-medium text-yellow-500">{{ t('ohmy.noModels') }}</h4>
            <p class="text-sm text-muted-foreground mt-1">{{ t('ohmy.noModelsDesc') }}</p>
          </div>
        </div>
      </div>
      
      <!-- 卸载按钮（仅已安装时显示） -->
      <div v-if="status?.ohmy_installed" class="pt-4 border-t border-border">
        <button
          @click="showUninstallConfirm = true"
          :disabled="uninstalling"
          class="px-4 py-2 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 text-sm font-medium transition-all flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <SvgIcon v-if="uninstalling" name="refresh" :size="16" class="animate-spin" />
          <SvgIcon v-else name="trash" :size="16" />
          {{ uninstalling ? t('ohmy.uninstalling') : t('ohmy.uninstall') }}
        </button>
      </div>
    </div>
  </div>
</template>

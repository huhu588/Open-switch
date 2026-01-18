<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()

interface McpServer {
  name: string
  server_type: string
  enabled: boolean
  url: string | null
  command: string[] | null
}

interface McpHealthResult {
  healthy: boolean
  message: string
}

interface RecommendedMcpServer {
  name: string
  description: string
  command: string[]
  url: string
}

interface AddRecommendedResult {
  added: string[]
  skipped: string[]
}


const servers = ref<McpServer[]>([])
const selectedServer = ref<string | null>(null)
const loading = ref(false)
const healthStatus = ref<Record<string, McpHealthResult | null>>({})

// 推荐MCP相关
const showRecommendedModal = ref(false)
const recommendedServers = ref<RecommendedMcpServer[]>([])
const selectedRecommended = ref<Set<string>>(new Set())
const addingRecommended = ref(false)

// 消息提示
const installMessage = ref('')

// 自定义MCP相关
const showCustomModal = ref(false)
const customName = ref('')
const customJson = ref(`{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-xxx"],
  "type": "stdio"
}`)
const customError = ref('')
const addingCustom = ref(false)


async function loadServers() {
  loading.value = true
  try {
    servers.value = await invoke<McpServer[]>('get_mcp_servers')
    if (servers.value.length > 0 && !selectedServer.value) {
      selectedServer.value = servers.value[0].name
    }
    // 检查所有服务器健康状态
    for (const server of servers.value) {
      checkServerHealth(server.name)
    }
  } catch (e) {
    console.error('加载 MCP 服务器失败:', e)
  } finally {
    loading.value = false
  }
}

// 检查单个服务器健康状态
async function checkServerHealth(name: string) {
  try {
    const result = await invoke<McpHealthResult>('check_mcp_server_health', { name })
    healthStatus.value[name] = result
  } catch (e) {
    healthStatus.value[name] = { healthy: false, message: '检查失败' }
  }
}

async function toggleServer(name: string) {
  try {
    await invoke('toggle_mcp_server', { name })
    await loadServers()
  } catch (e) {
    console.error('切换状态失败:', e)
  }
}

// 已安装的推荐MCP
const installedRecommended = ref<Set<string>>(new Set())

// 加载推荐MCP服务器列表
async function loadRecommendedServers() {
  try {
    recommendedServers.value = await invoke<RecommendedMcpServer[]>('get_recommended_mcp_servers')
    
    // 检查哪些已安装
    const existingNames = new Set(servers.value.map(s => s.name))
    installedRecommended.value = new Set(
      recommendedServers.value.filter(r => existingNames.has(r.name)).map(r => r.name)
    )
    
    // 默认选中未安装的
    selectedRecommended.value = new Set(
      recommendedServers.value.filter(r => !existingNames.has(r.name)).map(r => r.name)
    )
  } catch (e) {
    console.error('加载推荐MCP失败:', e)
  }
}

// 打开推荐MCP弹窗
async function openRecommendedModal() {
  await loadRecommendedServers()
  showRecommendedModal.value = true
}

// 切换选中状态
function toggleRecommendedSelect(name: string) {
  if (selectedRecommended.value.has(name)) {
    selectedRecommended.value.delete(name)
  } else {
    selectedRecommended.value.add(name)
  }
  // 触发响应式更新
  selectedRecommended.value = new Set(selectedRecommended.value)
}

// 添加选中的推荐MCP
async function addSelectedRecommended() {
  if (selectedRecommended.value.size === 0) return
  
  addingRecommended.value = true
  try {
    const result = await invoke<AddRecommendedResult>('add_recommended_mcp_servers', {
      serverNames: Array.from(selectedRecommended.value)
    })
    
    // 显示结果消息
    if (result.added.length > 0) {
      installMessage.value = t('mcp.serverAdded', { count: result.added.length })
    }
    if (result.skipped.length > 0) {
      installMessage.value += ' ' + t('mcp.serverSkipped', { count: result.skipped.length })
    }
    
    showRecommendedModal.value = false
    await loadServers()
    
    // 3秒后清除消息
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e) {
    console.error('添加推荐MCP失败:', e)
  } finally {
    addingRecommended.value = false
  }
}

const selectedCount = computed(() => selectedRecommended.value.size)

// 打开自定义MCP弹窗
function openCustomModal() {
  customName.value = ''
  customJson.value = `{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-xxx"],
  "type": "stdio"
}`
  customError.value = ''
  showCustomModal.value = true
}

// 添加自定义MCP
async function addCustomMcp() {
  customError.value = ''
  
  // 验证名称
  if (!customName.value.trim()) {
    customError.value = t('mcp.customNameRequired')
    return
  }
  
  // 解析JSON
  let config: any
  try {
    config = JSON.parse(customJson.value)
  } catch (e) {
    customError.value = t('mcp.customJsonInvalid')
    return
  }
  
  // 构建命令数组
  const command: string[] = []
  if (config.command) {
    command.push(config.command)
  }
  if (config.args && Array.isArray(config.args)) {
    command.push(...config.args)
  }
  
  if (command.length === 0) {
    customError.value = t('mcp.customCommandRequired')
    return
  }
  
  addingCustom.value = true
  try {
    await invoke('add_mcp_server', {
      input: {
        name: customName.value.trim(),
        server_type: 'local',
        enabled: true,
        command: command,
        environment: config.env || {},
        timeout: null,
        url: null,
        headers: null,
        oauth: null
      }
    })
    
    showCustomModal.value = false
    installMessage.value = t('mcp.customAdded', { name: customName.value })
    await loadServers()
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e: any) {
    customError.value = e.toString()
  } finally {
    addingCustom.value = false
  }
}

onMounted(() => {
  loadServers()
})

const currentServer = () => servers.value.find(s => s.name === selectedServer.value)
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- 顶部工具栏 -->
    <div class="flex items-center justify-between flex-shrink-0">
      <div class="flex items-center gap-3">
        <!-- 添加推荐MCP按钮 -->
        <button
          @click="openRecommendedModal"
          class="px-4 py-2 rounded-lg bg-accent/20 hover:bg-accent/30 text-accent font-medium text-sm transition-all flex items-center gap-2"
        >
          <span>✨</span>
          {{ t('mcp.addRecommended') }}
        </button>
        
        <!-- 自定义添加MCP按钮 -->
        <button
          @click="openCustomModal"
          class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground font-medium text-sm transition-all flex items-center gap-2"
        >
          <span>➕</span>
          {{ t('mcp.addCustom') }}
        </button>
      </div>
      
      <!-- 操作结果消息 -->
      <div v-if="installMessage" class="text-sm text-accent animate-pulse">
        {{ installMessage }}
      </div>
    </div>
    
    <!-- 主内容区 -->
    <div class="flex-1 flex gap-4 min-h-0">
      <!-- 服务器列表 -->
      <div class="w-72 flex-shrink-0">
        <div class="h-full flex flex-col rounded-xl bg-surface/30 border border-border overflow-hidden">
          <div class="flex items-center justify-between px-4 py-3 border-b border-border">
            <h3 class="font-semibold text-sm">{{ t('mcp.title') }}</h3>
            <span class="text-xs text-muted-foreground">({{ servers.length }})</span>
          </div>

          <div class="flex-1 overflow-auto">
            <div v-if="loading" class="p-4 text-center text-muted-foreground">
              {{ t('mcp.loading') }}
            </div>
            <div v-else-if="servers.length === 0" class="p-4 text-center text-muted-foreground">
              {{ t('mcp.noServers') }}
            </div>
            <ul v-else class="p-2 space-y-1">
              <li
                v-for="server in servers"
                :key="server.name"
                @click="selectedServer = server.name"
                class="px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
                :class="[
                  server.name === selectedServer
                    ? 'bg-accent/10 border border-accent/40'
                    : 'hover:bg-surface-hover border border-transparent'
                ]"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <!-- 健康状态指示器 -->
                    <span 
                      class="w-2 h-2 rounded-full flex-shrink-0"
                      :class="healthStatus[server.name]?.healthy ? 'bg-emerald-500' : 'bg-amber-500'"
                      :title="healthStatus[server.name]?.message || '检查中...'"
                    ></span>
                    <span>{{ server.server_type === 'local' ? '📦' : '🌐' }}</span>
                    <span class="font-medium text-sm truncate">{{ server.name }}</span>
                  </div>
                  <button
                    @click.stop="toggleServer(server.name)"
                    class="text-xs px-2 py-0.5 rounded"
                    :class="server.enabled ? 'bg-emerald-500/20 text-emerald-500' : 'bg-surface text-muted-foreground'"
                  >
                    {{ server.enabled ? t('mcp.enabled') : t('mcp.disabled') }}
                  </button>
                </div>
                <div class="mt-1 text-xs text-muted-foreground truncate">
                  {{ server.server_type === 'local' ? server.command?.join(' ') : server.url }}
                </div>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- 详情面板 -->
      <div class="flex-1">
        <div class="h-full rounded-xl bg-surface/30 border border-border p-4">
          <div v-if="!currentServer()" class="text-center text-muted-foreground py-8">
            {{ t('mcp.selectServer') }}
          </div>
          <div v-else class="space-y-4">
            <h3 class="font-semibold text-lg">{{ currentServer()?.name }}</h3>
            <div class="space-y-2 text-sm">
              <div class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.type') }}</span>
                <span>{{ currentServer()?.server_type === 'local' ? t('mcp.local') : t('mcp.remote') }}</span>
              </div>
              <div class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.status') }}</span>
                <span :class="currentServer()?.enabled ? 'text-emerald-500' : 'text-muted-foreground'">
                  {{ currentServer()?.enabled ? t('mcp.statusEnabled') : t('mcp.statusDisabled') }}
                </span>
              </div>
              <div v-if="currentServer()?.command" class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.command') }}</span>
                <span class="font-mono text-xs">{{ currentServer()?.command?.join(' ') }}</span>
              </div>
              <div v-if="currentServer()?.url" class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.url') }}</span>
                <span class="font-mono text-xs">{{ currentServer()?.url }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 推荐MCP弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showRecommendedModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showRecommendedModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[560px] max-h-[80vh] overflow-hidden shadow-2xl">
          <!-- 弹窗头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold">{{ t('mcp.recommended') }}</h2>
            <button 
              @click="showRecommendedModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              ✕
            </button>
          </div>
          
          <!-- 服务器列表 -->
          <div class="p-4 space-y-3 max-h-[50vh] overflow-auto">
            <div
              v-for="server in recommendedServers"
              :key="server.name"
              @click="!installedRecommended.has(server.name) && toggleRecommendedSelect(server.name)"
              class="p-4 rounded-xl border transition-all"
              :class="[
                installedRecommended.has(server.name)
                  ? 'border-emerald-500/30 bg-emerald-500/5 cursor-default'
                  : selectedRecommended.has(server.name) 
                    ? 'border-accent bg-accent/10 cursor-pointer' 
                    : 'border-border hover:border-accent/50 bg-surface/30 cursor-pointer'
              ]"
            >
              <div class="flex items-start gap-3">
                <!-- 选中指示器 -->
                <div 
                  v-if="!installedRecommended.has(server.name)"
                  class="w-5 h-5 rounded-md border-2 flex items-center justify-center flex-shrink-0 mt-0.5 transition-all"
                  :class="selectedRecommended.has(server.name) 
                    ? 'border-accent bg-accent text-white' 
                    : 'border-muted-foreground'"
                >
                  <span v-if="selectedRecommended.has(server.name)" class="text-xs">✓</span>
                </div>
                <!-- 已安装标记 -->
                <div 
                  v-else
                  class="w-5 h-5 rounded-md bg-emerald-500/20 flex items-center justify-center flex-shrink-0 mt-0.5"
                >
                  <span class="text-xs text-emerald-500">✓</span>
                </div>
                
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-semibold">{{ server.name }}</span>
                    <span v-if="installedRecommended.has(server.name)" class="text-xs px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-500">已安装</span>
                    <a 
                      :href="server.url" 
                      target="_blank"
                      @click.stop
                      class="text-xs text-accent hover:underline"
                    >
                      {{ t('mcp.visitSite') }} ↗
                    </a>
                  </div>
                  <p class="text-sm text-muted-foreground mt-1">{{ server.description }}</p>
                  <code class="text-xs text-muted-foreground/70 mt-2 block font-mono">
                    {{ server.command.join(' ') }}
                  </code>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 弹窗底部 -->
          <div class="px-6 py-4 border-t border-border flex items-center justify-between">
            <span class="text-sm text-muted-foreground">
              {{ t('mcp.addSelected', { count: selectedCount }) }}
            </span>
            <div class="flex gap-2">
              <button
                @click="showRecommendedModal = false"
                class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
              >
                取消
              </button>
              <button
                @click="addSelectedRecommended"
                :disabled="selectedCount === 0 || addingRecommended"
                class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ addingRecommended ? t('mcp.installing') : t('mcp.addAll') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 自定义MCP弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showCustomModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showCustomModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[500px] overflow-hidden shadow-2xl">
          <!-- 弹窗头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold">{{ t('mcp.customTitle') }}</h2>
            <button 
              @click="showCustomModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              ✕
            </button>
          </div>
          
          <!-- 表单内容 -->
          <div class="p-6 space-y-4">
            <!-- 名称输入 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('mcp.customName') }}</label>
              <input
                v-model="customName"
                type="text"
                :placeholder="t('mcp.customNamePlaceholder')"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-accent focus:outline-none text-sm"
              />
            </div>
            
            <!-- JSON配置 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('mcp.customConfig') }}</label>
              <textarea
                v-model="customJson"
                rows="8"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-accent focus:outline-none text-sm font-mono resize-none"
                spellcheck="false"
              ></textarea>
              <p class="text-xs text-muted-foreground mt-1">{{ t('mcp.customConfigHint') }}</p>
            </div>
            
            <!-- 错误提示 -->
            <div v-if="customError" class="text-sm text-red-500 bg-red-500/10 px-3 py-2 rounded-lg">
              {{ customError }}
            </div>
          </div>
          
          <!-- 弹窗底部 -->
          <div class="px-6 py-4 border-t border-border flex justify-end gap-2">
            <button
              @click="showCustomModal = false"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="addCustomMcp"
              :disabled="addingCustom"
              class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all disabled:opacity-50"
            >
              {{ addingCustom ? t('mcp.installing') : t('common.add') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

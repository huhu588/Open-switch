<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface McpServer {
  name: string
  server_type: string
  enabled: boolean
  url: string | null
  command: string[] | null
  install_path: string
  package_name: string | null
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

// 规则相关接口
interface InstalledRule {
  name: string
  location: string
  path: string
  description: string
  rule_type: string
  enabled: boolean
}

interface RecommendedRule {
  id: string
  name: string
  description: string
  category: string
  content: string
  file_type: string
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

// 规则管理相关
const showRecommendedRulesModal = ref(false)
const showCustomRuleModal = ref(false)
const recommendedRules = ref<RecommendedRule[]>([])
const selectedRules = ref<Set<string>>(new Set())
const installedRuleIds = ref<Set<string>>(new Set())
const addingRules = ref(false)
const ruleInstallLocation = ref('global_opencode')
const customRuleName = ref('')
const customRuleContent = ref('')
const customRuleError = ref('')
const addingCustomRule = ref(false)

// 已安装的规则列表
const installedRules = ref<InstalledRule[]>([])
const selectedRule = ref<InstalledRule | null>(null)
const selectedRuleContent = ref<string>('')
const rulesLoading = ref(false)

// 编辑规则相关
const showEditRuleModal = ref(false)
const editingRule = ref<InstalledRule | null>(null)
const editRuleContent = ref('')
const editRuleError = ref('')
const savingRule = ref(false)

// 加载已安装的规则
async function loadInstalledRules() {
  rulesLoading.value = true
  try {
    installedRules.value = await invoke<InstalledRule[]>('get_installed_rules')
    // 默认选中第一个
    if (installedRules.value.length > 0 && !selectedRule.value) {
      selectedRule.value = installedRules.value[0]
    }
  } catch (e) {
    console.error('加载已安装规则失败:', e)
  } finally {
    rulesLoading.value = false
  }
}

// 删除规则
async function deleteInstalledRule(rule: InstalledRule) {
  if (!confirm(t('rule.deleteConfirm', { name: rule.name }))) return
  try {
    await invoke('delete_rule', { path: rule.path })
    installMessage.value = t('rule.deleted', { name: rule.name })
    await loadInstalledRules()
    if (selectedRule.value?.path === rule.path) {
      selectedRule.value = installedRules.value[0] || null
    }
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e: any) {
    console.error('删除规则失败:', e)
  }
}

// 打开编辑规则弹窗
async function openEditRuleModal(rule: InstalledRule) {
  editingRule.value = rule
  editRuleError.value = ''
  try {
    editRuleContent.value = await invoke<string>('read_rule_content', { path: rule.path })
    showEditRuleModal.value = true
  } catch (e: any) {
    console.error('读取规则内容失败:', e)
  }
}

// 保存编辑的规则
async function saveEditedRule() {
  if (!editingRule.value) return
  
  editRuleError.value = ''
  if (!editRuleContent.value.trim()) {
    editRuleError.value = t('rule.customContentRequired')
    return
  }
  
  savingRule.value = true
  try {
    // 直接写入文件
    await invoke('save_rule_content', {
      path: editingRule.value.path,
      content: editRuleContent.value
    })
    
    showEditRuleModal.value = false
    installMessage.value = t('rule.saved', { name: editingRule.value.name })
    await loadInstalledRules()
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e: any) {
    editRuleError.value = e.toString()
  } finally {
    savingRule.value = false
  }
}

// 获取位置标签
function getLocationLabel(location: string): string {
  const labels: Record<string, string> = {
    'global_opencode': '全局',
    'project_opencode': '项目',
    'project_root': '根目录',
    'global_claude': 'Claude 全局',
    'project_claude': 'Claude 项目'
  }
  return labels[location] || location
}

// 选中规则时加载内容
async function selectRule(rule: InstalledRule) {
  selectedRule.value = rule
  selectedServer.value = null
  try {
    selectedRuleContent.value = await invoke<string>('read_rule_content', { path: rule.path })
  } catch (e) {
    selectedRuleContent.value = ''
    console.error('读取规则内容失败:', e)
  }
}

// 切换规则启用状态
async function toggleRule(rule: InstalledRule, event: Event) {
  event.stopPropagation()
  const newEnabled = !rule.enabled
  try {
    const newPath = await invoke<string>('toggle_rule_enabled', {
      path: rule.path,
      enabled: newEnabled
    })
    // 更新本地状态
    rule.enabled = newEnabled
    rule.path = newPath
    // 如果当前选中的是这个规则，也更新
    if (selectedRule.value?.path === rule.path || selectedRule.value?.name === rule.name) {
      selectedRule.value = { ...rule }
    }
  } catch (e: any) {
    console.error('切换规则状态失败:', e)
  }
}

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

// 删除 MCP 服务器
async function deleteMcpServer(name: string) {
  if (!confirm(t('mcp.deleteConfirm', { name }))) return
  try {
    await invoke('delete_mcp_server', { name })
    installMessage.value = t('mcp.serverDeleted', { name })
    if (selectedServer.value === name) {
      selectedServer.value = null
    }
    await loadServers()
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e: any) {
    console.error('删除 MCP 服务器失败:', e)
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
      server_names: Array.from(selectedRecommended.value)
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

// ============ 规则管理函数 ============

// 加载推荐规则列表
async function loadRecommendedRules() {
  try {
    recommendedRules.value = await invoke<RecommendedRule[]>('get_recommended_rules')
    
    // 检查已安装的规则
    const installed = await invoke<InstalledRule[]>('get_installed_rules')
    const installedNames = new Set(installed.map(r => r.name.replace('.md', '')))
    installedRuleIds.value = new Set(
      recommendedRules.value.filter(r => installedNames.has(r.id) || installedNames.has(r.name)).map(r => r.id)
    )
    
    // 默认选中未安装的
    selectedRules.value = new Set(
      recommendedRules.value.filter(r => !installedRuleIds.value.has(r.id)).map(r => r.id)
    )
  } catch (e) {
    console.error('加载推荐规则失败:', e)
  }
}

// 打开推荐规则弹窗
async function openRecommendedRulesModal() {
  await loadRecommendedRules()
  showRecommendedRulesModal.value = true
}

// 切换规则选中状态
function toggleRuleSelect(id: string) {
  if (selectedRules.value.has(id)) {
    selectedRules.value.delete(id)
  } else {
    selectedRules.value.add(id)
  }
  selectedRules.value = new Set(selectedRules.value)
}

// 安装选中的推荐规则
async function addSelectedRules() {
  if (selectedRules.value.size === 0) return
  
  addingRules.value = true
  let successCount = 0
  let failCount = 0
  
  try {
    for (const ruleId of selectedRules.value) {
      try {
        await invoke('install_rule', {
          ruleId: ruleId,
          content: '',
          location: ruleInstallLocation.value
        })
        successCount++
      } catch (e) {
        console.error(`安装规则 ${ruleId} 失败:`, e)
        failCount++
      }
    }
    
    if (successCount > 0) {
      installMessage.value = t('rule.rulesAdded', { count: successCount })
    }
    if (failCount > 0) {
      installMessage.value += ` ${t('rule.rulesFailed', { count: failCount })}`
    }
    
    showRecommendedRulesModal.value = false
    // 刷新规则列表
    await loadInstalledRules()
    setTimeout(() => { installMessage.value = '' }, 3000)
  } finally {
    addingRules.value = false
  }
}

const selectedRulesCount = computed(() => selectedRules.value.size)

// 打开自定义规则弹窗
function openCustomRuleModal() {
  customRuleName.value = ''
  customRuleContent.value = `# 我的自定义规则

## 编码规范
- 规则 1
- 规则 2

## 注意事项
- 注意事项 1
`
  customRuleError.value = ''
  showCustomRuleModal.value = true
}

// 添加自定义规则
async function addCustomRule() {
  customRuleError.value = ''
  
  if (!customRuleName.value.trim()) {
    customRuleError.value = t('rule.customNameRequired')
    return
  }
  
  if (!customRuleContent.value.trim()) {
    customRuleError.value = t('rule.customContentRequired')
    return
  }
  
  addingCustomRule.value = true
  try {
    await invoke('install_rule', {
      ruleId: customRuleName.value.trim().toLowerCase().replace(/\s+/g, '-'),
      content: customRuleContent.value,
      location: ruleInstallLocation.value
    })
    
    showCustomRuleModal.value = false
    installMessage.value = t('rule.customAdded', { name: customRuleName.value })
    // 刷新规则列表
    await loadInstalledRules()
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e: any) {
    customRuleError.value = e.toString()
  } finally {
    addingCustomRule.value = false
  }
}

// 获取规则分类标签
function getRuleCategoryLabel(category: string): string {
  const labels: Record<string, string> = {
    'code_style': '代码风格',
    'project': '项目结构',
    'review': '代码审查',
    'testing': '测试',
    'workflow': '工作流',
    'api': 'API',
    'security': '安全',
    'documentation': '文档'
  }
  return labels[category] || category
}

onMounted(() => {
  loadServers()
  loadInstalledRules()
})

const currentServer = () => servers.value.find(s => s.name === selectedServer.value)
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- 顶部工具栏 -->
    <div class="flex items-center justify-between flex-shrink-0">
      <div class="flex items-center gap-3">
        <!-- MCP 按钮组 -->
        <div class="flex items-center gap-1 px-1 py-1 rounded-lg bg-surface/50">
          <button
            @click="openRecommendedModal"
            class="px-3 py-1.5 rounded-md bg-accent/20 hover:bg-accent/30 text-accent font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="star" :size="12" />
            {{ t('mcp.addRecommended') }}
          </button>
          <button
            @click="openCustomModal"
            class="px-3 py-1.5 rounded-md hover:bg-surface-hover text-foreground font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="plus" :size="12" />
            {{ t('mcp.addCustom') }}
          </button>
        </div>
        
        <!-- 分隔符 -->
        <div class="w-px h-6 bg-border"></div>
        
        <!-- 规则按钮组 -->
        <div class="flex items-center gap-1 px-1 py-1 rounded-lg bg-surface/50">
          <button
            @click="openRecommendedRulesModal"
            class="px-3 py-1.5 rounded-md bg-violet-500/20 hover:bg-violet-500/30 text-violet-400 font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="code" :size="12" />
            {{ t('rule.addRecommended') }}
          </button>
          <button
            @click="openCustomRuleModal"
            class="px-3 py-1.5 rounded-md hover:bg-surface-hover text-foreground font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="edit" :size="12" />
            {{ t('rule.addCustom') }}
          </button>
        </div>
      </div>
      
      <!-- 操作结果消息 -->
      <div v-if="installMessage" class="text-sm text-accent animate-pulse">
        {{ installMessage }}
      </div>
    </div>
    
    <!-- 主内容区 -->
    <div class="flex-1 flex gap-4 min-h-0">
      <!-- 左侧列表区域 -->
      <div class="w-72 flex-shrink-0 flex flex-col gap-3">
        <!-- MCP 服务器列表 -->
        <div class="flex-1 min-h-0 flex flex-col rounded-xl bg-surface/30 border border-border overflow-hidden">
          <div class="flex items-center justify-between px-4 py-2 border-b border-border">
            <h3 class="font-semibold text-sm flex items-center gap-1.5">
              <SvgIcon name="terminal" :size="14" class="text-accent" /> MCP
            </h3>
            <span class="text-xs text-muted-foreground">({{ servers.length }})</span>
          </div>

          <div class="flex-1 overflow-auto">
            <div v-if="loading" class="p-4 text-center text-muted-foreground text-xs">
              {{ t('mcp.loading') }}
            </div>
            <div v-else-if="servers.length === 0" class="p-3 text-center text-muted-foreground text-xs">
              {{ t('mcp.noServers') }}
            </div>
            <ul v-else class="p-1.5 space-y-1">
              <li
                v-for="server in servers"
                :key="server.name"
                @click="selectedServer = server.name; selectedRule = null"
                class="px-2.5 py-2 rounded-lg cursor-pointer transition-all duration-150"
                :class="[
                  server.name === selectedServer && !selectedRule
                    ? 'bg-accent/10 border border-accent/40'
                    : 'hover:bg-surface-hover border border-transparent'
                ]"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span 
                      class="w-1.5 h-1.5 rounded-full flex-shrink-0"
                      :class="[
                        healthStatus[server.name] === undefined ? 'bg-gray-400 animate-pulse' :
                        healthStatus[server.name]?.healthy ? 'bg-emerald-500' : 'bg-amber-500'
                      ]"
                    ></span>
                    <SvgIcon :name="server.server_type === 'local' ? 'box' : 'globe'" :size="12" class="text-muted-foreground" />
                    <span class="font-medium text-xs truncate max-w-[120px]">{{ server.name }}</span>
                  </div>
                  <button
                    @click.stop="toggleServer(server.name)"
                    class="text-[10px] px-1.5 py-0.5 rounded"
                    :class="server.enabled ? 'bg-emerald-500/20 text-emerald-500' : 'bg-surface text-muted-foreground'"
                  >
                    {{ server.enabled ? t('mcp.enabled') : t('mcp.disabled') }}
                  </button>
                </div>
              </li>
            </ul>
          </div>
        </div>
        
        <!-- 规则列表 -->
        <div class="flex-1 min-h-0 flex flex-col rounded-xl bg-surface/30 border border-border overflow-hidden">
          <div class="flex items-center justify-between px-4 py-2 border-b border-border">
            <h3 class="font-semibold text-sm flex items-center gap-1.5">
              <SvgIcon name="code" :size="14" class="text-violet-400" /> {{ t('rule.title') }}
            </h3>
            <span class="text-xs text-muted-foreground">({{ installedRules.length }})</span>
          </div>

          <div class="flex-1 overflow-auto">
            <div v-if="rulesLoading" class="p-3 text-center text-muted-foreground text-xs">
              {{ t('mcp.loading') }}
            </div>
            <div v-else-if="installedRules.length === 0" class="p-3 text-center text-muted-foreground text-xs">
              {{ t('rule.noRules') }}
            </div>
            <ul v-else class="p-1.5 space-y-1">
              <li
                v-for="rule in installedRules"
                :key="rule.path"
                @click="selectRule(rule)"
                class="px-2.5 py-2 rounded-lg cursor-pointer transition-all duration-150"
                :class="[
                  selectedRule?.path === rule.path
                    ? 'bg-violet-500/10 border border-violet-500/40'
                    : 'hover:bg-surface-hover border border-transparent'
                ]"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <SvgIcon name="book" :size="12" class="text-muted-foreground" />
                    <span 
                      class="font-medium text-xs truncate max-w-[140px]"
                      :class="!rule.enabled && 'text-muted-foreground line-through'"
                    >{{ rule.name }}</span>
                  </div>
                  <button
                    @click="toggleRule(rule, $event)"
                    class="text-[10px] px-1.5 py-0.5 rounded transition-colors"
                    :class="rule.enabled ? 'bg-emerald-500/20 text-emerald-500' : 'bg-surface text-muted-foreground'"
                  >
                    {{ rule.enabled ? t('mcp.enabled') : t('mcp.disabled') }}
                  </button>
                </div>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- 详情面板 -->
      <div class="flex-1">
        <div class="h-full rounded-xl bg-surface/30 border border-border p-4 overflow-auto">
          <!-- 未选中任何项 -->
          <div v-if="!currentServer() && !selectedRule" class="text-center text-muted-foreground py-8">
            {{ t('mcp.selectItem') }}
          </div>
          
          <!-- MCP 详情 -->
          <div v-else-if="currentServer() && !selectedRule" class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="font-semibold text-lg flex items-center gap-2">
                <SvgIcon name="terminal" :size="18" class="text-accent" /> {{ currentServer()?.name }}
              </h3>
              <button
                @click="deleteMcpServer(currentServer()!.name)"
                class="px-3 py-1.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 text-xs transition-all flex items-center gap-1"
              >
                <SvgIcon name="trash" :size="12" /> {{ t('common.delete') }}
              </button>
            </div>
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
              <div class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.installPath') }}</span>
                <span class="font-mono text-xs break-all">{{ currentServer()?.install_path }}</span>
              </div>
              <div v-if="currentServer()?.package_name" class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.package') }}</span>
                <span class="font-mono text-xs text-accent">{{ currentServer()?.package_name }}</span>
              </div>
              <div class="flex gap-3">
                <span class="text-muted-foreground w-20">{{ t('mcp.effective') }}</span>
                <span 
                  :class="[
                    currentServer()?.enabled && healthStatus[currentServer()!.name]?.healthy 
                      ? 'text-emerald-500' 
                      : 'text-amber-500'
                  ]"
                >
                  {{ 
                    currentServer()?.enabled 
                      ? (healthStatus[currentServer()!.name]?.healthy 
                          ? t('mcp.effectiveYes') 
                          : t('mcp.effectiveNo')) 
                      : t('mcp.effectiveDisabled')
                  }}
                </span>
              </div>
            </div>
          </div>
          
          <!-- 规则详情 -->
          <div v-else-if="selectedRule" class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="font-semibold text-lg flex items-center gap-2">
                <SvgIcon name="code" :size="18" class="text-violet-400" /> {{ selectedRule.name }}
              </h3>
              <div class="flex gap-2">
                <button
                  @click="openEditRuleModal(selectedRule)"
                  class="px-3 py-1.5 rounded-lg bg-accent/10 hover:bg-accent/20 text-accent text-xs transition-all flex items-center gap-1"
                >
                  <SvgIcon name="edit" :size="12" /> {{ t('common.edit') }}
                </button>
                <button
                  @click="deleteInstalledRule(selectedRule)"
                  class="px-3 py-1.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 text-xs transition-all flex items-center gap-1"
                >
                  <SvgIcon name="trash" :size="12" /> {{ t('common.delete') }}
                </button>
              </div>
            </div>
            <div class="space-y-3 text-sm">
              <div class="flex gap-3">
                <span class="text-muted-foreground w-16 flex-shrink-0">{{ t('mcp.status') }}</span>
                <button
                  @click="toggleRule(selectedRule, $event)"
                  class="px-2 py-0.5 rounded text-xs transition-colors"
                  :class="selectedRule.enabled 
                    ? 'bg-emerald-500/20 text-emerald-500 hover:bg-emerald-500/30' 
                    : 'bg-amber-500/20 text-amber-500 hover:bg-amber-500/30'"
                >
                  {{ selectedRule.enabled ? t('mcp.statusEnabled') : t('mcp.statusDisabled') }}
                </button>
              </div>
              <div class="flex gap-3">
                <span class="text-muted-foreground w-16 flex-shrink-0">{{ t('rule.installLocation') }}</span>
                <span class="text-violet-400">{{ getLocationLabel(selectedRule.location) }}</span>
              </div>
              <div class="flex gap-3">
                <span class="text-muted-foreground w-16 flex-shrink-0">{{ t('rule.type') }}</span>
                <span>{{ selectedRule.rule_type }}</span>
              </div>
              <div class="flex gap-3">
                <span class="text-muted-foreground w-16 flex-shrink-0">{{ t('rule.path') }}</span>
                <span class="font-mono text-xs break-all text-muted-foreground">{{ selectedRule.path }}</span>
              </div>
            </div>
            
            <!-- 规则内容预览 -->
            <div class="mt-4">
              <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium">{{ t('rule.content') || '规则内容' }}</span>
              </div>
              <div class="bg-surface/50 border border-border rounded-lg p-3 max-h-[400px] overflow-auto">
                <pre class="text-xs font-mono whitespace-pre-wrap text-muted-foreground leading-relaxed">{{ selectedRuleContent || selectedRule.description }}</pre>
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
              <SvgIcon name="close" :size="16" />
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
              <SvgIcon name="close" :size="16" />
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
    
    <!-- 推荐规则弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showRecommendedRulesModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showRecommendedRulesModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[600px] max-h-[80vh] overflow-hidden shadow-2xl">
          <!-- 弹窗头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold flex items-center gap-2"><SvgIcon name="code" :size="18" class="text-violet-400" /> {{ t('rule.recommendedTitle') }}</h2>
            <button 
              @click="showRecommendedRulesModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 安装位置选择 -->
          <div class="px-6 py-3 border-b border-border bg-surface/30">
            <label class="text-sm font-medium mr-3">{{ t('rule.installLocation') }}</label>
            <select 
              v-model="ruleInstallLocation"
              class="px-3 py-1.5 rounded-lg bg-surface border border-border text-sm focus:border-accent focus:outline-none"
            >
              <option value="global_opencode">{{ t('rule.locationOptions.globalOpencode') }}</option>
              <option value="project_opencode">{{ t('rule.locationOptions.projectOpencode') }}</option>
              <option value="global_claude">{{ t('rule.locationOptions.globalClaude') }}</option>
              <option value="project_claude">{{ t('rule.locationOptions.projectClaude') }}</option>
            </select>
          </div>
          
          <!-- 规则列表 -->
          <div class="p-4 space-y-3 max-h-[45vh] overflow-auto">
            <div
              v-for="rule in recommendedRules"
              :key="rule.id"
              @click="!installedRuleIds.has(rule.id) && toggleRuleSelect(rule.id)"
              class="p-4 rounded-xl border transition-all"
              :class="[
                installedRuleIds.has(rule.id)
                  ? 'border-emerald-500/30 bg-emerald-500/5 cursor-default'
                  : selectedRules.has(rule.id) 
                    ? 'border-violet-500 bg-violet-500/10 cursor-pointer' 
                    : 'border-border hover:border-violet-500/50 bg-surface/30 cursor-pointer'
              ]"
            >
              <div class="flex items-start gap-3">
                <!-- 选中指示器 -->
                <div 
                  v-if="!installedRuleIds.has(rule.id)"
                  class="w-5 h-5 rounded-md border-2 flex items-center justify-center flex-shrink-0 mt-0.5 transition-all"
                  :class="selectedRules.has(rule.id) 
                    ? 'border-violet-500 bg-violet-500 text-white' 
                    : 'border-muted-foreground'"
                >
                  <span v-if="selectedRules.has(rule.id)" class="text-xs">✓</span>
                </div>
                <!-- 已安装标记 -->
                <div 
                  v-else
                  class="w-5 h-5 rounded-md bg-emerald-500/20 flex items-center justify-center flex-shrink-0 mt-0.5"
                >
                  <span class="text-xs text-emerald-500">✓</span>
                </div>
                
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="font-semibold">{{ rule.name }}</span>
                    <span class="text-xs px-1.5 py-0.5 rounded bg-violet-500/20 text-violet-400">
                      {{ getRuleCategoryLabel(rule.category) }}
                    </span>
                    <span v-if="installedRuleIds.has(rule.id)" class="text-xs px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-500">
                      {{ t('rule.installed') }}
                    </span>
                  </div>
                  <p class="text-sm text-muted-foreground mt-1">{{ rule.description }}</p>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 弹窗底部 -->
          <div class="px-6 py-4 border-t border-border flex items-center justify-between">
            <span class="text-sm text-muted-foreground">
              {{ t('rule.selectedCount', { count: selectedRulesCount }) }}
            </span>
            <div class="flex gap-2">
              <button
                @click="showRecommendedRulesModal = false"
                class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                @click="addSelectedRules"
                :disabled="selectedRulesCount === 0 || addingRules"
                class="px-4 py-2 rounded-lg bg-violet-500 hover:bg-violet-600 text-white text-sm font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ addingRules ? t('rule.installing') : t('rule.addAll') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 自定义规则弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showCustomRuleModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showCustomRuleModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[550px] overflow-hidden shadow-2xl">
          <!-- 弹窗头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold flex items-center gap-2"><SvgIcon name="edit" :size="18" class="text-violet-400" /> {{ t('rule.customTitle') }}</h2>
            <button 
              @click="showCustomRuleModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 表单内容 -->
          <div class="p-6 space-y-4">
            <!-- 名称输入 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('rule.customName') }}</label>
              <input
                v-model="customRuleName"
                type="text"
                :placeholder="t('rule.customNamePlaceholder')"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-violet-500 focus:outline-none text-sm"
              />
            </div>
            
            <!-- 安装位置 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('rule.installLocation') }}</label>
              <select 
                v-model="ruleInstallLocation"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border text-sm focus:border-violet-500 focus:outline-none"
              >
                <option value="global_opencode">{{ t('rule.locationOptions.globalOpencode') }}</option>
                <option value="project_opencode">{{ t('rule.locationOptions.projectOpencode') }}</option>
                <option value="global_claude">{{ t('rule.locationOptions.globalClaude') }}</option>
                <option value="project_claude">{{ t('rule.locationOptions.projectClaude') }}</option>
              </select>
            </div>
            
            <!-- 规则内容 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('rule.customContent') }}</label>
              <textarea
                v-model="customRuleContent"
                rows="10"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-violet-500 focus:outline-none text-sm font-mono resize-none"
                spellcheck="false"
              ></textarea>
              <p class="text-xs text-muted-foreground mt-1">{{ t('rule.customContentHint') }}</p>
            </div>
            
            <!-- 错误提示 -->
            <div v-if="customRuleError" class="text-sm text-red-500 bg-red-500/10 px-3 py-2 rounded-lg">
              {{ customRuleError }}
            </div>
          </div>
          
          <!-- 弹窗底部 -->
          <div class="px-6 py-4 border-t border-border flex justify-end gap-2">
            <button
              @click="showCustomRuleModal = false"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="addCustomRule"
              :disabled="addingCustomRule"
              class="px-4 py-2 rounded-lg bg-violet-500 hover:bg-violet-600 text-white text-sm font-medium transition-all disabled:opacity-50"
            >
              {{ addingCustomRule ? t('rule.installing') : t('common.add') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 编辑规则弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showEditRuleModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showEditRuleModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[650px] max-h-[85vh] overflow-hidden shadow-2xl flex flex-col">
          <!-- 弹窗头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between flex-shrink-0">
            <h2 class="text-lg font-semibold flex items-center gap-2"><SvgIcon name="edit" :size="18" class="text-violet-400" /> {{ t('rule.editTitle') }} - {{ editingRule?.name }}</h2>
            <button 
              @click="showEditRuleModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 编辑内容 -->
          <div class="p-6 flex-1 overflow-auto">
            <div class="space-y-4">
              <!-- 规则内容 -->
              <div>
                <label class="block text-sm font-medium mb-2">{{ t('rule.customContent') }}</label>
                <textarea
                  v-model="editRuleContent"
                  rows="18"
                  class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-violet-500 focus:outline-none text-sm font-mono resize-none"
                  spellcheck="false"
                ></textarea>
              </div>
              
              <!-- 错误提示 -->
              <div v-if="editRuleError" class="text-sm text-red-500 bg-red-500/10 px-3 py-2 rounded-lg">
                {{ editRuleError }}
              </div>
            </div>
          </div>
          
          <!-- 弹窗底部 -->
          <div class="px-6 py-4 border-t border-border flex justify-end gap-2 flex-shrink-0">
            <button
              @click="showEditRuleModal = false"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="saveEditedRule"
              :disabled="savingRule"
              class="px-4 py-2 rounded-lg bg-violet-500 hover:bg-violet-600 text-white text-sm font-medium transition-all disabled:opacity-50"
            >
              {{ savingRule ? t('common.saving') : t('common.save') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

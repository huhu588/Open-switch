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

interface McpOAuthConfig {
  clientId?: string
  clientSecret?: string
  scope?: string
  client_id?: string
  client_secret?: string
}

interface McpServerDetail {
  server_type: string
  enabled: boolean
  timeout: number | null
  command: string[] | null
  environment: Record<string, string>
  url: string | null
  headers: Record<string, string>
  oauth: McpOAuthConfig | null
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

// 各应用 MCP 状态
interface AppMcpStatus {
  app_name: string
  is_configured: boolean
  server_count: number
  server_names: string[]
}

// 导入 MCP 结果
interface ImportMcpResult {
  imported: string[]
  skipped: string[]
  failed: string[]
}

// 聚合的 MCP 管理信息
interface ManagedMcp {
  name: string
  server_type: string
  command: string[] | null
  url: string | null
  package_name: string | null
  opencode_enabled: boolean
  claude_enabled: boolean
  codex_enabled: boolean
  gemini_enabled: boolean
  cursor_enabled: boolean
}

// MCP 统计信息
interface McpStats {
  opencode_count: number
  claude_count: number
  codex_count: number
  gemini_count: number
  cursor_count: number
}

const servers = ref<McpServer[]>([])
const appsMcpStatus = ref<AppMcpStatus[]>([])
const selectedServer = ref<string | null>(null)
const loading = ref(false)
const healthStatus = ref<Record<string, McpHealthResult | null>>({})
const importingApp = ref<string | null>(null)

// 当前活动标签页: 'mcp' | 'rules'
const activeTab = ref<'mcp' | 'rules'>('mcp')
// 已部署应用展开状态
const deployedAppsExpanded = ref(false)

// MCP 管理弹窗
const showMcpManageModal = ref(false)
const managedMcps = ref<ManagedMcp[]>([])
const mcpStats = ref<McpStats>({ opencode_count: 0, claude_count: 0, codex_count: 0, gemini_count: 0, cursor_count: 0 })
const togglingMcp = ref<string | null>(null)
const mcpManageSearchQuery = ref('')

// 聚合的规则管理信息
interface ManagedRule {
  name: string
  description: string
  content: string
  rule_type: string
  opencode_enabled: boolean
  claude_enabled: boolean
  codex_enabled: boolean
  gemini_enabled: boolean
  cursor_enabled: boolean
  source_path: string | null
}

// 规则统计信息
interface RuleStats {
  opencode_count: number
  claude_count: number
  codex_count: number
  gemini_count: number
  cursor_count: number
}

// 规则管理弹窗
const showRuleManageModal = ref(false)
const managedRules = ref<ManagedRule[]>([])
const ruleStats = ref<RuleStats>({ opencode_count: 0, claude_count: 0, codex_count: 0, gemini_count: 0, cursor_count: 0 })
const togglingRule = ref<string | null>(null)
const ruleManageSearchQuery = ref('')

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
  "args": ["-y", "@modelcontextprotocol/server-memory"],
  "type": "stdio"
}`)
const customError = ref('')
const addingCustom = ref(false)

// 编辑 MCP 相关
const showEditMcpModal = ref(false)
const editingServerName = ref<string | null>(null)
const editingServerDetail = ref<McpServerDetail | null>(null)
const editName = ref('')
const editJson = ref('')
const editError = ref('')
const savingEdit = ref(false)

// 规则管理相关
const showRecommendedRulesModal = ref(false)
const showCustomRuleModal = ref(false)
const recommendedRules = ref<RecommendedRule[]>([])
const selectedRules = ref<Set<string>>(new Set())
const installedRuleIds = ref<Set<string>>(new Set())
const addingRules = ref(false)
const customRuleName = ref('')
const customRuleContent = ref('')
const customRuleError = ref('')
const addingCustomRule = ref(false)

// 规则多选安装位置（OpenCode、Claude Code、Codex、Gemini、Cursor）
const ruleInstallTargets = ref({
  opencode: true,      // ~/.config/opencode/rules/
  claudeCode: false,   // ~/.claude/rules/ + CLAUDE.md
  codex: false,        // ~/.codex/ + AGENTS.md  
  gemini: false,       // ~/.gemini/ + GEMINI.md
  cursor: false,       // ~/.cursor/rules/
})

// 已安装的规则列表
const installedRules = ref<InstalledRule[]>([])
const selectedRule = ref<InstalledRule | null>(null)
const selectedRuleContent = ref<string>('')
const rulesLoading = ref(false)

// 去重后的规则（聚合同名规则的部署状态）
interface UniqueRule extends InstalledRule {
  deployedTo: string[]  // 部署到哪些应用
  allPaths: { location: string; path: string }[]  // 所有副本路径
}

// 计算去重后的规则列表
const uniqueInstalledRules = computed<UniqueRule[]>(() => {
  const ruleMap = new Map<string, UniqueRule>()
  
  for (const rule of installedRules.value) {
    const key = rule.name
    if (!ruleMap.has(key)) {
      ruleMap.set(key, {
        ...rule,
        deployedTo: [getLocationLabel(rule.location)],
        allPaths: [{ location: rule.location, path: rule.path }]
      })
    } else {
      const existing = ruleMap.get(key)!
      const label = getLocationLabel(rule.location)
      if (!existing.deployedTo.includes(label)) {
        existing.deployedTo.push(label)
      }
      existing.allPaths.push({ location: rule.location, path: rule.path })
    }
  }
  
  return Array.from(ruleMap.values()).sort((a, b) => a.name.localeCompare(b.name))
})

// 获取选中规则的部署状态
const selectedRuleDeployment = computed(() => {
  if (!selectedRule.value) return null
  return uniqueInstalledRules.value.find(r => r.name === selectedRule.value?.name)
})

// 编辑规则相关
const showEditRuleModal = ref(false)
const editingRule = ref<InstalledRule | null>(null)
const editRuleContent = ref('')
const editRuleError = ref('')
const savingRule = ref(false)

// 编辑模式的 CLI 工具同步选项
const editSyncToClaudeMd = ref(false)
const editSyncToAgentsMd = ref(false)
const editSyncToGeminiMd = ref(false)

// 加载已安装的规则
async function loadInstalledRules() {
  rulesLoading.value = true
  try {
    installedRules.value = await invoke<InstalledRule[]>('get_installed_rules')
    // 默认选中第一个（使用去重后的列表）
    if (uniqueInstalledRules.value.length > 0 && !selectedRule.value) {
      selectedRule.value = uniqueInstalledRules.value[0]
    }
  } catch (e) {
    console.error('加载已安装规则失败:', e)
  } finally {
    rulesLoading.value = false
  }
}

// 删除规则（删除所有副本）
async function deleteInstalledRule(rule: InstalledRule) {
  // 获取该规则的所有副本路径
  const uniqueRule = uniqueInstalledRules.value.find(r => r.name === rule.name) as UniqueRule | undefined
  const pathsToDelete = uniqueRule?.allPaths || [{ path: rule.path }]
  
  const confirmMsg = pathsToDelete.length > 1 
    ? `确定要从所有应用中删除规则 "${rule.name}" 吗？（共 ${pathsToDelete.length} 个副本）`
    : t('rule.deleteConfirm', { name: rule.name })
  
  if (!confirm(confirmMsg)) return
  
  try {
    // 删除所有副本
    for (const { path } of pathsToDelete) {
      await invoke('delete_rule', { path })
    }
    installMessage.value = t('rule.deleted', { name: rule.name })
    await loadInstalledRules()
    if (selectedRule.value?.name === rule.name) {
      selectedRule.value = uniqueInstalledRules.value[0] || null
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
  // 重置 CLI 工具同步选项
  editSyncToClaudeMd.value = false
  editSyncToAgentsMd.value = false
  editSyncToGeminiMd.value = false
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
    // 1. 保存规则文件
    await invoke('save_rule_content', {
      path: editingRule.value.path,
      content: editRuleContent.value
    })
    
    // 2. 同步到 CLI 工具（如果选中）
    const syncTargets: string[] = []
    if (editSyncToClaudeMd.value) syncTargets.push('claude')
    if (editSyncToAgentsMd.value) syncTargets.push('codex')
    if (editSyncToGeminiMd.value) syncTargets.push('gemini')
    
    if (syncTargets.length > 0) {
      await invoke('sync_prompt', {
        content: editRuleContent.value,
        targets: syncTargets
      })
    }
    
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
    'global_opencode': 'OpenCode',
    'project_opencode': 'OpenCode 项目',
    'project_root': '根目录',
    'global_claude': 'Claude',
    'project_claude': 'Claude 项目',
    'global_cursor': 'Cursor',
    'global_codex': 'Codex',
    'global_gemini': 'Gemini'
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

// 加载各应用的 MCP 状态
async function loadAppsMcpStatus() {
  try {
    appsMcpStatus.value = await invoke<AppMcpStatus[]>('get_apps_mcp_status')
  } catch (e) {
    console.error('加载应用 MCP 状态失败:', e)
  }
}

// ==================== MCP 管理功能 ====================

// 加载管理的 MCP 列表
async function loadManagedMcps() {
  try {
    managedMcps.value = await invoke<ManagedMcp[]>('get_managed_mcps')
    mcpStats.value = await invoke<McpStats>('get_mcp_stats')
  } catch (e) {
    console.error('加载 managed MCPs 失败:', e)
  }
}

// 打开 MCP 管理弹窗
async function openMcpManageModal() {
  await loadManagedMcps()
  mcpManageSearchQuery.value = ''
  showMcpManageModal.value = true
}

// 过滤后的管理 MCP 列表
const filteredManagedMcps = computed(() => {
  if (!mcpManageSearchQuery.value.trim()) {
    return managedMcps.value
  }
  const query = mcpManageSearchQuery.value.toLowerCase()
  return managedMcps.value.filter(mcp => 
    mcp.name.toLowerCase().includes(query) || 
    (mcp.package_name && mcp.package_name.toLowerCase().includes(query))
  )
})

// 切换 MCP 的应用启用状态
async function toggleMcpApp(mcp: ManagedMcp, app: 'opencode' | 'claude' | 'codex' | 'gemini' | 'cursor') {
  const key = `${mcp.name}-${app}`
  if (togglingMcp.value) return
  
  togglingMcp.value = key
  
  const currentEnabled = app === 'opencode' ? mcp.opencode_enabled :
                         app === 'claude' ? mcp.claude_enabled :
                         app === 'codex' ? mcp.codex_enabled :
                         app === 'gemini' ? mcp.gemini_enabled :
                         mcp.cursor_enabled
  
  try {
    await invoke('toggle_mcp_app', {
      mcpName: mcp.name,
      app: app,
      enabled: !currentEnabled
    })
    
    // 更新本地状态
    if (app === 'opencode') mcp.opencode_enabled = !currentEnabled
    else if (app === 'claude') mcp.claude_enabled = !currentEnabled
    else if (app === 'codex') mcp.codex_enabled = !currentEnabled
    else if (app === 'gemini') mcp.gemini_enabled = !currentEnabled
    else if (app === 'cursor') mcp.cursor_enabled = !currentEnabled
    
    // 刷新统计
    mcpStats.value = await invoke<McpStats>('get_mcp_stats')
  } catch (e) {
    console.error('切换 MCP 应用状态失败:', e)
    installMessage.value = '切换失败: ' + (e as any).message || e
  } finally {
    togglingMcp.value = null
  }
}

// 从所有应用删除 MCP
async function deleteMcpFromAll(mcp: ManagedMcp) {
  if (!confirm(`确定要从所有应用中删除 "${mcp.name}" 吗？`)) return
  
  try {
    await invoke('delete_mcp_from_all', { mcpName: mcp.name })
    await loadManagedMcps()
    await loadServers()
    await loadAppsMcpStatus()
    installMessage.value = `已删除 ${mcp.name}`
  } catch (e) {
    console.error('删除 MCP 失败:', e)
    installMessage.value = '删除失败: ' + (e as any).message || e
  }
}

// ==================== 规则管理功能 ====================

// 加载管理的规则列表
async function loadManagedRules() {
  try {
    managedRules.value = await invoke<ManagedRule[]>('get_managed_rules')
    ruleStats.value = await invoke<RuleStats>('get_rule_stats')
  } catch (e) {
    console.error('加载 managed rules 失败:', e)
  }
}

// 打开规则管理弹窗
async function openRuleManageModal() {
  await loadManagedRules()
  ruleManageSearchQuery.value = ''
  showRuleManageModal.value = true
}

// 过滤后的管理规则列表
const filteredManagedRules = computed(() => {
  if (!ruleManageSearchQuery.value.trim()) {
    return managedRules.value
  }
  const query = ruleManageSearchQuery.value.toLowerCase()
  return managedRules.value.filter(rule => 
    rule.name.toLowerCase().includes(query) || 
    rule.description.toLowerCase().includes(query)
  )
})

// 切换规则的应用启用状态
async function toggleRuleApp(rule: ManagedRule, app: 'opencode' | 'claude' | 'codex' | 'gemini' | 'cursor') {
  const key = `${rule.name}-${app}`
  if (togglingRule.value) return
  
  togglingRule.value = key
  
  const currentEnabled = app === 'opencode' ? rule.opencode_enabled :
                         app === 'claude' ? rule.claude_enabled :
                         app === 'codex' ? rule.codex_enabled :
                         app === 'gemini' ? rule.gemini_enabled :
                         rule.cursor_enabled
  
  try {
    await invoke('toggle_rule_app', {
      ruleName: rule.name,
      app: app,
      enabled: !currentEnabled,
      content: rule.content
    })
    
    // 更新本地状态
    if (app === 'opencode') rule.opencode_enabled = !currentEnabled
    else if (app === 'claude') rule.claude_enabled = !currentEnabled
    else if (app === 'codex') rule.codex_enabled = !currentEnabled
    else if (app === 'gemini') rule.gemini_enabled = !currentEnabled
    else if (app === 'cursor') rule.cursor_enabled = !currentEnabled
    
    // 刷新统计
    ruleStats.value = await invoke<RuleStats>('get_rule_stats')
  } catch (e) {
    console.error('切换规则应用状态失败:', e)
    installMessage.value = '切换失败: ' + (e as any).message || e
  } finally {
    togglingRule.value = null
  }
}

// 从所有应用删除规则
async function deleteRuleFromAll(rule: ManagedRule) {
  if (!confirm(`确定要从所有应用中删除 "${rule.name}" 吗？`)) return
  
  try {
    await invoke('delete_rule_from_all', { ruleName: rule.name })
    await loadManagedRules()
    await loadInstalledRules()
    installMessage.value = `已删除 ${rule.name}`
  } catch (e) {
    console.error('删除规则失败:', e)
    installMessage.value = '删除失败: ' + (e as any).message || e
  }
}

// 从应用导入 MCP
async function importMcpFromApp(appName: string) {
  importingApp.value = appName
  try {
    const result = await invoke<ImportMcpResult>('import_mcp_from_apps', { appName })
    
    // 构建消息
    const messages: string[] = []
    if (result.imported.length > 0) {
      messages.push(t('mcp.importedCount', { count: result.imported.length }))
    }
    if (result.skipped.length > 0) {
      messages.push(t('mcp.skippedCount', { count: result.skipped.length }))
    }
    if (result.failed.length > 0) {
      messages.push(t('mcp.failedCount', { count: result.failed.length }))
    }
    
    installMessage.value = messages.join(', ') || t('mcp.noMcpToImport')
    
    // 刷新列表
    await loadServers()
    await loadAppsMcpStatus()
  } catch (e: any) {
    console.error('导入 MCP 失败:', e)
    installMessage.value = t('mcp.importFailed') + ': ' + (e.message || e)
  } finally {
    importingApp.value = null
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

function normalizeStringRecord(value: unknown): Record<string, string> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {}
  const record: Record<string, string> = {}
  for (const [key, rawValue] of Object.entries(value)) {
    if (rawValue === null || rawValue === undefined) continue
    record[key] = String(rawValue)
  }
  return record
}

function buildEditableConfig(server: McpServerDetail): string {
  const config: Record<string, any> = {
    type: server.server_type === 'remote' ? 'remote' : 'stdio',
    enabled: server.enabled
  }

  if (server.timeout !== null && server.timeout !== undefined) {
    config.timeout = server.timeout
  }

  if (server.command && server.command.length > 0) {
    config.command = server.command[0]
    if (server.command.length > 1) {
      config.args = server.command.slice(1)
    }
  }

  if (server.environment && Object.keys(server.environment).length > 0) {
    config.env = server.environment
  }

  if (server.url) {
    config.url = server.url
  }

  if (server.headers && Object.keys(server.headers).length > 0) {
    config.headers = server.headers
  }

  if (server.oauth) {
    const oauth: Record<string, string> = {}
    const clientId = server.oauth.clientId ?? server.oauth.client_id
    const clientSecret = server.oauth.clientSecret ?? server.oauth.client_secret
    if (clientId) oauth.clientId = clientId
    if (clientSecret) oauth.clientSecret = clientSecret
    if (server.oauth.scope) oauth.scope = server.oauth.scope
    if (Object.keys(oauth).length > 0) {
      config.oauth = oauth
    }
  }

  return JSON.stringify(config, null, 2)
}

function parseMcpConfig(
  jsonText: string,
  fallbackEnabled: boolean
): { input?: {
  server_type: 'local' | 'remote'
  enabled: boolean
  timeout: number | null
  command: string[] | null
  environment: Record<string, string> | null
  url: string | null
  headers: Record<string, string> | null
  oauth: { client_id?: string; client_secret?: string; scope?: string } | null
}; error?: string } {
  let config: any
  try {
    config = JSON.parse(jsonText)
  } catch (e) {
    return { error: t('mcp.customJsonInvalid') }
  }

  const typeValue = typeof config.type === 'string' ? config.type.toLowerCase() : ''
  const urlValue = typeof config.url === 'string' ? config.url.trim() : ''

  const command: string[] = []
  if (typeof config.command === 'string' && config.command.trim()) {
    command.push(config.command.trim())
  }
  if (Array.isArray(config.args)) {
    for (const arg of config.args) {
      if (typeof arg === 'string' && arg.trim()) {
        command.push(arg.trim())
      }
    }
  }

  const hasCommand = command.length > 0
  const hasUrl = urlValue.length > 0

  let serverType: 'local' | 'remote' = 'local'
  if (typeValue === 'remote') {
    serverType = 'remote'
  } else if (typeValue === 'local' || typeValue === 'stdio') {
    serverType = 'local'
  } else if (hasUrl) {
    serverType = 'remote'
  } else if (hasCommand) {
    serverType = 'local'
  }

  if (serverType === 'local' && !hasCommand) {
    return { error: t('mcp.customCommandRequired') }
  }

  if (serverType === 'remote' && !hasUrl) {
    return { error: t('mcp.customUrlRequired') }
  }

  const enabled = typeof config.enabled === 'boolean' ? config.enabled : fallbackEnabled
  const timeout = Number.isFinite(config.timeout) ? config.timeout : null
  const environment = normalizeStringRecord(config.env ?? config.environment)
  const headers = normalizeStringRecord(config.headers)

  let oauth: { client_id?: string; client_secret?: string; scope?: string } | null = null
  if (config.oauth && typeof config.oauth === 'object' && !Array.isArray(config.oauth)) {
    const client_id = config.oauth.clientId ?? config.oauth.client_id
    const client_secret = config.oauth.clientSecret ?? config.oauth.client_secret
    const scope = config.oauth.scope
    if (client_id || client_secret || scope) {
      oauth = { client_id, client_secret, scope }
    }
  }

  return {
    input: {
      server_type: serverType,
      enabled,
      timeout,
      command: serverType === 'local' ? command : null,
      environment: serverType === 'local' ? environment : null,
      url: serverType === 'remote' ? urlValue : null,
      headers: serverType === 'remote' ? headers : null,
      oauth: serverType === 'remote' ? oauth : null
    }
  }
}

// 打开编辑 MCP 弹窗
async function openEditMcpModal(server: McpServer) {
  editError.value = ''
  editingServerName.value = server.name
  editName.value = server.name
  editingServerDetail.value = null

  const fallbackDetail: McpServerDetail = {
    server_type: server.server_type,
    enabled: server.enabled,
    timeout: null,
    command: server.command,
    environment: {},
    url: server.url,
    headers: {},
    oauth: null
  }

  try {
    const detail = await invoke<McpServerDetail | null>('get_mcp_server', { name: server.name })
    editingServerDetail.value = detail ?? fallbackDetail
  } catch (e) {
    editingServerDetail.value = fallbackDetail
  }

  editJson.value = buildEditableConfig(editingServerDetail.value ?? fallbackDetail)
  showEditMcpModal.value = true
}

// 保存编辑后的 MCP
async function saveEditedMcp() {
  editError.value = ''
  if (!editingServerName.value) return

  if (!editName.value.trim()) {
    editError.value = t('mcp.customNameRequired')
    return
  }

  const parsed = parseMcpConfig(editJson.value, editingServerDetail.value?.enabled ?? true)
  if (!parsed.input) {
    editError.value = parsed.error || ''
    return
  }

  savingEdit.value = true
  try {
    await invoke('update_mcp_server', {
      old_name: editingServerName.value,
      input: {
        name: editName.value.trim(),
        ...parsed.input
      }
    })

    showEditMcpModal.value = false
    installMessage.value = t('mcp.customUpdated', { name: editName.value })
    await loadServers()
    selectedServer.value = editName.value.trim()
    setTimeout(() => { installMessage.value = '' }, 3000)
  } catch (e: any) {
    editError.value = e.toString()
  } finally {
    savingEdit.value = false
  }
}

// 打开自定义MCP弹窗
function openCustomModal() {
  customName.value = ''
  customJson.value = `{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-memory"],
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
  
  const parsed = parseMcpConfig(customJson.value, true)
  if (!parsed.input) {
    customError.value = parsed.error || ''
    return
  }
  
  addingCustom.value = true
  try {
    await invoke('add_mcp_server', {
      input: {
        name: customName.value.trim(),
        ...parsed.input
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

// 安装选中的推荐规则（支持多目标）
async function addSelectedRules() {
  if (selectedRules.value.size === 0) return
  
  // 检查是否至少选择了一个安装位置
  const hasTarget = ruleInstallTargets.value.opencode || 
                    ruleInstallTargets.value.claudeCode || 
                    ruleInstallTargets.value.codex || 
                    ruleInstallTargets.value.gemini ||
                    ruleInstallTargets.value.cursor
  if (!hasTarget) {
    installMessage.value = t('rule.selectInstallTarget')
    setTimeout(() => { installMessage.value = '' }, 3000)
    return
  }
  
  addingRules.value = true
  let successCount = 0
  let failCount = 0
  
  try {
    for (const ruleId of selectedRules.value) {
      const rule = recommendedRules.value.find(r => r.id === ruleId)
      if (!rule) continue
      
      try {
        // 1. 安装到 OpenCode
        if (ruleInstallTargets.value.opencode) {
          await invoke('install_rule', {
            ruleId: ruleId,
            content: '',
            location: 'global_opencode'
          })
        }
        
        // 2. 安装到 Claude Code（rules 目录 + CLAUDE.md）
        if (ruleInstallTargets.value.claudeCode) {
          await invoke('install_rule', {
            ruleId: ruleId,
            content: '',
            location: 'global_claude'
          })
        }
        
        // 3. 安装到 Cursor（rules 目录）
        if (ruleInstallTargets.value.cursor) {
          await invoke('install_rule', {
            ruleId: ruleId,
            content: '',
            location: 'global_cursor'
          })
        }
        
        // 4. 同步到 CLI 工具的系统提示文件
        const syncTargets: string[] = []
        if (ruleInstallTargets.value.claudeCode) syncTargets.push('claude')
        if (ruleInstallTargets.value.codex) syncTargets.push('codex')
        if (ruleInstallTargets.value.gemini) syncTargets.push('gemini')
        
        if (syncTargets.length > 0 && rule.content) {
          await invoke('sync_prompt', {
            content: rule.content,
            targets: syncTargets
          })
        }
        
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
  // 重置安装位置选项（默认只选中 OpenCode）
  ruleInstallTargets.value = {
    opencode: true,
    claudeCode: false,
    codex: false,
    gemini: false,
    cursor: false,
  }
  showCustomRuleModal.value = true
}

// 添加自定义规则（支持多目标）
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
  
  // 检查是否至少选择了一个安装位置
  const hasTarget = ruleInstallTargets.value.opencode || 
                    ruleInstallTargets.value.claudeCode || 
                    ruleInstallTargets.value.codex || 
                    ruleInstallTargets.value.gemini ||
                    ruleInstallTargets.value.cursor
  if (!hasTarget) {
    customRuleError.value = t('rule.selectInstallTarget')
    return
  }
  
  addingCustomRule.value = true
  try {
    const ruleId = customRuleName.value.trim().toLowerCase().replace(/\s+/g, '-')
    
    // 1. 安装到 OpenCode
    if (ruleInstallTargets.value.opencode) {
      await invoke('install_rule', {
        ruleId: ruleId,
        content: customRuleContent.value,
        location: 'global_opencode'
      })
    }
    
    // 2. 安装到 Claude Code（rules 目录）
    if (ruleInstallTargets.value.claudeCode) {
      await invoke('install_rule', {
        ruleId: ruleId,
        content: customRuleContent.value,
        location: 'global_claude'
      })
    }
    
    // 3. 安装到 Cursor（rules 目录）
    if (ruleInstallTargets.value.cursor) {
      await invoke('install_rule', {
        ruleId: ruleId,
        content: customRuleContent.value,
        location: 'global_cursor'
      })
    }
    
    // 4. 同步到 CLI 工具的系统提示文件
    const syncTargets: string[] = []
    if (ruleInstallTargets.value.claudeCode) syncTargets.push('claude')
    if (ruleInstallTargets.value.codex) syncTargets.push('codex')
    if (ruleInstallTargets.value.gemini) syncTargets.push('gemini')
    
    if (syncTargets.length > 0) {
      await invoke('sync_prompt', {
        content: customRuleContent.value,
        targets: syncTargets
      })
    }
    
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
  loadAppsMcpStatus()
})

const currentServer = () => servers.value.find(s => s.name === selectedServer.value)
</script>

<template>
  <div class="h-full flex flex-col gap-3">
    <!-- 顶部标签页 + 工具栏 -->
    <div class="flex items-center justify-between flex-shrink-0">
      <!-- 标签页切换 -->
      <div class="flex items-center gap-2">
        <div class="flex rounded-lg bg-surface/50 p-1">
          <button
            @click="activeTab = 'mcp'; selectedRule = null"
            class="px-4 py-1.5 rounded-md text-sm font-medium transition-all flex items-center gap-2"
            :class="activeTab === 'mcp' 
              ? 'bg-accent text-white shadow-sm' 
              : 'text-muted-foreground hover:text-foreground'"
          >
            <SvgIcon name="terminal" :size="14" />
            MCP
            <span class="text-xs opacity-70">({{ servers.length }})</span>
          </button>
          <button
            @click="activeTab = 'rules'; selectedServer = null"
            class="px-4 py-1.5 rounded-md text-sm font-medium transition-all flex items-center gap-2"
            :class="activeTab === 'rules' 
              ? 'bg-violet-500 text-white shadow-sm' 
              : 'text-muted-foreground hover:text-foreground'"
          >
            <SvgIcon name="code" :size="14" />
            {{ t('rule.title') }}
            <span class="text-xs opacity-70">({{ uniqueInstalledRules.length }})</span>
          </button>
        </div>
      </div>

      <!-- 右侧操作按钮 -->
      <div class="flex items-center gap-2">
        <!-- 操作结果消息 -->
        <div v-if="installMessage" class="text-sm text-accent animate-pulse mr-2">
          {{ installMessage }}
        </div>
        
        <!-- MCP 按钮组 -->
        <template v-if="activeTab === 'mcp'">
          <button
            @click="openMcpManageModal"
            class="px-3 py-1.5 rounded-lg bg-blue-500/20 hover:bg-blue-500/30 text-blue-400 font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="settings" :size="12" />
            {{ t('mcp.manage') }}
          </button>
          <button
            @click="openRecommendedModal"
            class="px-3 py-1.5 rounded-lg bg-accent/20 hover:bg-accent/30 text-accent font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="star" :size="12" />
            {{ t('mcp.addRecommended') }}
          </button>
          <button
            @click="openCustomModal"
            class="px-3 py-1.5 rounded-lg hover:bg-surface-hover text-foreground font-medium text-xs transition-all flex items-center gap-1.5 border border-border"
          >
            <SvgIcon name="plus" :size="12" />
            {{ t('mcp.addCustom') }}
          </button>
        </template>
        
        <!-- 规则按钮组 -->
        <template v-else>
          <button
            @click="openRuleManageModal"
            class="px-3 py-1.5 rounded-lg bg-blue-500/20 hover:bg-blue-500/30 text-blue-400 font-medium text-xs transition-all flex items-center justify-center gap-1.5"
          >
            <SvgIcon name="settings" :size="12" />
            {{ t('rule.manage') }}
          </button>
          <button
            @click="openRecommendedRulesModal"
            class="px-3 py-1.5 rounded-lg bg-violet-500/20 hover:bg-violet-500/30 text-violet-400 font-medium text-xs transition-all flex items-center gap-1.5"
          >
            <SvgIcon name="code" :size="12" />
            {{ t('rule.addRecommended') }}
          </button>
          <button
            @click="openCustomRuleModal"
            class="px-3 py-1.5 rounded-lg hover:bg-surface-hover text-foreground font-medium text-xs transition-all flex items-center gap-1.5 border border-border"
          >
            <SvgIcon name="edit" :size="12" />
            {{ t('rule.addCustom') }}
          </button>
        </template>
      </div>
    </div>

    <!-- 已部署应用状态栏 (仅在 MCP 标签页显示) -->
    <div v-if="activeTab === 'mcp' && appsMcpStatus.length > 0" class="flex-shrink-0">
      <div class="rounded-lg bg-surface/30 border border-border px-3 py-2">
        <div class="flex items-center gap-4 flex-wrap">
          <span class="text-xs text-muted-foreground flex items-center gap-1.5">
            <SvgIcon name="layers" :size="12" class="text-emerald-400" />
            {{ t('mcp.deployedApps') }}:
          </span>
          <div class="flex items-center gap-3 flex-wrap">
            <div
              v-for="app in appsMcpStatus"
              :key="app.app_name"
              class="flex items-center gap-1.5 text-xs"
            >
              <span 
                class="w-1.5 h-1.5 rounded-full"
                :class="app.server_count > 0 ? 'bg-emerald-500' : 'bg-gray-400'"
              ></span>
              <span class="text-muted-foreground">{{ app.app_name }}</span>
              <span 
                class="px-1.5 py-0.5 rounded text-[10px]"
                :class="app.server_count > 0 ? 'bg-emerald-500/20 text-emerald-500' : 'bg-surface text-muted-foreground'"
              >
                {{ app.server_count }}
              </span>
              <!-- 导入按钮 -->
              <button
                v-if="app.server_count > 0 && app.app_name !== 'OpenCode'"
                @click="importMcpFromApp(app.app_name)"
                :disabled="importingApp === app.app_name"
                class="text-[10px] px-1 py-0.5 rounded bg-accent/10 text-accent hover:bg-accent/20 transition-colors disabled:opacity-50"
                :title="t('mcp.importFromApp')"
              >
                {{ importingApp === app.app_name ? '...' : '↓' }}
              </button>
            </div>
          </div>
          <button
            @click="loadAppsMcpStatus"
            class="ml-auto text-muted-foreground hover:text-foreground transition-colors"
            title="刷新"
          >
            <SvgIcon name="refresh" :size="12" />
          </button>
        </div>
      </div>
    </div>
    
    <!-- 主内容区 -->
    <div class="flex-1 flex gap-4 min-h-0">
      <!-- 左侧列表区域 -->
      <div class="w-80 flex-shrink-0 flex flex-col rounded-xl bg-surface/30 border border-border overflow-hidden">
        <!-- MCP 列表 -->
        <template v-if="activeTab === 'mcp'">
          <div class="flex-1 overflow-auto">
            <div v-if="loading" class="p-4 text-center text-muted-foreground text-sm">
              {{ t('mcp.loading') }}
            </div>
            <div v-else-if="servers.length === 0" class="p-6 text-center text-muted-foreground">
              <SvgIcon name="terminal" :size="32" class="mx-auto mb-2 opacity-30" />
              <p class="text-sm">{{ t('mcp.noServers') }}</p>
              <p class="text-xs mt-1">点击右上角添加 MCP 服务器</p>
            </div>
            <ul v-else class="p-2 space-y-1">
              <li
                v-for="server in servers"
                :key="server.name"
                @click="selectedServer = server.name; selectedRule = null"
                class="px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
                :class="[
                  server.name === selectedServer
                    ? 'bg-accent/10 border border-accent/40'
                    : 'hover:bg-surface-hover border border-transparent'
                ]"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span 
                      class="w-2 h-2 rounded-full flex-shrink-0"
                      :class="[
                        healthStatus[server.name] === undefined ? 'bg-gray-400 animate-pulse' :
                        healthStatus[server.name]?.healthy ? 'bg-emerald-500' : 'bg-amber-500'
                      ]"
                    ></span>
                    <SvgIcon :name="server.server_type === 'local' ? 'box' : 'globe'" :size="14" class="text-muted-foreground" />
                    <span class="font-medium text-sm truncate max-w-[160px]">{{ server.name }}</span>
                  </div>
                  <button
                    @click.stop="toggleServer(server.name)"
                    class="text-xs px-2 py-0.5 rounded transition-colors"
                    :class="server.enabled ? 'bg-emerald-500/20 text-emerald-500' : 'bg-surface text-muted-foreground'"
                  >
                    {{ server.enabled ? t('mcp.enabled') : t('mcp.disabled') }}
                  </button>
                </div>
                <div v-if="server.package_name" class="mt-1 pl-6 text-xs text-muted-foreground truncate">
                  {{ server.package_name }}
                </div>
              </li>
            </ul>
          </div>
        </template>

        <!-- 规则列表 -->
        <template v-else>
          <div class="flex-1 overflow-auto">
            <div v-if="rulesLoading" class="p-4 text-center text-muted-foreground text-sm">
              {{ t('mcp.loading') }}
            </div>
            <div v-else-if="uniqueInstalledRules.length === 0" class="p-6 text-center text-muted-foreground">
              <SvgIcon name="code" :size="32" class="mx-auto mb-2 opacity-30" />
              <p class="text-sm">{{ t('rule.noRules') }}</p>
              <p class="text-xs mt-1">点击右上角添加规则</p>
            </div>
            <ul v-else class="p-2 space-y-1">
              <li
                v-for="rule in uniqueInstalledRules"
                :key="rule.name"
                @click="selectRule(rule)"
                class="px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
                :class="[
                  selectedRule?.name === rule.name
                    ? 'bg-violet-500/10 border border-violet-500/40'
                    : 'hover:bg-surface-hover border border-transparent'
                ]"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <SvgIcon name="book" :size="14" class="text-muted-foreground" />
                    <span 
                      class="font-medium text-sm truncate max-w-[180px]"
                      :class="!rule.enabled && 'text-muted-foreground line-through'"
                    >{{ rule.name }}</span>
                  </div>
                  <button
                    @click="toggleRule(rule, $event)"
                    class="text-xs px-2 py-0.5 rounded transition-colors"
                    :class="rule.enabled ? 'bg-emerald-500/20 text-emerald-500' : 'bg-surface text-muted-foreground'"
                  >
                    {{ rule.enabled ? t('mcp.enabled') : t('mcp.disabled') }}
                  </button>
                </div>
                <div class="mt-1 pl-6 flex flex-wrap gap-1">
                  <span 
                    v-for="app in rule.deployedTo" 
                    :key="app" 
                    class="text-xs px-1.5 py-0.5 rounded bg-violet-500/10 text-violet-400"
                  >
                    {{ app }}
                  </span>
                </div>
              </li>
            </ul>
          </div>
        </template>
      </div>

      <!-- 详情面板 -->
      <div class="flex-1">
        <div class="h-full rounded-xl bg-surface/30 border border-border p-5 overflow-auto">
          <!-- 未选中任何项 -->
          <div v-if="(activeTab === 'mcp' && !selectedServer) || (activeTab === 'rules' && !selectedRule)" class="h-full flex items-center justify-center">
            <div class="text-center text-muted-foreground">
              <SvgIcon :name="activeTab === 'mcp' ? 'terminal' : 'code'" :size="48" class="mx-auto mb-3 opacity-20" />
              <p>{{ t('mcp.selectItem') }}</p>
            </div>
          </div>
          
          <!-- MCP 详情 -->
          <div v-else-if="activeTab === 'mcp' && currentServer()" class="space-y-5">
            <div class="flex items-center justify-between">
              <h3 class="font-semibold text-xl flex items-center gap-2">
                <SvgIcon name="terminal" :size="20" class="text-accent" /> {{ currentServer()?.name }}
              </h3>
              <div class="flex gap-2">
                <button
                  @click="openEditMcpModal(currentServer()!)"
                  class="px-3 py-1.5 rounded-lg bg-accent/10 hover:bg-accent/20 text-accent text-sm transition-all flex items-center gap-1.5"
                >
                  <SvgIcon name="edit" :size="14" /> {{ t('common.edit') }}
                </button>
                <button
                  @click="deleteMcpServer(currentServer()!.name)"
                  class="px-3 py-1.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 text-sm transition-all flex items-center gap-1.5"
                >
                  <SvgIcon name="trash" :size="14" /> {{ t('common.delete') }}
                </button>
              </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div class="p-3 rounded-lg bg-surface/50 border border-border">
                <span class="text-xs text-muted-foreground block mb-1">{{ t('mcp.type') }}</span>
                <span class="text-sm font-medium">{{ currentServer()?.server_type === 'local' ? t('mcp.local') : t('mcp.remote') }}</span>
              </div>
              <div class="p-3 rounded-lg bg-surface/50 border border-border">
                <span class="text-xs text-muted-foreground block mb-1">{{ t('mcp.status') }}</span>
                <span 
                  class="text-sm font-medium"
                  :class="currentServer()?.enabled ? 'text-emerald-500' : 'text-muted-foreground'"
                >
                  {{ currentServer()?.enabled ? t('mcp.statusEnabled') : t('mcp.statusDisabled') }}
                </span>
              </div>
              <div class="p-3 rounded-lg bg-surface/50 border border-border">
                <span class="text-xs text-muted-foreground block mb-1">{{ t('mcp.effective') }}</span>
                <span 
                  class="text-sm font-medium"
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
              <div v-if="currentServer()?.package_name" class="p-3 rounded-lg bg-surface/50 border border-border">
                <span class="text-xs text-muted-foreground block mb-1">{{ t('mcp.package') }}</span>
                <span class="text-sm font-medium text-accent">{{ currentServer()?.package_name }}</span>
              </div>
            </div>
            
            <div v-if="currentServer()?.command" class="space-y-2">
              <span class="text-sm text-muted-foreground">{{ t('mcp.command') }}</span>
              <div class="p-3 rounded-lg bg-surface/50 border border-border font-mono text-sm break-all">
                {{ currentServer()?.command?.join(' ') }}
              </div>
            </div>
            
            <div v-if="currentServer()?.url" class="space-y-2">
              <span class="text-sm text-muted-foreground">{{ t('mcp.url') }}</span>
              <div class="p-3 rounded-lg bg-surface/50 border border-border font-mono text-sm break-all">
                {{ currentServer()?.url }}
              </div>
            </div>
            
            <div class="space-y-2">
              <span class="text-sm text-muted-foreground">{{ t('mcp.installPath') }}</span>
              <div class="p-3 rounded-lg bg-surface/50 border border-border font-mono text-xs break-all text-muted-foreground">
                {{ currentServer()?.install_path }}
              </div>
            </div>
          </div>
          
          <!-- 规则详情 -->
          <div v-else-if="activeTab === 'rules' && selectedRule" class="space-y-5">
            <div class="flex items-center justify-between">
              <h3 class="font-semibold text-xl flex items-center gap-2">
                <SvgIcon name="code" :size="20" class="text-violet-400" /> {{ selectedRule.name }}
              </h3>
              <div class="flex gap-2">
                <button
                  @click="openEditRuleModal(selectedRule)"
                  class="px-3 py-1.5 rounded-lg bg-accent/10 hover:bg-accent/20 text-accent text-sm transition-all flex items-center gap-1.5"
                >
                  <SvgIcon name="edit" :size="14" /> {{ t('common.edit') }}
                </button>
                <button
                  @click="deleteInstalledRule(selectedRule)"
                  class="px-3 py-1.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 text-sm transition-all flex items-center gap-1.5"
                >
                  <SvgIcon name="trash" :size="14" /> {{ t('common.delete') }}
                </button>
              </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4">
              <div class="p-3 rounded-lg bg-surface/50 border border-border">
                <span class="text-xs text-muted-foreground block mb-1">{{ t('mcp.status') }}</span>
                <button
                  @click="toggleRule(selectedRule, $event)"
                  class="px-2 py-0.5 rounded text-sm font-medium transition-colors"
                  :class="selectedRule.enabled 
                    ? 'bg-emerald-500/20 text-emerald-500 hover:bg-emerald-500/30' 
                    : 'bg-amber-500/20 text-amber-500 hover:bg-amber-500/30'"
                >
                  {{ selectedRule.enabled ? t('mcp.statusEnabled') : t('mcp.statusDisabled') }}
                </button>
              </div>
              <div class="p-3 rounded-lg bg-surface/50 border border-border">
                <span class="text-xs text-muted-foreground block mb-1">{{ t('rule.type') }}</span>
                <span class="text-sm font-medium">{{ selectedRule.rule_type }}</span>
              </div>
            </div>
            
            <!-- 部署状态 -->
            <div class="p-3 rounded-lg bg-surface/50 border border-border">
              <span class="text-xs text-muted-foreground block mb-2">已部署到</span>
              <div class="flex flex-wrap gap-2">
                <span 
                  v-for="app in (selectedRuleDeployment?.deployedTo || [getLocationLabel(selectedRule.location)])" 
                  :key="app" 
                  class="px-2 py-1 rounded-lg bg-violet-500/20 text-violet-400 text-sm font-medium"
                >
                  ✓ {{ app }}
                </span>
              </div>
            </div>
            
            <div class="space-y-2">
              <span class="text-sm text-muted-foreground">{{ t('rule.path') }}</span>
              <div class="p-3 rounded-lg bg-surface/50 border border-border font-mono text-xs break-all text-muted-foreground">
                {{ selectedRule.path }}
              </div>
            </div>
            
            <!-- 规则内容预览 -->
            <div class="space-y-2">
              <span class="text-sm font-medium">{{ t('rule.content') || '规则内容' }}</span>
              <div class="bg-surface/50 border border-border rounded-lg p-4 max-h-[350px] overflow-auto">
                <pre class="text-sm font-mono whitespace-pre-wrap text-muted-foreground leading-relaxed">{{ selectedRuleContent || selectedRule.description }}</pre>
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
    
    <!-- 编辑MCP弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showEditMcpModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showEditMcpModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[500px] overflow-hidden shadow-2xl">
          <!-- 弹窗头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold">{{ t('mcp.editTitle') }}</h2>
            <button 
              @click="showEditMcpModal = false"
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
                v-model="editName"
                type="text"
                :placeholder="t('mcp.customNamePlaceholder')"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-accent focus:outline-none text-sm"
              />
            </div>
            
            <!-- JSON配置 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('mcp.customConfig') }}</label>
              <textarea
                v-model="editJson"
                rows="8"
                class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-accent focus:outline-none text-sm font-mono resize-none"
                spellcheck="false"
              ></textarea>
              <p class="text-xs text-muted-foreground mt-1">{{ t('mcp.customConfigHint') }}</p>
            </div>
            
            <!-- 错误提示 -->
            <div v-if="editError" class="text-sm text-red-500 bg-red-500/10 px-3 py-2 rounded-lg">
              {{ editError }}
            </div>
          </div>
          
          <!-- 弹窗底部 -->
          <div class="px-6 py-4 border-t border-border flex justify-end gap-2">
            <button
              @click="showEditMcpModal = false"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="saveEditedMcp"
              :disabled="savingEdit"
              class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all disabled:opacity-50"
            >
              {{ savingEdit ? t('common.saving') : t('common.save') }}
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
          
          <!-- 安装位置选择（多选） -->
          <div class="px-6 py-3 border-b border-border bg-surface/30">
            <label class="text-sm font-medium block mb-2">{{ t('rule.installLocation') }}</label>
            <div class="flex flex-wrap gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="ruleInstallTargets.opencode" class="w-4 h-4 rounded border-border accent-violet-500" />
                <span class="text-sm">OpenCode</span>
                <span class="text-xs text-muted-foreground">(~/.config/opencode/rules/)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="ruleInstallTargets.claudeCode" class="w-4 h-4 rounded border-border accent-orange-500" />
                <span class="text-sm">Claude Code</span>
                <span class="text-xs text-muted-foreground">(~/.claude/)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="ruleInstallTargets.codex" class="w-4 h-4 rounded border-border accent-green-500" />
                <span class="text-sm">Codex</span>
                <span class="text-xs text-muted-foreground">(AGENTS.md)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="ruleInstallTargets.gemini" class="w-4 h-4 rounded border-border accent-blue-500" />
                <span class="text-sm">Gemini</span>
                <span class="text-xs text-muted-foreground">(GEMINI.md)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="ruleInstallTargets.cursor" class="w-4 h-4 rounded border-border accent-purple-500" />
                <span class="text-sm">Cursor</span>
                <span class="text-xs text-muted-foreground">(~/.cursor/rules/)</span>
              </label>
            </div>
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
            
            <!-- 安装位置（多选） -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('rule.installLocation') }}</label>
              <div class="grid grid-cols-2 gap-2">
                <label class="flex items-center gap-2 cursor-pointer p-2 rounded-lg bg-surface/50 border border-border hover:border-violet-500/50">
                  <input type="checkbox" v-model="ruleInstallTargets.opencode" class="w-4 h-4 rounded border-border accent-violet-500" />
                  <div>
                    <span class="text-sm font-medium">OpenCode</span>
                    <p class="text-xs text-muted-foreground">~/.config/opencode/rules/</p>
                  </div>
                </label>
                <label class="flex items-center gap-2 cursor-pointer p-2 rounded-lg bg-surface/50 border border-border hover:border-orange-500/50">
                  <input type="checkbox" v-model="ruleInstallTargets.claudeCode" class="w-4 h-4 rounded border-border accent-orange-500" />
                  <div>
                    <span class="text-sm font-medium">Claude Code</span>
                    <p class="text-xs text-muted-foreground">~/.claude/ + CLAUDE.md</p>
                  </div>
                </label>
                <label class="flex items-center gap-2 cursor-pointer p-2 rounded-lg bg-surface/50 border border-border hover:border-green-500/50">
                  <input type="checkbox" v-model="ruleInstallTargets.codex" class="w-4 h-4 rounded border-border accent-green-500" />
                  <div>
                    <span class="text-sm font-medium">Codex</span>
                    <p class="text-xs text-muted-foreground">AGENTS.md</p>
                  </div>
                </label>
                <label class="flex items-center gap-2 cursor-pointer p-2 rounded-lg bg-surface/50 border border-border hover:border-blue-500/50">
                  <input type="checkbox" v-model="ruleInstallTargets.gemini" class="w-4 h-4 rounded border-border accent-blue-500" />
                  <div>
                    <span class="text-sm font-medium">Gemini</span>
                    <p class="text-xs text-muted-foreground">GEMINI.md</p>
                  </div>
                </label>
                <label class="flex items-center gap-2 cursor-pointer p-2 rounded-lg bg-surface/50 border border-border hover:border-purple-500/50">
                  <input type="checkbox" v-model="ruleInstallTargets.cursor" class="w-4 h-4 rounded border-border accent-purple-500" />
                  <div>
                    <span class="text-sm font-medium">Cursor</span>
                    <p class="text-xs text-muted-foreground">~/.cursor/rules/</p>
                  </div>
                </label>
              </div>
            </div>
            
            <!-- 规则内容 -->
            <div>
              <label class="block text-sm font-medium mb-2">{{ t('rule.customContent') }}</label>
              <textarea
                v-model="customRuleContent"
                rows="8"
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
                  rows="14"
                  class="w-full px-3 py-2 rounded-lg bg-surface border border-border focus:border-violet-500 focus:outline-none text-sm font-mono resize-none"
                  spellcheck="false"
                ></textarea>
              </div>
              
              <!-- CLI 工具同步选项 -->
              <div class="pt-2 border-t border-border">
                <label class="block text-sm font-medium mb-2">{{ t('rule.syncToCliTools') }}</label>
                <div class="flex flex-wrap gap-3">
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input type="checkbox" v-model="editSyncToClaudeMd" class="w-4 h-4 rounded border-border" />
                    <span class="text-sm">Claude Code (CLAUDE.md)</span>
                  </label>
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input type="checkbox" v-model="editSyncToAgentsMd" class="w-4 h-4 rounded border-border" />
                    <span class="text-sm">Codex (AGENTS.md)</span>
                  </label>
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input type="checkbox" v-model="editSyncToGeminiMd" class="w-4 h-4 rounded border-border" />
                    <span class="text-sm">Gemini (GEMINI.md)</span>
                  </label>
                </div>
                <p class="text-xs text-muted-foreground mt-1">{{ t('rule.syncToCliToolsHint') }}</p>
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
    
    <!-- MCP 管理弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showMcpManageModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showMcpManageModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[850px] max-h-[85vh] overflow-hidden shadow-2xl flex flex-col">
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between flex-shrink-0">
            <h2 class="text-lg font-semibold flex items-center gap-2">
              <SvgIcon name="settings" :size="20" class="text-blue-400" />
              {{ t('mcp.manageTitle') }}
            </h2>
            <button 
              @click="showMcpManageModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 统计信息 -->
          <div class="px-6 py-3 border-b border-border bg-surface/30 text-sm text-muted-foreground flex-shrink-0">
            {{ t('mcp.installed') }} · OpenCode: {{ mcpStats.opencode_count }} · Claude: {{ mcpStats.claude_count }} · Codex: {{ mcpStats.codex_count }} · Gemini: {{ mcpStats.gemini_count }} · Cursor: {{ mcpStats.cursor_count }}
          </div>
          
          <!-- 搜索框 -->
          <div class="px-6 py-3 border-b border-border flex-shrink-0">
            <input
              v-model="mcpManageSearchQuery"
              type="text"
              :placeholder="t('mcp.searchMcp')"
              class="w-full px-4 py-2 rounded-lg bg-surface border border-border focus:border-accent focus:outline-none text-sm"
            />
          </div>
          
          <!-- MCP 列表 -->
          <div class="flex-1 overflow-auto p-4 space-y-3 min-h-0">
            <div v-if="filteredManagedMcps.length === 0" class="text-center text-muted-foreground py-8">
              {{ t('mcp.noServers') }}
            </div>
            
            <div
              v-for="mcp in filteredManagedMcps"
              :key="mcp.name"
              class="p-4 rounded-xl border border-border bg-surface/30 hover:bg-surface/50 transition-all"
            >
              <div class="flex items-start justify-between gap-4">
                <!-- 左侧：名称和描述 -->
                <div class="flex-1 min-w-0">
                  <h3 class="font-semibold text-foreground">{{ mcp.name }}</h3>
                  <p class="text-sm text-muted-foreground mt-1 truncate">
                    {{ mcp.package_name || (mcp.command ? mcp.command.join(' ') : mcp.url) || '暂无描述' }}
                  </p>
                  <span class="inline-block mt-2 text-xs px-2 py-0.5 rounded" 
                        :class="mcp.server_type === 'local' ? 'bg-accent/20 text-accent' : 'bg-violet-500/20 text-violet-400'">
                    {{ mcp.server_type === 'local' ? t('mcp.local') : t('mcp.remote') }}
                  </span>
                </div>
                
                <!-- 右侧：应用开关 -->
                <div class="flex flex-col gap-2 flex-shrink-0">
                  <!-- OpenCode 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">OpenCode</span>
                    <button
                      @click="toggleMcpApp(mcp, 'opencode')"
                      :disabled="togglingMcp !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="mcp.opencode_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="mcp.opencode_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Claude 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Claude</span>
                    <button
                      @click="toggleMcpApp(mcp, 'claude')"
                      :disabled="togglingMcp !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="mcp.claude_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="mcp.claude_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Codex 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Codex</span>
                    <button
                      @click="toggleMcpApp(mcp, 'codex')"
                      :disabled="togglingMcp !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="mcp.codex_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="mcp.codex_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Gemini 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Gemini</span>
                    <button
                      @click="toggleMcpApp(mcp, 'gemini')"
                      :disabled="togglingMcp !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="mcp.gemini_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="mcp.gemini_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Cursor 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Cursor</span>
                    <button
                      @click="toggleMcpApp(mcp, 'cursor')"
                      :disabled="togglingMcp !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="mcp.cursor_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="mcp.cursor_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                </div>
                
                <!-- 删除按钮 -->
                <button
                  @click="deleteMcpFromAll(mcp)"
                  class="p-2 rounded-lg text-muted-foreground hover:text-red-500 hover:bg-red-500/10 transition-colors flex-shrink-0"
                  :title="t('mcp.deleteFromAll')"
                >
                  <SvgIcon name="delete" :size="16" />
                </button>
              </div>
            </div>
          </div>
          
          <!-- 底部 -->
          <div class="px-6 py-4 border-t border-border flex items-center justify-between flex-shrink-0">
            <span class="text-sm text-muted-foreground">
              {{ t('mcp.totalMcps', { count: managedMcps.length }) }}
            </span>
            <div class="flex gap-2">
              <button
                @click="loadManagedMcps"
                class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all flex items-center gap-2"
              >
                <SvgIcon name="refresh" :size="14" />
                {{ t('common.refresh') }}
              </button>
              <button
                @click="showMcpManageModal = false"
                class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all"
              >
                {{ t('common.confirm') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 规则管理弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showRuleManageModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showRuleManageModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[850px] max-h-[85vh] overflow-hidden shadow-2xl flex flex-col">
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between flex-shrink-0">
            <h2 class="text-lg font-semibold flex items-center gap-2">
              <SvgIcon name="settings" :size="20" class="text-violet-400" />
              {{ t('rule.manageTitle') }}
            </h2>
            <button 
              @click="showRuleManageModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 统计信息 -->
          <div class="px-6 py-3 border-b border-border bg-surface/30 text-sm text-muted-foreground flex-shrink-0">
            {{ t('rule.installed') }} · OpenCode: {{ ruleStats.opencode_count }} · Claude: {{ ruleStats.claude_count }} · Codex: {{ ruleStats.codex_count }} · Gemini: {{ ruleStats.gemini_count }} · Cursor: {{ ruleStats.cursor_count }}
          </div>
          
          <!-- 搜索框 -->
          <div class="px-6 py-3 border-b border-border flex-shrink-0">
            <input
              v-model="ruleManageSearchQuery"
              type="text"
              :placeholder="t('rule.searchRule')"
              class="w-full px-4 py-2 rounded-lg bg-surface border border-border focus:border-violet-500 focus:outline-none text-sm"
            />
          </div>
          
          <!-- 规则列表 -->
          <div class="flex-1 overflow-auto p-4 space-y-3 min-h-0">
            <div v-if="filteredManagedRules.length === 0" class="text-center text-muted-foreground py-8">
              {{ t('rule.noRules') }}
            </div>
            
            <div
              v-for="rule in filteredManagedRules"
              :key="rule.name"
              class="p-4 rounded-xl border border-border bg-surface/30 hover:bg-surface/50 transition-all"
            >
              <div class="flex items-start justify-between gap-4">
                <!-- 左侧：名称和描述 -->
                <div class="flex-1 min-w-0">
                  <h3 class="font-semibold text-foreground">{{ rule.name }}</h3>
                  <p class="text-sm text-muted-foreground mt-1 line-clamp-2">{{ rule.description || '暂无描述' }}</p>
                  <span class="inline-block mt-2 text-xs px-2 py-0.5 rounded bg-violet-500/20 text-violet-400">
                    {{ rule.rule_type }}
                  </span>
                </div>
                
                <!-- 右侧：应用开关 -->
                <div class="flex flex-col gap-2 flex-shrink-0">
                  <!-- OpenCode 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">OpenCode</span>
                    <button
                      @click="toggleRuleApp(rule, 'opencode')"
                      :disabled="togglingRule !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="rule.opencode_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="rule.opencode_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Claude 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Claude</span>
                    <button
                      @click="toggleRuleApp(rule, 'claude')"
                      :disabled="togglingRule !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="rule.claude_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="rule.claude_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Codex 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Codex</span>
                    <button
                      @click="toggleRuleApp(rule, 'codex')"
                      :disabled="togglingRule !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="rule.codex_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="rule.codex_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Gemini 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Gemini</span>
                    <button
                      @click="toggleRuleApp(rule, 'gemini')"
                      :disabled="togglingRule !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="rule.gemini_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="rule.gemini_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                  
                  <!-- Cursor 开关 -->
                  <div class="flex items-center justify-between gap-3 min-w-[130px]">
                    <span class="text-sm text-muted-foreground">Cursor</span>
                    <button
                      @click="toggleRuleApp(rule, 'cursor')"
                      :disabled="togglingRule !== null"
                      class="relative w-11 h-6 rounded-full transition-colors duration-200"
                      :class="rule.cursor_enabled ? 'bg-emerald-500' : 'bg-gray-600'"
                    >
                      <span
                        class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200"
                        :class="rule.cursor_enabled ? 'translate-x-5' : 'translate-x-0'"
                      ></span>
                    </button>
                  </div>
                </div>
                
                <!-- 删除按钮 -->
                <button
                  @click="deleteRuleFromAll(rule)"
                  class="p-2 rounded-lg text-muted-foreground hover:text-red-500 hover:bg-red-500/10 transition-colors flex-shrink-0"
                  :title="t('rule.deleteFromAll')"
                >
                  <SvgIcon name="delete" :size="16" />
                </button>
              </div>
            </div>
          </div>
          
          <!-- 底部 -->
          <div class="px-6 py-4 border-t border-border flex items-center justify-between flex-shrink-0">
            <span class="text-sm text-muted-foreground">
              {{ t('rule.totalRules', { count: managedRules.length }) }}
            </span>
            <div class="flex gap-2">
              <button
                @click="loadManagedRules"
                class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all flex items-center gap-2"
              >
                <SvgIcon name="refresh" :size="14" />
                {{ t('common.refresh') }}
              </button>
              <button
                @click="showRuleManageModal = false"
                class="px-4 py-2 rounded-lg bg-violet-500 hover:bg-violet-600 text-white text-sm font-medium transition-all"
              >
                {{ t('common.confirm') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

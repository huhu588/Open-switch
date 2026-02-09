<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { save, open } from '@tauri-apps/plugin-dialog'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

// ============================================================================
// 备份相关类型
// ============================================================================

interface ExportedModel { id: string; name: string; reasoning_effort?: string }
interface ExportedProvider { name: string; base_url: string; api_key: string; npm?: string; description?: string; model_type?: string; enabled: boolean; models: ExportedModel[] }
interface ExportedOAuthConfig { client_id?: string; client_secret?: string; scope?: string }
interface ExportedMcpServer { name: string; server_type: string; enabled: boolean; timeout?: number; command?: string[]; environment?: Record<string, string>; url?: string; headers?: Record<string, string>; oauth?: ExportedOAuthConfig }
interface ExportedRule { name: string; location: string; rule_type: string; content: string; file_ext?: string }
interface ExportedSkills { name: string; location: string; content: string }
interface ExportedCodexProvider { name: string; base_url: string; env_key?: string; requires_openai_auth?: boolean }
interface ExportedCodexMcpServer { name: string; command: string[]; env?: Record<string, string> }
interface ExportedCodexConfig { model_providers: ExportedCodexProvider[]; mcp_servers: ExportedCodexMcpServer[] }
interface ExportedGeminiEnv { gemini_api_key?: string; google_gemini_api_key?: string; google_gemini_base_url?: string; gemini_model?: string }
interface ExportedGeminiMcpServer { name: string; command?: string; args?: string[]; env?: Record<string, string>; url?: string }
interface ExportedGeminiConfig { env: ExportedGeminiEnv; mcp_servers: ExportedGeminiMcpServer[] }
interface ExportedUsageRecord { session_id: string; timestamp: number; model: string; source: string; input_tokens: number; output_tokens: number; cache_read_tokens: number; cache_creation_tokens: number; cost?: number }
interface BackupData { version: string; created_at: string; app_name: string; providers: ExportedProvider[]; mcp_servers: ExportedMcpServer[]; rules: ExportedRule[]; skills: ExportedSkills[]; codex_config?: ExportedCodexConfig; gemini_config?: ExportedGeminiConfig; usage_stats?: ExportedUsageRecord[]; dev_envs?: { id: string; name: string; version?: string }[] }
interface ExportedDevEnv { id: string; name: string; version?: string }
interface DevEnvInfo { id: string; name: string; installed: boolean; current_version: string | null }
interface ExportStats { providers: number; models: number; mcp_servers: number; rules: number; skills: number; codex_providers: number; codex_mcp_servers: number; gemini_configured: boolean; gemini_mcp_servers: number; usage_records: number; chat_conversations: number }
interface ImportResult { success: boolean; providers_imported: number; providers_skipped: number; mcp_imported: number; mcp_skipped: number; rules_imported: number; rules_skipped: number; skills_imported: number; skills_skipped: number; codex_imported: number; codex_skipped: number; gemini_imported: number; gemini_skipped: number; usage_imported: number; usage_skipped: number; errors: string[] }

// ============================================================================
// 对话迁移相关类型
// ============================================================================

interface ChatSourceInfo { name: string; key: string; path: string | null; conversationCount: number; available: boolean }
interface ChatScanResult { sources: ChatSourceInfo[] }
interface ExtractedMessage { role: string; content: string; model?: string; timestamp?: string; toolUse?: unknown }
interface ExtractedConversation { messages: ExtractedMessage[]; source: string; sessionId?: string; name?: string; createdAt?: number }
interface ExtractionResult { source: string; conversations: ExtractedConversation[]; total: number }
interface ChatExportResult { exported: number; filePath: string }
interface MigrationImportResult { imported: number; skipped: number; total: number }

// ============================================================================
// 备份状态
// ============================================================================

const isExporting = ref(false)
const isImportingBackup = ref(false)
const previewData = ref<BackupData | null>(null)
const showBackupPreview = ref(false)
const selectedFilePath = ref('')
const exportMessage = ref('')
const exportMessageType = ref<'success' | 'error'>('success')
const importMessage = ref('')
const importMessageType = ref<'success' | 'error'>('success')

// 导出选择面板状态
const showExportPanel = ref(false)
const isLoadingExportData = ref(false)
const fullBackupData = ref<BackupData | null>(null)
const selectedProviders = ref<string[]>([])
const selectedMcps = ref<string[]>([])
const selectedRules = ref<string[]>([])
const selectedSkills = ref<string[]>([])
const selectedCodexProviders = ref<string[]>([])
const selectedCodexMcps = ref<string[]>([])
const includeGeminiEnv = ref(true)
const selectedGeminiMcps = ref<string[]>([])
const selectedUsageSources = ref<string[]>([])
const selectedChatSources = ref<string[]>([])
// 开发环境（从后端检测得到）
const detectedEnvs = ref<DevEnvInfo[]>([])
const selectedDevEnvs = ref<Record<string, string>>({}) // id -> version

// 区块折叠状态
const sectionExpanded = ref<Record<string, boolean>>({
  providers: true, claude: true, codex: true, gemini: true,
  mcp: true, rules: true, skills: true, usage: false, chatRecords: false,
})
function toggleSection(key: string) { sectionExpanded.value[key] = !sectionExpanded.value[key] }

// 使用统计可选来源
const usageSourceOptions = [
  { key: 'claude', label: 'Claude', color: 'amber' },
  { key: 'codex', label: 'Codex', color: 'green' },
  { key: 'gemini', label: 'Gemini', color: 'pink' },
  { key: 'opencode', label: 'Opencode', color: 'blue' },
  { key: 'cursor', label: 'Cursor', color: 'cyan' },
]

const importOptions = ref({
  import_providers: true, import_mcp: true, import_rules: true,
  import_skills: true, import_codex: true, import_gemini: true,
  overwrite_existing: false, import_usage_stats: true,
})

const previewStats = computed(() => {
  if (!previewData.value) return null
  const codexConfig = previewData.value.codex_config
  const geminiConfig = previewData.value.gemini_config
  return {
    providers: previewData.value.providers.length,
    models: previewData.value.providers.reduce((sum, p) => sum + p.models.length, 0),
    mcp_servers: previewData.value.mcp_servers.length,
    rules: previewData.value.rules.length,
    skills: previewData.value.skills.length,
    codex_providers: codexConfig?.model_providers?.length || 0,
    codex_mcp_servers: codexConfig?.mcp_servers?.length || 0,
    gemini_configured: !!(geminiConfig?.env?.gemini_api_key || geminiConfig?.env?.google_gemini_api_key),
    gemini_mcp_servers: geminiConfig?.mcp_servers?.length || 0,
    usage_records: previewData.value.usage_stats?.length || 0,
  }
})

// ============================================================================
// 对话迁移状态
// ============================================================================

const isScanning = ref(false)
const scanResult = ref<ChatScanResult | null>(null)
const extractedData = ref<Map<string, ExtractedConversation[]>>(new Map())
const extractingSource = ref<string | null>(null)
const isChatExporting = ref(false)
const isChatImporting = ref(false)
const chatStatusMessage = ref('')
const chatStatusType = ref<'success' | 'error' | 'info'>('info')
const previewConversation = ref<ExtractedConversation | null>(null)
const showChatPreview = ref(false)

const toolMeta: Record<string, { icon: string; color: string }> = {
  cursor: { icon: 'terminal', color: '#00B4D8' },
  claude: { icon: 'robot', color: '#D97706' },
  codex: { icon: 'layers', color: '#10B981' },
  windsurf: { icon: 'activity', color: '#8B5CF6' },
  trae: { icon: 'server', color: '#EC4899' },
  trae_cn: { icon: 'server', color: '#F43F5E' },
}

const allConversations = computed(() => {
  const all: ExtractedConversation[] = []
  for (const convs of extractedData.value.values()) all.push(...convs)
  return all
})
const totalConversations = computed(() => allConversations.value.length)
const totalMessages = computed(() => allConversations.value.reduce((s, c) => s + c.messages.length, 0))
const hasExtractedData = computed(() => totalConversations.value > 0)
const availableSources = computed(() => scanResult.value?.sources.filter(s => s.available) ?? [])
const extractedSourceList = computed(() => [...extractedData.value.entries()])

// ============================================================================
// 备份方法
// ============================================================================

// 获取 provider 应用到的工具列表
function getProviderTools(modelType?: string): string[] {
  switch (modelType || 'claude') {
    case 'claude': return ['Claude Code', 'OpenCode']
    case 'codex': return ['Codex CLI', 'OpenCode']
    case 'gemini': return ['Gemini CLI', 'OpenCode']
    default: return ['OpenCode']
  }
}

// 通用列表项切换
function toggleItem(list: string[], key: string) {
  const idx = list.indexOf(key)
  if (idx >= 0) list.splice(idx, 1)
  else list.push(key)
}

// 服务商分组 computed
const claudeProviders = computed(() => fullBackupData.value?.providers.filter(p => (p.model_type || 'claude') === 'claude') || [])
const codexOcProviders = computed(() => fullBackupData.value?.providers.filter(p => p.model_type === 'codex') || [])
const geminiOcProviders = computed(() => fullBackupData.value?.providers.filter(p => p.model_type === 'gemini') || [])
const codexCliProviders = computed(() => fullBackupData.value?.codex_config?.model_providers || [])
const hasGeminiEnv = computed(() => {
  const env = fullBackupData.value?.gemini_config?.env
  return !!(env?.gemini_api_key || env?.google_gemini_api_key)
})

// 统一 MCP 列表
const codexCliMcps = computed(() => fullBackupData.value?.codex_config?.mcp_servers || [])
const geminiCliMcps = computed(() => fullBackupData.value?.gemini_config?.mcp_servers || [])
const totalMcpCount = computed(() => (fullBackupData.value?.mcp_servers.length || 0) + codexCliMcps.value.length + geminiCliMcps.value.length)
const selectedMcpTotalCount = computed(() => selectedMcps.value.length + selectedCodexMcps.value.length + selectedGeminiMcps.value.length)

// Claude 分组全选
const allClaudeSelected = computed(() => claudeProviders.value.length > 0 && claudeProviders.value.every(p => selectedProviders.value.includes(p.name)))
function toggleAllClaude() {
  if (allClaudeSelected.value) { for (const p of claudeProviders.value) { const i = selectedProviders.value.indexOf(p.name); if (i >= 0) selectedProviders.value.splice(i, 1) } }
  else { for (const p of claudeProviders.value) { if (!selectedProviders.value.includes(p.name)) selectedProviders.value.push(p.name) } }
}
// Codex 分组全选
const allCodexSelected = computed(() => {
  const ocOk = codexOcProviders.value.length === 0 || codexOcProviders.value.every(p => selectedProviders.value.includes(p.name))
  const cliOk = codexCliProviders.value.length === 0 || codexCliProviders.value.every(p => selectedCodexProviders.value.includes(p.name))
  return ocOk && cliOk && (codexOcProviders.value.length + codexCliProviders.value.length > 0)
})
function toggleAllCodex() {
  if (allCodexSelected.value) {
    for (const p of codexOcProviders.value) { const i = selectedProviders.value.indexOf(p.name); if (i >= 0) selectedProviders.value.splice(i, 1) }
    selectedCodexProviders.value = []
  } else {
    for (const p of codexOcProviders.value) { if (!selectedProviders.value.includes(p.name)) selectedProviders.value.push(p.name) }
    selectedCodexProviders.value = codexCliProviders.value.map(p => p.name)
  }
}
// Gemini 分组全选
const allGeminiSelected = computed(() => {
  const ocOk = geminiOcProviders.value.length === 0 || geminiOcProviders.value.every(p => selectedProviders.value.includes(p.name))
  const envOk = !hasGeminiEnv.value || includeGeminiEnv.value
  return ocOk && envOk && (geminiOcProviders.value.length + (hasGeminiEnv.value ? 1 : 0) > 0)
})
function toggleAllGemini() {
  if (allGeminiSelected.value) {
    for (const p of geminiOcProviders.value) { const i = selectedProviders.value.indexOf(p.name); if (i >= 0) selectedProviders.value.splice(i, 1) }
    includeGeminiEnv.value = false
  } else {
    for (const p of geminiOcProviders.value) { if (!selectedProviders.value.includes(p.name)) selectedProviders.value.push(p.name) }
    includeGeminiEnv.value = hasGeminiEnv.value
  }
}
// 服务商总计
const allProvidersGroupSelected = computed(() => allClaudeSelected.value && allCodexSelected.value && allGeminiSelected.value)
const selectedProvidersGroupCount = computed(() => {
  let c = selectedProviders.value.length + selectedCodexProviders.value.length
  if (includeGeminiEnv.value && hasGeminiEnv.value) c++
  return c
})
const totalProvidersGroupCount = computed(() => {
  return (fullBackupData.value?.providers.length || 0) + codexCliProviders.value.length + (hasGeminiEnv.value ? 1 : 0)
})
function toggleAllProvidersGroup() {
  if (allProvidersGroupSelected.value) {
    // 取消全选
    selectedProviders.value = []; selectedCodexProviders.value = []; includeGeminiEnv.value = false
  } else {
    // 强制全选所有服务商（不使用 toggle 函数，避免已选中的被反选）
    selectedProviders.value = fullBackupData.value?.providers.map(p => p.name) || []
    selectedCodexProviders.value = codexCliProviders.value.map(p => p.name)
    includeGeminiEnv.value = hasGeminiEnv.value
  }
}

// MCP 统一全选
const allMcpsSelected = computed(() => {
  const ocOk = (fullBackupData.value?.mcp_servers.length || 0) === 0 || fullBackupData.value!.mcp_servers.every(m => selectedMcps.value.includes(m.name))
  const cxOk = codexCliMcps.value.length === 0 || codexCliMcps.value.every(m => selectedCodexMcps.value.includes(m.name))
  const gmOk = geminiCliMcps.value.length === 0 || geminiCliMcps.value.every(m => selectedGeminiMcps.value.includes(m.name))
  return totalMcpCount.value > 0 && ocOk && cxOk && gmOk
})
function toggleAllMcps() {
  if (allMcpsSelected.value) { selectedMcps.value = []; selectedCodexMcps.value = []; selectedGeminiMcps.value = [] }
  else {
    selectedMcps.value = fullBackupData.value?.mcp_servers.map(m => m.name) || []
    selectedCodexMcps.value = codexCliMcps.value.map(m => m.name)
    selectedGeminiMcps.value = geminiCliMcps.value.map(m => m.name)
  }
}

// 规则全选
const allRulesSelected = computed(() => fullBackupData.value ? fullBackupData.value.rules.length > 0 && fullBackupData.value.rules.every(r => selectedRules.value.includes(`${r.name}|${r.location}`)) : false)
function toggleAllRules() { selectedRules.value = allRulesSelected.value ? [] : (fullBackupData.value?.rules.map(r => `${r.name}|${r.location}`) || []) }

// Skills 全选
const allSkillsSelected = computed(() => fullBackupData.value ? fullBackupData.value.skills.length > 0 && fullBackupData.value.skills.every(s => selectedSkills.value.includes(`${s.name}|${s.location}`)) : false)
function toggleAllSkills() { selectedSkills.value = allSkillsSelected.value ? [] : (fullBackupData.value?.skills.map(s => `${s.name}|${s.location}`) || []) }

// 使用统计全选
const allUsageSourcesSelected = computed(() => selectedUsageSources.value.length === usageSourceOptions.length)
function toggleAllUsageSources() { selectedUsageSources.value = allUsageSourcesSelected.value ? [] : usageSourceOptions.map(s => s.key) }

// 开发环境全选/取消
function toggleAllDevEnvs() {
  if (Object.keys(selectedDevEnvs.value).length === detectedEnvs.value.length) {
    selectedDevEnvs.value = {}
  } else {
    selectedDevEnvs.value = {}
    detectedEnvs.value.forEach(e => {
      if (e.current_version) selectedDevEnvs.value[e.id] = e.current_version
    })
  }
}

// 对话源全选
const allChatSourcesSelected = computed(() => extractedData.value.size > 0 && [...extractedData.value.keys()].every(k => selectedChatSources.value.includes(k)))
function toggleAllChatSources() { selectedChatSources.value = allChatSourcesSelected.value ? [] : [...extractedData.value.keys()] }

// 总选中数
const totalSelectedCount = computed(() =>
  selectedProviders.value.length + selectedCodexProviders.value.length + (includeGeminiEnv.value && hasGeminiEnv.value ? 1 : 0) +
  selectedMcps.value.length + selectedCodexMcps.value.length + selectedGeminiMcps.value.length +
  selectedRules.value.length + selectedSkills.value.length +
  selectedUsageSources.value.length + selectedChatSources.value.length
)

// 打开导出选择面板
async function openExportPanel() {
  isLoadingExportData.value = true; showExportPanel.value = true; exportMessage.value = ''
  try {
    fullBackupData.value = await invoke<BackupData>('create_backup')
    // 检测本机开发环境（仅备份版本号）
    const envs = await invoke<DevEnvInfo[]>('detect_all_dev_envs')
    detectedEnvs.value = envs.filter(e => e.installed)
    selectedDevEnvs.value = {}
    for (const e of detectedEnvs.value) { if (e.current_version) selectedDevEnvs.value[e.id] = e.current_version }
    selectedProviders.value = fullBackupData.value.providers.map(p => p.name)
    selectedMcps.value = fullBackupData.value.mcp_servers.map(m => m.name)
    selectedRules.value = fullBackupData.value.rules.map(r => `${r.name}|${r.location}`)
    selectedSkills.value = fullBackupData.value.skills.map(s => `${s.name}|${s.location}`)
    selectedCodexProviders.value = fullBackupData.value.codex_config?.model_providers?.map(p => p.name) || []
    selectedCodexMcps.value = fullBackupData.value.codex_config?.mcp_servers?.map(m => m.name) || []
    includeGeminiEnv.value = !!(fullBackupData.value.gemini_config?.env?.gemini_api_key || fullBackupData.value.gemini_config?.env?.google_gemini_api_key)
    selectedGeminiMcps.value = fullBackupData.value.gemini_config?.mcp_servers?.map(m => m.name) || []
    selectedUsageSources.value = []
    selectedChatSources.value = [...extractedData.value.keys()]
  } catch (e) { exportMessage.value = t('backup.exportFailed') + ': ' + String(e); exportMessageType.value = 'error'; showExportPanel.value = false }
  finally { isLoadingExportData.value = false }
}

function cancelExport() { showExportPanel.value = false; fullBackupData.value = null; exportMessage.value = '' }

// 确认导出
async function confirmFilteredExport() {
  try {
    isExporting.value = true; exportMessage.value = ''
    const filePath = await save({ defaultPath: `aiswitch-backup-${new Date().toISOString().split('T')[0]}.json`, filters: [{ name: 'JSON', extensions: ['json'] }] })
    if (!filePath) { isExporting.value = false; return }
    const chatConvs: ExtractedConversation[] = []
    for (const [key, convs] of extractedData.value) { if (selectedChatSources.value.includes(key)) chatConvs.push(...convs) }
    const stats = await invoke<ExportStats>('export_backup_filtered', {
      filePath,
      options: {
        provider_names: selectedProviders.value, mcp_names: selectedMcps.value,
        rule_ids: selectedRules.value, skill_ids: selectedSkills.value,
        codex_provider_names: selectedCodexProviders.value,
        codex_mcp_names: selectedCodexMcps.value,
        include_gemini_env: includeGeminiEnv.value,
        gemini_mcp_names: selectedGeminiMcps.value,
        usage_sources: selectedUsageSources.value,
        dev_envs: Object.entries(selectedDevEnvs.value).map(([id, version]) => ({ id, version, name: detectedEnvs.value.find(e => e.id === id)?.name || id })) as ExportedDevEnv[],
      },
      chatConversations: chatConvs.length > 0 ? chatConvs : null,
    })
    let msg = t('backup.exportSuccess', { providers: stats.providers, models: stats.models, mcp: stats.mcp_servers, rules: stats.rules, skills: stats.skills })
    if (stats.usage_records > 0) msg += `, ${stats.usage_records} ${t('backup.usageRecords')}`
    if (stats.chat_conversations > 0) msg += `, ${stats.chat_conversations} ${t('backup.chatRecords')}`
    exportMessage.value = msg; exportMessageType.value = 'success'; showExportPanel.value = false; fullBackupData.value = null
  } catch (e) { exportMessage.value = t('backup.exportFailed') + ': ' + String(e); exportMessageType.value = 'error' }
  finally { isExporting.value = false }
}

async function handleSelectFile() {
  try {
    const filePath = await open({ filters: [{ name: 'JSON', extensions: ['json'] }], multiple: false })
    if (!filePath || Array.isArray(filePath)) return
    selectedFilePath.value = filePath
    const data = await invoke<BackupData>('preview_backup', { filePath })
    previewData.value = data; showBackupPreview.value = true; importMessage.value = ''
  } catch (e) { importMessage.value = t('backup.previewFailed') + ': ' + String(e); importMessageType.value = 'error' }
}

async function handleBackupImport() {
  if (!selectedFilePath.value) return
  try {
    isImportingBackup.value = true; importMessage.value = ''
    const result = await invoke<ImportResult>('import_backup', { filePath: selectedFilePath.value, options: importOptions.value })
    let msg = result.success
      ? t('backup.importSuccess', { providers: result.providers_imported, mcp: result.mcp_imported, rules: result.rules_imported, skills: result.skills_imported, codex: result.codex_imported, gemini: result.gemini_imported })
      : t('backup.importPartial', { providers: result.providers_imported, mcp: result.mcp_imported, rules: result.rules_imported, skills: result.skills_imported, codex: result.codex_imported, gemini: result.gemini_imported, errors: result.errors.length })
    if (result.usage_imported > 0 || result.usage_skipped > 0) { msg += ` | ${t('backup.includeUsageStats')}: +${result.usage_imported}, -${result.usage_skipped}` }
    importMessage.value = msg; importMessageType.value = 'success'
    showBackupPreview.value = false; previewData.value = null; selectedFilePath.value = ''
  } catch (e) { importMessage.value = t('backup.importFailed') + ': ' + String(e); importMessageType.value = 'error' }
  finally { isImportingBackup.value = false }
}

function handleCancelImport() {
  showBackupPreview.value = false; previewData.value = null; selectedFilePath.value = ''; importMessage.value = ''
}

function formatDate(dateStr: string): string { try { return new Date(dateStr).toLocaleString() } catch { return dateStr } }
function maskApiKey(key: string): string { return key.length <= 8 ? '***' : key.slice(0, 4) + '****' + key.slice(-4) }

// ============================================================================
// 对话迁移方法
// ============================================================================

async function handleScan() {
  try {
    isScanning.value = true; chatStatusMessage.value = ''; sectionExpanded.value.chatRecords = true
    const unlisten = await listen('chat-migration-progress', () => {})
    scanResult.value = await invoke<ChatScanResult>('scan_chat_sources')
    unlisten()
    setChatStatus('success', t('chatMigration.scanComplete'))
  } catch (e) { setChatStatus('error', String(e)) }
  finally { isScanning.value = false }
}

async function handleExtract(sourceKey: string) {
  try {
    extractingSource.value = sourceKey; chatStatusMessage.value = ''
    const result = await invoke<ExtractionResult>('extract_conversations', { source: sourceKey })
    extractedData.value.set(sourceKey, result.conversations)
    extractedData.value = new Map(extractedData.value)
    setChatStatus('success', `${t('chatMigration.extractComplete')} - ${result.total} ${t('chatMigration.conversations')}`)
  } catch (e) { setChatStatus('error', String(e)) }
  finally { extractingSource.value = null }
}

async function handleExtractAll() {
  if (!scanResult.value) return
  for (const source of scanResult.value.sources) {
    if (source.available && source.conversationCount > 0) await handleExtract(source.key)
  }
}

async function handleChatExport() {
  if (!hasExtractedData.value) { setChatStatus('error', t('chatMigration.noConversations')); return }
  try {
    isChatExporting.value = true; chatStatusMessage.value = ''
    const ts = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
    const filePath = await save({ defaultPath: `chat-migration-${ts}.jsonl`, filters: [{ name: 'JSONL', extensions: ['jsonl'] }] })
    if (!filePath) { isChatExporting.value = false; return }
    const result = await invoke<ChatExportResult>('export_conversations', { conversations: allConversations.value, filePath })
    setChatStatus('success', t('chatMigration.exportSuccess', { count: result.exported, path: result.filePath }))
  } catch (e) { setChatStatus('error', t('chatMigration.exportFailed') + ': ' + String(e)) }
  finally { isChatExporting.value = false }
}

async function handleChatImport() {
  try {
    isChatImporting.value = true; chatStatusMessage.value = ''
    const filePath = await open({ filters: [{ name: 'JSONL', extensions: ['jsonl'] }] })
    if (!filePath) { isChatImporting.value = false; return }
    const result = await invoke<MigrationImportResult>('import_migration_file', { filePath })
    setChatStatus('success', t('chatMigration.importSuccess', { imported: result.imported, skipped: result.skipped }))
  } catch (e) { setChatStatus('error', t('chatMigration.importFailed') + ': ' + String(e)) }
  finally { isChatImporting.value = false }
}

function closeChatPreview() { showChatPreview.value = false; previewConversation.value = null }
function handleClearExtracted() { extractedData.value = new Map(); chatStatusMessage.value = '' }
function setChatStatus(type: 'success' | 'error' | 'info', msg: string) { chatStatusType.value = type; chatStatusMessage.value = msg }
function getSourceConvCount(key: string): number { return extractedData.value.get(key)?.length || 0 }

function formatTimestamp(ts?: string | number | null): string {
  if (!ts) return '-'
  let date: Date
  if (typeof ts === 'number') { date = ts > 1e12 ? new Date(ts) : new Date(ts * 1000) }
  else { const n = Number(ts); date = !isNaN(n) ? (n > 1e12 ? new Date(n) : new Date(n * 1000)) : new Date(ts) }
  return isNaN(date.getTime()) ? String(ts) : date.toLocaleString()
}

</script>

<template>
  <div class="h-full overflow-auto pb-6">
    <div class="max-w-3xl mx-auto space-y-6">

    <!-- ================================================================ -->
    <!-- 导出备份 -->
    <!-- ================================================================ -->
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-4">
        <SvgIcon name="save" :size="28" class="text-accent" />
        <h2 class="text-lg font-semibold">{{ t('backup.exportTitle') }}</h2>
      </div>
      <p class="text-sm text-muted-foreground mb-4">{{ t('backup.exportDesc') }}</p>

      <!-- 未展开：导出按钮 -->
      <div v-if="!showExportPanel" class="flex items-center gap-4">
        <button @click="openExportPanel" :disabled="isExporting"
          class="px-4 py-2 bg-accent text-accent-foreground rounded-lg font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2">
          <SvgIcon name="download" :size="18" />
          {{ t('backup.exportBtn') }}
        </button>
        <span v-if="exportMessage" class="text-sm" :class="exportMessageType === 'error' ? 'text-red-500' : 'text-green-500'">{{ exportMessage }}</span>
      </div>

      <!-- 展开：详细选择面板 -->
      <div v-if="showExportPanel" class="space-y-3">
        <!-- 加载中 -->
        <div v-if="isLoadingExportData" class="flex items-center justify-center py-8 gap-2 text-muted-foreground">
          <SvgIcon name="loading" :size="20" class="animate-spin" />
          <span class="text-sm">{{ t('backup.loadingData') }}</span>
        </div>

        <template v-else-if="fullBackupData">
          <!-- ========== 服务商（按工具分组） ========== -->
          <div class="bg-surface rounded-lg overflow-hidden">
            <!-- 服务商总标题 -->
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('providers')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox" :checked="allProvidersGroupSelected" @change="toggleAllProvidersGroup" class="w-3.5 h-3.5 rounded border-border accent-accent" />
                <span class="text-sm font-medium">{{ t('backup.providers') }}</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{{ selectedProvidersGroupCount }}/{{ totalProvidersGroupCount }}</span>
                <SvgIcon :name="sectionExpanded.providers ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.providers" class="border-t border-border/30">
              <!-- Claude Code 子分组 -->
              <div v-if="claudeProviders.length > 0">
                <div class="flex items-center justify-between px-4 py-2 cursor-pointer select-none bg-amber-500/5" @click="toggleSection('claude')">
                  <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                    <input type="checkbox" :checked="allClaudeSelected" @change="toggleAllClaude" class="w-3 h-3 rounded border-border accent-amber-500" />
                    <span class="text-xs font-medium text-amber-500">Claude Code</span>
                  </label>
                  <div class="flex items-center gap-1">
                    <span class="text-[10px] text-muted-foreground">{{ claudeProviders.filter(p => selectedProviders.includes(p.name)).length }}/{{ claudeProviders.length }}</span>
                    <SvgIcon :name="sectionExpanded.claude ? 'arrow-up' : 'arrow-down'" :size="12" class="text-muted-foreground" />
                  </div>
                </div>
                <div v-if="sectionExpanded.claude" class="px-4 pb-2 space-y-1">
                  <div v-for="provider in claudeProviders" :key="provider.name" class="flex items-center gap-2 px-2.5 py-2 rounded-md hover:bg-background/50 transition-colors">
                    <input type="checkbox" :checked="selectedProviders.includes(provider.name)" @change="toggleItem(selectedProviders, provider.name)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-medium truncate">{{ provider.name }}</span>
                        <span class="text-[10px] px-1.5 py-0.5 bg-accent/15 text-accent rounded-full">{{ provider.models.length }} {{ t('backup.models') }}</span>
                      </div>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-[10px] font-mono text-muted-foreground/70">{{ maskApiKey(provider.api_key) }}</span>
                        <span class="text-[10px] text-muted-foreground/50 truncate">{{ provider.base_url }}</span>
                      </div>
                    </div>
                    <div class="flex items-center gap-1 shrink-0">
                      <span v-for="tool in getProviderTools(provider.model_type)" :key="tool" class="text-[9px] px-1.5 py-0.5 rounded-full border" :class="tool.includes('Claude') ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' : 'bg-blue-500/10 text-blue-400 border-blue-500/20'">{{ tool }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <!-- Codex 子分组 -->
              <div v-if="codexOcProviders.length > 0 || codexCliProviders.length > 0">
                <div class="flex items-center justify-between px-4 py-2 cursor-pointer select-none bg-green-500/5 border-t border-border/20" @click="toggleSection('codex')">
                  <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                    <input type="checkbox" :checked="allCodexSelected" @change="toggleAllCodex" class="w-3 h-3 rounded border-border accent-green-500" />
                    <span class="text-xs font-medium text-green-500">Codex</span>
                  </label>
                  <div class="flex items-center gap-1">
                    <span class="text-[10px] text-muted-foreground">{{ codexOcProviders.filter(p => selectedProviders.includes(p.name)).length + selectedCodexProviders.length }}/{{ codexOcProviders.length + codexCliProviders.length }}</span>
                    <SvgIcon :name="sectionExpanded.codex ? 'arrow-up' : 'arrow-down'" :size="12" class="text-muted-foreground" />
                  </div>
                </div>
                <div v-if="sectionExpanded.codex" class="px-4 pb-2 space-y-1">
                  <!-- OpenCode Codex providers -->
                  <div v-for="provider in codexOcProviders" :key="provider.name" class="flex items-center gap-2 px-2.5 py-2 rounded-md hover:bg-background/50 transition-colors">
                    <input type="checkbox" :checked="selectedProviders.includes(provider.name)" @change="toggleItem(selectedProviders, provider.name)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-medium truncate">{{ provider.name }}</span>
                        <span class="text-[10px] px-1.5 py-0.5 bg-accent/15 text-accent rounded-full">{{ provider.models.length }} {{ t('backup.models') }}</span>
                        <span class="text-[9px] px-1.5 py-0.5 rounded-full border bg-blue-500/10 text-blue-400 border-blue-500/20">OpenCode</span>
                      </div>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-[10px] font-mono text-muted-foreground/70">{{ maskApiKey(provider.api_key) }}</span>
                        <span class="text-[10px] text-muted-foreground/50 truncate">{{ provider.base_url }}</span>
                      </div>
                    </div>
                  </div>
                  <!-- Codex CLI providers -->
                  <div v-for="cp in codexCliProviders" :key="'codex-cli-' + cp.name" class="flex items-center gap-2 px-2.5 py-2 rounded-md hover:bg-background/50 transition-colors">
                    <input type="checkbox" :checked="selectedCodexProviders.includes(cp.name)" @change="toggleItem(selectedCodexProviders, cp.name)" class="w-3.5 h-3.5 rounded border-border accent-green-500 shrink-0" />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-medium truncate">{{ cp.name }}</span>
                        <span class="text-[9px] px-1.5 py-0.5 rounded-full border bg-green-500/10 text-green-400 border-green-500/20">Codex CLI</span>
                      </div>
                      <div class="text-[10px] text-muted-foreground/50 truncate mt-0.5">{{ cp.base_url }}</div>
                    </div>
                  </div>
                </div>
              </div>
              <!-- Gemini 子分组 -->
              <div v-if="geminiOcProviders.length > 0 || hasGeminiEnv">
                <div class="flex items-center justify-between px-4 py-2 cursor-pointer select-none bg-pink-500/5 border-t border-border/20" @click="toggleSection('gemini')">
                  <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                    <input type="checkbox" :checked="allGeminiSelected" @change="toggleAllGemini" class="w-3 h-3 rounded border-border accent-pink-500" />
                    <span class="text-xs font-medium text-pink-500">Gemini</span>
                  </label>
                  <div class="flex items-center gap-1">
                    <span class="text-[10px] text-muted-foreground">{{ geminiOcProviders.filter(p => selectedProviders.includes(p.name)).length + (includeGeminiEnv ? 1 : 0) }}/{{ geminiOcProviders.length + (hasGeminiEnv ? 1 : 0) }}</span>
                    <SvgIcon :name="sectionExpanded.gemini ? 'arrow-up' : 'arrow-down'" :size="12" class="text-muted-foreground" />
                  </div>
                </div>
                <div v-if="sectionExpanded.gemini" class="px-4 pb-2 space-y-1">
                  <!-- OpenCode Gemini providers -->
                  <div v-for="provider in geminiOcProviders" :key="provider.name" class="flex items-center gap-2 px-2.5 py-2 rounded-md hover:bg-background/50 transition-colors">
                    <input type="checkbox" :checked="selectedProviders.includes(provider.name)" @change="toggleItem(selectedProviders, provider.name)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-medium truncate">{{ provider.name }}</span>
                        <span class="text-[10px] px-1.5 py-0.5 bg-accent/15 text-accent rounded-full">{{ provider.models.length }} {{ t('backup.models') }}</span>
                        <span class="text-[9px] px-1.5 py-0.5 rounded-full border bg-blue-500/10 text-blue-400 border-blue-500/20">OpenCode</span>
                      </div>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-[10px] font-mono text-muted-foreground/70">{{ maskApiKey(provider.api_key) }}</span>
                        <span class="text-[10px] text-muted-foreground/50 truncate">{{ provider.base_url }}</span>
                      </div>
                    </div>
                  </div>
                  <!-- Gemini CLI env 配置 -->
                  <label v-if="hasGeminiEnv" class="flex items-center gap-2 px-2.5 py-2 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                    <input type="checkbox" v-model="includeGeminiEnv" class="w-3.5 h-3.5 rounded border-border accent-pink-500 shrink-0" />
                    <span class="text-sm text-pink-500">Gemini CLI</span>
                    <span class="text-[10px] text-muted-foreground">{{ t('backup.geminiEnvConfig') }}</span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          <!-- ========== MCP 服务器（统一三来源） ========== -->
          <div v-if="totalMcpCount > 0" class="bg-surface rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('mcp')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox" :checked="allMcpsSelected" @change="toggleAllMcps" class="w-3.5 h-3.5 rounded border-border accent-accent" />
                <span class="text-sm font-medium">MCP</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{{ selectedMcpTotalCount }}/{{ totalMcpCount }}</span>
                <SvgIcon :name="sectionExpanded.mcp ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.mcp" class="px-4 pb-2 space-y-1 max-h-[240px] overflow-y-auto border-t border-border/30">
              <!-- OpenCode MCP -->
              <label v-for="mcp in fullBackupData.mcp_servers" :key="mcp.name" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="selectedMcps.includes(mcp.name)" @change="toggleItem(selectedMcps, mcp.name)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                <span class="text-sm truncate">{{ mcp.name }}</span>
                <span class="text-[10px] text-muted-foreground shrink-0">{{ mcp.server_type }}</span>
                <span class="text-[9px] px-1.5 py-0.5 rounded-full border bg-blue-500/10 text-blue-400 border-blue-500/20 shrink-0">OpenCode</span>
              </label>
              <!-- Codex CLI MCP -->
              <label v-for="mcp in codexCliMcps" :key="'cx-' + mcp.name" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="selectedCodexMcps.includes(mcp.name)" @change="toggleItem(selectedCodexMcps, mcp.name)" class="w-3.5 h-3.5 rounded border-border accent-green-500 shrink-0" />
                <span class="text-sm truncate">{{ mcp.name }}</span>
                <span class="text-[9px] px-1.5 py-0.5 rounded-full border bg-green-500/10 text-green-400 border-green-500/20 shrink-0">Codex</span>
              </label>
              <!-- Gemini CLI MCP -->
              <label v-for="mcp in geminiCliMcps" :key="'gm-' + mcp.name" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="selectedGeminiMcps.includes(mcp.name)" @change="toggleItem(selectedGeminiMcps, mcp.name)" class="w-3.5 h-3.5 rounded border-border accent-pink-500 shrink-0" />
                <span class="text-sm truncate">{{ mcp.name }}</span>
                <span class="text-[9px] px-1.5 py-0.5 rounded-full border bg-pink-500/10 text-pink-400 border-pink-500/20 shrink-0">Gemini</span>
              </label>
            </div>
          </div>

          <!-- ========== 规则 ========== -->
          <div v-if="fullBackupData.rules.length > 0" class="bg-surface rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('rules')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox" :checked="allRulesSelected" @change="toggleAllRules" class="w-3.5 h-3.5 rounded border-border accent-accent" />
                <span class="text-sm font-medium">{{ t('backup.rules') }}</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{{ selectedRules.length }}/{{ fullBackupData.rules.length }}</span>
                <SvgIcon :name="sectionExpanded.rules ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.rules" class="px-4 pb-2 space-y-1 max-h-[180px] overflow-y-auto border-t border-border/30">
              <label v-for="rule in fullBackupData.rules" :key="rule.name + '|' + rule.location" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="selectedRules.includes(rule.name + '|' + rule.location)" @change="toggleItem(selectedRules, rule.name + '|' + rule.location)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                <span class="text-sm truncate">{{ rule.name }}</span>
                <span class="text-[10px] text-muted-foreground shrink-0">{{ rule.location }}</span>
              </label>
            </div>
          </div>

          <!-- ========== Skills ========== -->
          <div v-if="fullBackupData.skills.length > 0" class="bg-surface rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('skills')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox" :checked="allSkillsSelected" @change="toggleAllSkills" class="w-3.5 h-3.5 rounded border-border accent-accent" />
                <span class="text-sm font-medium">Skills</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{{ selectedSkills.length }}/{{ fullBackupData.skills.length }}</span>
                <SvgIcon :name="sectionExpanded.skills ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.skills" class="px-4 pb-2 space-y-1 max-h-[180px] overflow-y-auto border-t border-border/30">
              <label v-for="skill in fullBackupData.skills" :key="skill.name + '|' + skill.location" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="selectedSkills.includes(skill.name + '|' + skill.location)" @change="toggleItem(selectedSkills, skill.name + '|' + skill.location)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                <span class="text-sm truncate">{{ skill.name }}</span>
                <span class="text-[10px] text-muted-foreground shrink-0">{{ skill.location }}</span>
              </label>
            </div>
          </div>

          <!-- ========== 开发环境（仅版本号） ========== -->
          <div v-if="detectedEnvs.length > 0" class="bg-surface rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('devenvs')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox"
                  :checked="Object.keys(selectedDevEnvs).length === detectedEnvs.length"
                  @change="toggleAllDevEnvs"
                  class="w-3.5 h-3.5 rounded border-border accent-accent" />
                <span class="text-sm font-medium">开发环境（版本号）</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{{ Object.keys(selectedDevEnvs).length }}/{{ detectedEnvs.length }}</span>
                <SvgIcon :name="sectionExpanded.devenvs ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.devenvs" class="px-4 pb-2 space-y-1 max-h-[200px] overflow-y-auto border-top border-border/30">
              <label v-for="env in detectedEnvs" :key="env.id" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="!!selectedDevEnvs[env.id]" @change="() => { if (selectedDevEnvs[env.id]) delete selectedDevEnvs[env.id]; else if (env.current_version) selectedDevEnvs[env.id] = env.current_version }" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                <span class="text-sm truncate">{{ env.name }}</span>
                <span class="text-[10px] text-muted-foreground">v{{ env.current_version }}</span>
              </label>
            </div>
          </div>

          <!-- ========== 使用统计（按来源可选） ========== -->
          <div class="bg-surface rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('usage')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox" :checked="allUsageSourcesSelected" @change="toggleAllUsageSources" class="w-3.5 h-3.5 rounded border-border accent-amber-500" />
                <span class="text-sm font-medium text-amber-500">{{ t('backup.includeUsageStats') }}</span>
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{{ selectedUsageSources.length }}/{{ usageSourceOptions.length }}</span>
                <SvgIcon :name="sectionExpanded.usage ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.usage" class="px-4 pb-2 space-y-1 border-t border-border/30">
              <label v-for="src in usageSourceOptions" :key="src.key" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                <input type="checkbox" :checked="selectedUsageSources.includes(src.key)" @change="toggleItem(selectedUsageSources, src.key)" class="w-3.5 h-3.5 rounded border-border shrink-0" :class="'accent-' + src.color + '-500'" />
                <span class="text-sm">{{ src.label }}</span>
              </label>
            </div>
          </div>

          <!-- ========== 对话记录（集成对话迁移功能） ========== -->
          <div class="bg-surface rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none" @click="toggleSection('chatRecords')">
              <label class="flex items-center gap-2 cursor-pointer" @click.stop>
                <input type="checkbox" :checked="allChatSourcesSelected" @change="toggleAllChatSources" class="w-3.5 h-3.5 rounded border-border accent-accent" />
                <span class="text-sm font-medium">{{ t('backup.chatRecords') }}</span>
              </label>
              <div class="flex items-center gap-2">
                <span v-if="extractedSourceList.length > 0" class="text-xs text-muted-foreground">{{ selectedChatSources.length }}/{{ extractedSourceList.length }}</span>
                <SvgIcon :name="sectionExpanded.chatRecords ? 'arrow-up' : 'arrow-down'" :size="14" class="text-muted-foreground" />
              </div>
            </div>
            <div v-if="sectionExpanded.chatRecords" class="border-t border-border/30">
              <!-- 操作栏：扫描/提取/导入 -->
              <div class="px-4 pt-2 pb-1 flex items-center gap-2 flex-wrap">
                <button @click="handleScan" :disabled="isScanning" class="px-3 py-1.5 text-xs font-medium rounded-lg bg-accent text-accent-foreground hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-1.5">
                  <SvgIcon v-if="isScanning" name="loading" :size="14" class="animate-spin" /><SvgIcon v-else name="search" :size="14" />
                  {{ isScanning ? t('chatMigration.scanning') : t('chatMigration.scan') }}
                </button>
                <button v-if="scanResult && availableSources.length > 0" @click="handleExtractAll" :disabled="!!extractingSource" class="px-3 py-1.5 text-xs font-medium rounded-lg border border-border hover:bg-surface-hover transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5">
                  <SvgIcon v-if="extractingSource" name="loading" :size="14" class="animate-spin" /><SvgIcon v-else name="layers" :size="14" />
                  {{ t('chatMigration.extractAll') }}
                </button>
                <button @click="handleChatImport" :disabled="isChatImporting" class="px-3 py-1.5 text-xs font-medium rounded-lg border border-border hover:bg-surface-hover transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5">
                  <SvgIcon v-if="isChatImporting" name="loading" :size="14" class="animate-spin" /><SvgIcon v-else name="upload" :size="14" />
                  {{ t('chatMigration.importFile') }}
                </button>
              </div>
              <!-- 状态消息 -->
              <div v-if="chatStatusMessage" class="mx-4 mb-2 px-3 py-2 rounded-lg text-sm" :class="{ 'bg-green-500/10 text-green-400 border border-green-500/20': chatStatusType === 'success', 'bg-red-500/10 text-red-400 border border-red-500/20': chatStatusType === 'error', 'bg-blue-500/10 text-blue-400 border border-blue-500/20': chatStatusType === 'info' }">
                {{ chatStatusMessage }}
              </div>
              <!-- 扫描结果：工具卡片 -->
              <div v-if="scanResult" class="px-4 pb-2">
                <div class="grid grid-cols-2 lg:grid-cols-3 gap-2">
                  <div v-for="source in scanResult.sources" :key="source.key" class="rounded-lg border border-border p-2.5 transition-all duration-200" :class="source.available ? 'bg-surface/30 hover:border-accent/40' : 'bg-surface/10 opacity-60'">
                    <div class="flex items-center justify-between mb-1.5">
                      <div class="flex items-center gap-1.5">
                        <div class="flex h-5 w-5 items-center justify-center rounded" :style="{ backgroundColor: (toolMeta[source.key]?.color || '#666') + '20' }">
                          <SvgIcon :name="toolMeta[source.key]?.icon || 'terminal'" :size="12" :style="{ color: toolMeta[source.key]?.color || '#666' }" />
                        </div>
                        <span class="font-semibold text-[11px]">{{ source.name }}</span>
                      </div>
                      <span v-if="source.available" class="px-1 py-0.5 text-[8px] font-medium rounded-full bg-green-500/10 text-green-400 border border-green-500/20">{{ t('chatMigration.detected') }}</span>
                    </div>
                    <div class="flex items-center gap-1 text-[10px] text-muted-foreground mb-1.5">
                      <span>{{ source.conversationCount }}</span>
                      <span v-if="getSourceConvCount(source.key) > 0" class="text-green-400">({{ getSourceConvCount(source.key) }} {{ t('chatMigration.extract') }})</span>
                    </div>
                    <button v-if="source.available && source.conversationCount > 0" @click="handleExtract(source.key)" :disabled="extractingSource === source.key" class="w-full px-2 py-1 text-[10px] font-medium rounded border border-accent/30 text-accent hover:bg-accent/10 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-1">
                      <SvgIcon v-if="extractingSource === source.key" name="loading" :size="10" class="animate-spin" /><SvgIcon v-else name="download" :size="10" />
                      {{ extractingSource === source.key ? t('chatMigration.extracting') : t('chatMigration.extract') }}
                    </button>
                  </div>
                </div>
              </div>
              <div v-else class="px-4 pb-2">
                <p class="text-xs text-muted-foreground/60 text-center py-3">{{ t('chatMigration.scanFirst') }}</p>
              </div>
              <!-- 已提取对话源选择 -->
              <div v-if="extractedSourceList.length > 0" class="px-4 pb-2 space-y-1 border-t border-border/20">
                <div class="flex items-center justify-between py-1.5">
                  <span class="text-[10px] text-muted-foreground">{{ t('backup.chatRecords') }} ({{ totalConversations }} {{ t('chatMigration.conversations') }}, {{ totalMessages }} {{ t('chatMigration.messages') }})</span>
                  <div class="flex items-center gap-1.5">
                    <button @click="handleClearExtracted" class="px-2 py-0.5 text-[10px] font-medium rounded border border-border text-muted-foreground hover:bg-surface-hover transition-all">{{ t('chatMigration.clearAll') }}</button>
                    <button @click="handleChatExport" :disabled="isChatExporting" class="px-2 py-0.5 text-[10px] font-medium rounded bg-accent text-accent-foreground hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-1">
                      <SvgIcon v-if="isChatExporting" name="loading" :size="10" class="animate-spin" /><SvgIcon v-else name="download" :size="10" />
                      {{ t('chatMigration.exportAll') }}
                    </button>
                  </div>
                </div>
                <label v-for="[key, convs] in extractedSourceList" :key="key" class="flex items-center gap-2 px-2.5 py-1.5 rounded-md hover:bg-background/50 cursor-pointer transition-colors">
                  <input type="checkbox" :checked="selectedChatSources.includes(key)" @change="toggleItem(selectedChatSources, key)" class="w-3.5 h-3.5 rounded border-border accent-accent shrink-0" />
                  <div class="flex h-5 w-5 items-center justify-center rounded" :style="{ backgroundColor: (toolMeta[key]?.color || '#666') + '20' }">
                    <SvgIcon :name="toolMeta[key]?.icon || 'terminal'" :size="12" :style="{ color: toolMeta[key]?.color || '#666' }" />
                  </div>
                  <span class="text-sm">{{ key }}</span>
                  <span class="text-[10px] text-muted-foreground">{{ convs.length }} {{ t('chatMigration.conversations') }}</span>
                </label>
              </div>
            </div>
          </div>
        </template>

        <!-- 底部操作 -->
        <div class="flex items-center gap-3 pt-3">
          <button @click="confirmFilteredExport" :disabled="isExporting || totalSelectedCount === 0"
            class="px-4 py-2 bg-accent text-accent-foreground rounded-lg font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2">
            <SvgIcon v-if="isExporting" name="loading" :size="18" class="animate-spin" />
            <SvgIcon v-else name="download" :size="18" />
            {{ isExporting ? t('backup.exporting') : t('backup.confirmExport') }}
          </button>
          <button @click="cancelExport" :disabled="isExporting"
            class="px-4 py-2 border border-border rounded-lg font-medium hover:bg-surface transition-all">{{ t('common.cancel') }}</button>
          <span v-if="exportMessage" class="text-sm ml-2" :class="exportMessageType === 'error' ? 'text-red-500' : 'text-green-500'">{{ exportMessage }}</span>
        </div>
      </div>
    </div>

    <!-- ================================================================ -->
    <!-- 导入配置 -->
    <!-- ================================================================ -->
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-4">
        <SvgIcon name="upload" :size="28" class="text-accent" />
        <h2 class="text-lg font-semibold">{{ t('backup.importTitle') }}</h2>
      </div>
      <p class="text-sm text-muted-foreground mb-4">{{ t('backup.importDesc') }}</p>

      <!-- 选择文件 -->
      <div v-if="!showBackupPreview" class="flex items-center gap-4">
        <button @click="handleSelectFile"
          class="px-4 py-2 bg-surface border border-border rounded-lg font-medium hover:bg-surface/80 transition-all flex items-center gap-2">
          <SvgIcon name="folder" :size="18" />{{ t('backup.selectFile') }}
        </button>
        <span v-if="importMessage && !showBackupPreview" class="text-sm" :class="importMessageType === 'error' ? 'text-red-500' : 'text-green-500'">{{ importMessage }}</span>
      </div>

      <!-- 预览 -->
      <div v-if="showBackupPreview && previewData" class="space-y-4">
        <div class="bg-surface rounded-lg p-4 space-y-2">
          <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">{{ t('backup.backupVersion') }}</span><span class="text-sm font-mono">{{ previewData.version }}</span></div>
          <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">{{ t('backup.backupTime') }}</span><span class="text-sm">{{ formatDate(previewData.created_at) }}</span></div>
        </div>

        <div v-if="previewStats" class="space-y-3">
          <div class="grid grid-cols-5 gap-3">
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-accent">{{ previewStats.providers }}</div><div class="text-xs text-muted-foreground">{{ t('backup.providers') }}</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-blue-500">{{ previewStats.models }}</div><div class="text-xs text-muted-foreground">{{ t('backup.models') }}</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-purple-500">{{ previewStats.mcp_servers }}</div><div class="text-xs text-muted-foreground">MCP</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-orange-500">{{ previewStats.rules }}</div><div class="text-xs text-muted-foreground">{{ t('backup.rules') }}</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-green-500">{{ previewStats.skills }}</div><div class="text-xs text-muted-foreground">Skills</div></div>
          </div>
          <div v-if="previewStats.codex_providers > 0 || previewStats.codex_mcp_servers > 0 || previewStats.gemini_configured || previewStats.gemini_mcp_servers > 0" class="grid grid-cols-4 gap-3">
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-cyan-500">{{ previewStats.codex_providers }}</div><div class="text-xs text-muted-foreground">Codex {{ t('backup.providers') }}</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-cyan-400">{{ previewStats.codex_mcp_servers }}</div><div class="text-xs text-muted-foreground">Codex MCP</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-pink-500">{{ previewStats.gemini_configured ? '1' : '0' }}</div><div class="text-xs text-muted-foreground">Gemini ENV</div></div>
            <div class="bg-surface rounded-lg p-3 text-center"><div class="text-2xl font-bold text-pink-400">{{ previewStats.gemini_mcp_servers }}</div><div class="text-xs text-muted-foreground">Gemini MCP</div></div>
          </div>
        </div>

        <!-- 导入选项 -->
        <div class="bg-surface rounded-lg p-4 space-y-3">
          <h3 class="font-medium text-sm mb-3">{{ t('backup.importOptions') }}</h3>
          <label class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_providers" class="w-4 h-4 rounded border-border accent-accent" /><span class="text-sm">{{ t('backup.importProviders') }}</span><span class="text-xs text-muted-foreground">({{ previewStats?.providers || 0 }} {{ t('backup.items') }})</span></label>
          <label class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_mcp" class="w-4 h-4 rounded border-border accent-accent" /><span class="text-sm">{{ t('backup.importMcp') }}</span><span class="text-xs text-muted-foreground">({{ previewStats?.mcp_servers || 0 }} {{ t('backup.items') }})</span></label>
          <label class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_rules" class="w-4 h-4 rounded border-border accent-accent" /><span class="text-sm">{{ t('backup.importRules') }}</span><span class="text-xs text-muted-foreground">({{ previewStats?.rules || 0 }} {{ t('backup.items') }})</span></label>
          <label class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_skills" class="w-4 h-4 rounded border-border accent-accent" /><span class="text-sm">{{ t('backup.importSkills') }}</span><span class="text-xs text-muted-foreground">({{ previewStats?.skills || 0 }} {{ t('backup.items') }})</span></label>
          <label v-if="previewStats && (previewStats.codex_providers > 0 || previewStats.codex_mcp_servers > 0)" class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_codex" class="w-4 h-4 rounded border-border accent-cyan-500" /><span class="text-sm text-cyan-500">{{ t('backup.importCodex') }}</span><span class="text-xs text-muted-foreground">({{ (previewStats?.codex_providers || 0) + (previewStats?.codex_mcp_servers || 0) }} {{ t('backup.items') }})</span></label>
          <label v-if="previewStats && (previewStats.gemini_configured || previewStats.gemini_mcp_servers > 0)" class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_gemini" class="w-4 h-4 rounded border-border accent-pink-500" /><span class="text-sm text-pink-500">{{ t('backup.importGemini') }}</span><span class="text-xs text-muted-foreground">({{ (previewStats?.gemini_configured ? 1 : 0) + (previewStats?.gemini_mcp_servers || 0) }} {{ t('backup.items') }})</span></label>
          <label v-if="previewStats && previewStats.usage_records > 0" class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.import_usage_stats" class="w-4 h-4 rounded border-border accent-amber-500" /><span class="text-sm text-amber-500">{{ t('backup.importUsageStats') }}</span><span class="text-xs text-muted-foreground">({{ previewStats.usage_records }} {{ t('backup.usageRecords') }})</span></label>
          <div class="border-t border-border pt-3 mt-3">
            <label class="flex items-center gap-3 cursor-pointer"><input type="checkbox" v-model="importOptions.overwrite_existing" class="w-4 h-4 rounded border-border accent-orange-500" /><span class="text-sm text-orange-500">{{ t('backup.overwriteExisting') }}</span></label>
            <p class="text-xs text-muted-foreground mt-1 ml-7">{{ t('backup.overwriteHint') }}</p>
          </div>
        </div>

        <!-- Provider 预览 -->
        <div v-if="previewData.providers.length > 0" class="bg-surface rounded-lg p-4">
          <h3 class="font-medium text-sm mb-3">{{ t('backup.providerPreview') }}</h3>
          <div class="space-y-2 max-h-40 overflow-y-auto">
            <div v-for="provider in previewData.providers" :key="provider.name" class="flex items-center justify-between text-sm p-2 bg-background/50 rounded">
              <div class="flex items-center gap-2"><span class="font-medium">{{ provider.name }}</span><span class="text-xs px-1.5 py-0.5 bg-accent/20 text-accent rounded">{{ provider.models.length }} {{ t('backup.models') }}</span></div>
              <span class="text-xs text-muted-foreground font-mono">{{ maskApiKey(provider.api_key) }}</span>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-3 pt-2">
          <button @click="handleBackupImport" :disabled="isImportingBackup" class="px-4 py-2 bg-accent text-accent-foreground rounded-lg font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2">
            <SvgIcon v-if="isImportingBackup" name="loading" :size="18" class="animate-spin" /><SvgIcon v-else name="upload" :size="18" />
            {{ isImportingBackup ? t('backup.importing') : t('backup.importBtn') }}
          </button>
          <button @click="handleCancelImport" :disabled="isImportingBackup" class="px-4 py-2 border border-border rounded-lg font-medium hover:bg-surface transition-all">{{ t('common.cancel') }}</button>
          <span v-if="importMessage" class="text-sm ml-2" :class="importMessageType === 'error' ? 'text-red-500' : 'text-green-500'">{{ importMessage }}</span>
        </div>
      </div>
    </div>

    <!-- ================================================================ -->
    <!-- 说明信息 -->
    <!-- ================================================================ -->
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-start gap-3">
        <SvgIcon name="info" :size="20" class="text-accent flex-shrink-0 mt-0.5" />
        <div class="space-y-3 text-sm">
          <div>
            <p class="font-medium">{{ t('backup.whatIncluded') }}</p>
            <ul class="text-muted-foreground mt-1 space-y-1">
              <li>• {{ t('backup.includeProviders') }}</li>
              <li>• {{ t('backup.includeMcp') }}</li>
              <li>• {{ t('backup.includeRules') }}</li>
              <li>• {{ t('backup.includeSkills') }}</li>
              <li>• {{ t('backup.includeUsageStatsDesc') }}</li>
            </ul>
          </div>
          <div class="pt-2 border-t border-border">
            <p class="text-orange-500 flex items-center gap-1.5"><SvgIcon name="warning" :size="16" />{{ t('backup.securityWarning') }}</p>
          </div>
        </div>
      </div>
    </div>

    </div>
  </div>

  <!-- ================================================================ -->
  <!-- 对话预览弹窗 -->
  <!-- ================================================================ -->
  <Teleport to="body">
    <div v-if="showChatPreview && previewConversation" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="closeChatPreview">
      <div class="w-[700px] max-h-[80vh] bg-surface border border-border rounded-2xl shadow-xl flex flex-col overflow-hidden">
        <div class="flex items-center justify-between px-5 py-3 border-b border-border">
          <div class="flex items-center gap-2">
            <span class="px-1.5 py-0.5 text-[10px] font-mono rounded border"
              :class="{ 'bg-blue-500/10 text-blue-400 border-blue-500/20': previewConversation.source.includes('cursor'), 'bg-amber-500/10 text-amber-400 border-amber-500/20': previewConversation.source.includes('claude'), 'bg-green-500/10 text-green-400 border-green-500/20': previewConversation.source.includes('codex'), 'bg-purple-500/10 text-purple-400 border-purple-500/20': previewConversation.source.includes('windsurf'), 'bg-pink-500/10 text-pink-400 border-pink-500/20': previewConversation.source.includes('trae') }">
              {{ previewConversation.source }}</span>
            <h3 class="text-sm font-semibold truncate">{{ previewConversation.name || previewConversation.sessionId?.slice(0, 16) || t('chatMigration.preview') }}</h3>
            <span class="text-[10px] text-muted-foreground">{{ previewConversation.messages.length }} {{ t('chatMigration.messages') }}</span>
          </div>
          <button @click="closeChatPreview" class="p-1 rounded-lg hover:bg-surface-hover transition-colors"><SvgIcon name="close" :size="16" class="text-muted-foreground" /></button>
        </div>
        <div class="flex-1 overflow-y-auto px-5 py-4 space-y-3">
          <div v-for="(msg, mIdx) in previewConversation.messages" :key="mIdx" class="flex gap-3">
            <div class="flex h-6 w-6 items-center justify-center rounded-full shrink-0 mt-0.5 text-[10px] font-bold" :class="msg.role === 'user' ? 'bg-blue-500/20 text-blue-400' : 'bg-green-500/20 text-green-400'">{{ msg.role === 'user' ? 'U' : 'A' }}</div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-xs font-medium">{{ msg.role === 'user' ? t('chatMigration.user') : t('chatMigration.assistant') }}</span>
                <span v-if="msg.model" class="text-[10px] text-muted-foreground font-mono">{{ msg.model }}</span>
                <span v-if="msg.timestamp" class="text-[10px] text-muted-foreground/50">{{ formatTimestamp(msg.timestamp) }}</span>
              </div>
              <div class="text-xs text-muted-foreground whitespace-pre-wrap break-words bg-background/50 rounded-lg px-3 py-2 border border-border/30 max-h-[200px] overflow-y-auto">{{ msg.content.length > 2000 ? msg.content.slice(0, 2000) + '...' : msg.content }}</div>
              <div v-if="msg.toolUse" class="mt-1 text-[10px] text-amber-400/70 flex items-center gap-1"><SvgIcon name="terminal" :size="10" />{{ t('chatMigration.toolUse') }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

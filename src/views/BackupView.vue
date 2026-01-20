<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

// 备份数据类型
interface ExportedModel {
  id: string
  name: string
  reasoning_effort?: string
}

interface ExportedProvider {
  name: string
  base_url: string
  api_key: string
  npm?: string
  description?: string
  model_type?: string
  enabled: boolean
  models: ExportedModel[]
}

interface ExportedOAuthConfig {
  client_id?: string
  client_secret?: string
  scope?: string
}

interface ExportedMcpServer {
  name: string
  server_type: string
  enabled: boolean
  timeout?: number
  command?: string[]
  environment?: Record<string, string>
  url?: string
  headers?: Record<string, string>
  oauth?: ExportedOAuthConfig
}

interface ExportedRule {
  name: string
  location: string
  rule_type: string
  content: string
  file_ext?: string
}

interface ExportedSkills {
  name: string
  location: string
  content: string
}

interface BackupData {
  version: string
  created_at: string
  app_name: string
  providers: ExportedProvider[]
  mcp_servers: ExportedMcpServer[]
  rules: ExportedRule[]
  skills: ExportedSkills[]
}

interface ExportStats {
  providers: number
  models: number
  mcp_servers: number
  rules: number
  skills: number
}

interface ImportResult {
  success: boolean
  providers_imported: number
  providers_skipped: number
  mcp_imported: number
  mcp_skipped: number
  rules_imported: number
  rules_skipped: number
  skills_imported: number
  skills_skipped: number
  errors: string[]
}

// 状态
const isExporting = ref(false)
const isImporting = ref(false)
const previewData = ref<BackupData | null>(null)
const showPreview = ref(false)
const selectedFilePath = ref('')
const exportMessage = ref('')
const importMessage = ref('')

// 导入选项
const importOptions = ref({
  import_providers: true,
  import_mcp: true,
  import_rules: true,
  import_skills: true,
  overwrite_existing: false,
})

// 计算统计
const previewStats = computed(() => {
  if (!previewData.value) return null
  return {
    providers: previewData.value.providers.length,
    models: previewData.value.providers.reduce((sum, p) => sum + p.models.length, 0),
    mcp_servers: previewData.value.mcp_servers.length,
    rules: previewData.value.rules.length,
    skills: previewData.value.skills.length,
  }
})

// 导出备份
async function handleExport() {
  try {
    isExporting.value = true
    exportMessage.value = ''
    
    // 选择保存路径
    const filePath = await save({
      defaultPath: `openswitch-backup-${new Date().toISOString().split('T')[0]}.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    
    if (!filePath) {
      isExporting.value = false
      return
    }
    
    const stats = await invoke<ExportStats>('export_backup', { filePath })
    
    exportMessage.value = t('backup.exportSuccess', {
      providers: stats.providers,
      models: stats.models,
      mcp: stats.mcp_servers,
      rules: stats.rules,
      skills: stats.skills,
    })
  } catch (e) {
    exportMessage.value = t('backup.exportFailed') + ': ' + String(e)
  } finally {
    isExporting.value = false
  }
}

// 选择导入文件
async function handleSelectFile() {
  try {
    const filePath = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      multiple: false,
    })
    
    if (!filePath || Array.isArray(filePath)) return
    
    selectedFilePath.value = filePath
    
    // 预览备份内容
    const data = await invoke<BackupData>('preview_backup', { filePath })
    previewData.value = data
    showPreview.value = true
    importMessage.value = ''
  } catch (e) {
    importMessage.value = t('backup.previewFailed') + ': ' + String(e)
  }
}

// 执行导入
async function handleImport() {
  if (!selectedFilePath.value) return
  
  try {
    isImporting.value = true
    importMessage.value = ''
    
    const result = await invoke<ImportResult>('import_backup', {
      filePath: selectedFilePath.value,
      options: importOptions.value,
    })
    
    if (result.success) {
      importMessage.value = t('backup.importSuccess', {
        providers: result.providers_imported,
        mcp: result.mcp_imported,
        rules: result.rules_imported,
        skills: result.skills_imported,
      })
    } else {
      importMessage.value = t('backup.importPartial', {
        providers: result.providers_imported,
        mcp: result.mcp_imported,
        rules: result.rules_imported,
        skills: result.skills_imported,
        errors: result.errors.length,
      })
    }
    
    // 清理状态
    showPreview.value = false
    previewData.value = null
    selectedFilePath.value = ''
  } catch (e) {
    importMessage.value = t('backup.importFailed') + ': ' + String(e)
  } finally {
    isImporting.value = false
  }
}

// 取消导入
function handleCancelImport() {
  showPreview.value = false
  previewData.value = null
  selectedFilePath.value = ''
  importMessage.value = ''
}

// 格式化日期
function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleString()
  } catch {
    return dateStr
  }
}

// 掩码 API Key
function maskApiKey(key: string): string {
  if (key.length <= 8) return '***'
  return key.slice(0, 4) + '****' + key.slice(-4)
}
</script>

<template>
  <div class="h-full overflow-auto pb-6">
    <div class="max-w-3xl mx-auto space-y-6">
    <!-- 导出区域 -->
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-4">
        <SvgIcon name="save" :size="28" class="text-accent" />
        <h2 class="text-lg font-semibold">{{ t('backup.exportTitle') }}</h2>
      </div>
      
      <p class="text-sm text-muted-foreground mb-4">
        {{ t('backup.exportDesc') }}
      </p>
      
      <div class="flex items-center gap-4">
        <button
          @click="handleExport"
          :disabled="isExporting"
          class="px-4 py-2 bg-accent text-accent-foreground rounded-lg font-medium
                 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed
                 transition-all flex items-center gap-2"
        >
          <SvgIcon v-if="isExporting" name="loading" :size="18" class="animate-spin" />
          <SvgIcon v-else name="download" :size="18" />
          {{ isExporting ? t('backup.exporting') : t('backup.exportBtn') }}
        </button>
        
        <span v-if="exportMessage" class="text-sm" 
              :class="exportMessage.includes('失败') || exportMessage.includes('Failed') ? 'text-red-500' : 'text-green-500'">
          {{ exportMessage }}
        </span>
      </div>
    </div>
    
    <!-- 导入区域 -->
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-4">
        <SvgIcon name="upload" :size="28" class="text-accent" />
        <h2 class="text-lg font-semibold">{{ t('backup.importTitle') }}</h2>
      </div>
      
      <p class="text-sm text-muted-foreground mb-4">
        {{ t('backup.importDesc') }}
      </p>
      
      <!-- 选择文件按钮 -->
      <div v-if="!showPreview" class="flex items-center gap-4">
        <button
          @click="handleSelectFile"
          class="px-4 py-2 bg-surface border border-border rounded-lg font-medium
                 hover:bg-surface/80 transition-all flex items-center gap-2"
        >
          <SvgIcon name="folder" :size="18" />
          {{ t('backup.selectFile') }}
        </button>
        
        <span v-if="importMessage && !showPreview" class="text-sm"
              :class="importMessage.includes('失败') || importMessage.includes('Failed') ? 'text-red-500' : 'text-green-500'">
          {{ importMessage }}
        </span>
      </div>
      
      <!-- 预览区域 -->
      <div v-if="showPreview && previewData" class="space-y-4">
        <!-- 备份信息 -->
        <div class="bg-surface rounded-lg p-4 space-y-2">
          <div class="flex justify-between items-center">
            <span class="text-sm text-muted-foreground">{{ t('backup.backupVersion') }}</span>
            <span class="text-sm font-mono">{{ previewData.version }}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-sm text-muted-foreground">{{ t('backup.backupTime') }}</span>
            <span class="text-sm">{{ formatDate(previewData.created_at) }}</span>
          </div>
        </div>
        
        <!-- 统计信息 -->
        <div v-if="previewStats" class="grid grid-cols-5 gap-3">
          <div class="bg-surface rounded-lg p-3 text-center">
            <div class="text-2xl font-bold text-accent">{{ previewStats.providers }}</div>
            <div class="text-xs text-muted-foreground">{{ t('backup.providers') }}</div>
          </div>
          <div class="bg-surface rounded-lg p-3 text-center">
            <div class="text-2xl font-bold text-blue-500">{{ previewStats.models }}</div>
            <div class="text-xs text-muted-foreground">{{ t('backup.models') }}</div>
          </div>
          <div class="bg-surface rounded-lg p-3 text-center">
            <div class="text-2xl font-bold text-purple-500">{{ previewStats.mcp_servers }}</div>
            <div class="text-xs text-muted-foreground">MCP</div>
          </div>
          <div class="bg-surface rounded-lg p-3 text-center">
            <div class="text-2xl font-bold text-orange-500">{{ previewStats.rules }}</div>
            <div class="text-xs text-muted-foreground">{{ t('backup.rules') }}</div>
          </div>
          <div class="bg-surface rounded-lg p-3 text-center">
            <div class="text-2xl font-bold text-green-500">{{ previewStats.skills }}</div>
            <div class="text-xs text-muted-foreground">skills</div>
          </div>
        </div>
        
        <!-- 导入选项 -->
        <div class="bg-surface rounded-lg p-4 space-y-3">
          <h3 class="font-medium text-sm mb-3">{{ t('backup.importOptions') }}</h3>
          
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="importOptions.import_providers" 
                   class="w-4 h-4 rounded border-border accent-accent" />
            <span class="text-sm">{{ t('backup.importProviders') }}</span>
            <span class="text-xs text-muted-foreground">({{ previewStats?.providers || 0 }} {{ t('backup.items') }})</span>
          </label>
          
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="importOptions.import_mcp"
                   class="w-4 h-4 rounded border-border accent-accent" />
            <span class="text-sm">{{ t('backup.importMcp') }}</span>
            <span class="text-xs text-muted-foreground">({{ previewStats?.mcp_servers || 0 }} {{ t('backup.items') }})</span>
          </label>
          
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="importOptions.import_rules"
                   class="w-4 h-4 rounded border-border accent-accent" />
            <span class="text-sm">{{ t('backup.importRules') }}</span>
            <span class="text-xs text-muted-foreground">({{ previewStats?.rules || 0 }} {{ t('backup.items') }})</span>
          </label>
          
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="importOptions.import_skills"
                   class="w-4 h-4 rounded border-border accent-accent" />
            <span class="text-sm">{{ t('backup.importSkills') }}</span>
            <span class="text-xs text-muted-foreground">({{ previewStats?.skills || 0 }} {{ t('backup.items') }})</span>
          </label>
          
          <div class="border-t border-border pt-3 mt-3">
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="importOptions.overwrite_existing"
                     class="w-4 h-4 rounded border-border accent-orange-500" />
              <span class="text-sm text-orange-500">{{ t('backup.overwriteExisting') }}</span>
            </label>
            <p class="text-xs text-muted-foreground mt-1 ml-7">{{ t('backup.overwriteHint') }}</p>
          </div>
        </div>
        
        <!-- Provider 详情预览 -->
        <div v-if="previewData.providers.length > 0" class="bg-surface rounded-lg p-4">
          <h3 class="font-medium text-sm mb-3">{{ t('backup.providerPreview') }}</h3>
          <div class="space-y-2 max-h-40 overflow-y-auto">
            <div v-for="provider in previewData.providers" :key="provider.name"
                 class="flex items-center justify-between text-sm p-2 bg-background/50 rounded">
              <div class="flex items-center gap-2">
                <span class="font-medium">{{ provider.name }}</span>
                <span class="text-xs px-1.5 py-0.5 bg-accent/20 text-accent rounded">
                  {{ provider.models.length }} {{ t('backup.models') }}
                </span>
              </div>
              <span class="text-xs text-muted-foreground font-mono">{{ maskApiKey(provider.api_key) }}</span>
            </div>
          </div>
        </div>
        
        <!-- 操作按钮 -->
        <div class="flex items-center gap-3 pt-2">
          <button
            @click="handleImport"
            :disabled="isImporting"
            class="px-4 py-2 bg-accent text-accent-foreground rounded-lg font-medium
                   hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed
                   transition-all flex items-center gap-2"
          >
            <SvgIcon v-if="isImporting" name="loading" :size="18" class="animate-spin" />
            <SvgIcon v-else name="upload" :size="18" />
            {{ isImporting ? t('backup.importing') : t('backup.importBtn') }}
          </button>
          
          <button
            @click="handleCancelImport"
            :disabled="isImporting"
            class="px-4 py-2 border border-border rounded-lg font-medium
                   hover:bg-surface transition-all"
          >
            {{ t('common.cancel') }}
          </button>
          
          <span v-if="importMessage" class="text-sm ml-2"
                :class="importMessage.includes('失败') || importMessage.includes('Failed') ? 'text-red-500' : 'text-green-500'">
            {{ importMessage }}
          </span>
        </div>
      </div>
    </div>
    
    <!-- 说明信息 -->
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
            </ul>
          </div>
          <div class="pt-2 border-t border-border">
            <p class="text-orange-500 flex items-center gap-1.5">
              <SvgIcon name="warning" :size="16" />
              {{ t('backup.securityWarning') }}
            </p>
          </div>
        </div>
      </div>
    </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

interface AppStatus {
  has_global_config: boolean
  has_project_config: boolean
  active_provider: string | null
  provider_count: number
  mcp_server_count: number
  config_paths: {
    global_config_dir: string
    global_opencode_dir: string
    project_opencode_dir: string | null
  }
}

const status = ref<AppStatus | null>(null)
const version = ref('')
const loading = ref(true)

async function loadStatus() {
  loading.value = true
  try {
    status.value = await invoke<AppStatus>('get_status')
    version.value = await invoke<string>('get_version')
  } catch (e) {
    console.error('加载状态失败:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadStatus()
})
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-6">
        <SvgIcon name="activity" :size="32" class="text-accent" />
        <h2 class="text-xl font-semibold">{{ t('status.title') }}</h2>
      </div>

      <div v-if="loading" class="py-8 text-center text-muted-foreground">
        {{ t('common.loading') }}
      </div>

      <div v-else-if="status" class="space-y-6">
        <!-- 版本信息 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3">
            {{ t('status.appInfo') }}
          </h3>
          <div class="grid grid-cols-2 gap-4">
            <div class="bg-surface rounded-lg p-4">
              <div class="text-2xl font-bold">v{{ version }}</div>
              <div class="text-xs text-muted-foreground">{{ t('status.currentVersion') }}</div>
            </div>
            <div class="bg-surface rounded-lg p-4">
              <div class="text-2xl font-bold">{{ status.provider_count }}</div>
              <div class="text-xs text-muted-foreground">{{ t('status.providerCount') }}</div>
            </div>
          </div>
        </section>

        <!-- 配置状态 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3">
            {{ t('status.configStatus') }}
          </h3>
          <div class="space-y-3">
            <div class="flex items-center justify-between py-2 border-b border-border">
              <span class="text-sm">{{ t('status.globalConfig') }}</span>
              <span class="text-emerald-500">{{ t('status.configured') }}</span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-border">
              <span class="text-sm">{{ t('status.projectConfig') }}</span>
              <span :class="status.has_project_config ? 'text-emerald-500' : 'text-muted-foreground'">
                {{ status.has_project_config ? t('status.configured') : t('status.notConfigured') }}
              </span>
            </div>
            <div class="flex items-center justify-between py-2 border-b border-border">
              <span class="text-sm">{{ t('status.currentProvider') }}</span>
              <span class="font-mono text-sm">{{ status.active_provider || '-' }}</span>
            </div>
            <div class="flex items-center justify-between py-2">
              <span class="text-sm">{{ t('status.mcpServers') }}</span>
              <span>{{ t('status.count', { count: status.mcp_server_count }) }}</span>
            </div>
          </div>
        </section>

        <!-- 配置路径 -->
        <section>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3">
            {{ t('status.configPaths') }}
          </h3>
          <div class="space-y-2 text-sm">
            <div class="flex items-start gap-3">
              <span class="text-muted-foreground w-20 shrink-0">{{ t('status.globalConfig') }}</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.global_config_dir }}</span>
            </div>
            <div class="flex items-start gap-3">
              <span class="text-muted-foreground w-20 shrink-0">{{ t('status.openCode') }}</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.global_opencode_dir }}</span>
            </div>
            <div v-if="status.config_paths.project_opencode_dir" class="flex items-start gap-3">
              <span class="text-muted-foreground w-20 shrink-0">{{ t('status.projectConfig') }}</span>
              <span class="font-mono text-xs break-all">{{ status.config_paths.project_opencode_dir }}</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

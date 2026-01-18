<script setup lang="ts">
import { ref, onMounted } from 'vue'
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

const servers = ref<McpServer[]>([])
const selectedServer = ref<string | null>(null)
const loading = ref(false)

async function loadServers() {
  loading.value = true
  try {
    servers.value = await invoke<McpServer[]>('get_mcp_servers')
    if (servers.value.length > 0 && !selectedServer.value) {
      selectedServer.value = servers.value[0].name
    }
  } catch (e) {
    console.error('加载 MCP 服务器失败:', e)
  } finally {
    loading.value = false
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

onMounted(() => {
  loadServers()
})

const currentServer = () => servers.value.find(s => s.name === selectedServer.value)
</script>

<template>
  <div class="h-full flex gap-4">
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
</template>

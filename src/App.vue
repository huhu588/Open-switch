<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watchEffect, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link'
import LanguageSwitch from '@/components/LanguageSwitch.vue'
import SvgIcon from '@/components/SvgIcon.vue'
import DeepLinkDialog from '@/components/DeepLinkDialog.vue'

const route = useRoute()
const router = useRouter()
const { t, locale } = useI18n()

// 深链接对话框引用
const deepLinkDialogRef = ref<InstanceType<typeof DeepLinkDialog> | null>(null)
let unlistenDeepLink: (() => void) | null = null
let unlistenSingleInstance: (() => void) | null = null
let isProcessingDeepLink = false // 防止并发处理深链接
const pendingDeepLinks: string[] = []

// 处理深链接的统一方法（带并发保护）
async function handleDeepLinkUrl(url: string) {
  // 如果正在处理深链接，将新请求加入队列
  if (isProcessingDeepLink) {
    pendingDeepLinks.push(url)
    if (import.meta.env.DEV) {
      console.log('深链接处理中，排队:', url)
    }
    return
  }
  
  isProcessingDeepLink = true
  
  try {
    if (import.meta.env.DEV) {
      console.log('收到深链接:', url)
    }
    // 先导航到首页，确保用户能看到对话框
    await router.push('/')
    // 等待一帧确保路由完成
    await new Promise(resolve => requestAnimationFrame(resolve))
    // 然后打开对话框处理深链接
    await deepLinkDialogRef.value?.handleDeepLink(url)
  } finally {
    isProcessingDeepLink = false
  }

  // 处理排队的深链接（按顺序）
  if (pendingDeepLinks.length > 0) {
    const nextUrl = pendingDeepLinks.shift()
    if (nextUrl) {
      await handleDeepLinkUrl(nextUrl)
    }
  }
}

// 设置深链接监听器
async function setupDeepLinkListener() {
  // 1. 首先检查是否有初始深链接（应用通过深链接启动的情况）
  // 使用 plugin API 直接获取，避免事件丢失问题
  try {
    const initialUrls = await getCurrent()
    if (initialUrls && initialUrls.length > 0) {
      // 延迟处理，确保组件已完全挂载
      // 只处理第一个 URL，避免多个对话框同时打开的竞态条件
      setTimeout(() => {
        handleDeepLinkUrl(initialUrls[0])
      }, 100)
    }
  } catch (e) {
    if (import.meta.env.DEV) {
      console.error('获取初始深链接失败:', e)
    }
  }
  
  // 2. 监听后续的深链接事件（应用已运行时收到的深链接）
  // onOpenUrl 仅在 macOS/iOS/Android 上工作
  try {
    unlistenDeepLink = await onOpenUrl((urls: string[]) => {
      // 只处理第一个 URL
      if (urls.length > 0) {
        handleDeepLinkUrl(urls[0])
      }
    })
  } catch (e) {
    if (import.meta.env.DEV) {
      console.error('设置深链接监听器失败:', e)
    }
  }
  
  // 3. 监听 single-instance 插件发送的事件（Windows/Linux 上深链接触发）
  // 在 Windows/Linux 上，OS 会启动新实例并将 URL 作为 CLI 参数
  // single-instance 插件会检测到并发送 deep-link-received 事件给第一个实例
  try {
    unlistenSingleInstance = await listen<string>('deep-link-received', (event) => {
      handleDeepLinkUrl(event.payload)
    })
  } catch (e) {
    if (import.meta.env.DEV) {
      console.error('设置单实例深链接监听器失败:', e)
    }
  }
}

// 关闭确认对话框状态
const showCloseDialog = ref(false)
const closeDialogRef = ref<HTMLElement | null>(null)
const isClosing = ref(false) // 防止重复关闭操作
let unlistenClose: (() => void) | null = null

// 对话框打开时自动聚焦以支持 Escape 键
watch(showCloseDialog, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      closeDialogRef.value?.focus()
    })
  }
})

// 处理关闭请求（使用 Tauri 2.0 的内置 onCloseRequested API）
async function setupCloseListener() {
  const appWindow = getCurrentWindow()
  unlistenClose = await appWindow.onCloseRequested(async (event) => {
    // 如果对话框已经打开或正在处理关闭，忽略新的关闭请求（防止状态混乱）
    if (showCloseDialog.value || isClosing.value) {
      event.preventDefault()
      return
    }
    
    // 先阻止关闭，然后根据设置决定是否真正关闭
    event.preventDefault()
    
    // 立即设置 isClosing 标志，防止并发关闭请求竞争
    isClosing.value = true
    
    try {
      // 获取当前关闭行为设置
      const closeAction = await invoke<string>('get_close_action')
      
      if (closeAction === 'quit') {
        // 直接退出
        await invoke('handle_close_choice', { choice: 'quit' })
      } else if (closeAction === 'tray') {
        // 最小化到托盘
        await invoke('handle_close_choice', { choice: 'tray' })
        // 操作完成后重置标志，允许后续关闭请求
        isClosing.value = false
      } else {
        // 询问用户（默认行为）
        showCloseDialog.value = true
        // 对话框模式下重置标志（对话框本身会阻止新请求）
        isClosing.value = false
      }
    } catch (e) {
      console.error('处理关闭请求失败:', e)
      // 发生错误时重置标志并显示询问对话框
      isClosing.value = false
      showCloseDialog.value = true
    }
  })
}

// 关闭对话框操作（统一由后端处理窗口操作）
async function handleCloseChoice(choice: 'tray' | 'quit') {
  if (isClosing.value) return // 防止重复操作
  isClosing.value = true
  
  try {
    await invoke('handle_close_choice', { choice })
    // 操作成功后清除对话框状态
    showCloseDialog.value = false
    // 重置关闭标志（对于 'tray'，用户可能从托盘恢复窗口后再次关闭；对于 'quit'，应用会退出所以这不重要）
    isClosing.value = false
  } catch (e) {
    console.error('处理关闭选择失败:', e)
    // 操作失败时恢复状态，允许用户重试
    isClosing.value = false
  }
}

// 取消关闭对话框（用户点击外部或按 Escape）
function cancelCloseDialog() {
  showCloseDialog.value = false
  // 不执行任何关闭操作，窗口保持打开
}

// 动态更新文档标题
watchEffect(() => {
  document.title = t('app.title')
  document.documentElement.lang = locale.value === 'zh-CN' ? 'zh-CN' : 'en'
})

// Theme state
const isDark = ref(true)

// 导航图标配置（使用 iconfont 图标名称）
const navItems = computed(() => [
  { 
    name: t('nav.providers'), 
    path: '/', 
    icon: 'server' // 服务商图标
  },
  { 
    name: t('nav.mcp'), 
    path: '/mcp', 
    icon: 'terminal' // 终端/MCP 图标
  },
  {
    name: t('nav.skills'),
    path: '/skills',
    icon: 'layers' // 层级/技能图标
  },
  {
    name: t('nav.ohmy'),
    path: '/ohmy',
    icon: 'robot' // oh-my-opencode 图标
  },
  { 
    name: t('nav.backup'),
    path: '/backup', 
    icon: 'save' // 保存/备份图标
  },
  { 
    name: t('nav.status'), 
    path: '/status', 
    icon: 'activity' // 状态/活动图标
  },
])

const version = ref('')
const localIp = ref('')

function toggleTheme() {
  isDark.value = !isDark.value
  if (isDark.value) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
  localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
}

function initTheme() {
  const savedTheme = localStorage.getItem('theme')
  if (savedTheme === 'light') {
    isDark.value = false
    document.documentElement.classList.remove('dark')
  } else {
    isDark.value = true
    document.documentElement.classList.add('dark')
  }
}

onMounted(async () => {
  initTheme()
  // 设置关闭事件监听器
  await setupCloseListener()
  // 设置深链接监听器
  await setupDeepLinkListener()
  try {
    version.value = await invoke<string>('get_version')
  } catch (e) {
    version.value = '1.0.0'
  }
  // 获取本地IP
  try {
    localIp.value = await invoke<string>('get_local_ip')
  } catch (e) {
    localIp.value = '127.0.0.1'
  }
})

onUnmounted(() => {
  if (unlistenClose) {
    unlistenClose()
  }
  if (unlistenDeepLink) {
    unlistenDeepLink()
  }
  if (unlistenSingleInstance) {
    unlistenSingleInstance()
  }
})
</script>

<template>
  <div class="flex h-screen w-full overflow-hidden bg-background text-primary font-sans relative">
    <!-- Grid Background -->
    <div class="absolute inset-0 bg-grid-tech opacity-20 pointer-events-none z-0"></div>

    <!-- Sidebar -->
    <aside class="flex w-[260px] flex-col border-r border-border bg-surface/50 backdrop-blur-md z-10">
      <!-- Logo Area -->
      <div class="flex h-16 items-center px-6 border-b border-border/50">
        <div class="flex items-center gap-3 group cursor-default">
          <!-- Logo：粉色背景无限符号设计 -->
          <div 
            class="flex h-8 w-8 items-center justify-center rounded-lg shadow-sm transition-all duration-300 group-hover:scale-105 overflow-hidden"
          >
            <svg viewBox="0 0 1024 1024" class="h-full w-full">
              <defs>
                <linearGradient id="logo_bg" x1="0" y1="0" x2="1024" y2="1024" gradientUnits="userSpaceOnUse">
                  <stop offset="0%" stop-color="#F472B6"/>
                  <stop offset="100%" stop-color="#BE185D"/>
                </linearGradient>
                <linearGradient id="logo_symbol" x1="100" y1="512" x2="924" y2="512" gradientUnits="userSpaceOnUse">
                  <stop offset="0%" stop-color="#34D399"/>
                  <stop offset="40%" stop-color="#22D3EE"/>
                  <stop offset="100%" stop-color="#D8B4FE"/>
                </linearGradient>
              </defs>
              <rect x="64" y="64" width="896" height="896" rx="220" fill="url(#logo_bg)"/>
              <g transform="translate(512, 512) scale(0.8)">
                <path d="M 220 160 C 100 160 0 0 -220 -160 C -340 -160 -420 -80 -420 0 C -420 80 -340 160 -220 160 C -100 160 0 0 220 -160 C 340 -160 420 -80 420 0 C 420 80 340 160 220 160 Z" 
                      fill="none" stroke="url(#logo_symbol)" stroke-width="80" stroke-linecap="round" stroke-linejoin="round"/>
              </g>
            </svg>
          </div>
          <div>
            <h1 class="font-bold text-lg tracking-tight text-glow">Open Switch</h1>
            <p class="text-[10px] font-mono text-muted-foreground">v{{ version }}</p>
          </div>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 px-3 py-6 space-y-1">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="group relative flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-all duration-200"
          :class="[
            route.path === item.path
              ? 'bg-accent/10 text-accent'
              : 'text-muted-foreground hover:bg-surface-hover hover:text-primary'
          ]"
        >
          <!-- Active Line Indicator -->
          <div 
            v-if="route.path === item.path"
            class="absolute left-0 top-1/2 -translate-y-1/2 h-5 w-1 rounded-r-full bg-accent shadow-[0_0_8px_currentColor]"
          ></div>

          <SvgIcon 
            :name="item.icon" 
            :size="18" 
            class="transition-transform duration-200 group-hover:scale-110"
          />
          <span class="tracking-wide">{{ item.name }}</span>
        </router-link>
      </nav>

      <!-- Footer / Language & Theme Toggle -->
      <div class="p-4 border-t border-border/50 space-y-2">
        <!-- IP 地址显示 -->
        <div class="flex items-center gap-2 px-3 py-2 rounded-md bg-background/50 border border-border text-xs font-mono">
          <SvgIcon name="network" :size="14" class="text-muted-foreground" />
          <span class="text-muted-foreground">IP:</span>
          <span class="text-accent truncate" :title="localIp">{{ localIp }}</span>
        </div>
        <LanguageSwitch />
        <button
          @click="toggleTheme"
          class="flex w-full items-center justify-between rounded-md border border-border bg-background/50 px-3 py-2 text-xs font-medium transition-all hover:border-accent/40 hover:bg-surface-hover active:scale-[0.98]"
        >
          <div class="flex items-center gap-2">
            <SvgIcon v-if="isDark" name="moon" :size="14" />
            <SvgIcon v-else name="sun" :size="14" />
            <span class="text-muted-foreground group-hover:text-primary transition-colors">
              {{ isDark ? t('system.darkMode') : t('system.lightMode') }}
            </span>
          </div>
          <div class="h-3 w-6 rounded-full bg-surface-hover border border-border p-0.5 transition-colors group-hover:border-accent/50 relative">
             <div class="h-full w-2.5 rounded-full bg-muted-foreground transition-all duration-300 absolute left-0.5 top-0.5" :style="{ transform: isDark ? 'translateX(100%)' : 'translateX(0)', backgroundColor: isDark ? 'var(--accent)' : '' }"></div>
          </div>
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex flex-1 flex-col overflow-hidden relative z-10">
      <!-- Header -->
      <header class="flex h-16 shrink-0 items-center justify-between border-b border-border/50 px-8 bg-background/30 backdrop-blur-sm">
        <div class="flex items-center gap-2">
          <h2 class="text-lg font-bold tracking-tight text-primary">
            {{ navItems.find(item => item.path === route.path)?.name || t('app.title') }}
          </h2>
        </div>
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2 rounded-full border border-border/50 bg-surface/30 px-3 py-1 backdrop-blur-sm">
            <span class="relative flex h-2 w-2">
              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
              <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
            </span>
            <span class="text-[10px] font-mono text-muted-foreground uppercase tracking-wider">{{ t('system.operational') }}</span>
          </div>
        </div>
      </header>

      <!-- Page View -->
      <div class="flex-1 overflow-auto p-6">
        <router-view v-slot="{ Component }">
          <transition 
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="opacity-0 translate-y-2 scale-[0.99]"
            enter-to-class="opacity-100 translate-y-0 scale-100"
            leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="opacity-100 translate-y-0 scale-100"
            leave-to-class="opacity-0 -translate-y-2 scale-[0.99]"
            mode="out-in"
          >
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
    
    <!-- 关闭确认对话框（全局） -->
    <Teleport to="body">
      <div 
        v-if="showCloseDialog" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="cancelCloseDialog"
        @keydown.escape="cancelCloseDialog"
        tabindex="0"
        ref="closeDialogRef"
      >
        <div class="bg-background border border-border rounded-2xl w-[400px] overflow-hidden shadow-2xl">
          <div class="px-6 py-4 border-b border-border">
            <h2 class="text-lg font-semibold">{{ t('settings.closeDialogTitle') }}</h2>
          </div>
          <div class="p-6">
            <p class="text-muted-foreground">{{ t('settings.closeDialogMessage') }}</p>
          </div>
          <div class="px-6 py-4 border-t border-border flex justify-end gap-2">
            <button
              @click="cancelCloseDialog"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-muted-foreground text-sm transition-all"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="handleCloseChoice('tray')"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all flex items-center gap-2"
            >
              <SvgIcon name="monitor" :size="14" />
              {{ t('settings.closeTray') }}
            </button>
            <button
              @click="handleCloseChoice('quit')"
              class="px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white text-sm font-medium transition-all flex items-center gap-2"
            >
              <SvgIcon name="close" :size="14" />
              {{ t('settings.closeQuit') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 深链接确认对话框 -->
    <DeepLinkDialog ref="deepLinkDialogRef" />
  </div>
</template>

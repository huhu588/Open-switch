<script setup lang="ts">
import { ref, onMounted, computed, watchEffect } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import LanguageSwitch from '@/components/LanguageSwitch.vue'
import SvgIcon from '@/components/SvgIcon.vue'

const route = useRoute()
const { t, locale } = useI18n()

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
    name: t('nav.skill'),
    path: '/skill',
    icon: 'layers' // 层级/技能图标
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
          <div class="flex h-8 w-8 items-center justify-center rounded bg-accent text-accent-foreground shadow-[0_0_15px_-3px_rgba(245,158,11,0.4)] transition-all duration-300 group-hover:scale-105 group-hover:shadow-[0_0_20px_-3px_rgba(245,158,11,0.6)]">
            <SvgIcon name="layers" :size="20" />
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
            {{ navItems.find(item => item.path === route.path)?.name }}
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
      <div class="flex-1 overflow-hidden p-6">
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
  </div>
</template>

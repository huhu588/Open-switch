<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useProvidersStore } from '@/stores/providers'
import SvgIcon from './SvgIcon.vue'

const { t } = useI18n()
const store = useProvidersStore()

// 对话框状态
const isOpen = ref(false)
const isAdding = ref(false)
const error = ref('')
const success = ref(false)
const dialogRef = ref<HTMLElement | null>(null)

// 对话框打开时自动聚焦以支持 Escape 键
watch(isOpen, (open) => {
  if (open) {
    nextTick(() => {
      dialogRef.value?.focus()
    })
  }
})

// 深链接数据
interface DeepLinkModel {
  id: string
  name?: string
}

interface DeepLinkProvider {
  name: string
  api_key: string
  base_url: string
  model_type: string
  models?: DeepLinkModel[]
  description?: string
}

interface ParsedDeepLink {
  action: string
  provider?: DeepLinkProvider
  error?: string
}

const parsedData = ref<ParsedDeepLink | null>(null)

// 计算属性：显示的 API Key（脱敏）
const maskedApiKey = computed(() => {
  if (!parsedData.value?.provider?.api_key) return ''
  const key = parsedData.value.provider.api_key
  if (key.length <= 8) return '****'
  return key.substring(0, 4) + '****' + key.substring(key.length - 4)
})

// 处理深链接 URL
async function handleDeepLink(url: string) {
  error.value = ''
  success.value = false
  parsedData.value = null
  
  try {
    // 调用后端解析深链接
    const result = await invoke<ParsedDeepLink>('parse_deep_link', { url })
    
    if (result.error) {
      error.value = result.error
      isOpen.value = true // 打开对话框显示错误
      return
    }
    
    if (result.action === 'add-provider' && result.provider) {
      parsedData.value = result
      isOpen.value = true
    }
  } catch (e) {
    if (import.meta.env.DEV) {
      console.error('解析深链接失败:', e)
    }
    error.value = String(e)
    isOpen.value = true // 打开对话框显示错误
  }
}

// 确认添加 Provider
async function confirmAdd() {
  if (!parsedData.value?.provider) return
  
  isAdding.value = true
  error.value = ''
  
  try {
    const provider = parsedData.value.provider
    
    // 添加 Provider
    await invoke('add_provider', {
      input: {
        name: provider.name,
        api_key: provider.api_key,
        base_url: provider.base_url,
        model_type: provider.model_type,
        description: provider.description || null,
        npm: null,
        auto_add_v1_suffix: false, // 深链接配置不自动添加预设模型
      }
    })
    
    // 如果有自定义模型，逐个添加（保留显示名称）
    if (provider.models && provider.models.length > 0) {
      for (const model of provider.models) {
        await invoke('add_model', {
          providerName: provider.name,
          input: {
            id: model.id,
            name: model.name || model.id, // 使用深链接中指定的显示名称
            reasoning_effort: null,
          }
        })
      }
    }
    
    // 刷新 Provider 列表，确保用户能看到新添加的 Provider
    await store.loadProviders()
    
    success.value = true
    
    // 2秒后关闭对话框
    setTimeout(() => {
      closeDialog()
    }, 2000)
    
  } catch (e) {
    if (import.meta.env.DEV) {
      console.error('添加 Provider 失败:', e)
    }
    error.value = String(e)
  } finally {
    isAdding.value = false
  }
}

// 关闭对话框
function closeDialog() {
  isOpen.value = false
  parsedData.value = null
  error.value = ''
  success.value = false
}

// 暴露方法供外部调用
defineExpose({
  handleDeepLink
})
</script>

<template>
  <Teleport to="body">
    <div 
      v-if="isOpen" 
      class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
      @click.self="closeDialog"
      @keydown.escape="closeDialog"
    >
      <div 
        ref="dialogRef"
        tabindex="-1"
        class="bg-background border border-border rounded-2xl w-[500px] max-w-[90vw] overflow-hidden shadow-2xl outline-none">
        <!-- 头部 -->
        <div class="px-6 py-4 border-b border-border flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-accent/20 flex items-center justify-center">
            <SvgIcon name="link" :size="20" class="text-accent" />
          </div>
          <div>
            <h2 class="text-lg font-semibold">{{ t('deepLink.title') }}</h2>
            <p class="text-xs text-muted-foreground">{{ t('deepLink.subtitle') }}</p>
          </div>
        </div>
        
        <!-- 内容 -->
        <div class="p-6 space-y-4">
          <!-- 成功提示 -->
          <div v-if="success" class="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/30 rounded-xl">
            <SvgIcon name="check" :size="24" class="text-green-500" />
            <span class="text-green-500 font-medium">{{ t('deepLink.success') }}</span>
          </div>
          
          <!-- 错误提示 -->
          <div v-else-if="error" class="flex items-center gap-3 p-4 bg-red-500/10 border border-red-500/30 rounded-xl">
            <SvgIcon name="close" :size="24" class="text-red-500" />
            <span class="text-red-500">{{ error }}</span>
          </div>
          
          <!-- Provider 信息预览 -->
          <div v-else-if="parsedData?.provider" class="space-y-3">
            <p class="text-sm text-muted-foreground">{{ t('deepLink.confirmMessage') }}</p>
            
            <div class="bg-surface rounded-xl p-4 space-y-3">
              <!-- 名称 -->
              <div class="flex items-center justify-between">
                <span class="text-sm text-muted-foreground">{{ t('deepLink.providerName') }}</span>
                <span class="font-medium">{{ parsedData.provider.name }}</span>
              </div>
              
              <!-- Base URL -->
              <div class="flex items-center justify-between">
                <span class="text-sm text-muted-foreground">{{ t('deepLink.baseUrl') }}</span>
                <span class="font-mono text-sm truncate max-w-[250px]">{{ parsedData.provider.base_url }}</span>
              </div>
              
              <!-- API Key (脱敏) -->
              <div class="flex items-center justify-between">
                <span class="text-sm text-muted-foreground">{{ t('deepLink.apiKey') }}</span>
                <span class="font-mono text-sm">{{ maskedApiKey }}</span>
              </div>
              
              <!-- API 协议 -->
              <div class="flex items-center justify-between">
                <span class="text-sm text-muted-foreground">{{ t('deepLink.modelType') }}</span>
                <span class="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded-full">
                  {{ parsedData.provider.model_type }}
                </span>
              </div>
              
              <!-- 自定义模型 -->
              <div v-if="parsedData.provider.models && parsedData.provider.models.length > 0">
                <span class="text-sm text-muted-foreground">{{ t('deepLink.customModels') }}</span>
                <div class="flex flex-wrap gap-1.5 mt-2">
                  <span 
                    v-for="model in parsedData.provider.models" 
                    :key="model.id"
                    class="px-2 py-0.5 bg-surface-hover text-xs rounded-full"
                  >
                    {{ model.name || model.id }}
                  </span>
                </div>
              </div>
              
              <!-- 描述 -->
              <div v-if="parsedData.provider.description" class="pt-2 border-t border-border">
                <span class="text-sm text-muted-foreground">{{ t('deepLink.description') }}</span>
                <p class="text-sm mt-1">{{ parsedData.provider.description }}</p>
              </div>
            </div>
            
            <!-- 安全提示 -->
            <div class="flex items-start gap-2 p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg">
              <SvgIcon name="info" :size="16" class="text-amber-500 mt-0.5 flex-shrink-0" />
              <p class="text-xs text-amber-500">{{ t('deepLink.securityNote') }}</p>
            </div>
          </div>
        </div>
        
        <!-- 底部按钮 -->
        <div class="px-6 py-4 border-t border-border flex justify-end gap-2">
          <button
            @click="closeDialog"
            :disabled="isAdding"
            class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-muted-foreground text-sm transition-all disabled:opacity-50"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            v-if="!success && !error"
            @click="confirmAdd"
            :disabled="isAdding"
            class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all disabled:opacity-50 flex items-center gap-2"
          >
            <svg v-if="isAdding" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ isAdding ? t('deepLink.adding') : t('deepLink.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

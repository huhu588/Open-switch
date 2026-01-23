<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProvidersStore, type DeployedProviderItem } from '@/stores/providers'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()
const store = useProvidersStore()

interface Props {
  visible: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'imported': []
}>()

const deployedProviders = ref<DeployedProviderItem[]>([])
const loading = ref(false)
const deleting = ref<string | null>(null)
const importing = ref<string | null>(null)
const error = ref<string | null>(null)
const showModelTypeDialog = ref(false)
const importingProvider = ref<DeployedProviderItem | null>(null)
const syncingAll = ref(false)

// 加载已部署的服务商
async function loadData() {
  loading.value = true
  error.value = null
  try {
    deployedProviders.value = await store.loadDeployedProviders()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// 当对话框打开时加载数据
watch(() => props.visible, (visible) => {
  if (visible) {
    loadData()
  }
})

function close() {
  emit('update:visible', false)
}

// 删除已部署的服务商
async function removeProvider(provider: DeployedProviderItem) {
  if (deleting.value) return
  
  deleting.value = provider.name
  try {
    await store.removeDeployedProvider(
      provider.name,
      provider.source === 'global',
      provider.source === 'project'
    )
    // 重新加载列表
    await loadData()
  } catch (e) {
    error.value = String(e)
  } finally {
    deleting.value = null
  }
}

// 批量删除所有已部署的服务商
async function removeAll() {
  if (deleting.value || deployedProviders.value.length === 0) return
  
  deleting.value = 'all'
  try {
    for (const provider of deployedProviders.value) {
      await store.removeDeployedProvider(
        provider.name,
        provider.source === 'global',
        provider.source === 'project'
      )
    }
    // 重新加载列表
    await loadData()
  } catch (e) {
    error.value = String(e)
  } finally {
    deleting.value = null
  }
}

// 开始导入流程：如果可以推断则直接导入，否则选择模型类型
function startImport(provider: DeployedProviderItem) {
  importingProvider.value = provider
  // 如果能够推断出 model_type，直接导入
  if (provider.inferred_model_type) {
    importProvider(provider.inferred_model_type)
  } else {
    // 否则显示手动选择对话框
    showModelTypeDialog.value = true
  }
}

// 导入已部署的服务商
async function importProvider(modelType: string) {
  if (!importingProvider.value || importing.value) return
  
  importing.value = importingProvider.value.name
  try {
    await store.importDeployedProvider(importingProvider.value.name, modelType)
    showModelTypeDialog.value = false
    importingProvider.value = null
    // 发出事件通知父组件刷新列表
    emit('imported')
  } catch (e) {
    error.value = String(e)
  } finally {
    importing.value = null
  }
}

// 一键同步所有已部署的服务商
async function syncAll() {
  if (syncingAll.value || deployedProviders.value.length === 0) return
  
  syncingAll.value = true
  error.value = null
  let successCount = 0
  let failCount = 0
  
  try {
    for (const provider of deployedProviders.value) {
      try {
        // 优先使用推断的模型类型，如果没有则跳过
        if (provider.inferred_model_type) {
          await store.importDeployedProvider(provider.name, provider.inferred_model_type)
          successCount++
        } else {
          // 如果无法推断模型类型，尝试默认使用 codex
          await store.importDeployedProvider(provider.name, 'codex')
          successCount++
        }
      } catch (e) {
        console.error(`导入 ${provider.name} 失败:`, e)
        failCount++
      }
    }
    
    // 显示结果提示
    if (failCount === 0) {
      error.value = null
    } else {
      error.value = `成功导入 ${successCount} 个，失败 ${failCount} 个`
    }
    
    // 通知父组件刷新列表
    emit('imported')
  } finally {
    syncingAll.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="close">
        <div class="w-full max-w-lg rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <!-- 标题 -->
          <div class="px-5 py-4 border-b border-border flex items-center justify-between">
            <div class="flex items-center gap-3">
              <SvgIcon name="setting" :size="20" class="text-accent" />
              <h3 class="font-semibold text-lg">{{ t('deployed.title') }}</h3>
            </div>
            <button @click="close" class="p-1 rounded-lg hover:bg-surface-hover transition-colors">
              <SvgIcon name="close" :size="18" class="text-muted-foreground" />
            </button>
          </div>

          <!-- 内容 -->
          <div class="px-5 py-4 max-h-[400px] overflow-y-auto">
            <!-- 错误提示 -->
            <div v-if="error" class="mb-4 px-3 py-2 rounded-lg bg-error-500/10 border border-error-500/30 text-error-500 text-sm">
              {{ error }}
            </div>

            <!-- 加载中 -->
            <div v-if="loading" class="py-8 text-center text-muted-foreground">
              {{ t('common.loading') }}
            </div>

            <!-- 空状态 -->
            <div v-else-if="deployedProviders.length === 0" class="py-8 text-center text-muted-foreground">
              <SvgIcon name="check" :size="48" class="mx-auto mb-3 opacity-50" />
              <p>{{ t('deployed.noProviders') }}</p>
            </div>

            <!-- 服务商列表 -->
            <div v-else class="space-y-2">
              <p class="text-sm text-muted-foreground mb-3">
                {{ t('deployed.description') }}
              </p>
              
              <div
                v-for="provider in deployedProviders"
                :key="provider.name"
                class="flex items-center justify-between p-3 rounded-lg bg-surface border border-border hover:border-accent/50 transition-colors"
              >
                <div class="flex-1 min-w-0 mr-3">
                  <div class="flex items-center gap-2">
                    <span class="font-medium truncate">{{ provider.name }}</span>
                    <span 
                      class="px-1.5 py-0.5 text-xs rounded"
                      :class="provider.source === 'global' 
                        ? 'bg-blue-500/10 text-blue-500' 
                        : 'bg-emerald-500/10 text-emerald-500'"
                    >
                      {{ provider.source === 'global' ? t('deployed.global') : t('deployed.project') }}
                    </span>
                  </div>
                  <div class="text-xs text-muted-foreground mt-1 truncate">
                    {{ provider.base_url }} · {{ provider.model_count }} {{ t('deployed.models') }}
                  </div>
                </div>
                
                <div class="flex items-center gap-1">
                  <!-- 导入按钮 -->
                  <button
                    @click="startImport(provider)"
                    :disabled="importing !== null || deleting !== null"
                    class="p-2 rounded-lg text-muted-foreground hover:text-accent hover:bg-accent/10 transition-colors disabled:opacity-50"
                    title="导入到管理界面"
                  >
                    <svg v-if="importing === provider.name" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <SvgIcon v-else name="download" :size="16" />
                  </button>
                  
                  <!-- 删除按钮 -->
                  <button
                    @click="removeProvider(provider)"
                    :disabled="deleting !== null || importing !== null"
                    class="p-2 rounded-lg text-muted-foreground hover:text-error-500 hover:bg-error-500/10 transition-colors disabled:opacity-50"
                    :title="t('common.delete')"
                  >
                    <svg v-if="deleting === provider.name" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <SvgIcon v-else name="delete" :size="16" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="px-5 py-4 flex justify-between gap-3 border-t border-border">
            <button
              v-if="deployedProviders.length > 0"
              @click="removeAll"
              :disabled="deleting !== null || syncingAll"
              class="px-4 py-2 text-sm font-medium rounded-lg text-error-500 border border-error-500/30 hover:bg-error-500/10 disabled:opacity-50 transition-colors"
            >
              {{ deleting === 'all' ? t('common.loading') : t('deployed.removeAll') }}
            </button>
            <button
              v-if="deployedProviders.length > 0"
              @click="syncAll"
              :disabled="syncingAll || deleting !== null || importing !== null"
              class="px-4 py-2 text-sm font-medium rounded-lg text-accent border border-accent/30 hover:bg-accent/10 disabled:opacity-50 transition-colors"
            >
              {{ syncingAll ? t('common.loading') : t('deployed.syncAll') }}
            </button>
            <div class="flex-1"></div>
            <button
              @click="close"
              class="px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover transition-colors"
            >
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 模型类型选择对话框 -->
    <Transition name="fade">
      <div v-if="showModelTypeDialog" class="fixed inset-0 z-[60] flex items-center justify-center p-4 bg-black/50" @click.self="showModelTypeDialog = false">
        <div class="w-full max-w-sm rounded-xl bg-background border border-border shadow-xl animate-slide-up">
          <div class="px-5 py-4 border-b border-border">
            <h3 class="font-semibold text-lg">选择模型类型</h3>
            <p class="text-sm text-muted-foreground mt-1">
              为 <span class="font-medium text-foreground">{{ importingProvider?.name }}</span> 选择所属的模型类型
            </p>
          </div>
          
          <div class="px-5 py-4 space-y-2">
            <button
              @click="importProvider('claude')"
              :disabled="importing !== null"
              class="w-full px-4 py-3 text-left rounded-lg border border-border hover:border-accent hover:bg-accent/5 transition-colors disabled:opacity-50"
            >
              <div class="font-medium">Claude</div>
              <div class="text-xs text-muted-foreground mt-0.5">Anthropic Claude 模型</div>
            </button>
            
            <button
              @click="importProvider('codex')"
              :disabled="importing !== null"
              class="w-full px-4 py-3 text-left rounded-lg border border-border hover:border-accent hover:bg-accent/5 transition-colors disabled:opacity-50"
            >
              <div class="font-medium">Codex</div>
              <div class="text-xs text-muted-foreground mt-0.5">OpenAI GPT / Codex 模型</div>
            </button>
            
            <button
              @click="importProvider('gemini')"
              :disabled="importing !== null"
              class="w-full px-4 py-3 text-left rounded-lg border border-border hover:border-accent hover:bg-accent/5 transition-colors disabled:opacity-50"
            >
              <div class="font-medium">Gemini</div>
              <div class="text-xs text-muted-foreground mt-0.5">Google Gemini 模型</div>
            </button>
          </div>
          
          <div class="px-5 py-4 border-t border-border">
            <button
              @click="showModelTypeDialog = false"
              :disabled="importing !== null"
              class="w-full px-4 py-2 text-sm font-medium rounded-lg border border-border hover:bg-surface-hover transition-colors disabled:opacity-50"
            >
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useProvidersStore } from '@/stores/providers'

const { t } = useI18n()
import ProviderList from '@/components/ProviderList.vue'
import ProviderDialog from '@/components/ProviderDialog.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ApplyDialog from '@/components/ApplyDialog.vue'
import ModelTypeSelector from '@/components/ModelTypeSelector.vue'
import DeployedProvidersDialog from '@/components/DeployedProvidersDialog.vue'
import { type ModelType } from '@/config/modelTypes'

const store = useProvidersStore()

// ProviderList 组件引用
const providerListRef = ref<InstanceType<typeof ProviderList> | null>(null)

// 模型厂家筛选
const selectedModelType = ref<ModelType>('claude')

// 已部署的服务商映射：base_url -> tools[]
const deployedToolsMap = ref<Map<string, string[]>>(new Map())

// 切换模型厂家时清空选中的 Provider
watch(selectedModelType, () => {
  store.selectedProvider = null
  store.models = []
  store.selectedModel = null
})

// 根据模型厂家筛选 Provider
const filteredProviders = computed(() => {
  return store.providers.filter(p => {
    // 根据 provider 的 model_type 字段筛选，如果没有则默认显示在 claude
    const providerModelType = p.model_type || 'claude'
    return providerModelType === selectedModelType.value
  })
})

// 获取启用的 Provider 名称列表（用于应用配置）
const enabledProviderNames = computed(() => {
  return filteredProviders.value
    .filter(p => p.enabled)
    .map(p => p.name)
})

// 对话框状态
const showProviderDialog = ref(false)
const showDeleteDialog = ref(false)
const showApplyDialog = ref(false)
const showDeployedDialog = ref(false)
const editingProvider = ref<string | null>(null)
const deleteTarget = ref<{ type: 'provider'; name: string } | null>(null)

// 加载已部署的服务商信息
async function loadDeployedTools() {
  try {
    const deployed = await store.loadAllDeployedProviders()
    const map = new Map<string, string[]>()
    
    for (const item of deployed) {
      const key = item.base_url
      if (!map.has(key)) {
        map.set(key, [])
      }
      if (item.tool) {
        const tools = map.get(key)!
        if (!tools.includes(item.tool)) {
          tools.push(item.tool)
        }
      }
    }
    
    deployedToolsMap.value = map
  } catch (e) {
    console.error('加载部署信息失败:', e)
  }
}

// 加载数据
onMounted(() => {
  store.loadProviders()
  loadDeployedTools()
})

// 添加 Provider
function openAddProvider() {
  editingProvider.value = null
  showProviderDialog.value = true
}

// 编辑 Provider
function openEditProvider(name: string) {
  editingProvider.value = name
  showProviderDialog.value = true
}

// 删除 Provider
function openDeleteProvider(name: string) {
  deleteTarget.value = { type: 'provider', name }
  showDeleteDialog.value = true
}

// 确认删除
async function confirmDelete() {
  if (!deleteTarget.value) return
  
  try {
    await store.deleteProvider(deleteTarget.value.name)
  } catch (e) {
    console.error('删除失败:', e)
  }
  
  showDeleteDialog.value = false
  deleteTarget.value = null
}

// 应用配置
function openApplyDialog() {
  if (enabledProviderNames.value.length > 0) {
    showApplyDialog.value = true
  }
}

// 切换 Provider 启用状态
async function handleToggleProvider(name: string, enabled: boolean) {
  try {
    await store.toggleProvider(name, enabled)
  } catch (e) {
    console.error('切换启用状态失败:', e)
  }
}

// 处理测速请求
async function handleSpeedTest(providerName: string) {
  try {
    // 获取 provider 信息
    const provider = store.providers.find(p => p.name === providerName)
    if (!provider) return

    // 获取 provider 详情以获取 API Key
    const detail = await invoke<any>('get_provider', { name: providerName })
    if (!detail) return

    // 获取所有 URL
    const urls = provider.base_urls?.map(u => u.url) || [provider.base_url]
    
    // 调用测试并自动选择最快 URL 的命令
    await invoke('test_and_auto_select_fastest', {
      providerName,
      urls,
      apiKey: detail.options.api_key,
      modelType: provider.model_type
    })

    // 重新加载 providers 以更新显示
    await store.loadProviders()

    // 通知 ProviderList 测试完成
    providerListRef.value?.setTestingComplete(providerName)
  } catch (e) {
    console.error('测速失败:', e)
    providerListRef.value?.setTestingComplete(providerName)
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- 顶部模型厂家选择器 -->
    <div class="flex-shrink-0 flex justify-center">
      <ModelTypeSelector v-model="selectedModelType" />
    </div>

    <!-- 主内容区 - 全宽 Provider 列表 -->
    <div class="flex-1 min-h-0">
      <ProviderList
        ref="providerListRef"
        :providers="filteredProviders"
        :selected="store.selectedProvider"
        :deployed-tools-map="deployedToolsMap"
        @select="store.selectProvider"
        @add="openAddProvider"
        @edit="openEditProvider"
        @delete="openDeleteProvider"
        @apply="openApplyDialog"
        @toggle="handleToggleProvider"
        @view-deployed="showDeployedDialog = true"
        @speed-test="handleSpeedTest"
      />
    </div>

    <!-- Provider 对话框（包含模型管理和延迟测试） -->
    <ProviderDialog
      v-model:visible="showProviderDialog"
      :editing="editingProvider"
      :default-model-type="selectedModelType"
      @saved="store.loadProviders()"
    />

    <!-- 删除确认对话框 -->
    <ConfirmDialog
      v-model:visible="showDeleteDialog"
      :title="t('confirm.deleteTitle')"
      :message="t('confirm.deleteProvider', { name: deleteTarget?.name })"
      :confirm-text="t('common.delete')"
      danger
      @confirm="confirmDelete"
    />

    <!-- 应用配置对话框 -->
    <ApplyDialog
      v-model:visible="showApplyDialog"
      :provider-names="enabledProviderNames"
      :model-type="selectedModelType"
      @applied="() => {}"
    />

    <!-- 已部署服务商管理对话框 -->
    <DeployedProvidersDialog 
      v-model:visible="showDeployedDialog" 
      @imported="store.loadProviders()" 
    />
  </div>
</template>

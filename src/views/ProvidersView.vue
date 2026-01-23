<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProvidersStore } from '@/stores/providers'

const { t } = useI18n()
import ProviderList from '@/components/ProviderList.vue'
import ModelList from '@/components/ModelList.vue'
import ProviderDialog from '@/components/ProviderDialog.vue'
import ModelDialog from '@/components/ModelDialog.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ApplyDialog from '@/components/ApplyDialog.vue'
import FetchModelsDialog from '@/components/FetchModelsDialog.vue'
import ModelTypeSelector from '@/components/ModelTypeSelector.vue'
import DeployedProvidersDialog from '@/components/DeployedProvidersDialog.vue'
import { type ModelType } from '@/config/modelTypes'

const store = useProvidersStore()

// 模型厂家筛选
const selectedModelType = ref<ModelType>('claude')

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
const showModelDialog = ref(false)
const showDeleteDialog = ref(false)
const showApplyDialog = ref(false)
const showFetchModelsDialog = ref(false)
const showDeployedDialog = ref(false)
const editingProvider = ref<string | null>(null)
const editingModel = ref<{ id: string; name: string } | null>(null)
const deleteTarget = ref<{ type: 'provider' | 'model'; name: string } | null>(null)

// 加载数据
onMounted(() => {
  store.loadProviders()
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

// 添加 Model
function openAddModel() {
  editingModel.value = null
  showModelDialog.value = true
}

// 编辑 Model
function openEditModel(model: { id: string; name: string }) {
  editingModel.value = model
  showModelDialog.value = true
}

// 删除 Model
function openDeleteModel(id: string) {
  deleteTarget.value = { type: 'model', name: id }
  showDeleteDialog.value = true
}

// 确认删除
async function confirmDelete() {
  if (!deleteTarget.value) return
  
  try {
    if (deleteTarget.value.type === 'provider') {
      await store.deleteProvider(deleteTarget.value.name)
    } else {
      await store.deleteModel(deleteTarget.value.name)
    }
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

// 获取站点模型
function openFetchModels() {
  if (store.selectedProvider) {
    showFetchModelsDialog.value = true
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
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- 顶部模型厂家选择器 -->
    <div class="flex-shrink-0 flex justify-center">
      <ModelTypeSelector v-model="selectedModelType" />
    </div>

    <!-- 主内容区 -->
    <div class="flex-1 flex gap-4 min-h-0">
      <!-- Provider 列表 -->
      <div class="flex-[3] min-w-0">
        <ProviderList
          :providers="filteredProviders"
          :selected="store.selectedProvider"
          @select="store.selectProvider"
          @add="openAddProvider"
          @edit="openEditProvider"
          @delete="openDeleteProvider"
          @apply="openApplyDialog"
          @toggle="handleToggleProvider"
          @view-deployed="showDeployedDialog = true"
        />
      </div>

      <!-- Model 列表 -->
      <div class="flex-[2] min-w-0">
        <ModelList
          :models="store.models"
          :selected="store.selectedModel"
          :disabled="!store.selectedProvider"
          @select="id => store.selectedModel = id"
          @add="openAddModel"
          @edit="openEditModel"
          @delete="openDeleteModel"
          @fetch="openFetchModels"
        />
      </div>
    </div>

    <!-- Provider 对话框 -->
    <ProviderDialog
      v-model:visible="showProviderDialog"
      :editing="editingProvider"
      :default-model-type="selectedModelType"
      @saved="store.loadProviders()"
    />

    <!-- Model 对话框 -->
    <ModelDialog
      v-model:visible="showModelDialog"
      :provider-name="store.selectedProvider"
      :editing="editingModel"
      @saved="store.loadModels()"
    />

    <!-- 删除确认对话框 -->
    <ConfirmDialog
      v-model:visible="showDeleteDialog"
      :title="t('confirm.deleteTitle')"
      :message="deleteTarget?.type === 'provider' 
        ? t('confirm.deleteProvider', { name: deleteTarget?.name }) 
        : t('confirm.deleteModel', { name: deleteTarget?.name })"
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

    <!-- 获取站点模型对话框 -->
    <FetchModelsDialog
      v-model:visible="showFetchModelsDialog"
      :provider-name="store.selectedProvider"
      @added="store.loadModels()"
    />

    <!-- 已部署服务商管理对话框 -->
    <DeployedProvidersDialog 
      v-model:visible="showDeployedDialog" 
      @imported="store.loadProviders()" 
    />
  </div>
</template>

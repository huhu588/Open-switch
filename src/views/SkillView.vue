<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

// 类型定义
interface InstalledSkill {
  name: string
  path: string
  location: string
  content_preview: string
}

interface RecommendedSkill {
  id: string
  name: string
  description: string
  category: string
  repo: string
  repo_url: string
  raw_url: string
}

interface InstallSkillResult {
  success: boolean
  message: string
  installed_path: string | null
}

// 状态
const installedSkills = ref<InstalledSkill[]>([])
const recommendedSkills = ref<RecommendedSkill[]>([])
const loading = ref(false)
const selectedSkill = ref<InstalledSkill | null>(null)
const skillContent = ref('')

// 推荐 Skill 弹窗
const showRecommendedModal = ref(false)
const selectedRecommended = ref<Set<string>>(new Set())
const installLocation = ref('global_opencode')
const installing = ref(false)

// 消息提示
const message = ref('')
const messageType = ref<'success' | 'error'>('success')

// 删除确认弹窗
const showDeleteConfirm = ref(false)
const skillToDelete = ref<InstalledSkill | null>(null)

// 查看内容弹窗
const showContentModal = ref(false)

// 按位置分组的已安装 Skills
const groupedSkills = computed(() => {
  const groups: Record<string, InstalledSkill[]> = {
    'GlobalOpenCode': [],
    'GlobalClaude': [],
    'ProjectOpenCode': [],
    'ProjectClaude': [],
  }
  
  for (const skill of installedSkills.value) {
    if (groups[skill.location]) {
      groups[skill.location].push(skill)
    }
  }
  
  return groups
})

// 位置标签
const locationLabels: Record<string, string> = {
  'GlobalOpenCode': '全局 OpenCode (~/.config/opencode/skill/)',
  'GlobalClaude': '全局 Claude (~/.claude/skills/)',
  'ProjectOpenCode': '项目 OpenCode (.opencode/skill/)',
  'ProjectClaude': '项目 Claude (.claude/skills/)',
}

// 加载已安装的 Skills
async function loadInstalledSkills() {
  loading.value = true
  try {
    installedSkills.value = await invoke<InstalledSkill[]>('get_installed_skills')
  } catch (e) {
    console.error('加载 Skills 失败:', e)
  } finally {
    loading.value = false
  }
}

// 加载推荐 Skills
async function loadRecommendedSkills() {
  try {
    recommendedSkills.value = await invoke<RecommendedSkill[]>('get_recommended_skills')
  } catch (e) {
    console.error('加载推荐 Skills 失败:', e)
  }
}

// 打开推荐弹窗
async function openRecommendedModal() {
  await loadRecommendedSkills()
  
  // 已安装的 skill 名称
  const installedNames = new Set(installedSkills.value.map(s => s.name))
  
  // 默认选中未安装的
  selectedRecommended.value = new Set(
    recommendedSkills.value
      .filter(r => !installedNames.has(r.id))
      .map(r => r.id)
  )
  
  showRecommendedModal.value = true
}

// 切换选中
function toggleSelect(id: string) {
  if (selectedRecommended.value.has(id)) {
    selectedRecommended.value.delete(id)
  } else {
    selectedRecommended.value.add(id)
  }
  selectedRecommended.value = new Set(selectedRecommended.value)
}

// 检查是否已安装
function isInstalled(skillId: string): boolean {
  return installedSkills.value.some(s => s.name === skillId)
}

// 安装选中的 Skills
async function installSelected() {
  if (selectedRecommended.value.size === 0) return
  
  installing.value = true
  let successCount = 0
  let failCount = 0
  
  try {
    for (const skillId of selectedRecommended.value) {
      const skill = recommendedSkills.value.find(s => s.id === skillId)
      if (!skill) continue
      
      try {
        const result = await invoke<InstallSkillResult>('install_skill', {
          input: {
            skill_id: skill.id,
            raw_url: skill.raw_url,
            location: installLocation.value
          }
        })
        
        if (result.success) {
          successCount++
        } else {
          failCount++
        }
      } catch (e) {
        console.error(`安装 ${skill.name} 失败:`, e)
        failCount++
      }
    }
    
    showRecommendedModal.value = false
    await loadInstalledSkills()
    
    if (successCount > 0) {
      showMessage(`成功安装 ${successCount} 个 Skill`, 'success')
    }
    if (failCount > 0) {
      showMessage(`${failCount} 个 Skill 安装失败`, 'error')
    }
  } finally {
    installing.value = false
  }
}

// 查看 Skill 内容
async function viewSkillContent(skill: InstalledSkill) {
  selectedSkill.value = skill
  try {
    skillContent.value = await invoke<string>('read_skill_content', { skillPath: skill.path })
    showContentModal.value = true
  } catch (e) {
    console.error('读取 Skill 内容失败:', e)
    showMessage('读取失败', 'error')
  }
}

// 确认删除
function confirmDelete(skill: InstalledSkill) {
  skillToDelete.value = skill
  showDeleteConfirm.value = true
}

// 执行删除
async function deleteSkill() {
  if (!skillToDelete.value) return
  
  try {
    await invoke('delete_skill', { skillPath: skillToDelete.value.path })
    showDeleteConfirm.value = false
    await loadInstalledSkills()
    showMessage('删除成功', 'success')
  } catch (e) {
    console.error('删除失败:', e)
    showMessage('删除失败', 'error')
  }
}

// 显示消息
function showMessage(msg: string, type: 'success' | 'error') {
  message.value = msg
  messageType.value = type
  setTimeout(() => { message.value = '' }, 3000)
}

// 按分类分组的推荐 Skills
const groupedRecommended = computed(() => {
  const groups: Record<string, RecommendedSkill[]> = {}
  for (const skill of recommendedSkills.value) {
    if (!groups[skill.repo]) {
      groups[skill.repo] = []
    }
    groups[skill.repo].push(skill)
  }
  return groups
})

onMounted(() => {
  loadInstalledSkills()
})
</script>

<template>
  <div class="h-full flex flex-col gap-4">
    <!-- 顶部工具栏 -->
    <div class="flex items-center justify-between flex-shrink-0">
      <div class="flex items-center gap-3">
        <button
          @click="openRecommendedModal"
          class="px-4 py-2 rounded-lg bg-accent/20 hover:bg-accent/30 text-accent font-medium text-sm transition-all flex items-center gap-2"
        >
          <SvgIcon name="star" :size="16" />
          {{ t('skill.addRecommended') }}
        </button>
        
        <button
          @click="loadInstalledSkills"
          class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground font-medium text-sm transition-all flex items-center gap-2"
        >
          <SvgIcon name="refresh" :size="16" />
          {{ t('skill.refresh') }}
        </button>
      </div>
      
      <!-- 消息提示 -->
      <div 
        v-if="message" 
        class="text-sm animate-pulse"
        :class="messageType === 'success' ? 'text-emerald-500' : 'text-red-500'"
      >
        {{ message }}
      </div>
    </div>
    
    <!-- 主内容区 -->
    <div class="flex-1 overflow-auto">
      <div v-if="loading" class="text-center py-8 text-muted-foreground">
        {{ t('common.loading') }}
      </div>
      
      <div v-else-if="installedSkills.length === 0" class="text-center py-12">
        <div class="mb-4 flex justify-center"><SvgIcon name="layers" :size="48" class="text-muted-foreground opacity-30" /></div>
        <p class="text-muted-foreground mb-4">{{ t('skill.noSkills') }}</p>
        <button
          @click="openRecommendedModal"
          class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white font-medium text-sm transition-all"
        >
          {{ t('skill.installFirst') }}
        </button>
      </div>
      
      <!-- 按位置分组显示 -->
      <div v-else class="space-y-6">
        <template v-for="(skills, location) in groupedSkills" :key="location">
          <div v-if="skills.length > 0" class="rounded-xl bg-surface/30 border border-border overflow-hidden">
            <div class="px-4 py-3 border-b border-border bg-surface/50">
              <h3 class="font-semibold text-sm flex items-center gap-2">
                <SvgIcon :name="location.includes('Claude') ? 'robot' : 'box'" :size="16" class="text-accent" />
                {{ locationLabels[location] }}
                <span class="text-xs text-muted-foreground">({{ skills.length }})</span>
              </h3>
            </div>
            
            <div class="p-3 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              <div
                v-for="skill in skills"
                :key="skill.path"
                class="p-4 rounded-lg border border-border bg-background hover:border-accent/50 transition-all group"
              >
                <div class="flex items-start justify-between mb-2">
                  <h4 class="font-medium">{{ skill.name }}</h4>
                  <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      @click="viewSkillContent(skill)"
                      class="p-1.5 rounded hover:bg-surface text-muted-foreground hover:text-foreground transition-colors"
                      :title="t('skill.view')"
                    >
                      <SvgIcon name="eye" :size="14" />
                    </button>
                    <button
                      @click="confirmDelete(skill)"
                      class="p-1.5 rounded hover:bg-red-500/10 text-muted-foreground hover:text-red-500 transition-colors"
                      :title="t('common.delete')"
                    >
                      <SvgIcon name="trash" :size="14" />
                    </button>
                  </div>
                </div>
                <p class="text-xs text-muted-foreground line-clamp-2">
                  {{ skill.content_preview }}
                </p>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
    
    <!-- 推荐 Skill 弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showRecommendedModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showRecommendedModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[640px] max-h-[80vh] overflow-hidden shadow-2xl">
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold">{{ t('skill.recommended') }}</h2>
            <button 
              @click="showRecommendedModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 安装位置选择 -->
          <div class="px-6 py-3 border-b border-border bg-surface/30">
            <label class="text-sm font-medium mb-2 block">{{ t('skill.installLocation') }}</label>
            <div class="flex flex-wrap gap-2">
              <label 
                v-for="(label, key) in { global_opencode: '全局 OpenCode', global_claude: '全局 Claude', project_opencode: '项目 OpenCode', project_claude: '项目 Claude' }" 
                :key="key"
                class="flex items-center gap-2 px-3 py-1.5 rounded-lg cursor-pointer transition-all text-sm"
                :class="installLocation === key ? 'bg-accent/20 text-accent' : 'bg-surface hover:bg-surface-hover'"
              >
                <input
                  type="radio"
                  :value="key"
                  v-model="installLocation"
                  class="hidden"
                />
                {{ label }}
              </label>
            </div>
          </div>
          
          <!-- Skill 列表 -->
          <div class="p-4 space-y-4 max-h-[45vh] overflow-auto">
            <div v-for="(skills, repo) in groupedRecommended" :key="repo" class="space-y-2">
              <h3 class="text-sm font-semibold text-muted-foreground flex items-center gap-2">
                <SvgIcon name="book" :size="14" /> {{ repo }}
                <a 
                  :href="skills[0]?.repo_url" 
                  target="_blank" 
                  class="text-xs text-accent hover:underline"
                >
                  GitHub ↗
                </a>
              </h3>
              
              <div class="grid gap-2">
                <div
                  v-for="skill in skills"
                  :key="skill.id"
                  @click="!isInstalled(skill.id) && toggleSelect(skill.id)"
                  class="p-3 rounded-xl border transition-all"
                  :class="[
                    isInstalled(skill.id)
                      ? 'border-emerald-500/30 bg-emerald-500/5 cursor-default'
                      : selectedRecommended.has(skill.id)
                        ? 'border-accent bg-accent/10 cursor-pointer'
                        : 'border-border hover:border-accent/50 bg-surface/30 cursor-pointer'
                  ]"
                >
                  <div class="flex items-start gap-3">
                    <!-- 选中指示器 -->
                    <div 
                      v-if="!isInstalled(skill.id)"
                      class="w-5 h-5 rounded-md border-2 flex items-center justify-center flex-shrink-0 mt-0.5 transition-all"
                      :class="selectedRecommended.has(skill.id) 
                        ? 'border-accent bg-accent text-white' 
                        : 'border-muted-foreground'"
                    >
                      <span v-if="selectedRecommended.has(skill.id)" class="text-xs">✓</span>
                    </div>
                    <!-- 已安装标记 -->
                    <div 
                      v-else
                      class="w-5 h-5 rounded-md bg-emerald-500/20 flex items-center justify-center flex-shrink-0 mt-0.5"
                    >
                      <span class="text-xs text-emerald-500">✓</span>
                    </div>
                    
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="font-semibold">{{ skill.name }}</span>
                        <span v-if="isInstalled(skill.id)" class="text-xs px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-500">
                          {{ t('skill.installed') }}
                        </span>
                      </div>
                      <p class="text-sm text-muted-foreground mt-1">{{ skill.description }}</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 底部 -->
          <div class="px-6 py-4 border-t border-border flex items-center justify-between">
            <span class="text-sm text-muted-foreground">
              {{ t('skill.selected', { count: selectedRecommended.size }) }}
            </span>
            <div class="flex gap-2">
              <button
                @click="showRecommendedModal = false"
                class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                @click="installSelected"
                :disabled="selectedRecommended.size === 0 || installing"
                class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white text-sm font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ installing ? t('skill.installing') : t('skill.installAll') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 内容查看弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showContentModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showContentModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[700px] max-h-[80vh] overflow-hidden shadow-2xl">
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold">{{ selectedSkill?.name }} - SKILL.md</h2>
            <button 
              @click="showContentModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          <div class="p-6 max-h-[60vh] overflow-auto">
            <pre class="text-sm font-mono whitespace-pre-wrap text-foreground/90">{{ skillContent }}</pre>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 删除确认弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showDeleteConfirm" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showDeleteConfirm = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[400px] overflow-hidden shadow-2xl">
          <div class="px-6 py-4 border-b border-border">
            <h2 class="text-lg font-semibold">{{ t('confirm.deleteTitle') }}</h2>
          </div>
          <div class="p-6">
            <p class="text-muted-foreground">
              {{ t('skill.deleteConfirm', { name: skillToDelete?.name }) }}
            </p>
          </div>
          <div class="px-6 py-4 border-t border-border flex justify-end gap-2">
            <button
              @click="showDeleteConfirm = false"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="deleteSkill"
              class="px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white text-sm font-medium transition-all"
            >
              {{ t('common.delete') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>

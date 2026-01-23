<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()

// 在默认浏览器中打开 URL
async function openUrl(url: string) {
  try {
    await open(url)
  } catch (e) {
    console.error('打开 URL 失败:', e)
  }
}

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

interface SkillsRepository {
  id: string
  name: string
  url: string
  index_url: string
  builtin: boolean
  enabled: boolean
}

// 状态
const installedSkills = ref<InstalledSkill[]>([])
const recommendedSkills = ref<RecommendedSkill[]>([])
const loading = ref(false)
const selectedSkill = ref<InstalledSkill | null>(null)
const skillContent = ref('')

// 推荐 skills 弹窗
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

// 仓库管理
const showRepoModal = ref(false)
const repos = ref<SkillsRepository[]>([])
const newRepoUrl = ref('')
const addingRepo = ref(false)

// 发现技能（从远程仓库）
const showDiscoverModal = ref(false)
const discoveringSkills = ref(false)
const discoveredSkills = ref<RecommendedSkill[]>([])
const discoverSearchQuery = ref('')
const selectedDiscovered = ref<Set<string>>(new Set())

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

// 位置标签（使用 computed 以支持 i18n 动态切换）
// 添加 Record<string, string> 类型注解，以支持 v-for 遍历时的字符串索引访问
const locationLabels = computed((): Record<string, string> => ({
  'GlobalOpenCode': t('skills.locationLabels.GlobalOpenCode'),
  'GlobalClaude': t('skills.locationLabels.GlobalClaude'),
  'ProjectOpenCode': t('skills.locationLabels.ProjectOpenCode'),
  'ProjectClaude': t('skills.locationLabels.ProjectClaude'),
}))

// 安装位置选项（使用 computed 以支持 i18n）
const installLocationOptions = computed(() => [
  { key: 'global_opencode', label: t('skills.locations.globalOpencode') },
  { key: 'global_claude', label: t('skills.locations.globalClaude') },
  { key: 'project_opencode', label: t('skills.locations.projectOpencode') },
  { key: 'project_claude', label: t('skills.locations.projectClaude') },
])

// 加载已安装的 skills
async function loadInstalledSkills() {
  loading.value = true
  try {
    installedSkills.value = await invoke<InstalledSkill[]>('get_installed_skills')
  } catch (e) {
    console.error('加载 skills 失败:', e)
  } finally {
    loading.value = false
  }
}

// 加载推荐 skills
async function loadRecommendedSkills() {
  try {
    recommendedSkills.value = await invoke<RecommendedSkill[]>('get_recommended_skills')
  } catch (e) {
    console.error('加载推荐 skills 失败:', e)
  }
}

// 打开推荐弹窗
async function openRecommendedModal() {
  await loadRecommendedSkills()
  
  // 默认不选中任何技能，让用户自己选择
  selectedRecommended.value = new Set()
  
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
function isInstalled(skillsId: string): boolean {
  return installedSkills.value.some(s => s.name === skillsId)
}

// 将 raw URL 转换为 GitHub 网页 URL
function getGithubViewUrl(rawUrl: string): string {
  // 从 https://raw.githubusercontent.com/owner/repo/branch/path 
  // 转换为 https://github.com/owner/repo/blob/branch/path
  // 例如: https://raw.githubusercontent.com/anthropics/skills/main/skills/algorithmic-art/SKILL.md
  //    -> https://github.com/anthropics/skills/blob/main/skills/algorithmic-art/SKILL.md
  
  const match = rawUrl.match(/raw\.githubusercontent\.com\/([^/]+)\/([^/]+)\/([^/]+)\/(.+)/)
  if (match) {
    const [, owner, repo, branch, path] = match
    return `https://github.com/${owner}/${repo}/blob/${branch}/${path}`
  }
  // 如果无法解析，返回原 URL
  return rawUrl
}

// 安装选中的 skills
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
        const result = await invoke<InstallSkillResult>('install_skills', {
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
      showMessage(`成功安装 ${successCount} 个 skills`, 'success')
    }
    if (failCount > 0) {
      showMessage(`${failCount} 个 skills 安装失败`, 'error')
    }
  } finally {
    installing.value = false
  }
}

// 查看 Skill 内容
async function viewSkillContent(skill: InstalledSkill) {
  selectedSkill.value = skill
  try {
    skillContent.value = await invoke<string>('read_skills_content', { skills_path: skill.path })
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
async function deleteskills() {
  if (!skillToDelete.value) return
  
  try {
    await invoke('delete_skills', { skills_path: skillToDelete.value.path })
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

// 按仓库分组的发现技能（支持搜索过滤）
const groupedDiscovered = computed(() => {
  const groups: Record<string, RecommendedSkill[]> = {}
  const query = discoverSearchQuery.value.toLowerCase().trim()
  
  for (const skill of discoveredSkills.value) {
    // 搜索过滤：匹配名称或描述
    if (query && !skill.name.toLowerCase().includes(query) && !skill.description.toLowerCase().includes(query)) {
      continue
    }
    
    if (!groups[skill.repo]) {
      groups[skill.repo] = []
    }
    groups[skill.repo].push(skill)
  }
  return groups
})

// 过滤后的技能总数
const filteredDiscoveredCount = computed(() => {
  return Object.values(groupedDiscovered.value).reduce((sum, skills) => sum + skills.length, 0)
})

// ==================== 仓库管理 ====================

// 加载仓库列表
async function loadRepos() {
  try {
    repos.value = await invoke<SkillsRepository[]>('get_skills_repos')
  } catch (e) {
    console.error('加载仓库列表失败:', e)
  }
}

// 打开仓库管理弹窗
async function openRepoModal() {
  await loadRepos()
  showRepoModal.value = true
}

// 添加仓库
async function addRepo() {
  if (!newRepoUrl.value.trim()) return
  
  addingRepo.value = true
  try {
    // 从 URL 提取仓库信息
    const url = newRepoUrl.value.trim()
    const parts = url.replace(/\/$/, '').split('/')
    const repoName = parts[parts.length - 1] || 'custom-repo'
    const owner = parts[parts.length - 2] || 'unknown'
    
    const repo: SkillsRepository = {
      id: `custom-${Date.now()}`,
      name: `${owner}/${repoName}`,
      url: url,
      index_url: url.replace('github.com', 'raw.githubusercontent.com') + '/main/index.json',
      builtin: false,
      enabled: true
    }
    
    repos.value = await invoke<SkillsRepository[]>('add_skills_repo', { repo })
    newRepoUrl.value = ''
    showMessage('仓库添加成功', 'success')
  } catch (e) {
    showMessage(`添加失败: ${e}`, 'error')
  } finally {
    addingRepo.value = false
  }
}

// 删除仓库
async function deleteRepo(repoId: string) {
  try {
    repos.value = await invoke<SkillsRepository[]>('delete_skills_repo', { repoId })
    showMessage('仓库已删除', 'success')
  } catch (e) {
    showMessage(`删除失败: ${e}`, 'error')
  }
}

// 切换仓库启用状态
async function toggleRepo(repoId: string) {
  try {
    repos.value = await invoke<SkillsRepository[]>('toggle_skills_repo', { repoId })
  } catch (e) {
    showMessage(`操作失败: ${e}`, 'error')
  }
}

// ==================== 发现技能 ====================

// 打开发现技能弹窗
async function openDiscoverModal() {
  showDiscoverModal.value = true
  discoverSearchQuery.value = '' // 重置搜索
  await discoverSkills()
}

// 发现技能
async function discoverSkills() {
  discoveringSkills.value = true
  discoveredSkills.value = []
  
  try {
    discoveredSkills.value = await invoke<RecommendedSkill[]>('discover_skills')
    
    // 默认不选中任何技能，让用户自己选择
    selectedDiscovered.value = new Set()
  } catch (e) {
    console.error('发现技能失败:', e)
    showMessage('获取技能列表失败', 'error')
  } finally {
    discoveringSkills.value = false
  }
}

// 切换发现的技能选中状态
function toggleDiscoveredSelect(id: string) {
  if (selectedDiscovered.value.has(id)) {
    selectedDiscovered.value.delete(id)
  } else {
    selectedDiscovered.value.add(id)
  }
  selectedDiscovered.value = new Set(selectedDiscovered.value)
}

// 安装发现的技能
async function installDiscoveredSkills() {
  if (selectedDiscovered.value.size === 0) return
  
  installing.value = true
  let successCount = 0
  let failCount = 0
  
  try {
    for (const skillId of selectedDiscovered.value) {
      const skill = discoveredSkills.value.find(s => s.id === skillId)
      if (!skill) continue
      
      try {
        const result = await invoke<InstallSkillResult>('install_skills', {
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
    
    showDiscoverModal.value = false
    await loadInstalledSkills()
    
    if (successCount > 0) {
      showMessage(`成功安装 ${successCount} 个 skills`, 'success')
    }
    if (failCount > 0) {
      showMessage(`${failCount} 个 skills 安装失败`, 'error')
    }
  } finally {
    installing.value = false
  }
}

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
          {{ t('skills.addRecommended') }}
        </button>
        
        <!-- 发现新技能按钮 -->
        <button
          @click="openDiscoverModal"
          class="px-4 py-2 rounded-lg bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 font-medium text-sm transition-all flex items-center gap-2"
        >
          <SvgIcon name="search" :size="16" />
          {{ t('skills.discover') }}
        </button>
        
        <!-- 仓库管理按钮 -->
        <button
          @click="openRepoModal"
          class="px-4 py-2 rounded-lg bg-purple-500/20 hover:bg-purple-500/30 text-purple-400 font-medium text-sm transition-all flex items-center gap-2"
        >
          <SvgIcon name="repository" :size="16" />
          {{ t('skills.manageRepos') }}
        </button>
        
        <button
          @click="loadInstalledSkills"
          class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground font-medium text-sm transition-all flex items-center gap-2"
        >
          <SvgIcon name="refresh" :size="16" />
          {{ t('skills.refresh') }}
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
        <p class="text-muted-foreground mb-4">{{ t('skills.noSkills') }}</p>
        <button
          @click="openRecommendedModal"
          class="px-4 py-2 rounded-lg bg-accent hover:bg-accent/90 text-white font-medium text-sm transition-all"
        >
          {{ t('skills.installFirst') }}
        </button>
      </div>
      
      <!-- 按位置分组显示 -->
      <div v-else class="space-y-6">
        <template v-for="(locationSkills, location) in groupedSkills" :key="location">
          <div v-if="locationSkills.length > 0" class="rounded-xl bg-surface/30 border border-border overflow-hidden">
            <div class="px-4 py-3 border-b border-border bg-surface/50">
              <h3 class="font-semibold text-sm flex items-center gap-2">
                <SvgIcon :name="location.includes('Claude') ? 'robot' : 'box'" :size="16" class="text-accent" />
                {{ locationLabels[location] }}
                <span class="text-xs text-muted-foreground">({{ locationSkills.length }})</span>
              </h3>
            </div>
            
            <div class="p-3 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              <div
                v-for="skill in locationSkills"
                :key="skill.path"
                class="p-4 rounded-lg border border-border bg-background hover:border-accent/50 transition-all group"
              >
                <div class="flex items-start justify-between mb-2">
                  <h4 class="font-medium">{{ skill.name }}</h4>
                  <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      @click="viewSkillContent(skill)"
                      class="p-1.5 rounded hover:bg-surface text-muted-foreground hover:text-foreground transition-colors"
                      :title="t('skills.view')"
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
    
    <!-- 推荐 skills 弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showRecommendedModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showRecommendedModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[640px] max-h-[80vh] overflow-hidden shadow-2xl">
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold">{{ t('skills.recommended') }}</h2>
            <button 
              @click="showRecommendedModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 安装位置选择 -->
          <div class="px-6 py-3 border-b border-border bg-surface/30">
            <label class="text-sm font-medium mb-2 block">{{ t('skills.installLocation') }}</label>
            <div class="flex flex-wrap gap-2">
              <label 
                v-for="option in installLocationOptions" 
                :key="option.key"
                class="flex items-center gap-2 px-3 py-1.5 rounded-lg cursor-pointer transition-all text-sm"
                :class="installLocation === option.key ? 'bg-accent/20 text-accent' : 'bg-surface hover:bg-surface-hover'"
              >
                <input
                  type="radio"
                  :value="option.key"
                  v-model="installLocation"
                  class="hidden"
                />
                {{ option.label }}
              </label>
            </div>
          </div>
          
          <!-- skills 列表 -->
          <div class="p-4 space-y-4 max-h-[45vh] overflow-auto">
            <div v-for="(repoSkills, repo) in groupedRecommended" :key="repo" class="space-y-2">
              <h3 class="text-sm font-semibold text-muted-foreground flex items-center gap-2">
                <SvgIcon name="book" :size="14" /> {{ repo }}
                <button 
                  @click.stop="openUrl(repoSkills[0]?.repo_url)"
                  class="text-xs text-accent hover:underline cursor-pointer"
                >
                  GitHub ↗
                </button>
              </h3>
              
              <div class="grid gap-2">
                <div
                  v-for="skill in repoSkills"
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
                          {{ t('skills.installed') }}
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
              {{ t('skills.selected', { count: selectedRecommended.size }) }}
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
                {{ installing ? t('skills.installing') : t('skills.installAll') }}
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
              {{ t('skills.deleteConfirm', { name: skillToDelete?.name }) }}
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
              @click="deleteskills"
              class="px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white text-sm font-medium transition-all"
            >
              {{ t('common.delete') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 发现技能弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showDiscoverModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showDiscoverModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[720px] max-h-[85vh] overflow-hidden shadow-2xl">
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold flex items-center gap-2">
              <SvgIcon name="search" :size="20" class="text-emerald-400" />
              {{ t('skills.discoverTitle') }}
              <span class="text-sm font-normal text-muted-foreground">
                ({{ filteredDiscoveredCount }}/{{ discoveredSkills.length }})
              </span>
            </h2>
            <button 
              @click="showDiscoverModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 搜索栏 -->
          <div class="px-6 py-3 border-b border-border bg-surface/30">
            <div class="relative">
              <SvgIcon name="search" :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
              <input
                v-model="discoverSearchQuery"
                type="text"
                :placeholder="t('skills.searchPlaceholder')"
                class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-background border border-border text-sm focus:outline-none focus:border-accent transition-colors"
              />
              <button
                v-if="discoverSearchQuery"
                @click="discoverSearchQuery = ''"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                <SvgIcon name="close" :size="14" />
              </button>
            </div>
          </div>
          
          <!-- 安装位置选择 -->
          <div class="px-6 py-3 border-b border-border bg-surface/20">
            <label class="text-sm font-medium mb-2 block">{{ t('skills.installLocation') }}</label>
            <div class="flex flex-wrap gap-2">
              <label 
                v-for="option in installLocationOptions" 
                :key="option.key"
                class="flex items-center gap-2 px-3 py-1.5 rounded-lg cursor-pointer transition-all text-sm"
                :class="installLocation === option.key ? 'bg-accent/20 text-accent' : 'bg-surface hover:bg-surface-hover'"
              >
                <input type="radio" :value="option.key" v-model="installLocation" class="hidden" />
                {{ option.label }}
              </label>
            </div>
          </div>
          
          <!-- 技能列表 -->
          <div class="p-4 space-y-4 max-h-[50vh] overflow-auto">
            <div v-if="discoveringSkills" class="py-12 text-center">
              <div class="inline-block animate-spin rounded-full h-8 w-8 border-2 border-accent border-t-transparent mb-3"></div>
              <p class="text-muted-foreground">{{ t('skills.discovering') }}</p>
            </div>
            
            <div v-else-if="discoveredSkills.length === 0" class="py-12 text-center text-muted-foreground">
              {{ t('skills.noSkillsFound') }}
            </div>
            
            <div v-else v-for="(repoSkills, repo) in groupedDiscovered" :key="repo" class="space-y-3">
              <h3 class="text-sm font-semibold text-muted-foreground flex items-center gap-2 sticky top-0 bg-background/95 backdrop-blur-sm py-2 -mx-1 px-1">
                <SvgIcon name="book" :size="14" /> {{ repo }}
                <span class="text-xs font-normal">({{ repoSkills.length }})</span>
                <button 
                  @click.stop="openUrl(repoSkills[0]?.repo_url)"
                  class="ml-auto text-xs text-accent hover:underline flex items-center gap-1 cursor-pointer"
                >
                  <SvgIcon name="link" :size="12" />
                  GitHub
                </button>
              </h3>
              
              <div class="grid gap-3">
                <div
                  v-for="skill in repoSkills"
                  :key="skill.id"
                  @click="!isInstalled(skill.id) && toggleDiscoveredSelect(skill.id)"
                  class="p-4 rounded-xl border transition-all"
                  :class="[
                    isInstalled(skill.id)
                      ? 'border-emerald-500/30 bg-emerald-500/5 cursor-default'
                      : selectedDiscovered.has(skill.id)
                        ? 'border-accent bg-accent/10 cursor-pointer'
                        : 'border-border hover:border-accent/50 bg-surface/30 cursor-pointer'
                  ]"
                >
                  <div class="flex items-start gap-3">
                    <div 
                      v-if="!isInstalled(skill.id)"
                      class="w-5 h-5 rounded-md border-2 flex items-center justify-center flex-shrink-0 mt-0.5 transition-all"
                      :class="selectedDiscovered.has(skill.id) ? 'border-accent bg-accent text-white' : 'border-muted-foreground'"
                    >
                      <span v-if="selectedDiscovered.has(skill.id)" class="text-xs">✓</span>
                    </div>
                    <div v-else class="w-5 h-5 rounded-md bg-emerald-500/20 flex items-center justify-center flex-shrink-0 mt-0.5">
                      <span class="text-xs text-emerald-500">✓</span>
                    </div>
                    
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="font-semibold">{{ skill.name }}</span>
                        <span v-if="isInstalled(skill.id)" class="text-xs px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-500">
                          {{ t('skills.installed') }}
                        </span>
                      </div>
                      <!-- GitHub 仓库路径 -->
                      <button 
                        @click.stop="openUrl(skill.repo_url)"
                        class="text-xs text-muted-foreground hover:text-accent flex items-center gap-1 mt-1 cursor-pointer"
                      >
                        <SvgIcon name="link" :size="10" />
                        {{ skill.description }}
                      </button>
                      <!-- 操作按钮 -->
                      <div class="flex items-center gap-3 mt-2">
                        <button 
                          @click.stop="openUrl(getGithubViewUrl(skill.raw_url))"
                          class="text-xs text-accent/70 hover:text-accent hover:underline flex items-center gap-1 cursor-pointer"
                          title="在 GitHub 上查看"
                        >
                          <SvgIcon name="eye" :size="12" />
                          {{ t('skills.viewSource') }}
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 底部 -->
          <div class="px-6 py-4 border-t border-border flex items-center justify-between">
            <span class="text-sm text-muted-foreground">
              {{ t('skills.selected', { count: selectedDiscovered.size }) }}
            </span>
            <div class="flex gap-2">
              <button
                @click="discoverSkills"
                :disabled="discoveringSkills"
                class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all flex items-center gap-2"
              >
                <SvgIcon name="refresh" :size="14" />
                {{ t('skills.refresh') }}
              </button>
              <button
                @click="installDiscoveredSkills"
                :disabled="selectedDiscovered.size === 0 || installing"
                class="px-4 py-2 rounded-lg bg-emerald-500 hover:bg-emerald-600 text-white text-sm font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ installing ? t('skills.installing') : t('skills.installSelected') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- 仓库管理弹窗 -->
    <Teleport to="body">
      <div 
        v-if="showRepoModal" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showRepoModal = false"
      >
        <div class="bg-background border border-border rounded-2xl w-[560px] max-h-[80vh] overflow-hidden shadow-2xl">
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-border flex items-center justify-between">
            <h2 class="text-lg font-semibold flex items-center gap-2">
              <SvgIcon name="repository" :size="20" class="text-purple-400" />
              {{ t('skills.repoManagement') }}
            </h2>
            <button 
              @click="showRepoModal = false"
              class="text-muted-foreground hover:text-foreground transition-colors"
            >
              <SvgIcon name="close" :size="16" />
            </button>
          </div>
          
          <!-- 添加新仓库 -->
          <div class="px-6 py-4 border-b border-border bg-surface/30">
            <label class="text-sm font-medium mb-2 block">{{ t('skills.addRepo') }}</label>
            <div class="flex gap-2">
              <input
                v-model="newRepoUrl"
                type="text"
                :placeholder="t('skills.repoUrlPlaceholder')"
                class="flex-1 px-3 py-2 rounded-lg bg-background border border-border text-sm focus:outline-none focus:border-accent"
              />
              <button
                @click="addRepo"
                :disabled="!newRepoUrl.trim() || addingRepo"
                class="px-4 py-2 rounded-lg bg-purple-500 hover:bg-purple-600 text-white text-sm font-medium transition-all disabled:opacity-50"
              >
                {{ addingRepo ? '...' : t('common.add') }}
              </button>
            </div>
            <p class="text-xs text-muted-foreground mt-2">{{ t('skills.repoUrlHint') }}</p>
          </div>
          
          <!-- 仓库列表 -->
          <div class="p-4 space-y-3 max-h-[45vh] overflow-auto">
            <div
              v-for="repo in repos"
              :key="repo.id"
              class="p-4 rounded-xl border border-border bg-surface/30 hover:border-accent/30 transition-all"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-semibold">{{ repo.name }}</span>
                    <span v-if="repo.builtin" class="text-xs px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400">
                      {{ t('skills.builtin') }}
                    </span>
                    <span 
                      class="text-xs px-1.5 py-0.5 rounded"
                      :class="repo.enabled ? 'bg-emerald-500/20 text-emerald-400' : 'bg-gray-500/20 text-gray-400'"
                    >
                      {{ repo.enabled ? t('skills.enabled') : t('skills.disabled') }}
                    </span>
                  </div>
                  <button @click="openUrl(repo.url)" class="text-xs text-muted-foreground hover:text-accent mt-1 block truncate text-left cursor-pointer">
                    {{ repo.url }}
                  </button>
                </div>
                
                <div class="flex items-center gap-2 ml-4">
                  <!-- 启用/禁用切换 -->
                  <button
                    @click="toggleRepo(repo.id)"
                    class="p-2 rounded-lg hover:bg-surface transition-colors"
                    :title="repo.enabled ? t('skills.disable') : t('skills.enable')"
                  >
                    <SvgIcon :name="repo.enabled ? 'eye' : 'eye-off'" :size="16" class="text-muted-foreground" />
                  </button>
                  <!-- 删除按钮（非内置仓库） -->
                  <button
                    v-if="!repo.builtin"
                    @click="deleteRepo(repo.id)"
                    class="p-2 rounded-lg hover:bg-red-500/10 transition-colors"
                    :title="t('common.delete')"
                  >
                    <SvgIcon name="trash" :size="16" class="text-red-400" />
                  </button>
                </div>
              </div>
            </div>
            
            <div v-if="repos.length === 0" class="py-8 text-center text-muted-foreground">
              {{ t('skills.noRepos') }}
            </div>
          </div>
          
          <!-- 底部 -->
          <div class="px-6 py-4 border-t border-border flex justify-end">
            <button
              @click="showRepoModal = false"
              class="px-4 py-2 rounded-lg bg-surface hover:bg-surface-hover text-foreground text-sm transition-all"
            >
              {{ t('common.confirm') }}
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
  /* 添加标准属性以兼容现代浏览器 */
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>

/**
 * Claude 模型预设配置
 * 支持 Anthropic 协议和 OpenAI 协议
 */

// API 协议类型
export type ApiProtocol = 'anthropic' | 'openai'

// Claude 模型定义
export interface ClaudeModel {
  id: string           // 模型 ID
  name: string         // 显示名称
}

// 预设供应商定义
export interface ProviderPreset {
  name: string                // 供应商名称
  websiteUrl: string          // 官网地址
  apiKeyUrl?: string          // 获取 API Key 的地址
  baseUrl: string             // 默认 API 地址
  defaultProtocol: ApiProtocol // 默认协议
  supportedProtocols: ApiProtocol[] // 支持的协议
  models: ClaudeModel[]       // 预设模型列表
  description?: string        // 描述
  category: 'official' | 'cn_official' | 'third_party' | 'aggregator' | 'custom'
}

// Claude 模型列表
export const CLAUDE_MODELS: ClaudeModel[] = [
  { id: 'claude-4-sonnet', name: 'Claude 4 Sonnet' },
  { id: 'claude-4.1-opus', name: 'Claude 4.1 Opus' },
  { id: 'claude-4.5-haiku', name: 'Claude 4.5 Haiku' },
  { id: 'claude-4.5-opus', name: 'Claude 4.5 Opus' },
  { id: 'claude-4.5-sonnet', name: 'Claude 4.5 Sonnet' },
]

// Codex (GPT) 模型列表
export const CODEX_MODELS: ClaudeModel[] = [
  { id: 'gpt-5.2-codex', name: 'GPT-5.2 Codex' },
  { id: 'gpt-5.2', name: 'GPT-5.2' },
  { id: 'gpt-5.1-codex-max', name: 'GPT-5.1 Codex Max' },
  { id: 'gpt-5.1-codex-mini', name: 'GPT-5.1 Codex Mini' },
  { id: 'gpt-5.1', name: 'GPT-5.1' },
]

// Gemini 模型列表
export const GEMINI_MODELS: ClaudeModel[] = [
  { id: 'gemini-3-pro', name: 'Gemini 3 Pro' },
  { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro' },
]

// 根据模型类型获取模型列表
export function getModelsByType(modelType: string): ClaudeModel[] {
  switch (modelType) {
    case 'claude':
      return CLAUDE_MODELS
    case 'codex':
      return CODEX_MODELS
    case 'gemini':
      return GEMINI_MODELS
    default:
      return CLAUDE_MODELS
  }
}

// 预设供应商列表
export const PROVIDER_PRESETS: ProviderPreset[] = [
  // i7 Relay 聚合
  {
    name: 'i7 Relay',
    websiteUrl: 'https://i7dc.com',
    apiKeyUrl: 'https://docs.qq.com/doc/DZWhNb0dvaUdJVWpo',
    baseUrl: 'https://i7dc.com/api',
    defaultProtocol: 'anthropic',
    supportedProtocols: ['anthropic', 'openai'],
    models: CLAUDE_MODELS,
    description: 'i7 Relay 聚合 API',
    category: 'aggregator',
  },
  // 智谱 AI
  {
    name: '智谱 AI',
    websiteUrl: 'https://open.bigmodel.cn',
    apiKeyUrl: 'https://www.bigmodel.cn/glm-coding?ic=LOLVYRGC8E',
    baseUrl: 'https://open.bigmodel.cn/api/coding/paas/v4',
    defaultProtocol: 'openai',
    supportedProtocols: ['openai', 'anthropic'],
    models: [
      { id: 'glm-4.7', name: 'GLM-4.7' },
      { id: 'glm-4.6', name: 'GLM-4.6' },
    ],
    description: '智谱 GLM 编程专用 API',
    category: 'cn_official',
  },
  // 自定义
  {
    name: '自定义',
    websiteUrl: '',
    baseUrl: '',
    defaultProtocol: 'anthropic',
    supportedProtocols: ['anthropic', 'openai'],
    models: CLAUDE_MODELS,
    description: '自定义 API 端点',
    category: 'custom',
  },
]

// 根据 URL 匹配预设
export function matchPresetByUrl(url: string): ProviderPreset | undefined {
  if (!url) return undefined
  const lowerUrl = url.toLowerCase()
  
  return PROVIDER_PRESETS.find(preset => {
    if (!preset.baseUrl) return false
    return lowerUrl.includes(new URL(preset.baseUrl).hostname)
  })
}

// 根据名称获取预设
export function getPresetByName(name: string): ProviderPreset | undefined {
  return PROVIDER_PRESETS.find(preset => preset.name === name)
}

// 获取协议显示名称
export function getProtocolDisplayName(protocol: ApiProtocol): string {
  return protocol === 'anthropic' ? 'Anthropic 协议' : 'OpenAI 协议'
}

// 获取分类显示名称
export function getCategoryDisplayName(category: ProviderPreset['category']): string {
  const map: Record<ProviderPreset['category'], string> = {
    official: '官方',
    cn_official: '国内官方',
    third_party: '第三方中转',
    aggregator: '聚合平台',
    custom: '自定义',
  }
  return map[category]
}

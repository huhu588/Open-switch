// 模型厂家定义
export type ModelType = 'claude' | 'codex' | 'gemini'

export interface ModelTypeOption {
  id: ModelType
  name: string
  icon: string  // SVG 图标名称
  color: string
}

export const MODEL_TYPES: ModelTypeOption[] = [
  { id: 'claude', name: 'Claude', icon: 'claude', color: '#D97757' },
  { id: 'codex', name: 'Codex', icon: 'openai', color: '#10A37F' },
  { id: 'gemini', name: 'Gemini', icon: 'gemini', color: '#3186FF' },
]

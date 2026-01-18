// 模型厂家定义
export type ModelType = 'claude' | 'codex' | 'gemini'

export interface ModelTypeOption {
  id: ModelType
  name: string
  icon: string
  color: string
}

export const MODEL_TYPES: ModelTypeOption[] = [
  { id: 'claude', name: 'Claude', icon: '✳️', color: '#FF6B35' },
  { id: 'codex', name: 'Codex', icon: '⚙️', color: '#10A37F' },
  { id: 'gemini', name: 'Gemini', icon: '✦', color: '#4285F4' },
]

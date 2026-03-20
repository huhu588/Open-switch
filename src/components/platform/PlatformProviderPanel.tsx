import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, Zap, Loader2, Globe, Upload, Download, Search, Pencil, Package, Layers, ExternalLink, Star, ArrowLeft, Save, ChevronDown } from 'lucide-react';
import { useProviderStore } from '../../stores/useProviderStore';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../Toast';

export type ProviderModelType = 'claude' | 'codex' | 'gemini' | 'opencode' | 'openclaw';

type ProviderCategory = 'official' | 'cn_official' | 'aggregator' | 'third_party' | 'cloud_provider';
type ApiFormat = 'anthropic' | 'openai_chat' | 'openai_responses';
type AuthField = 'ANTHROPIC_AUTH_TOKEN' | 'ANTHROPIC_API_KEY';
type PanelView = 'list' | 'add' | 'edit';

interface ProviderPreset {
  name: string;
  base_url: string;
  description?: string;
  color?: string;
  category?: ProviderCategory;
  websiteUrl?: string;
  isOfficial?: boolean;
}

interface ModelConfig {
  mainModel: string;
  thinkingModel: string;
  haikuModel: string;
  sonnetModel: string;
  opusModel: string;
}

const EMPTY_MODEL_CONFIG: ModelConfig = { mainModel: '', thinkingModel: '', haikuModel: '', sonnetModel: '', opusModel: '' };

const CATEGORY_LABELS: Record<ProviderCategory, string> = {
  official: '官方服务',
  cn_official: '国内官方',
  aggregator: '聚合平台',
  third_party: '第三方中继',
  cloud_provider: '云服务商',
};

const CATEGORY_ORDER: ProviderCategory[] = ['official', 'cn_official', 'aggregator', 'third_party', 'cloud_provider'];

const API_FORMAT_OPTIONS: { value: ApiFormat; label: string; desc: string }[] = [
  { value: 'anthropic', label: 'Anthropic Messages (原生)', desc: '直接透传 Anthropic API 格式' },
  { value: 'openai_chat', label: 'OpenAI Chat Completions', desc: '兼容 OpenAI Chat API 格式' },
  { value: 'openai_responses', label: 'OpenAI Responses API', desc: '兼容 OpenAI Responses API 格式' },
];

const AUTH_FIELD_OPTIONS: { value: AuthField; label: string }[] = [
  { value: 'ANTHROPIC_AUTH_TOKEN', label: 'ANTHROPIC_AUTH_TOKEN（默认）' },
  { value: 'ANTHROPIC_API_KEY', label: 'ANTHROPIC_API_KEY' },
];

const PROVIDER_PRESETS: Record<ProviderModelType, ProviderPreset[]> = {
  claude: [
    { name: 'Anthropic Official', base_url: 'https://api.anthropic.com', description: 'Anthropic 官方 API', color: '#D97757', category: 'official', websiteUrl: 'https://www.anthropic.com/claude-code', isOfficial: true },
    { name: 'DeepSeek', base_url: 'https://api.deepseek.com/anthropic', description: 'DeepSeek 官方', color: '#1E88E5', category: 'cn_official', websiteUrl: 'https://platform.deepseek.com' },
    { name: 'Zhipu GLM', base_url: 'https://open.bigmodel.cn/api/anthropic', description: '智谱 GLM 官方', color: '#0F62FE', category: 'cn_official', websiteUrl: 'https://open.bigmodel.cn' },
    { name: 'Bailian', base_url: 'https://dashscope.aliyuncs.com/apps/anthropic', description: '阿里百炼', color: '#624AFF', category: 'cn_official', websiteUrl: 'https://bailian.console.aliyun.com' },
    { name: 'Bailian For Coding', base_url: 'https://coding.dashscope.aliyuncs.com/apps/anthropic', description: '阿里百炼编程版', color: '#624AFF', category: 'cn_official', websiteUrl: 'https://bailian.console.aliyun.com' },
    { name: 'Kimi', base_url: 'https://api.moonshot.cn/anthropic', description: '月之暗面 Kimi', color: '#6366F1', category: 'cn_official', websiteUrl: 'https://platform.moonshot.cn/console' },
    { name: 'Kimi For Coding', base_url: 'https://api.kimi.com/coding/', description: 'Kimi 编程版', color: '#6366F1', category: 'cn_official', websiteUrl: 'https://www.kimi.com/coding/docs/' },
    { name: 'StepFun', base_url: 'https://api.stepfun.ai/v1', description: '阶跃星辰', color: '#005AFF', category: 'cn_official', websiteUrl: 'https://platform.stepfun.ai' },
    { name: 'KAT-Coder', base_url: 'https://vanchin.streamlake.ai/api/gateway/v1/endpoints/', description: 'KAT-Coder 编程助手', color: '#FF6B35', category: 'cn_official', websiteUrl: 'https://console.streamlake.ai' },
    { name: 'Longcat', base_url: 'https://api.longcat.chat/anthropic', description: 'Longcat 长猫', color: '#29E154', category: 'cn_official', websiteUrl: 'https://longcat.chat/platform' },
    { name: 'MiniMax', base_url: 'https://api.minimaxi.com/anthropic', description: 'MiniMax 国内版', color: '#FF6B6B', category: 'cn_official', websiteUrl: 'https://platform.minimaxi.com' },
    { name: 'MiniMax en', base_url: 'https://api.minimax.io/anthropic', description: 'MiniMax 国际版', color: '#FF6B6B', category: 'cn_official', websiteUrl: 'https://platform.minimax.io' },
    { name: 'DouBaoSeed', base_url: 'https://ark.cn-beijing.volces.com/api/coding', description: '字节豆包', color: '#3370FF', category: 'cn_official', websiteUrl: 'https://www.volcengine.com/product/doubao' },
    { name: 'BaiLing', base_url: 'https://api.tbox.cn/api/anthropic', description: '支付宝百灵', color: '#1677FF', category: 'cn_official', websiteUrl: 'https://alipaytbox.yuque.com/sxs0ba/ling/get_started' },
    { name: 'Xiaomi MiMo', base_url: 'https://api.xiaomimimo.com/anthropic', description: '小米 MiMo', color: '#FF6900', category: 'cn_official', websiteUrl: 'https://platform.xiaomimimo.com' },
    { name: 'ModelScope', base_url: 'https://api-inference.modelscope.cn', description: '魔搭社区', color: '#624AFF', category: 'aggregator', websiteUrl: 'https://modelscope.cn' },
    { name: 'AiHubMix', base_url: 'https://aihubmix.com', description: 'AiHubMix 聚合', color: '#006FFB', category: 'aggregator', websiteUrl: 'https://aihubmix.com' },
    { name: 'SiliconFlow', base_url: 'https://api.siliconflow.cn', description: 'SiliconFlow 国内版', color: '#6E29F6', category: 'aggregator', websiteUrl: 'https://siliconflow.cn' },
    { name: 'SiliconFlow en', base_url: 'https://api.siliconflow.com', description: 'SiliconFlow 国际版', color: '#6E29F6', category: 'aggregator', websiteUrl: 'https://siliconflow.com' },
    { name: 'DMXAPI', base_url: 'https://www.dmxapi.cn', description: 'DMXAPI 聚合服务', color: '#f97316', category: 'aggregator', websiteUrl: 'https://www.dmxapi.cn' },
    { name: 'OpenRouter', base_url: 'https://openrouter.ai/api', description: 'OpenRouter 多模型聚合', color: '#6566F1', category: 'aggregator', websiteUrl: 'https://openrouter.ai' },
    { name: 'Novita AI', base_url: 'https://api.novita.ai/anthropic', description: 'Novita AI 推理平台', color: '#00C853', category: 'aggregator', websiteUrl: 'https://novita.ai' },
    { name: 'Nvidia', base_url: 'https://integrate.api.nvidia.com', description: 'NVIDIA 推理平台', color: '#76b900', category: 'aggregator', websiteUrl: 'https://build.nvidia.com' },
    { name: 'Compshare', base_url: 'https://api.modelverse.cn', description: 'UCloud 算力共享', color: '#0064FF', category: 'aggregator', websiteUrl: 'https://www.compshare.cn' },
    { name: 'PackyCode', base_url: 'https://www.packyapi.com', description: 'PackyCode 中继', color: '#0ea5e9', category: 'third_party', websiteUrl: 'https://www.packyapi.com' },
    { name: 'Cubence', base_url: 'https://api.cubence.com', description: 'Cubence 中继', color: '#4B0082', category: 'third_party', websiteUrl: 'https://cubence.com' },
    { name: 'AIGoCode', base_url: 'https://api.aigocode.com', description: 'AIGoCode 中继', color: '#5B7FFF', category: 'third_party', websiteUrl: 'https://aigocode.com' },
    { name: 'RightCode', base_url: 'https://www.right.codes/claude', description: 'RightCode 中继', color: '#E96B2C', category: 'third_party', websiteUrl: 'https://www.right.codes' },
    { name: 'AICodeMirror', base_url: 'https://api.aicodemirror.com/api/claudecode', description: 'AICodeMirror 中继', color: '#4A90D9', category: 'third_party', websiteUrl: 'https://www.aicodemirror.com' },
    { name: 'AICoding', base_url: 'https://api.aicoding.sh', description: 'AICoding 中继', color: '#00BCD4', category: 'third_party', websiteUrl: 'https://aicoding.sh' },
    { name: 'CrazyRouter', base_url: 'https://crazyrouter.com', description: 'CrazyRouter 中继', color: '#FF4500', category: 'third_party', websiteUrl: 'https://www.crazyrouter.com' },
    { name: 'SSSAiCode', base_url: 'https://node-hk.sssaicode.com/api', description: 'SSSAiCode 中继', color: '#9C27B0', category: 'third_party', websiteUrl: 'https://www.sssaicode.com' },
    { name: 'Micu', base_url: 'https://www.openclaudecode.cn', description: 'Micu 中继', color: '#607D8B', category: 'third_party', websiteUrl: 'https://www.openclaudecode.cn' },
    { name: 'X-Code API', base_url: 'https://x-code.cc', description: 'X-Code API 中继', color: '#333333', category: 'third_party', websiteUrl: 'https://x-code.cc' },
    { name: 'CTok.ai', base_url: 'https://api.ctok.ai', description: 'CTok.ai 中继', color: '#FF5722', category: 'third_party', websiteUrl: 'https://ctok.ai' },
    { name: 'AWS Bedrock (AKSK)', base_url: 'https://bedrock-runtime.us-east-1.amazonaws.com', description: 'AWS Bedrock AKSK 认证', color: '#FF9900', category: 'cloud_provider', websiteUrl: 'https://aws.amazon.com/bedrock/' },
    { name: 'AWS Bedrock (API Key)', base_url: 'https://bedrock-runtime.us-east-1.amazonaws.com', description: 'AWS Bedrock API Key 认证', color: '#FF9900', category: 'cloud_provider', websiteUrl: 'https://aws.amazon.com/bedrock/' },
  ],
  codex: [
    { name: 'OpenAI Official', base_url: 'https://api.openai.com/v1', description: 'OpenAI 官方 API', color: '#10A37F', category: 'official', websiteUrl: 'https://chatgpt.com/codex', isOfficial: true },
    { name: 'Azure OpenAI', base_url: 'https://YOUR_RESOURCE.openai.azure.com/openai', description: 'Azure OpenAI', color: '#0078D4', category: 'official', websiteUrl: 'https://learn.microsoft.com/en-us/azure/ai-foundry/openai/how-to/codex' },
    { name: 'AiHubMix', base_url: 'https://aihubmix.com/v1', description: 'AiHubMix 聚合', color: '#006FFB', category: 'aggregator', websiteUrl: 'https://aihubmix.com' },
    { name: 'DMXAPI', base_url: 'https://www.dmxapi.cn/v1', description: 'DMXAPI 聚合服务', color: '#f97316', category: 'aggregator', websiteUrl: 'https://www.dmxapi.cn' },
    { name: 'OpenRouter', base_url: 'https://openrouter.ai/api/v1', description: 'OpenRouter 多模型聚合', color: '#6566F1', category: 'aggregator', websiteUrl: 'https://openrouter.ai' },
    { name: 'Compshare', base_url: 'https://api.modelverse.cn/v1', description: 'UCloud 算力共享', color: '#0064FF', category: 'aggregator', websiteUrl: 'https://www.compshare.cn' },
    { name: 'PackyCode', base_url: 'https://www.packyapi.com/v1', description: 'PackyCode 中继', color: '#0ea5e9', category: 'third_party', websiteUrl: 'https://www.packyapi.com' },
    { name: 'Cubence', base_url: 'https://api.cubence.com/v1', description: 'Cubence 中继', color: '#4B0082', category: 'third_party', websiteUrl: 'https://cubence.com' },
    { name: 'AIGoCode', base_url: 'https://api.aigocode.com', description: 'AIGoCode 中继', color: '#5B7FFF', category: 'third_party', websiteUrl: 'https://aigocode.com' },
    { name: 'RightCode', base_url: 'https://right.codes/codex/v1', description: 'RightCode 中继', color: '#E96B2C', category: 'third_party', websiteUrl: 'https://www.right.codes' },
    { name: 'AICodeMirror', base_url: 'https://api.aicodemirror.com/api/codex/backend-api/codex', description: 'AICodeMirror 中继', color: '#4A90D9', category: 'third_party', websiteUrl: 'https://www.aicodemirror.com' },
    { name: 'AICoding', base_url: 'https://api.aicoding.sh', description: 'AICoding 中继', color: '#00BCD4', category: 'third_party', websiteUrl: 'https://aicoding.sh' },
    { name: 'CrazyRouter', base_url: 'https://crazyrouter.com/v1', description: 'CrazyRouter 中继', color: '#FF4500', category: 'third_party', websiteUrl: 'https://www.crazyrouter.com' },
    { name: 'SSSAiCode', base_url: 'https://node-hk.sssaicode.com/api/v1', description: 'SSSAiCode 中继', color: '#9C27B0', category: 'third_party', websiteUrl: 'https://www.sssaicode.com' },
    { name: 'Micu', base_url: 'https://www.openclaudecode.cn/v1', description: 'Micu 中继', color: '#607D8B', category: 'third_party', websiteUrl: 'https://www.openclaudecode.cn' },
    { name: 'X-Code API', base_url: 'https://x-code.cc/v1', description: 'X-Code API 中继', color: '#333333', category: 'third_party', websiteUrl: 'https://x-code.cc' },
    { name: 'CTok.ai', base_url: 'https://api.ctok.ai/v1', description: 'CTok.ai 中继', color: '#FF5722', category: 'third_party', websiteUrl: 'https://ctok.ai' },
  ],
  gemini: [
    { name: 'Google AI Studio', base_url: 'https://generativelanguage.googleapis.com/v1beta', description: 'Google AI Studio', color: '#3186FF', category: 'official', websiteUrl: 'https://aistudio.google.com', isOfficial: true },
    { name: 'Vertex AI', base_url: 'https://us-central1-aiplatform.googleapis.com', description: 'Google Cloud Vertex AI', color: '#4285F4', category: 'cloud_provider', websiteUrl: 'https://cloud.google.com/vertex-ai' },
    { name: 'OpenRouter', base_url: 'https://openrouter.ai/api/v1', description: 'OpenRouter 多模型聚合', color: '#6566F1', category: 'aggregator', websiteUrl: 'https://openrouter.ai' },
    { name: 'AiHubMix', base_url: 'https://aihubmix.com/v1', description: 'AiHubMix 聚合', color: '#006FFB', category: 'aggregator', websiteUrl: 'https://aihubmix.com' },
    { name: 'SiliconFlow', base_url: 'https://api.siliconflow.cn/v1', description: 'SiliconFlow 高性能推理', color: '#6E29F6', category: 'aggregator', websiteUrl: 'https://siliconflow.cn' },
  ],
  opencode: [
    { name: 'OpenAI Compatible', base_url: 'https://api.openai.com/v1', description: 'OpenAI 兼容接口', color: '#10A37F', category: 'official', websiteUrl: 'https://platform.openai.com' },
    { name: 'Anthropic', base_url: 'https://api.anthropic.com', description: 'Anthropic Claude', color: '#D97757', category: 'official', websiteUrl: 'https://www.anthropic.com' },
    { name: 'Google AI Studio', base_url: 'https://generativelanguage.googleapis.com/v1beta', description: 'Google AI Studio', color: '#3186FF', category: 'official', websiteUrl: 'https://aistudio.google.com' },
    { name: 'DeepSeek', base_url: 'https://api.deepseek.com/v1', description: 'DeepSeek 官方', color: '#1E88E5', category: 'cn_official', websiteUrl: 'https://platform.deepseek.com' },
    { name: 'Zhipu GLM', base_url: 'https://open.bigmodel.cn/api/paas/v4', description: '智谱 GLM', color: '#0F62FE', category: 'cn_official', websiteUrl: 'https://open.bigmodel.cn' },
    { name: 'Kimi', base_url: 'https://api.moonshot.cn/v1', description: '月之暗面 Kimi', color: '#6366F1', category: 'cn_official', websiteUrl: 'https://platform.moonshot.cn' },
    { name: 'DouBao', base_url: 'https://ark.cn-beijing.volces.com/api/v3', description: '字节豆包', color: '#3370FF', category: 'cn_official', websiteUrl: 'https://www.volcengine.com/product/doubao' },
    { name: 'OpenRouter', base_url: 'https://openrouter.ai/api/v1', description: 'OpenRouter 多模型聚合', color: '#6566F1', category: 'aggregator', websiteUrl: 'https://openrouter.ai' },
    { name: 'SiliconFlow', base_url: 'https://api.siliconflow.cn/v1', description: 'SiliconFlow 高性能推理', color: '#6E29F6', category: 'aggregator', websiteUrl: 'https://siliconflow.cn' },
    { name: 'DMXAPI', base_url: 'https://api.dmxapi.cn/v1', description: 'DMXAPI 聚合服务', color: '#f97316', category: 'aggregator', websiteUrl: 'https://www.dmxapi.cn' },
    { name: 'AiHubMix', base_url: 'https://aihubmix.com/v1', description: 'AiHubMix 聚合', color: '#006FFB', category: 'aggregator', websiteUrl: 'https://aihubmix.com' },
    { name: 'Nvidia', base_url: 'https://integrate.api.nvidia.com/v1', description: 'NVIDIA 推理平台', color: '#76b900', category: 'aggregator', websiteUrl: 'https://build.nvidia.com' },
  ],
  openclaw: [
    { name: 'OpenAI Compatible', base_url: 'https://api.openai.com/v1', description: 'OpenAI 兼容接口', color: '#10A37F', category: 'official', websiteUrl: 'https://platform.openai.com' },
    { name: 'Anthropic', base_url: 'https://api.anthropic.com', description: 'Anthropic Claude', color: '#D97757', category: 'official', websiteUrl: 'https://www.anthropic.com' },
    { name: 'Google AI Studio', base_url: 'https://generativelanguage.googleapis.com/v1beta', description: 'Google AI Studio', color: '#3186FF', category: 'official', websiteUrl: 'https://aistudio.google.com' },
    { name: 'DeepSeek', base_url: 'https://api.deepseek.com/v1', description: 'DeepSeek 官方', color: '#1E88E5', category: 'cn_official', websiteUrl: 'https://platform.deepseek.com' },
    { name: 'Zhipu GLM', base_url: 'https://open.bigmodel.cn/api/paas/v4', description: '智谱 GLM', color: '#0F62FE', category: 'cn_official', websiteUrl: 'https://open.bigmodel.cn' },
    { name: 'Kimi', base_url: 'https://api.moonshot.cn/v1', description: '月之暗面 Kimi', color: '#6366F1', category: 'cn_official', websiteUrl: 'https://platform.moonshot.cn' },
    { name: 'OpenRouter', base_url: 'https://openrouter.ai/api/v1', description: 'OpenRouter 多模型聚合', color: '#6566F1', category: 'aggregator', websiteUrl: 'https://openrouter.ai' },
    { name: 'SiliconFlow', base_url: 'https://api.siliconflow.cn/v1', description: 'SiliconFlow 高性能推理', color: '#6E29F6', category: 'aggregator', websiteUrl: 'https://siliconflow.cn' },
    { name: 'DMXAPI', base_url: 'https://api.dmxapi.cn/v1', description: 'DMXAPI 聚合服务', color: '#f97316', category: 'aggregator', websiteUrl: 'https://www.dmxapi.cn' },
    { name: 'AiHubMix', base_url: 'https://aihubmix.com/v1', description: 'AiHubMix 聚合', color: '#006FFB', category: 'aggregator', websiteUrl: 'https://aihubmix.com' },
    { name: 'Nvidia', base_url: 'https://integrate.api.nvidia.com/v1', description: 'NVIDIA 推理平台', color: '#76b900', category: 'aggregator', websiteUrl: 'https://build.nvidia.com' },
    { name: 'PackyCode', base_url: 'https://www.packyapi.com/v1', description: 'PackyCode 中继', color: '#0ea5e9', category: 'third_party', websiteUrl: 'https://www.packyapi.com' },
    { name: 'Cubence', base_url: 'https://api.cubence.com/v1', description: 'Cubence 中继', color: '#4B0082', category: 'third_party', websiteUrl: 'https://cubence.com' },
  ],
};

function getLatencyQuality(urls: { latency_ms: number | null }[] | undefined) {
  if (!urls?.length) return null;
  const tested = urls.filter(u => u.latency_ms != null);
  if (!tested.length) return null;
  const best = Math.min(...tested.map(u => u.latency_ms!));
  if (best < 300) return 'excellent';
  if (best < 800) return 'good';
  if (best < 2000) return 'fair';
  return 'poor';
}

function getLatencyMs(urls: { latency_ms: number | null }[] | undefined) {
  if (!urls?.length) return null;
  const tested = urls.filter(u => u.latency_ms != null);
  if (!tested.length) return null;
  return Math.min(...tested.map(u => u.latency_ms!));
}

const sLabel: React.CSSProperties = { fontSize: 12, fontWeight: 600, color: 'var(--text-secondary)', marginBottom: 6, display: 'block' };
const sHint: React.CSSProperties = { fontSize: 11, color: 'var(--text-tertiary)', marginTop: 4 };
const sSection: React.CSSProperties = { marginBottom: 24 };
const sSelect: React.CSSProperties = { width: '100%', padding: '8px 12px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--card-bg, #fff)', fontSize: 13, color: 'var(--text-primary)', appearance: 'none' as const, cursor: 'pointer' };

interface PlatformProviderPanelProps {
  modelType: ProviderModelType;
}

export function PlatformProviderPanel({ modelType }: PlatformProviderPanelProps) {
  const { t } = useTranslation();
  const store = useProviderStore();
  const toast = useToast();

  const [view, setView] = useState<PanelView>('list');
  const [selectedPreset, setSelectedPreset] = useState<ProviderPreset | null>(null);
  const [addForm, setAddForm] = useState({ name: '', api_key: '', base_url: '', description: '' });
  const [apiFormat, setApiFormat] = useState<ApiFormat>('anthropic');
  const [authField, setAuthField] = useState<AuthField>('ANTHROPIC_AUTH_TOKEN');
  const [modelConfig, setModelConfig] = useState<ModelConfig>({ ...EMPTY_MODEL_CONFIG });

  const [editingProvider, setEditingProvider] = useState<string | null>(null);
  const [editForm, setEditForm] = useState({ base_url: '', api_key: '', description: '' });
  const [editApiFormat, setEditApiFormat] = useState<ApiFormat>('anthropic');
  const [editAuthField, setEditAuthField] = useState<AuthField>('ANTHROPIC_AUTH_TOKEN');
  const [editModelConfig, setEditModelConfig] = useState<ModelConfig>({ ...EMPTY_MODEL_CONFIG });

  const [showDeleteDialog, setShowDeleteDialog] = useState<string | null>(null);
  const [addModelId, setAddModelId] = useState('');
  const [fetchingModels, setFetchingModels] = useState(false);
  const [fetchedModels, setFetchedModels] = useState<string[]>([]);
  const [selectedFetchedModels, setSelectedFetchedModels] = useState<Set<string>>(new Set());
  const [showFetchDialog, setShowFetchDialog] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [testingProvider, setTestingProvider] = useState<string | null>(null);

  const filteredProviders = useMemo(() => {
    let list = store.providers.filter(p => (p.model_type || 'claude') === modelType);
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter(p => p.name.toLowerCase().includes(q) || p.base_url.toLowerCase().includes(q));
    }
    return list;
  }, [store.providers, modelType, searchQuery]);

  useEffect(() => { store.loadProviders(); }, []);
  useEffect(() => { store.selectProvider(filteredProviders[0]?.name || ''); }, [modelType]);

  const showClaudeModels = modelType === 'claude' || modelType === 'openclaw';

  const resetAddForm = () => {
    setAddForm({ name: '', api_key: '', base_url: '', description: '' });
    setSelectedPreset(null);
    setApiFormat('anthropic');
    setAuthField('ANTHROPIC_AUTH_TOKEN');
    setModelConfig({ ...EMPTY_MODEL_CONFIG });
  };

  const handleAddProvider = async () => {
    const form = selectedPreset
      ? { name: addForm.name || selectedPreset.name, base_url: addForm.base_url || selectedPreset.base_url, api_key: addForm.api_key, description: addForm.description || selectedPreset.description || '' }
      : addForm;
    if (!form.name || !form.base_url) return;
    try {
      await invoke('add_provider', { input: { ...form, model_type: modelType } });
      await store.loadProviders();
      store.selectProvider(form.name);
      resetAddForm();
      setView('list');
      toast.success(t('providers.addSuccess', 'Provider 添加成功'));
    } catch (e) {
      toast.error(t('providers.addFailed', '添加失败: ') + String(e));
    }
  };

  const openEditView = async (name: string) => {
    try {
      const detail = await invoke<any>('get_provider', { name });
      setEditForm({ base_url: detail?.base_url || '', api_key: detail?.options?.api_key || '', description: detail?.description || '' });
      setEditApiFormat('anthropic');
      setEditAuthField('ANTHROPIC_AUTH_TOKEN');
      setEditModelConfig({ ...EMPTY_MODEL_CONFIG });
      setEditingProvider(name);
      setView('edit');
    } catch (e) { toast.error(String(e)); }
  };

  const handleEditProvider = async () => {
    if (!editingProvider) return;
    try {
      await store.updateProvider(editingProvider, editForm);
      setView('list');
      setEditingProvider(null);
      toast.success(t('providers.editSuccess', '修改已保存'));
    } catch (e) { toast.error(t('providers.editFailed', '保存失败: ') + String(e)); }
  };

  const handleDeleteProvider = async () => {
    if (!showDeleteDialog) return;
    try {
      await store.deleteProvider(showDeleteDialog);
      setShowDeleteDialog(null);
      toast.success(t('providers.deleteSuccess', 'Provider 已删除'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleAddModel = async () => {
    if (!addModelId.trim() || !store.selectedProvider) return;
    try {
      await invoke('add_model', { providerName: store.selectedProvider, input: { id: addModelId.trim() } });
      await store.loadModels(); await store.loadProviders();
      setAddModelId('');
      toast.success(t('providers.modelAdded', '模型已添加'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleDeleteModel = async (modelId: string) => {
    if (!store.selectedProvider) return;
    await invoke('delete_model', { providerName: store.selectedProvider, modelId });
    await store.loadModels(); await store.loadProviders();
  };

  const handleFetchModels = async () => {
    setFetchingModels(true);
    try {
      const models = await store.fetchSiteModels();
      setFetchedModels(models); setSelectedFetchedModels(new Set(models)); setShowFetchDialog(true);
    } catch (e) { toast.error(t('providers.fetchFailed', '拉取模型失败: ') + String(e)); }
    finally { setFetchingModels(false); }
  };

  const handleAddFetchedModels = async () => {
    const ids = Array.from(selectedFetchedModels);
    if (ids.length === 0) return;
    await store.addModelsBatch(ids);
    setShowFetchDialog(false); setFetchedModels([]); setSelectedFetchedModels(new Set());
    toast.success(t('providers.modelsAdded', `已添加 ${ids.length} 个模型`));
  };

  const handleApplyConfig = async () => {
    const enabledNames = filteredProviders.filter(p => p.enabled).map(p => p.name);
    if (enabledNames.length === 0) { toast.warning(t('providers.noEnabled', '没有启用的 Provider')); return; }
    try {
      await invoke('apply_config', { input: { provider_names: enabledNames, apply_to_global: true, apply_to_project: false } });
      await store.loadProviders();
      toast.success(t('providers.applySuccess', '配置已应用'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleSpeedTest = async (providerName: string) => {
    const provider = store.providers.find(p => p.name === providerName);
    if (!provider) return;
    setTestingProvider(providerName);
    try {
      const detail = await invoke<any>('get_provider', { name: providerName });
      const urls = provider.base_urls?.map(u => u.url) || [provider.base_url];
      await invoke('test_and_auto_select_fastest', { providerName, urls, apiKey: detail?.options?.api_key, modelType: provider.model_type });
      await store.loadProviders();
      toast.success(t('providers.speedTestDone', '测速完成'));
    } catch (e) { toast.error(String(e)); }
    finally { setTestingProvider(null); }
  };

  const handleExportProviders = async () => {
    try {
      const exportData = filteredProviders.map(p => ({ name: p.name, base_url: p.base_url, model_type: p.model_type, model_count: p.model_count, description: p.description, enabled: p.enabled, base_urls: p.base_urls }));
      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a'); a.href = url; a.download = `providers-${modelType}-${new Date().toISOString().slice(0, 10)}.json`; a.click(); URL.revokeObjectURL(url);
      toast.success(t('providers.exportSuccess', '导出成功'));
    } catch (e) { toast.error(t('providers.exportFailed', '导出失败: ') + String(e)); }
  };

  const handleImportProviders = async () => {
    const input = document.createElement('input'); input.type = 'file'; input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]; if (!file) return;
      try {
        const text = await file.text(); const imported = JSON.parse(text);
        const providers = Array.isArray(imported) ? imported : [imported]; let count = 0;
        for (const p of providers) { if (p.name && p.base_url) { try { await invoke('add_provider', { input: { name: p.name, base_url: p.base_url, api_key: p.api_key || '', description: p.description || '', model_type: p.model_type || modelType } }); count++; } catch { /* skip */ } } }
        await store.loadProviders();
        toast.success(t('providers.importSuccess', `成功导入 ${count} 个 Provider`));
      } catch (err) { toast.error(t('providers.importFailed', '导入失败: ') + String(err)); }
    };
    input.click();
  };

  const currentPresets = PROVIDER_PRESETS[modelType] || [];

  // --- Preset tag cloud (shared between add & edit) ---
  const renderPresetCloud = () => (
    <div style={{ padding: '16px 20px', borderRadius: 12, border: '1px solid var(--border)', background: 'var(--card-bg, transparent)' }}>
      {CATEGORY_ORDER.map(cat => {
        const items = currentPresets.filter(p => p.category === cat);
        if (items.length === 0) return null;
        return (
          <div key={cat} style={{ marginBottom: 16 }}>
            <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-tertiary)', marginBottom: 8, letterSpacing: '0.5px' }}>{CATEGORY_LABELS[cat]}</div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
              {items.map(preset => (
                <button key={preset.name} onClick={() => {
                  setSelectedPreset(preset);
                  setAddForm(f => ({ ...f, base_url: preset.base_url, name: f.name || '' }));
                }} style={{
                  display: 'inline-flex', alignItems: 'center', gap: 6, padding: '6px 14px', borderRadius: 20, fontSize: 13, fontWeight: 500,
                  cursor: 'pointer', transition: 'all 0.18s ease',
                  border: selectedPreset?.name === preset.name ? `2px solid ${preset.color || 'var(--primary)'}` : '1px solid var(--border)',
                  background: selectedPreset?.name === preset.name ? `${preset.color}18` : 'var(--card-bg, transparent)',
                  color: selectedPreset?.name === preset.name ? (preset.color || 'var(--primary)') : 'var(--text-primary)',
                  boxShadow: selectedPreset?.name === preset.name ? `0 0 0 1px ${preset.color}30` : 'none',
                }}>
                  <span style={{ width: 8, height: 8, borderRadius: '50%', background: preset.color || 'var(--primary)', flexShrink: 0 }} />
                  {preset.name}
                  {preset.isOfficial && <Star size={10} style={{ color: '#f59e0b', fill: '#f59e0b' }} />}
                </button>
              ))}
            </div>
          </div>
        );
      })}
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--text-tertiary)', marginTop: 4, paddingTop: 12, borderTop: '1px solid var(--border)' }}>
        <span style={{ fontSize: 14 }}>💡</span>
        <span>{t('providers.customHint', '自定义配置需手动填写所有必要字段')}</span>
      </div>
    </div>
  );

  // --- Select wrapper ---
  const SelectField = ({ label, hint, value, onChange, options }: { label: string; hint?: string; value: string; onChange: (v: string) => void; options: { value: string; label: string }[] }) => (
    <div style={sSection}>
      <label style={sLabel}>{label}</label>
      <div style={{ position: 'relative' }}>
        <select style={sSelect} value={value} onChange={e => onChange(e.target.value)}>
          {options.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
        </select>
        <ChevronDown size={14} style={{ position: 'absolute', right: 12, top: '50%', transform: 'translateY(-50%)', pointerEvents: 'none', color: 'var(--text-tertiary)' }} />
      </div>
      {hint && <div style={sHint}>{hint}</div>}
    </div>
  );

  // --- Model config fields ---
  const renderModelFields = (cfg: ModelConfig, setCfg: (c: ModelConfig) => void) => {
    if (modelType === 'codex') {
      return (
        <div style={sSection}>
          <label style={sLabel}>{t('providers.modelSettings', '模型设置')}</label>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>{t('providers.mainModel', '主模型')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="gpt-5.4" value={cfg.mainModel} onChange={e => setCfg({ ...cfg, mainModel: e.target.value })} />
            </div>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>{t('providers.thinkingModel', '推理模型 (Thinking)')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="gpt-5.4" value={cfg.thinkingModel} onChange={e => setCfg({ ...cfg, thinkingModel: e.target.value })} />
            </div>
          </div>
        </div>
      );
    }
    if (showClaudeModels) {
      return (
        <div style={sSection}>
          <label style={sLabel}>{t('providers.modelSettings', '模型设置')}</label>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>{t('providers.mainModel', '主模型')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="claude-sonnet-4-20250514" value={cfg.mainModel} onChange={e => setCfg({ ...cfg, mainModel: e.target.value })} />
            </div>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>{t('providers.thinkingModel', '推理模型 (Thinking)')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="" value={cfg.thinkingModel} onChange={e => setCfg({ ...cfg, thinkingModel: e.target.value })} />
            </div>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>Haiku {t('providers.defaultModel', '默认模型')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="claude-haiku-4-20250514" value={cfg.haikuModel} onChange={e => setCfg({ ...cfg, haikuModel: e.target.value })} />
            </div>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>Sonnet {t('providers.defaultModel', '默认模型')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="claude-sonnet-4-20250514" value={cfg.sonnetModel} onChange={e => setCfg({ ...cfg, sonnetModel: e.target.value })} />
            </div>
            <div>
              <label style={{ ...sLabel, fontSize: 11 }}>Opus {t('providers.defaultModel', '默认模型')}</label>
              <input className="input input-sm input-bordered w-full font-mono" placeholder="claude-sonnet-4-20250514" value={cfg.opusModel} onChange={e => setCfg({ ...cfg, opusModel: e.target.value })} />
            </div>
          </div>
        </div>
      );
    }
    return (
      <div style={sSection}>
        <label style={sLabel}>{t('providers.modelSettings', '模型设置')}</label>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
          <div>
            <label style={{ ...sLabel, fontSize: 11 }}>{t('providers.mainModel', '主模型')}</label>
            <input className="input input-sm input-bordered w-full font-mono" placeholder="gemini-2.5-pro" value={cfg.mainModel} onChange={e => setCfg({ ...cfg, mainModel: e.target.value })} />
          </div>
        </div>
      </div>
    );
  };

  // ===================== ADD VIEW =====================
  if (view === 'add') {
    return (
      <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', background: 'var(--page-bg, var(--fallback-b1))' }}>
        <ToastContainer toasts={toast.toasts} />
        <div style={{ padding: '16px 24px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', gap: 12 }}>
          <button className="btn btn-sm btn-ghost" onClick={() => { resetAddForm(); setView('list'); }}>
            <ArrowLeft size={18} />
          </button>
          <h2 style={{ fontSize: 18, fontWeight: 700, margin: 0 }}>{t('providers.addNewProvider', '添加新供应商')}</h2>
        </div>
        <div style={{ flex: 1, overflowY: 'auto', padding: '24px 32px' }}>
          <div style={{ maxWidth: 960, margin: '0 auto' }}>
            {renderPresetCloud()}

            <div style={{ height: 1, background: 'var(--border)', margin: '20px 0 24px' }} />

            {selectedPreset && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 20, padding: '12px 16px', borderRadius: 10, background: `${selectedPreset.color}10`, border: `1px solid ${selectedPreset.color}30` }}>
                <div style={{ width: 36, height: 36, borderRadius: 8, background: `${selectedPreset.color}20`, display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 700, fontSize: 16, color: selectedPreset.color }}>
                  {selectedPreset.name.charAt(0)}
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 600, fontSize: 14 }}>{selectedPreset.name}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)' }}>{selectedPreset.description}</div>
                </div>
                {selectedPreset.websiteUrl && (
                  <button className="btn btn-xs btn-ghost" onClick={() => window.open(selectedPreset.websiteUrl, '_blank')} title={t('providers.visitWebsite', '访问官网')}>
                    <ExternalLink size={14} /> {t('providers.website', '官网')}
                  </button>
                )}
              </div>
            )}

            {/* Base URL */}
            <div style={sSection}>
              <label style={sLabel}>Base URL</label>
              <div style={{ padding: '10px 14px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--warning-bg, #fef3c7)', fontSize: 12, color: 'var(--warning-text, #92400e)', marginBottom: 10 }}>
                💡 {t('providers.baseUrlHint', '填写兼容 API 的服务端点地址，不要以斜杠结尾')}
              </div>
              <input className="input input-bordered w-full font-mono" placeholder="https://api.example.com" value={addForm.base_url || (selectedPreset?.base_url ?? '')} onChange={e => setAddForm(f => ({ ...f, base_url: e.target.value }))} />
            </div>

            {/* API Format + Auth Field (Claude only) */}
            {showClaudeModels && (
              <>
                <SelectField label={t('providers.apiFormat', 'API 格式')} hint={t('providers.apiFormatHint', '选择供应商 API 的输入格式')} value={apiFormat} onChange={v => setApiFormat(v as ApiFormat)} options={API_FORMAT_OPTIONS.map(o => ({ value: o.value, label: o.label }))} />
                <SelectField label={t('providers.authField', '认证字段')} hint={t('providers.authFieldHint', '选择写入配置的认证环境变量名')} value={authField} onChange={v => setAuthField(v as AuthField)} options={AUTH_FIELD_OPTIONS} />
              </>
            )}

            {/* API Key */}
            <div style={sSection}>
              <label style={sLabel}>API Key</label>
              <input type="password" className="input input-bordered w-full font-mono" placeholder={t('providers.apiKeyPlaceholder', '输入 API Key')} value={addForm.api_key} onChange={e => setAddForm(f => ({ ...f, api_key: e.target.value }))} />
            </div>

            {/* Model config */}
            {renderModelFields(modelConfig, setModelConfig)}

            {/* Name & description */}
            <div style={sSection}>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                <div>
                  <label style={sLabel}>{t('providers.providerName', '供应商名称')}</label>
                  <input className="input input-bordered w-full" placeholder={selectedPreset ? `${t('providers.egName', '例如')}: ${selectedPreset.name}` : t('providers.namePlaceholder', '例如：Claude 官方')} value={addForm.name} onChange={e => setAddForm(f => ({ ...f, name: e.target.value }))} />
                </div>
                <div>
                  <label style={sLabel}>{t('providers.remarks', '备注')}</label>
                  <input className="input input-bordered w-full" placeholder={t('providers.remarksPlaceholder', '例如：公司专用账号')} value={addForm.description} onChange={e => setAddForm(f => ({ ...f, description: e.target.value }))} />
                </div>
              </div>
            </div>

            {selectedPreset?.websiteUrl && (
              <div style={sSection}>
                <label style={sLabel}>{t('providers.officialLink', '官网链接')}</label>
                <input className="input input-bordered w-full font-mono" value={selectedPreset.websiteUrl} readOnly style={{ opacity: 0.6 }} />
              </div>
            )}
          </div>
        </div>
        {/* Bottom bar */}
        <div style={{ padding: '12px 24px', borderTop: '1px solid var(--border)', display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button className="btn" onClick={() => { resetAddForm(); setView('list'); }}>{t('common.cancel', '取消')}</button>
          <button className="btn btn-primary" onClick={handleAddProvider} disabled={selectedPreset ? false : (!addForm.name || !addForm.base_url)}>
            <Plus size={14} /> {t('providers.addBtn', '添加')}
          </button>
        </div>
      </div>
    );
  }

  // ===================== EDIT VIEW =====================
  if (view === 'edit' && editingProvider) {
    return (
      <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', background: 'var(--page-bg, var(--fallback-b1))' }}>
        <ToastContainer toasts={toast.toasts} />
        <div style={{ padding: '16px 24px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', gap: 12 }}>
          <button className="btn btn-sm btn-ghost" onClick={() => { setView('list'); setEditingProvider(null); }}>
            <ArrowLeft size={18} />
          </button>
          <h2 style={{ fontSize: 18, fontWeight: 700, margin: 0 }}>{t('providers.editProvider', '编辑供应商')}</h2>
        </div>
        <div style={{ flex: 1, overflowY: 'auto', padding: '24px 32px' }}>
          <div style={{ maxWidth: 960, margin: '0 auto' }}>
            {/* Base URL */}
            <div style={sSection}>
              <div style={{ padding: '10px 14px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--warning-bg, #fef3c7)', fontSize: 12, color: 'var(--warning-text, #92400e)', marginBottom: 10 }}>
                💡 {t('providers.baseUrlHint', '填写兼容 API 的服务端点地址，不要以斜杠结尾')}
              </div>
              <label style={sLabel}>Base URL</label>
              <input className="input input-bordered w-full font-mono" value={editForm.base_url} onChange={e => setEditForm(f => ({ ...f, base_url: e.target.value }))} />
            </div>

            {showClaudeModels && (
              <>
                <SelectField label={t('providers.apiFormat', 'API 格式')} hint={t('providers.apiFormatHint', '选择供应商 API 的输入格式')} value={editApiFormat} onChange={v => setEditApiFormat(v as ApiFormat)} options={API_FORMAT_OPTIONS.map(o => ({ value: o.value, label: o.label }))} />
                <SelectField label={t('providers.authField', '认证字段')} hint={t('providers.authFieldHint', '选择写入配置的认证环境变量名')} value={editAuthField} onChange={v => setEditAuthField(v as AuthField)} options={AUTH_FIELD_OPTIONS} />
              </>
            )}

            <div style={sSection}>
              <label style={sLabel}>API Key</label>
              <input type="password" className="input input-bordered w-full font-mono" value={editForm.api_key} onChange={e => setEditForm(f => ({ ...f, api_key: e.target.value }))} />
            </div>

            {renderModelFields(editModelConfig, setEditModelConfig)}

            <div style={sSection}>
              <label style={sLabel}>{t('providers.remarks', '备注')}</label>
              <input className="input input-bordered w-full" value={editForm.description} onChange={e => setEditForm(f => ({ ...f, description: e.target.value }))} />
            </div>
          </div>
        </div>
        <div style={{ padding: '12px 24px', borderTop: '1px solid var(--border)', display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button className="btn" onClick={() => { setView('list'); setEditingProvider(null); }}>{t('common.cancel', '取消')}</button>
          <button className="btn btn-primary" onClick={handleEditProvider}>
            <Save size={14} /> {t('common.save', '保存')}
          </button>
        </div>
      </div>
    );
  }

  // ===================== LIST VIEW =====================
  return (
    <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', gap: 16, padding: 16 }}>
      <ToastContainer toasts={toast.toasts} />

      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: 6 }}>
        <button className="btn btn-sm btn-ghost" onClick={handleImportProviders} title={t('providers.import', '导入 Provider')}>
          <Download size={14} /> {t('providers.import', '导入')}
        </button>
        <button className="btn btn-sm btn-ghost" onClick={handleExportProviders} title={t('providers.export', '导出 Provider')}>
          <Upload size={14} /> {t('providers.export', '导出')}
        </button>
        <button className="btn btn-sm btn-ghost" onClick={handleApplyConfig} title={t('providers.applyGlobal', '应用到全局配置')}>
          <Upload size={14} /> {t('providers.apply', '应用配置')}
        </button>
      </div>

      <div style={{ flex: 1, minHeight: 0, display: 'flex', gap: 16 }}>
        {/* Provider List */}
        <div style={{ width: 288, display: 'flex', flexDirection: 'column', overflow: 'hidden', borderRadius: 'var(--radius-lg)', border: '1px solid var(--border)' }}>
          <div className="p-3 border-b border-base-content/10 flex flex-col gap-2">
            <div className="flex items-center justify-between">
              <h3 className="font-semibold text-sm">{t('providers.title', 'Providers')}</h3>
              <button className="btn btn-xs btn-primary" onClick={() => { resetAddForm(); setView('add'); }}>
                <Plus size={12} /> {t('providers.add', '添加')}
              </button>
            </div>
            <div className="oc-search-wrap">
              <Search size={14} className="oc-search-icon" />
              <input type="text" className="oc-search-input" placeholder={t('providers.search', '搜索 Provider...')} value={searchQuery} onChange={e => setSearchQuery(e.target.value)} />
            </div>
          </div>
          <div className="flex-1 overflow-y-auto">
            {filteredProviders.map(p => {
              const latency = getLatencyMs(p.base_urls);
              const quality = getLatencyQuality(p.base_urls);
              return (
                <div key={p.name} className={`oc-provider-card group ${store.selectedProvider === p.name ? 'is-selected' : ''}`} onClick={() => store.selectProvider(p.name)}>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className={`oc-toggle ${p.enabled ? 'is-on' : ''}`} role="switch" aria-checked={p.enabled} onClick={e => { e.stopPropagation(); store.toggleProvider(p.name, !p.enabled); }} />
                      <span className={`text-sm font-medium truncate ${!p.enabled ? 'opacity-50' : ''}`}>{p.name}</span>
                    </div>
                    <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button className="btn btn-xs btn-ghost" onClick={e => { e.stopPropagation(); handleSpeedTest(p.name); }} title={t('providers.speedTest', '测速')}>
                        {testingProvider === p.name ? <Loader2 size={12} className="animate-spin" /> : <Zap size={12} />}
                      </button>
                      <button className="btn btn-xs btn-ghost" onClick={e => { e.stopPropagation(); openEditView(p.name); }} title={t('providers.edit', '编辑')}>
                        <Pencil size={12} />
                      </button>
                      <button className="btn btn-xs btn-ghost text-error" onClick={e => { e.stopPropagation(); setShowDeleteDialog(p.name); }} title={t('providers.delete', '删除')}>
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                  <div className="text-xs opacity-50 mt-1 truncate font-mono">{p.base_url}</div>
                  <div className="oc-provider-meta">
                    <span>{p.model_count} {t('providers.modelsCount', 'models')}</span>
                    {latency != null && (<><span className="oc-provider-meta-dot" /><span className="oc-latency"><span className={`oc-latency-dot is-${quality}`} />{latency}ms</span></>)}
                  </div>
                </div>
              );
            })}
            {filteredProviders.length === 0 && (
              <div className="oc-empty-state">
                <div className="oc-empty-state-icon"><Package size={28} /></div>
                <div className="oc-empty-state-title">{t('providers.empty', '暂无 Provider')}</div>
                <div className="oc-empty-state-desc">{t('providers.emptyDesc', '点击上方 "添加" 按钮来配置你的第一个 Provider')}</div>
              </div>
            )}
          </div>
        </div>

        {/* Model Detail */}
        <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column', borderRadius: 'var(--radius-lg)', border: '1px solid var(--border)' }}>
          {store.selectedProvider ? (
            <>
              <div className="p-4 border-b border-base-content/10">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="font-semibold">{store.selectedProvider}</h3>
                    <p className="text-xs opacity-50 mt-0.5">{t('providers.modelsSubtitle', '管理此 Provider 的可用模型')}</p>
                  </div>
                  <button className="btn btn-sm btn-ghost" onClick={handleFetchModels} disabled={fetchingModels}>
                    {fetchingModels ? <Loader2 className="animate-spin" size={14} /> : <Globe size={14} />}
                    {t('providers.fetchModels', '拉取模型')}
                  </button>
                </div>
                {(() => {
                  const selectedP = filteredProviders.find(p => p.name === store.selectedProvider);
                  if (!selectedP?.base_urls?.length || selectedP.base_urls.length <= 1) return null;
                  return (
                    <div style={{ marginTop: 8, padding: '8px 0' }}>
                      <div style={{ fontSize: 11, color: 'var(--text-secondary)', marginBottom: 6, fontWeight: 600 }}>{t('providers.failoverUrls', '故障转移 URLs')} ({selectedP.base_urls.length})</div>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                        {selectedP.base_urls.map((urlItem, idx) => (
                          <div key={idx} style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 11 }}>
                            <span className={`oc-latency-dot is-${urlItem.quality}`} style={{ flexShrink: 0 }} />
                            <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-primary)', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{urlItem.url}</span>
                            {urlItem.latency_ms != null && <span style={{ color: 'var(--text-tertiary)', flexShrink: 0 }}>{urlItem.latency_ms}ms</span>}
                          </div>
                        ))}
                      </div>
                    </div>
                  );
                })()}
              </div>
              <div className="flex-1 overflow-y-auto p-4 space-y-2 oc-stagger">
                {store.models.map(m => (
                  <div key={m.id} className="flex items-center justify-between p-3 rounded-lg bg-base-300 group oc-card" style={{ border: '1px solid var(--border-light)' }}>
                    <div className="min-w-0">
                      <span className="font-mono text-sm">{m.id}</span>
                      {m.name && m.name !== m.id && <span className="text-xs opacity-50 ml-2">({m.name})</span>}
                      {m.reasoning_effort && <span className="badge badge-xs badge-info ml-2">{m.reasoning_effort}</span>}
                    </div>
                    <button className="btn btn-xs btn-ghost text-error opacity-0 group-hover:opacity-100" onClick={() => handleDeleteModel(m.id)}><Trash2 size={12} /></button>
                  </div>
                ))}
                {store.models.length === 0 && (
                  <div className="oc-empty-state">
                    <div className="oc-empty-state-icon"><Globe size={28} /></div>
                    <div className="oc-empty-state-title">{t('providers.noModels', '暂无模型')}</div>
                    <div className="oc-empty-state-desc">{t('providers.noModelsDesc', '在下方输入模型 ID 手动添加，或点击 "拉取模型" 从站点自动获取')}</div>
                  </div>
                )}
              </div>
              <div className="p-3 border-t border-base-content/10 flex gap-2">
                <input type="text" className="input input-sm input-bordered flex-1 font-mono" placeholder={t('providers.addModelPlaceholder', '输入模型 ID，如 claude-sonnet-4-20250514')} value={addModelId} onChange={e => setAddModelId(e.target.value)} onKeyDown={e => { if (e.key === 'Enter') handleAddModel(); }} />
                <button className="btn btn-sm btn-primary" onClick={handleAddModel} disabled={!addModelId.trim()}><Plus size={14} /></button>
              </div>
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <div className="oc-empty-state">
                <div className="oc-empty-state-icon"><Layers size={28} /></div>
                <div className="oc-empty-state-title">{t('providers.selectProvider', '选择一个 Provider')}</div>
                <div className="oc-empty-state-desc">{t('providers.selectProviderDesc', '从左侧列表选择或添加一个 Provider 开始配置')}</div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Delete Confirm (keep as modal) */}
      {showDeleteDialog && (
        <div className="oc-modal-overlay">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('confirm.deleteTitle', '确认删除')}</h3>
            <p className="py-4">{t('confirm.deleteProvider', `确定要删除 "${showDeleteDialog}" 吗？`)}</p>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowDeleteDialog(null)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-error" onClick={handleDeleteProvider}>{t('common.delete', '删除')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Fetch Models Dialog (keep as modal) */}
      {showFetchDialog && (
        <div className="oc-modal-overlay">
          <div className="modal-box max-w-lg">
            <h3 className="font-bold text-lg">{t('providers.fetchedModels', '可用模型')}</h3>
            <p className="text-xs opacity-50 mt-1">{t('providers.fetchedModelsDesc', '从站点拉取到的模型列表，勾选后批量添加')}</p>
            <div className="mt-4 max-h-64 overflow-y-auto space-y-1">
              {fetchedModels.map(m => (
                <label key={m} className="flex items-center gap-2 p-2 rounded hover:bg-base-200 cursor-pointer">
                  <input type="checkbox" className="checkbox checkbox-sm" checked={selectedFetchedModels.has(m)} onChange={() => setSelectedFetchedModels(prev => { const next = new Set(prev); next.has(m) ? next.delete(m) : next.add(m); return next; })} />
                  <span className="font-mono text-sm">{m}</span>
                </label>
              ))}
              {fetchedModels.length === 0 && <div className="text-center py-4 opacity-50">{t('providers.noModelsFound', '未找到模型')}</div>}
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowFetchDialog(false)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary" onClick={handleAddFetchedModels} disabled={selectedFetchedModels.size === 0}>{t('providers.addSelected', '添加选中')} ({selectedFetchedModels.size})</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

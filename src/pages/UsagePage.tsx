import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  BarChart3, RefreshCw, Trash2, Loader2, DollarSign, Cpu,
  MessageSquare, Upload, Search, Layers, Terminal,
  ChevronDown, ChevronRight, Globe, Edit2, X, Check, RotateCcw,
  FolderOpen, Plus, AlertCircle,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';
import { PlatformId } from '../types/platform';
import { renderPlatformIcon, getPlatformLabel } from '../utils/platformMeta';

interface UsageSummary {
  totalRequests: number;
  totalCost: string;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheCreationTokens: number;
  totalCacheReadTokens: number;
  activeDays: number;
  successRate: number;
}

interface UsageTrend {
  period: string;
  requestCount: number;
  totalCost: number;
  inputTokens: number;
  outputTokens: number;
  totalCacheCreationTokens: number;
  totalCacheReadTokens: number;
  totalTokens: number;
  topModel: string | null;
}

interface ProviderStats {
  providerId: string;
  providerName: string;
  requestCount: number;
  totalTokens: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheCreationTokens: number;
  totalCacheReadTokens: number;
  totalCost: string;
  successRate: number;
}

interface ProjectStats {
  projectName: string;
  requestCount: number;
  totalTokens: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheCreationTokens: number;
  totalCacheReadTokens: number;
  totalCost: string;
  providerCount: number;
  modelCount: number;
}

interface ModelPricing {
  modelId: string;
  displayName: string;
  inputCostPerMillion: string;
  outputCostPerMillion: string;
  cacheReadCostPerMillion: string;
  cacheCreationCostPerMillion: string;
}

interface ModelUsage {
  model: string;
  inputTokens: number;
  outputTokens: number;
  totalCacheCreationTokens: number;
  totalCacheReadTokens: number;
  totalTokens: number;
  totalCost: number;
  requestCount: number;
}

interface ModelTrendData {
  period: string;
  models: ModelUsage[];
  totalTokens: number;
  totalCost: number;
}

interface ProviderModelPricing {
  id: number | null;
  providerId: string;
  modelId: string;
  inputCostPerMillion: string;
  outputCostPerMillion: string;
  cacheReadCostPerMillion: string;
  cacheCreationCostPerMillion: string;
}

interface ScanResult {
  claudeFiles: number;
  claudeEntries: number;
  claudePath: string | null;
  codexFiles: number;
  codexEntries: number;
  codexPath: string | null;
  geminiFiles: number;
  geminiEntries: number;
  geminiPath: string | null;
  opencodeFiles: number;
  opencodeEntries: number;
  opencodePath: string | null;
  cursorFiles: number;
  cursorEntries: number;
  cursorPath: string | null;
  windsurfFiles: number;
  windsurfEntries: number;
  windsurfPath: string | null;
  kiroFiles: number;
  kiroEntries: number;
  kiroPath: string | null;
  antigravityFiles: number;
  antigravityEntries: number;
  antigravityPath: string | null;
  warpFiles: number;
  warpEntries: number;
  warpPath: string | null;
  augmentFiles: number;
  augmentEntries: number;
  augmentPath: string | null;
  githubCopilotFiles: number;
  githubCopilotEntries: number;
  githubCopilotPath: string | null;
  codebuddyFiles: number;
  codebuddyEntries: number;
  codebuddyPath: string | null;
  codebuddyCnFiles: number;
  codebuddyCnEntries: number;
  codebuddyCnPath: string | null;
  qoderFiles: number;
  qoderEntries: number;
  qoderPath: string | null;
  traeFiles: number;
  traeEntries: number;
  traePath: string | null;
  workbuddyFiles: number;
  workbuddyEntries: number;
  workbuddyPath: string | null;
  openclawFiles: number;
  openclawEntries: number;
  openclawPath: string | null;
  existingRecords: number;
}

interface PlatformEntry {
  providerId: string;
  platformId: PlatformId | null;
  fallbackName: string;
}

const ALL_PLATFORMS: PlatformEntry[] = [
  { providerId: 'claude_local', platformId: 'claude-code', fallbackName: 'Claude Code' },
  { providerId: 'codex_local', platformId: 'codex', fallbackName: 'Codex' },
  { providerId: 'gemini_local', platformId: 'gemini', fallbackName: 'Gemini CLI' },
  { providerId: 'cursor_local', platformId: 'cursor', fallbackName: 'Cursor' },
  { providerId: 'windsurf_local', platformId: 'windsurf', fallbackName: 'Windsurf' },
  { providerId: 'kiro_local', platformId: 'kiro', fallbackName: 'Kiro' },
  { providerId: 'antigravity_local', platformId: 'antigravity', fallbackName: 'Antigravity' },
  { providerId: 'warp_local', platformId: 'warp', fallbackName: 'Warp' },
  { providerId: 'augment_local', platformId: 'augment', fallbackName: 'Augment' },
  { providerId: 'opencode_local', platformId: 'opencode', fallbackName: 'OpenCode' },
  { providerId: 'github-copilot_local', platformId: 'github-copilot', fallbackName: 'GitHub Copilot' },
  { providerId: 'codebuddy_local', platformId: 'codebuddy', fallbackName: 'CodeBuddy' },
  { providerId: 'codebuddy_cn_local', platformId: 'codebuddy_cn', fallbackName: 'CodeBuddy CN' },
  { providerId: 'qoder_local', platformId: 'qoder', fallbackName: 'Qoder' },
  { providerId: 'trae_local', platformId: 'trae', fallbackName: 'Trae' },
  { providerId: 'workbuddy_local', platformId: 'workbuddy', fallbackName: 'WorkBuddy' },
  { providerId: 'openclaw_local', platformId: 'openclaw', fallbackName: 'OpenClaw' },
];

const KNOWN_PROVIDER_IDS = new Set(ALL_PLATFORMS.map(p => p.providerId));

const CHART_COLORS = [
  '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
  '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1',
  '#14b8a6', '#e11d48', '#a855f7', '#0ea5e9', '#22c55e',
  '#eab308', '#d946ef',
];

let usageAutoImportTriggered = false;
let cachedUsageScanResult: ScanResult | null = null;

function aggregateModelUsage(trendData: ModelTrendData[]): ModelUsage[] {
  const map = new Map<string, ModelUsage>();
  for (const td of trendData) {
    for (const m of td.models) {
      const existing = map.get(m.model);
      if (existing) {
        existing.inputTokens += m.inputTokens;
        existing.outputTokens += m.outputTokens;
        existing.totalCacheCreationTokens += m.totalCacheCreationTokens;
        existing.totalCacheReadTokens += m.totalCacheReadTokens;
        existing.totalTokens += m.totalTokens;
        existing.totalCost += m.totalCost;
        existing.requestCount += m.requestCount;
      } else {
        map.set(m.model, { ...m });
      }
    }
  }
  return Array.from(map.values()).sort((a, b) => b.totalTokens - a.totalTokens);
}

function normalizePricingLookupModel(model: string): string {
  return model
    .split('/')
    .pop()
    ?.split(':')[0]
    .trim()
    .replace(/@/g, '-')
    .replace(/-(max|medium|high|xhigh)-thinking-fast$/i, '')
    .replace(/-(max|medium|high|xhigh)-thinking$/i, '')
    .replace(/-(xhigh|high|medium|fast)$/i, '')
    .replace(/-thinking$/i, '')
    ?? model;
}

function matchesPricingModel(pricingModelId: string, models: ModelUsage[]): boolean {
  const normalizedPricingId = normalizePricingLookupModel(pricingModelId).toLowerCase();
  return models.some((modelUsage) => {
    const normalizedModel = normalizePricingLookupModel(modelUsage.model).toLowerCase();
    return (
      normalizedModel === normalizedPricingId ||
      normalizedModel.startsWith(`${normalizedPricingId}-`) ||
      normalizedPricingId.startsWith(`${normalizedModel}-`) ||
      normalizedModel.includes(normalizedPricingId) ||
      normalizedPricingId.includes(normalizedModel)
    );
  });
}

function formatCompactPath(path?: string | null): string {
  if (!path) return '未检测到本地路径';
  const normalized = path.replace(/\\/g, '/');
  return normalized.length > 44 ? `...${normalized.slice(-44)}` : normalized;
}

// ─── Donut Chart ───
function DonutChart({ data, size = 140, innerRatio = 0.6, centerLabel, centerValue }: {
  data: { label: string; value: number; color: string }[];
  size?: number;
  innerRatio?: number;
  centerLabel?: string;
  centerValue?: string;
}) {
  const total = data.reduce((s, d) => s + d.value, 0);
  if (total === 0) return null;

  const r = size / 2;
  const innerR = r * innerRatio;
  let cumAngle = -Math.PI / 2;

  const arcs = data.filter(d => d.value > 0).map(d => {
    const angle = (d.value / total) * Math.PI * 2;
    const startAngle = cumAngle;
    cumAngle += angle;
    const endAngle = cumAngle;
    const largeArc = angle > Math.PI ? 1 : 0;
    const x1 = r + r * Math.cos(startAngle);
    const y1 = r + r * Math.sin(startAngle);
    const x2 = r + r * Math.cos(endAngle);
    const y2 = r + r * Math.sin(endAngle);
    const ix1 = r + innerR * Math.cos(startAngle);
    const iy1 = r + innerR * Math.sin(startAngle);
    const ix2 = r + innerR * Math.cos(endAngle);
    const iy2 = r + innerR * Math.sin(endAngle);
    const path = `M ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} L ${ix2} ${iy2} A ${innerR} ${innerR} 0 ${largeArc} 0 ${ix1} ${iy1} Z`;
    return { ...d, path, pct: ((d.value / total) * 100).toFixed(1) };
  });

  return (
    <div className="flex flex-col items-center gap-2">
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        {arcs.map((arc, i) => (
          <path key={i} d={arc.path} fill={arc.color} opacity={0.85}>
            <title>{arc.label}: {arc.pct}%</title>
          </path>
        ))}
        {centerLabel && (
          <>
            <text x={r} y={r - 6} textAnchor="middle" fill="currentColor" fontSize={11} opacity={0.5}>{centerLabel}</text>
            <text x={r} y={r + 12} textAnchor="middle" fill="currentColor" fontSize={16} fontWeight="bold">{centerValue}</text>
          </>
        )}
      </svg>
      <div className="flex flex-wrap justify-center gap-x-3 gap-y-1" style={{ maxWidth: size + 80 }}>
        {arcs.map((arc, i) => (
          <div key={i} className="flex items-center gap-1" style={{ fontSize: 11 }}>
            <span style={{ width: 8, height: 8, borderRadius: 2, background: arc.color, flexShrink: 0 }} />
            <span className="opacity-70 truncate" style={{ maxWidth: 80 }}>{arc.label}</span>
            <span className="font-mono opacity-50">{arc.pct}%</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── Bar Chart ───
function BarChart({ data, height = 160 }: {
  data: { label: string; value: number; color: string }[];
  height?: number;
}) {
  const max = Math.max(...data.map(d => d.value), 1);
  const barWidth = Math.min(32, Math.max(12, Math.floor(300 / data.length)));

  return (
    <div className="flex flex-col items-center gap-1" style={{ width: '100%', overflow: 'hidden' }}>
      <div className="flex items-end justify-center gap-1" style={{ height, width: '100%' }}>
        {data.filter(d => d.value > 0).map((d, i) => {
          const h = Math.max((d.value / max) * (height - 20), 2);
          return (
            <div key={i} className="flex flex-col items-center" style={{ width: barWidth }}>
              <span className="font-mono" style={{ fontSize: 9, opacity: 0.6, marginBottom: 2 }}>
                {d.value >= 1_000_000 ? (d.value / 1_000_000).toFixed(1) + 'M' : d.value >= 1_000 ? (d.value / 1_000).toFixed(0) + 'K' : d.value}
              </span>
              <div
                style={{ width: barWidth - 4, height: h, background: d.color, borderRadius: '3px 3px 0 0', transition: 'height 0.3s ease' }}
                title={`${d.label}: ${d.value.toLocaleString()}`}
              />
            </div>
          );
        })}
      </div>
      <div className="flex justify-center gap-1" style={{ width: '100%' }}>
        {data.filter(d => d.value > 0).map((d, i) => (
          <div key={i} style={{ width: barWidth, textAlign: 'center' }}>
            <span className="truncate block" style={{ fontSize: 9, opacity: 0.5, maxWidth: barWidth }}>{d.label.split(' ')[0].slice(0, 5)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── Stacked Trend Chart (time + model breakdown) ───
const STACK_COLORS = [
  '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
  '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1',
];

function StackedTrendChart({ data, height = 260 }: {
  data: ModelTrendData[];
  height?: number;
}) {
  if (data.length === 0) return null;

  const allModels = Array.from(new Set(data.flatMap(d => d.models.map(m => m.model))));
  const topModels = allModels
    .map(m => ({ model: m, total: data.reduce((s, d) => s + (d.models.find(x => x.model === m)?.totalTokens || 0), 0) }))
    .sort((a, b) => b.total - a.total);
  const showModels = topModels.slice(0, STACK_COLORS.length).map(m => m.model);
  const modelColorMap = new Map(showModels.map((m, i) => [m, STACK_COLORS[i]]));

  const maxTokens = Math.max(...data.map(d => d.totalTokens), 1);
  const padding = { top: 20, right: 16, bottom: 36, left: 52 };
  const chartW = Math.max(data.length * 40, 320);
  const chartH = height - padding.top - padding.bottom;
  const barGap = 3;
  const barW = Math.min(28, Math.max(10, (chartW - data.length * barGap) / data.length));

  const yTicks = 4;
  const yLines = Array.from({ length: yTicks + 1 }, (_, i) => {
    const val = (maxTokens / yTicks) * i;
    return { val, y: padding.top + chartH - (val / maxTokens) * chartH };
  });

  const fmtY = (v: number) =>
    v >= 1_000_000 ? (v / 1_000_000).toFixed(1) + 'M'
      : v >= 1_000 ? (v / 1_000).toFixed(0) + 'K'
        : v.toFixed(0);

  const [hovered, setHovered] = useState<{ idx: number; model: string | null } | null>(null);

  return (
    <div>
      <div style={{ width: '100%', overflowX: 'auto' }}>
        <svg width={Math.max(chartW + padding.left + padding.right, 320)} height={height} style={{ display: 'block' }}>
          {yLines.map((yl, i) => (
            <g key={i}>
              <line x1={padding.left} y1={yl.y} x2={chartW + padding.left} y2={yl.y}
                stroke="currentColor" strokeOpacity={0.08} />
              <text x={padding.left - 6} y={yl.y + 3} textAnchor="end" fill="currentColor"
                fontSize={10} opacity={0.4}>{fmtY(yl.val)}</text>
            </g>
          ))}

          {data.map((d, i) => {
            const x = padding.left + i * (barW + barGap) + barW / 2;
            const totalH = Math.max((d.totalTokens / maxTokens) * chartH, 1);
            const barX = x - barW / 2;
            let curY = padding.top + chartH;
            const isHoveredBar = hovered?.idx === i;

            const segments = showModels
              .map(model => {
                const mu = d.models.find(m => m.model === model);
                return mu ? { model, tokens: mu.totalTokens, cost: mu.totalCost, color: modelColorMap.get(model) || '#666' } : null;
              })
              .filter(Boolean) as { model: string; tokens: number; cost: number; color: string }[];

            const otherTokens = d.totalTokens - segments.reduce((s, seg) => s + seg.tokens, 0);
            if (otherTokens > 0) {
              segments.push({ model: 'other', tokens: otherTokens, cost: 0, color: '#475569' });
            }

            return (
              <g key={i}
                onMouseEnter={() => setHovered({ idx: i, model: null })}
                onMouseLeave={() => setHovered(null)}
                style={{ cursor: 'pointer' }}
              >
                {segments.map((seg, si) => {
                  const segH = (seg.tokens / d.totalTokens) * totalH;
                  curY -= segH;
                  return (
                    <rect key={si} x={barX} y={curY} width={barW} height={Math.max(segH, 0.5)}
                      fill={seg.color} opacity={isHoveredBar ? 1 : 0.75}
                      onMouseEnter={(e) => { e.stopPropagation(); setHovered({ idx: i, model: seg.model }); }}
                    />
                  );
                })}
                {(data.length <= 31 || i % Math.ceil(data.length / 15) === 0) && (
                  <text x={x} y={padding.top + chartH + 14} textAnchor="middle"
                    fill="currentColor" fontSize={9} opacity={0.45}
                    transform={data.length > 12 ? `rotate(-35 ${x} ${padding.top + chartH + 14})` : ''}>
                    {d.period.length > 10 ? d.period.slice(11, 16) : d.period.slice(5)}
                  </text>
                )}
              </g>
            );
          })}
        </svg>
      </div>

      {hovered && data[hovered.idx] && (
        <div className="oc-mcp-card" style={{ marginTop: 8, padding: '8px 12px', fontSize: 12 }}>
          <div className="flex items-center justify-between gap-4 flex-wrap">
            <span className="font-semibold">{data[hovered.idx].period}</span>
            <span className="opacity-60">
              {fmtY(data[hovered.idx].totalTokens)} tokens &middot; ${data[hovered.idx].totalCost.toFixed(4)}
            </span>
          </div>
          <div className="flex flex-wrap gap-x-4 gap-y-1" style={{ marginTop: 6 }}>
            {data[hovered.idx].models
              .sort((a, b) => b.totalTokens - a.totalTokens)
              .slice(0, 8)
              .map((m, mi) => (
                <div key={mi} className="flex items-center gap-1.5" style={{
                  fontWeight: hovered.model === m.model ? 700 : 400,
                  opacity: hovered.model && hovered.model !== m.model ? 0.4 : 1,
                }}>
                  <span style={{ width: 8, height: 8, borderRadius: 2, background: modelColorMap.get(m.model) || '#475569', flexShrink: 0 }} />
                  <span className="font-mono truncate" style={{ maxWidth: 140 }}>{m.model}</span>
                  <span className="opacity-60">{fmtY(m.totalTokens)}</span>
                  <span className="text-success">${m.totalCost.toFixed(4)}</span>
                </div>
              ))}
          </div>
        </div>
      )}

      <div className="flex flex-wrap gap-x-3 gap-y-1" style={{ marginTop: 8 }}>
        {showModels.map((m, i) => (
          <div key={i} className="flex items-center gap-1" style={{ fontSize: 11 }}>
            <span style={{ width: 8, height: 8, borderRadius: 2, background: modelColorMap.get(m) || '#666', flexShrink: 0 }} />
            <span className="opacity-70 truncate font-mono" style={{ maxWidth: 100 }}>{m}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function UsagePage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [trend, setTrend] = useState<UsageTrend[]>([]);
  const [providerStats, setProviderStats] = useState<ProviderStats[]>([]);
  const [projectStats, setProjectStats] = useState<ProjectStats[]>([]);
  const [projectTrendDetails, setProjectTrendDetails] = useState<Record<string, UsageTrend[]>>({});
  const [pricing, setPricing] = useState<ModelPricing[]>([]);
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [activeTab, setActiveTab] = useState<'overview' | 'projects' | 'trend' | 'pricing'>('overview');
  const [period, setPeriod] = useState<string>('all');
  const [loadError, setLoadError] = useState<string | null>(null);
  const [pricingSearch, setPricingSearch] = useState('');
  const [projectSearch, setProjectSearch] = useState('');
  const [projectSort, setProjectSort] = useState<'tokens_desc' | 'cost_desc' | 'requests_desc' | 'name_asc'>('tokens_desc');
  const [expandedPlatform, setExpandedPlatform] = useState<string | null>(null);
  const [modelDetails, setModelDetails] = useState<Record<string, ModelUsage[]>>({});
  const [loadingDetails, setLoadingDetails] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<ScanResult | null>(() => cachedUsageScanResult);
  const [scanLoading, setScanLoading] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);
  const [scanExpanded, setScanExpanded] = useState(false);
  const [selectedSource, setSelectedSource] = useState<string | null>(null);

  // Custom pricing state
  const [editingPricing, setEditingPricing] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({ input: '', output: '', cacheRead: '', cacheCreation: '' });
  const [providerPricing, setProviderPricing] = useState<ProviderModelPricing[]>([]);
  const [selectedModelUsage, setSelectedModelUsage] = useState<ModelUsage[]>([]);
  const [modelTrendRaw, setModelTrendRaw] = useState<ModelTrendData[]>([]);
  const [addingPricing, setAddingPricing] = useState(false);
  const [newPricingValues, setNewPricingValues] = useState({ modelId: '', displayName: '', input: '', output: '', cacheRead: '', cacheCreation: '' });

  const selectedProviderId = selectedSource ? `${selectedSource}_local` : undefined;
  const selectedModelKey = selectedProviderId || '__total__';

  const loadData = useCallback(async (background = false) => {
    if (!background) setLoading(true);
    setLoadError(null);
    try {
      const [s, tr, ps, pj, pr, pp, mt] = await Promise.all([
        invoke<UsageSummary>('get_proxy_usage_summary', {
          period,
          providerId: selectedProviderId,
        }).catch(() => null),
        invoke<UsageTrend[]>('get_proxy_usage_trend', {
          period,
          providerId: selectedProviderId,
        }).catch(() => []),
        invoke<ProviderStats[]>('get_provider_stats', { period }).catch(() => []),
        invoke<ProjectStats[]>('get_project_stats', {
          period,
          providerId: selectedProviderId,
        }).catch(() => []),
        invoke<ModelPricing[]>('get_model_pricing_list').catch(() => []),
        invoke<ProviderModelPricing[]>('get_all_provider_pricing').catch(() => []),
        invoke<ModelTrendData[]>('get_proxy_usage_trend_by_model', {
          period,
          providerId: selectedProviderId,
        }).catch(() => []),
      ]);
      const mtArr = Array.isArray(mt) ? mt : [];
      const aggregatedModels = aggregateModelUsage(mtArr);
      setSummary(s);
      setTrend(Array.isArray(tr) ? tr : []);
      setProviderStats(Array.isArray(ps) ? ps : []);
      setProjectStats(Array.isArray(pj) ? pj : []);
      setPricing(Array.isArray(pr) ? pr : []);
      setProviderPricing(Array.isArray(pp) ? pp : []);
      setSelectedModelUsage(aggregatedModels);
      setModelTrendRaw(mtArr);
      setModelDetails(prev => ({ ...prev, [selectedModelKey]: aggregatedModels }));
    } catch (e) {
      setLoadError(String(e));
    } finally {
      if (!background) setLoading(false);
    }
  }, [period, selectedProviderId, selectedModelKey]);

  const loadScanResult = useCallback(async () => {
    setScanLoading(true);
    setScanError(null);
    try {
      const result = await invoke<ScanResult>('scan_local_logs');
      cachedUsageScanResult = result;
      setScanResult(result);
    } catch (e) {
      setScanError(String(e));
    } finally {
      setScanLoading(false);
    }
  }, []);

  const refreshUsageData = useCallback(async (background = false) => {
    await Promise.all([loadData(background), loadScanResult()]);
  }, [loadData, loadScanResult]);

  useEffect(() => {
    let cancelled = false;

    const init = async () => {
      await loadData();
      if (cancelled) return;

      if (!usageAutoImportTriggered) {
        usageAutoImportTriggered = true;
        setImporting(true);
        setScanLoading(true);
        setScanError(null);
        try {
          await invoke('auto_import_local_logs');
          if (!cancelled) {
            await loadData(true);
          }
        } catch {
          // 自动导入失败时保留当前统计数据，避免首次打开整页失败。
        } finally {
          if (!cancelled) {
            await loadScanResult();
            setImporting(false);
          }
        }
        return;
      }

      if (!cachedUsageScanResult) {
        void loadScanResult();
      }
    };

    void init();
    return () => {
      cancelled = true;
    };
  }, [loadData, loadScanResult]);

  const handleClear = async () => {
    if (!confirm(t('usage.confirmClear', '确定要清空所有用量数据吗？'))) return;
    try {
      await invoke('clear_proxy_usage_stats');
      setModelDetails({});
      setProjectTrendDetails({});
      setExpandedPlatform(null);
      setSelectedModelUsage([]);
      await refreshUsageData();
      toast.success(t('usage.cleared', '数据已清空'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleAutoImport = async () => {
    if (importing) return;
    setImporting(true);
    try {
      const count = await invoke<number>('auto_import_local_logs');
      usageAutoImportTriggered = true;
      await refreshUsageData(true);
      toast.success(t('usage.imported', '导入完成') + (count ? ` (+${count})` : ''));
    } catch (e) { toast.error(String(e)); }
    finally { setImporting(false); }
  };

  const togglePlatformDetail = async (providerId: string) => {
    if (expandedPlatform === providerId) {
      setExpandedPlatform(null);
      return;
    }
    setExpandedPlatform(providerId);
    if (modelDetails[providerId]) return;
    setLoadingDetails(providerId);
    try {
      const data = await invoke<ModelTrendData[]>('get_proxy_usage_trend_by_model', { period, providerId });
      const models = aggregateModelUsage(Array.isArray(data) ? data : []);
      setModelDetails(prev => ({ ...prev, [providerId]: models }));
    } catch {
      setModelDetails(prev => ({ ...prev, [providerId]: [] }));
    } finally { setLoadingDetails(null); }
  };

  const toggleTotalDetail = async () => {
    const key = '__total__';
    if (expandedPlatform === key) {
      setExpandedPlatform(null);
      return;
    }
    setExpandedPlatform(key);
    if (modelDetails[key]) return;
    setLoadingDetails(key);
    try {
      const data = await invoke<ModelTrendData[]>('get_proxy_usage_trend_by_model', { period });
      const models = aggregateModelUsage(Array.isArray(data) ? data : []);
      setModelDetails(prev => ({ ...prev, [key]: models }));
    } catch {
      setModelDetails(prev => ({ ...prev, [key]: [] }));
    } finally { setLoadingDetails(null); }
  };

  const toggleProjectDetail = async (projectName: string) => {
    const key = `project:${projectName}`;
    if (expandedPlatform === key) {
      setExpandedPlatform(null);
      return;
    }
    setExpandedPlatform(key);
    if (modelDetails[key] && projectTrendDetails[key]) return;
    setLoadingDetails(key);
    try {
      const [modelTrendData, projectTrendData] = await Promise.all([
        invoke<ModelTrendData[]>('get_proxy_usage_trend_by_model', {
          period,
          providerId: selectedProviderId,
          projectName,
        }),
        invoke<UsageTrend[]>('get_proxy_usage_trend', {
          period,
          providerId: selectedProviderId,
          projectName,
        }),
      ]);
      const models = aggregateModelUsage(Array.isArray(modelTrendData) ? modelTrendData : []);
      setModelDetails(prev => ({ ...prev, [key]: models }));
      setProjectTrendDetails(prev => ({
        ...prev,
        [key]: Array.isArray(projectTrendData) ? projectTrendData : [],
      }));
    } catch {
      setModelDetails(prev => ({ ...prev, [key]: [] }));
      setProjectTrendDetails(prev => ({ ...prev, [key]: [] }));
    } finally {
      setLoadingDetails(null);
    }
  };

  useEffect(() => {
    setModelDetails({});
    setProjectTrendDetails({});
    setExpandedPlatform(null);
    setSelectedModelUsage([]);
  }, [period, selectedProviderId]);

  // ─── Pricing CRUD ───
  const startEditPricing = (modelId: string, current: ModelPricing) => {
    setEditingPricing(modelId);
    setEditValues({
      input: current.inputCostPerMillion || '0',
      output: current.outputCostPerMillion || '0',
      cacheRead: current.cacheReadCostPerMillion || '0',
      cacheCreation: current.cacheCreationCostPerMillion || '0',
    });
  };

  const savePricing = async (modelId: string) => {
    try {
      await invoke('update_model_pricing', {
        modelId,
        inputCost: editValues.input,
        outputCost: editValues.output,
        cacheReadCost: editValues.cacheRead,
        cacheCreationCost: editValues.cacheCreation,
      });
      setEditingPricing(null);
      await loadData();
      toast.success(t('usage.pricingSaved', '价格已更新'));
    } catch (e) { toast.error(String(e)); }
  };

  const resetAllPricing = async () => {
    if (!confirm(t('usage.confirmResetPricing', '确定要重置所有模型价格为默认值吗？'))) return;
    try {
      await invoke('reset_model_pricing');
      await loadData();
      toast.success(t('usage.pricingReset', '已重置为默认价格'));
    } catch (e) { toast.error(String(e)); }
  };

  const saveNewPricing = async () => {
    if (!newPricingValues.modelId.trim()) return;
    try {
      await invoke('add_model_pricing', {
        modelId: newPricingValues.modelId.trim(),
        displayName: newPricingValues.displayName.trim() || newPricingValues.modelId.trim(),
        inputCost: newPricingValues.input || '0',
        outputCost: newPricingValues.output || '0',
        cacheReadCost: newPricingValues.cacheRead || '0',
        cacheCreationCost: newPricingValues.cacheCreation || '0',
      });
      setAddingPricing(false);
      setNewPricingValues({ modelId: '', displayName: '', input: '', output: '', cacheRead: '', cacheCreation: '' });
      await loadData();
      toast.success(t('usage.pricingAdded', '模型定价已添加'));
    } catch (e) { toast.error(String(e)); }
  };

  const unpricedModels = useMemo(() => {
    const pricedIds = new Set(pricing.map(p => p.modelId.toLowerCase()));
    return selectedModelUsage
      .filter(m => {
        const normalized = m.model.toLowerCase().split(',')[0].trim();
        return !pricedIds.has(normalized) && m.totalCost === 0 && m.totalTokens > 0;
      })
      .sort((a, b) => b.totalTokens - a.totalTokens);
  }, [selectedModelUsage, pricing]);

  const formatNumber = (n: number) => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return String(n);
  };

  const totalTokens = summary
    ? (
        summary.totalInputTokens +
        summary.totalOutputTokens +
        summary.totalCacheCreationTokens +
        summary.totalCacheReadTokens
      )
    : 0;
  const totalCost = summary ? parseFloat(summary.totalCost) || 0 : 0;

  const filteredPricing = useMemo(() => {
    const keyword = pricingSearch.trim().toLowerCase();
    let items = selectedModelUsage.length > 0
      ? pricing.filter((item) => matchesPricingModel(item.modelId, selectedModelUsage))
      : pricing.slice();

    if (keyword) {
      items = items.filter(item =>
        item.modelId.toLowerCase().includes(keyword) ||
        item.displayName.toLowerCase().includes(keyword)
      );
    }

    return items;
  }, [pricing, pricingSearch, selectedModelUsage]);

  const localSources = useMemo(
    () =>
      scanResult
        ? [
            {
              id: 'claude',
              label: getPlatformLabel('claude-code', t),
              platformId: 'claude-code' as PlatformId,
              files: scanResult.claudeFiles,
              entries: scanResult.claudeEntries,
              path: scanResult.claudePath,
            },
            {
              id: 'codex',
              label: getPlatformLabel('codex', t),
              platformId: 'codex' as PlatformId,
              files: scanResult.codexFiles,
              entries: scanResult.codexEntries,
              path: scanResult.codexPath,
            },
            {
              id: 'gemini',
              label: getPlatformLabel('gemini', t),
              platformId: 'gemini' as PlatformId,
              files: scanResult.geminiFiles,
              entries: scanResult.geminiEntries,
              path: scanResult.geminiPath,
            },
            {
              id: 'opencode',
              label: getPlatformLabel('opencode', t),
              platformId: 'opencode' as PlatformId,
              files: scanResult.opencodeFiles,
              entries: scanResult.opencodeEntries,
              path: scanResult.opencodePath,
            },
            {
              id: 'cursor',
              label: getPlatformLabel('cursor', t),
              platformId: 'cursor' as PlatformId,
              files: scanResult.cursorFiles,
              entries: scanResult.cursorEntries,
              path: scanResult.cursorPath,
            },
            {
              id: 'windsurf',
              label: getPlatformLabel('windsurf', t),
              platformId: 'windsurf' as PlatformId,
              files: scanResult.windsurfFiles,
              entries: scanResult.windsurfEntries,
              path: scanResult.windsurfPath,
            },
            {
              id: 'kiro',
              label: getPlatformLabel('kiro', t),
              platformId: 'kiro' as PlatformId,
              files: scanResult.kiroFiles,
              entries: scanResult.kiroEntries,
              path: scanResult.kiroPath,
            },
            {
              id: 'antigravity',
              label: getPlatformLabel('antigravity', t),
              platformId: 'antigravity' as PlatformId,
              files: scanResult.antigravityFiles,
              entries: scanResult.antigravityEntries,
              path: scanResult.antigravityPath,
            },
            {
              id: 'warp',
              label: getPlatformLabel('warp', t),
              platformId: 'warp' as PlatformId,
              files: scanResult.warpFiles,
              entries: scanResult.warpEntries,
              path: scanResult.warpPath,
            },
            {
              id: 'augment',
              label: getPlatformLabel('augment', t),
              platformId: 'augment' as PlatformId,
              files: scanResult.augmentFiles,
              entries: scanResult.augmentEntries,
              path: scanResult.augmentPath,
            },
            {
              id: 'github-copilot',
              label: getPlatformLabel('github-copilot', t),
              platformId: 'github-copilot' as PlatformId,
              files: scanResult.githubCopilotFiles,
              entries: scanResult.githubCopilotEntries,
              path: scanResult.githubCopilotPath,
            },
            {
              id: 'codebuddy',
              label: getPlatformLabel('codebuddy', t),
              platformId: 'codebuddy' as PlatformId,
              files: scanResult.codebuddyFiles,
              entries: scanResult.codebuddyEntries,
              path: scanResult.codebuddyPath,
            },
            {
              id: 'codebuddy_cn',
              label: getPlatformLabel('codebuddy_cn', t),
              platformId: 'codebuddy_cn' as PlatformId,
              files: scanResult.codebuddyCnFiles,
              entries: scanResult.codebuddyCnEntries,
              path: scanResult.codebuddyCnPath,
            },
            {
              id: 'qoder',
              label: getPlatformLabel('qoder', t),
              platformId: 'qoder' as PlatformId,
              files: scanResult.qoderFiles,
              entries: scanResult.qoderEntries,
              path: scanResult.qoderPath,
            },
            {
              id: 'trae',
              label: getPlatformLabel('trae', t),
              platformId: 'trae' as PlatformId,
              files: scanResult.traeFiles,
              entries: scanResult.traeEntries,
              path: scanResult.traePath,
            },
            {
              id: 'workbuddy',
              label: getPlatformLabel('workbuddy', t),
              platformId: 'workbuddy' as PlatformId,
              files: scanResult.workbuddyFiles,
              entries: scanResult.workbuddyEntries,
              path: scanResult.workbuddyPath,
            },
            {
              id: 'openclaw',
              label: getPlatformLabel('openclaw', t),
              platformId: 'openclaw' as PlatformId,
              files: scanResult.openclawFiles,
              entries: scanResult.openclawEntries,
              path: scanResult.openclawPath,
            },
          ]
        : [],
    [scanResult, t],
  );
  const detectedSourceCount = localSources.filter(
    (source) => source.files > 0 || source.entries > 0 || !!source.path,
  ).length;
  const estimatedLocalEntries = localSources.reduce((sum, source) => sum + source.entries, 0);

  const statsMap = new Map(providerStats.map(ps => [ps.providerId, ps]));
  const detectedLocalSources = useMemo(
    () => localSources
      .filter(source => source.files > 0 || source.entries > 0 || !!source.path)
      .sort((a, b) => {
        const aTokens = statsMap.get(`${a.id}_local`)?.totalTokens || 0;
        const bTokens = statsMap.get(`${b.id}_local`)?.totalTokens || 0;
        if (bTokens !== aTokens) return bTokens - aTokens;
        return b.entries - a.entries;
      }),
    [localSources, statsMap],
  );
  const selectedSourceInfo = selectedSource
    ? localSources.find(source => source.id === selectedSource) || null
    : null;
  const selectedProviderStats = selectedProviderId
    ? statsMap.get(selectedProviderId) || null
    : null;
  const activeProjects = projectStats.filter(project => project.totalTokens > 0);
  const filteredProjects = useMemo(() => {
    const keyword = projectSearch.trim().toLowerCase();
    const items = keyword
      ? activeProjects.filter(project => project.projectName.toLowerCase().includes(keyword))
      : activeProjects.slice();

    items.sort((a, b) => {
      switch (projectSort) {
        case 'cost_desc':
          return (parseFloat(b.totalCost) || 0) - (parseFloat(a.totalCost) || 0);
        case 'requests_desc':
          return b.requestCount - a.requestCount;
        case 'name_asc':
          return a.projectName.localeCompare(b.projectName);
        case 'tokens_desc':
        default:
          return b.totalTokens - a.totalTokens;
      }
    });

    return items;
  }, [activeProjects, projectSearch, projectSort]);

  const mergedPlatforms = ALL_PLATFORMS.map(platform => ({
    ...platform,
    stats: statsMap.get(platform.providerId) || null,
  }));

  const unknownProviders = providerStats.filter(ps => !KNOWN_PROVIDER_IDS.has(ps.providerId));

  useEffect(() => {
    if (detectedLocalSources.length === 0) return;
    if (!selectedSource || !localSources.some(source => source.id === selectedSource)) {
      setSelectedSource(detectedLocalSources[0].id);
    }
  }, [detectedLocalSources, localSources, selectedSource]);

  // ─── Chart data ───
  const chartTokenData = useMemo(() => {
    return trend
      .filter(point => point.totalTokens > 0)
      .map((point, i) => {
        const shortLabel = point.period.includes(' ')
          ? point.period.slice(11, 16)
          : point.period.length > 10
            ? point.period.slice(5, 10)
            : point.period.slice(-5);
        return {
          label: shortLabel,
          value: point.totalTokens,
          color: CHART_COLORS[i % CHART_COLORS.length],
        };
      });
  }, [trend]);

  const chartModelData = useMemo(() => {
    return selectedModelUsage
      .filter(model => model.totalTokens > 0)
      .sort((a, b) => b.totalTokens - a.totalTokens)
      .slice(0, 8)
      .map((model, i) => ({
        label: model.model,
        value: model.totalTokens,
        color: CHART_COLORS[i % CHART_COLORS.length],
      }));
  }, [selectedModelUsage]);

  const statCards = [
    { icon: <Cpu size={20} />, label: t('usage.totalTokens', '总 Token 数'), value: formatNumber(totalTokens), variant: 'blue' as const },
    { icon: <DollarSign size={20} />, label: t('usage.totalCost', '总费用'), value: `$${totalCost.toFixed(4)}`, variant: 'green' as const },
    { icon: <MessageSquare size={20} />, label: t('usage.requests', '请求数'), value: formatNumber(summary?.totalRequests || 0), variant: 'purple' as const },
    { icon: <Layers size={20} />, label: t('usage.activeDays', '活跃天数'), value: formatNumber(summary?.activeDays || 0), variant: 'orange' as const },
  ];

  const periodLabel = (p: string) => {
    switch (p) {
      case '24h': return '24 ' + t('usage.hours', '小时');
      case '7d': return '7 ' + t('usage.days', '天');
      case '30d': return '30 ' + t('usage.days', '天');
      case 'all': return t('usage.all', '全部');
      default: return p;
    }
  };

  // check if a model has provider-level custom pricing
  const getProviderCustomPrice = (providerId: string, modelId: string) => {
    return providerPricing.find(pp => pp.providerId === providerId && pp.modelId === modelId);
  };

  const renderModelDetails = (key: string) => {
    if (loadingDetails === key) {
      return (
        <div className="flex justify-center py-3">
          <Loader2 size={16} className="animate-spin" style={{ color: 'var(--primary)' }} />
        </div>
      );
    }
    const models = modelDetails[key];
    if (!models || models.length === 0) {
      return (
        <div className="text-center py-3 opacity-50" style={{ fontSize: '12px' }}>
          {t('usage.noModelData', '暂无模型数据')}
        </div>
      );
    }
    return (
      <div style={{ overflow: 'hidden' }}>
        <table className="table table-xs" style={{ fontSize: '12px', margin: 0 }}>
          <thead>
            <tr style={{ opacity: 0.6 }}>
              <th style={{ paddingLeft: 0 }}>{t('usage.model', '模型')}</th>
              <th className="text-right">{t('usage.inputTokensShort', '输入')}</th>
              <th className="text-right">{t('usage.outputTokensShort', '输出')}</th>
              <th className="text-right">{t('usage.cacheRead', '缓存')}</th>
              <th className="text-right">Tokens</th>
              <th className="text-right">{t('usage.costLabel', '费用')}</th>
              <th className="text-right">{t('usage.requestsShort', '请求')}</th>
            </tr>
          </thead>
          <tbody>
            {models.map(m => {
              const customPrice = key !== '__total__' && !key.startsWith('project:')
                ? getProviderCustomPrice(key, m.model)
                : null;
              return (
                <tr key={m.model}>
                  <td className="font-mono" style={{ paddingLeft: 0, maxWidth: 180, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {m.model}
                    {customPrice && <span style={{ fontSize: 9, color: 'var(--primary)', marginLeft: 4 }}>&#9733;</span>}
                  </td>
                  <td className="text-right font-mono">{formatNumber(m.inputTokens)}</td>
                  <td className="text-right font-mono">{formatNumber(m.outputTokens)}</td>
                  <td className="text-right font-mono">
                    {formatNumber(m.totalCacheCreationTokens + m.totalCacheReadTokens)}
                  </td>
                  <td className="text-right font-mono font-bold">{formatNumber(m.totalTokens)}</td>
                  <td className="text-right font-mono text-success">${m.totalCost.toFixed(4)}</td>
                  <td className="text-right font-mono">{m.requestCount}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    );
  };

  const renderProjectTrend = (key: string) => {
    const points = projectTrendDetails[key];
    if (!points || points.length === 0) return null;

    const visiblePoints = points.length > 10 ? points.slice(-10) : points;
    const maxTokens = Math.max(
      ...visiblePoints.map(point => point.totalTokens),
      1,
    );

    return (
      <div style={{ marginBottom: 10 }}>
        <div className="flex items-center justify-between" style={{ marginBottom: 6 }}>
          <div className="text-xs font-medium opacity-60">
            {t('usage.projectTrendTitle', '项目趋势')}
          </div>
          <div className="text-xs opacity-45">
            {points.length > visiblePoints.length
              ? t('usage.projectTrendRecentHint', '最近 {{count}} 个时间桶', { count: visiblePoints.length })
              : t('usage.projectTrendFullHint', '当前时间范围')}
          </div>
        </div>
        <div className="space-y-1">
          {visiblePoints.map(point => {
            const tokens = point.totalTokens;
            return (
              <div key={point.period} className="oc-trend-row" style={{ minHeight: 26 }}>
                <span className="oc-trend-date">
                  {point.period.length > 10 ? point.period.slice(11, 16) : point.period.slice(5)}
                </span>
                <div className="oc-trend-bar-wrap">
                  <div
                    className="oc-trend-bar"
                    style={{ width: `${Math.max((tokens / maxTokens) * 100, 1)}%` }}
                  />
                </div>
                <span className="oc-trend-tokens">{formatNumber(tokens)}</span>
                <span className="oc-trend-cost">${(point.totalCost || 0).toFixed(3)}</span>
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  const renderPlatformRow = (
    key: string,
    icon: React.ReactNode,
    name: string,
    stats: ProviderStats | null,
    expandKey: string,
    onToggle: () => void,
    isTotal = false,
  ) => {
    const hasData = stats && stats.totalTokens > 0;
    const isExpanded = expandedPlatform === expandKey;
    return (
      <div
        key={key}
        className="oc-mcp-card"
        style={{
          ...(hasData ? {} : { opacity: 0.45 }),
          ...(isTotal ? { borderLeft: '3px solid var(--primary)', background: 'rgba(59,130,246,0.03)' } : {}),
        }}
      >
        <div
          className="flex items-center justify-between"
          style={{ cursor: hasData ? 'pointer' : 'default', padding: 0 }}
          onClick={() => hasData && onToggle()}
        >
          <div className="flex items-center gap-3">
            {hasData ? (
              isExpanded ? <ChevronDown size={14} style={{ flexShrink: 0, opacity: 0.5 }} /> : <ChevronRight size={14} style={{ flexShrink: 0, opacity: 0.5 }} />
            ) : (
              <span style={{ width: 14, flexShrink: 0 }} />
            )}
            <div className="flex-shrink-0" style={{ width: 22, height: 22, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              {icon}
            </div>
            <div>
              <div className="text-sm font-medium">{name}</div>
              <div className="text-xs opacity-50">
                {stats ? `${stats.requestCount} requests` : '0 requests'}
              </div>
            </div>
          </div>
          <div className="text-right" style={{ minWidth: 120 }}>
            <div className="text-sm font-bold">
              {stats ? formatNumber(stats.totalTokens) : '0'} tokens
            </div>
            <div className="text-xs text-success">
              ${stats ? (parseFloat(stats.totalCost) || 0).toFixed(4) : '0.0000'}
            </div>
          </div>
        </div>
        {isExpanded && hasData && (
          <div style={{ borderTop: '1px solid var(--border)', marginTop: 8, paddingTop: 8 }}>
            {stats && (
              <div className="flex gap-4 flex-wrap" style={{ fontSize: '12px', marginBottom: 8, opacity: 0.7 }}>
                <span>{t('usage.inputTokensShort', '输入')}: {formatNumber(stats.totalInputTokens)}</span>
                <span>{t('usage.outputTokensShort', '输出')}: {formatNumber(stats.totalOutputTokens)}</span>
                {stats.totalCacheCreationTokens > 0 && <span>{t('usage.cacheWrite', '缓存写入')}: {formatNumber(stats.totalCacheCreationTokens)}</span>}
                {stats.totalCacheReadTokens > 0 && <span>{t('usage.cacheRead', '缓存读取')}: {formatNumber(stats.totalCacheReadTokens)}</span>}
              </div>
            )}
            {renderModelDetails(expandKey)}
          </div>
        )}
      </div>
    );
  };

  const renderProjectRow = (project: ProjectStats) => {
    const detailKey = `project:${project.projectName}`;
    const isExpanded = expandedPlatform === detailKey;

    return (
      <div
        key={project.projectName}
        className="oc-mcp-card"
        style={{
          borderLeft: '3px solid rgba(16,185,129,0.45)',
          background: isExpanded ? 'rgba(16,185,129,0.04)' : undefined,
        }}
      >
        <div
          className="flex items-center justify-between"
          style={{ cursor: 'pointer', padding: 0 }}
          onClick={() => void toggleProjectDetail(project.projectName)}
        >
          <div className="flex items-center gap-3 min-w-0">
            {isExpanded ? (
              <ChevronDown size={14} style={{ flexShrink: 0, opacity: 0.5 }} />
            ) : (
              <ChevronRight size={14} style={{ flexShrink: 0, opacity: 0.5 }} />
            )}
            <div
              className="flex-shrink-0"
              style={{
                width: 24,
                height: 24,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: 8,
                background: 'rgba(16,185,129,0.12)',
                color: '#10b981',
              }}
            >
              <FolderOpen size={15} />
            </div>
            <div className="min-w-0">
              <div
                className="text-sm font-medium truncate"
                title={project.projectName}
                style={{ maxWidth: 380 }}
              >
                {project.projectName}
              </div>
              <div className="text-xs opacity-50">
                {project.requestCount} requests · {project.providerCount} providers · {project.modelCount} models
              </div>
            </div>
          </div>
          <div className="text-right" style={{ minWidth: 132 }}>
            <div className="text-sm font-bold">{formatNumber(project.totalTokens)} tokens</div>
            <div className="text-xs text-success">
              ${(parseFloat(project.totalCost) || 0).toFixed(4)}
            </div>
          </div>
        </div>

        {isExpanded && (
          <div style={{ borderTop: '1px solid var(--border)', marginTop: 8, paddingTop: 8 }}>
            <div className="flex gap-4 flex-wrap" style={{ fontSize: '12px', marginBottom: 8, opacity: 0.7 }}>
              <span>{t('usage.inputTokensShort', '输入')}: {formatNumber(project.totalInputTokens)}</span>
              <span>{t('usage.outputTokensShort', '输出')}: {formatNumber(project.totalOutputTokens)}</span>
              {project.totalCacheCreationTokens > 0 && (
                <span>{t('usage.cacheWrite', '缓存写入')}: {formatNumber(project.totalCacheCreationTokens)}</span>
              )}
              {project.totalCacheReadTokens > 0 && (
                <span>{t('usage.cacheRead', '缓存读取')}: {formatNumber(project.totalCacheReadTokens)}</span>
              )}
            </div>
            {renderProjectTrend(detailKey)}
            {renderModelDetails(detailKey)}
          </div>
        )}
      </div>
    );
  };

  const totalStats: ProviderStats | null = summary ? {
    providerId: selectedModelKey,
    providerName: selectedSourceInfo?.label || t('usage.total', '总计'),
    requestCount: summary.totalRequests,
    totalTokens:
      summary.totalInputTokens +
      summary.totalOutputTokens +
      summary.totalCacheCreationTokens +
      summary.totalCacheReadTokens,
    totalInputTokens: summary.totalInputTokens,
    totalOutputTokens: summary.totalOutputTokens,
    totalCacheCreationTokens: summary.totalCacheCreationTokens,
    totalCacheReadTokens: summary.totalCacheReadTokens,
    totalCost: summary.totalCost,
    successRate: summary.successRate,
  } : null;

  return (
    <div className="h-full flex flex-col">
      <ToastContainer toasts={toast.toasts} />

      <div className="oc-page-header flex-shrink-0 px-4 pt-4">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--usage"><BarChart3 size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('usage.pageTitle', '用量统计')}</h2>
            <p className="oc-page-subtitle">{t('usage.pageSubtitle', '跟踪 Token 用量、成本和请求趋势')}</p>
          </div>
        </div>
        <div className="oc-page-header-actions">
          <button className="btn btn-sm btn-ghost" onClick={handleAutoImport} disabled={importing}>
            {importing ? <Loader2 size={14} className="animate-spin" /> : <Upload size={14} />}
            {' '}{t('usage.autoImport', '自动导入')}
          </button>
          <button
            className="btn btn-sm btn-ghost"
            onClick={() => { void refreshUsageData(); }}
            disabled={loading || scanLoading || importing}
          >
            <RefreshCw size={14} className={loading || scanLoading ? 'animate-spin' : ''} />
          </button>
          <button className="btn btn-sm btn-ghost text-error" onClick={handleClear}><Trash2 size={14} /></button>
        </div>
      </div>

      {importing && (
        <div className="flex items-center gap-2 px-4 mx-4 mt-2 py-2 rounded-lg flex-shrink-0" style={{ background: 'rgba(59,130,246,0.08)', border: '1px solid rgba(59,130,246,0.2)' }}>
          <Loader2 size={14} className="animate-spin" style={{ color: 'var(--primary)' }} />
          <span style={{ fontSize: '13px', color: 'var(--primary)' }}>{t('usage.importingHint', '正在扫描本地 AI 工具日志并导入...')}</span>
        </div>
      )}

      {loadError && (
        <div className="p-4 mx-4 mt-2 rounded-lg flex-shrink-0" style={{ background: 'rgba(239,68,68,0.1)', border: '1px solid rgba(239,68,68,0.3)' }}>
          <p style={{ fontSize: '13px', color: 'var(--danger)' }}>{loadError}</p>
          <button className="btn btn-sm btn-primary" onClick={() => { void loadData(); }} style={{ marginTop: '8px' }}>
            <RefreshCw size={14} /> {t('common.retry', '重试')}
          </button>
        </div>
      )}

      {loading && !importing ? (
        <div className="flex-1 flex justify-center items-center"><Loader2 className="animate-spin" size={32} /></div>
      ) : (
        <div className="flex-1 min-h-0 overflow-y-auto p-4 flex flex-col gap-4">
          <div className="flex items-center justify-end">
            <select className="select select-sm select-bordered" value={period} onChange={e => setPeriod(e.target.value)}>
              <option value="24h">{periodLabel('24h')}</option>
              <option value="7d">{periodLabel('7d')}</option>
              <option value="30d">{periodLabel('30d')}</option>
              <option value="all">{periodLabel('all')}</option>
            </select>
          </div>

          {(scanLoading || scanResult || scanError) && (
            <div className="oc-mcp-card">
              <div className="flex flex-col gap-3">
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-start gap-3 min-w-0">
                    <div
                      className="oc-page-icon"
                      style={{
                        width: 40,
                        height: 40,
                        borderRadius: 12,
                        background: 'rgba(59,130,246,0.10)',
                        color: 'var(--primary)',
                        boxShadow: 'none',
                        flexShrink: 0,
                      }}
                    >
                      {scanLoading ? <Loader2 size={18} className="animate-spin" /> : <Search size={18} />}
                    </div>
                    <div className="min-w-0">
                      <div className="text-sm font-semibold">{t('usage.localSourcesTitle', '本地来源扫描')}</div>
                      <div className="text-xs opacity-60" style={{ maxWidth: 680 }}>
                        {t(
                          'usage.localSourcesSubtitle',
                          '扫描本机日志和数据库，确认哪些 AI 客户端已经被用量统计覆盖。路径仅在本地读取，不会上传。',
                        )}
                      </div>
                    </div>
                  </div>
                  <button
                    type="button"
                    className="btn btn-xs btn-ghost gap-1 flex-shrink-0"
                    onClick={() => setScanExpanded(!scanExpanded)}
                    aria-expanded={scanExpanded}
                    title={scanExpanded ? t('common.collapse', '收起') : t('common.expand', '展开')}
                  >
                    {scanExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                    {scanExpanded ? t('common.collapse', '收起') : t('common.expand', '展开')}
                  </button>
                </div>

                {scanResult && (
                  <div className="flex gap-2 flex-wrap">
                    <span className="badge">
                      {t('usage.localDetected', '已检测')} {detectedSourceCount}/{localSources.length}
                    </span>
                    <span className="badge">
                      {t('usage.localEstimatedEntries', '预估记录')} {formatNumber(estimatedLocalEntries)}
                    </span>
                    <span className="badge">
                      {t('usage.localImportedRecords', '已入库')} {formatNumber(scanResult.existingRecords)}
                    </span>
                  </div>
                )}

                {!scanExpanded && scanError && (
                  <div
                    className="rounded-lg px-3 py-2"
                    style={{
                      border: '1px solid rgba(239,68,68,0.25)',
                      background: 'rgba(239,68,68,0.08)',
                      color: 'var(--danger)',
                      fontSize: 12,
                    }}
                  >
                    {scanError}
                  </div>
                )}

                {!scanExpanded && scanResult && (
                <div
                  className="flex flex-wrap gap-2"
                  style={{
                    marginTop: 12,
                    paddingTop: 12,
                    borderTop: '1px solid rgba(148,163,184,0.12)',
                  }}
                >
                  {localSources.filter(s => s.files > 0 || s.entries > 0 || !!s.path).map((source) => (
                    <div
                      key={source.id}
                      className="flex items-center gap-1.5"
                      style={{
                        padding: '4px 10px',
                        borderRadius: 8,
                        border: '1px solid rgba(59,130,246,0.18)',
                        background: 'rgba(59,130,246,0.06)',
                        fontSize: 12,
                        cursor: 'pointer',
                      }}
                      onClick={(e) => { e.stopPropagation(); setScanExpanded(true); setSelectedSource(source.id); }}
                    >
                      <div style={{ width: 16, height: 16, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                        {renderPlatformIcon(source.platformId, 12)}
                      </div>
                      <span className="font-medium">{source.label}</span>
                      <span className="opacity-50">·</span>
                      <span className="opacity-60">{formatNumber(source.entries)}</span>
                    </div>
                  ))}
                  {localSources.filter(s => !(s.files > 0 || s.entries > 0 || !!s.path)).length > 0 && (
                    <div
                      style={{
                        padding: '4px 10px',
                        borderRadius: 8,
                        border: '1px solid var(--border)',
                        background: 'var(--bg-tertiary)',
                        fontSize: 12,
                        opacity: 0.5,
                        cursor: 'pointer',
                      }}
                      onClick={(e) => { e.stopPropagation(); setScanExpanded(true); }}
                    >
                      +{localSources.filter(s => !(s.files > 0 || s.entries > 0 || !!s.path)).length} {t('usage.localSourceNotDetected', '未检测')}
                    </div>
                  )}
                </div>
              )}
              </div>

              {scanExpanded && (
                <div style={{ marginTop: 16 }}>
                  {scanError ? (
                    <div
                      className="rounded-lg px-3 py-2"
                      style={{
                        border: '1px solid rgba(239,68,68,0.25)',
                        background: 'rgba(239,68,68,0.08)',
                        color: 'var(--danger)',
                        fontSize: 12,
                      }}
                    >
                      {scanError}
                    </div>
                  ) : scanLoading && !scanResult ? (
                    <div className="flex items-center gap-2" style={{ fontSize: 12, opacity: 0.7 }}>
                      <Loader2 size={14} className="animate-spin" />
                      <span>{t('usage.localScanning', '正在扫描本地日志和数据库来源...')}</span>
                    </div>
                  ) : (
                    <div className="flex flex-col gap-3">
                      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))', gap: 12 }}>
                        {localSources.map((source) => {
                          const hasData = source.files > 0 || source.entries > 0 || !!source.path;
                          const isSelected = selectedSource === source.id;
                          return (
                            <div
                              key={source.id}
                              style={{
                                padding: 12,
                                borderRadius: 'var(--radius-lg)',
                                border: isSelected
                                  ? '1.5px solid var(--primary)'
                                  : hasData
                                    ? '1px solid rgba(59,130,246,0.18)'
                                    : '1px solid var(--border)',
                                background: isSelected
                                  ? 'linear-gradient(135deg, rgba(59,130,246,0.15), rgba(16,185,129,0.06))'
                                  : hasData
                                    ? 'linear-gradient(135deg, rgba(59,130,246,0.10), rgba(16,185,129,0.04))'
                                    : 'var(--bg-tertiary)',
                                opacity: hasData ? 1 : 0.58,
                                cursor: 'pointer',
                                transition: 'all 0.15s ease',
                              }}
                              onClick={(e) => {
                                e.stopPropagation();
                                setSelectedSource(source.id);
                              }}
                            >
                              <div className="flex items-center gap-2">
                                <div
                                  style={{
                                    width: 28,
                                    height: 28,
                                    borderRadius: 10,
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    background: hasData ? 'rgba(255,255,255,0.55)' : 'rgba(148,163,184,0.12)',
                                    flexShrink: 0,
                                  }}
                                >
                                  {renderPlatformIcon(source.platformId, 16)}
                                </div>
                                <div style={{ minWidth: 0 }}>
                                  <div className="text-sm font-medium truncate">{source.label}</div>
                                  <div className="text-xs opacity-55">
                                    {hasData
                                      ? t('usage.localSourceReady', '已检测到本地数据')
                                      : t('usage.localSourceMissing', '暂未检测到数据')}
                                  </div>
                                </div>
                                {hasData && (
                                  <div style={{ marginLeft: 'auto', width: 6, height: 6, borderRadius: '50%', background: '#10b981', flexShrink: 0 }} />
                                )}
                              </div>

                              <div
                                className="flex flex-wrap gap-x-3 gap-y-1"
                                style={{ marginTop: 10, fontSize: 12, opacity: 0.78 }}
                              >
                                <span>{t('usage.localSourceFiles', '文件/库')}: {formatNumber(source.files)}</span>
                                <span>{t('usage.localSourceEntries', '记录')}: {formatNumber(source.entries)}</span>
                              </div>

                              <div
                                className="font-mono"
                                style={{
                                  marginTop: 10,
                                  fontSize: 11,
                                  opacity: 0.52,
                                  overflow: 'hidden',
                                  textOverflow: 'ellipsis',
                                  whiteSpace: 'nowrap',
                                }}
                                title={source.path || ''}
                              >
                                {formatCompactPath(source.path)}
                              </div>
                            </div>
                          );
                        })}
                      </div>

                      {selectedSource && (() => {
                        const source = localSources.find(s => s.id === selectedSource);
                        if (!source) return null;
                        const hasData = source.files > 0 || source.entries > 0 || !!source.path;
                        const sourceStats = statsMap.get(`${source.id}_local`);
                        return (
                          <div
                            style={{
                              padding: 16,
                              borderRadius: 'var(--radius-lg)',
                              border: '1px solid rgba(59,130,246,0.15)',
                              background: 'rgba(59,130,246,0.03)',
                            }}
                          >
                            <div className="flex items-center justify-between">
                              <div className="flex items-center gap-3">
                                <div style={{ width: 32, height: 32, borderRadius: 10, display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'rgba(59,130,246,0.10)' }}>
                                  {renderPlatformIcon(source.platformId, 20)}
                                </div>
                                <div>
                                  <div className="text-sm font-semibold">{source.label} — {t('usage.localSourceDetail', '详细信息')}</div>
                                  <div className="text-xs opacity-50">{hasData ? t('usage.localSourceReady', '已检测到本地数据') : t('usage.localSourceMissing', '暂未检测到数据')}</div>
                                </div>
                              </div>
                              <div className="badge badge-outline">{t('usage.boundContext', '已绑定当前视图')}</div>
                            </div>

                            <div className="grid grid-cols-4 gap-4" style={{ marginTop: 12 }}>
                              <div style={{ padding: '8px 12px', borderRadius: 8, background: 'var(--bg-secondary)', border: '1px solid var(--border)' }}>
                                <div className="text-xs opacity-50">{t('usage.localSourceFiles', '文件/库')}</div>
                                <div className="text-lg font-bold">{source.files}</div>
                              </div>
                              <div style={{ padding: '8px 12px', borderRadius: 8, background: 'var(--bg-secondary)', border: '1px solid var(--border)' }}>
                                <div className="text-xs opacity-50">{t('usage.localSourceEntries', '记录')}</div>
                                <div className="text-lg font-bold">{formatNumber(source.entries)}</div>
                              </div>
                              {sourceStats && (
                                <>
                                  <div style={{ padding: '8px 12px', borderRadius: 8, background: 'var(--bg-secondary)', border: '1px solid var(--border)' }}>
                                    <div className="text-xs opacity-50">Tokens</div>
                                    <div className="text-lg font-bold">{formatNumber(sourceStats.totalTokens)}</div>
                                  </div>
                                  <div style={{ padding: '8px 12px', borderRadius: 8, background: 'var(--bg-secondary)', border: '1px solid var(--border)' }}>
                                    <div className="text-xs opacity-50">{t('usage.costLabel', '费用')}</div>
                                    <div className="text-lg font-bold text-success">${(parseFloat(sourceStats.totalCost) || 0).toFixed(4)}</div>
                                  </div>
                                </>
                              )}
                            </div>

                            <div className="flex items-center gap-2" style={{ marginTop: 12 }}>
                              <FolderOpen size={13} style={{ opacity: 0.5, flexShrink: 0 }} />
                              <span className="font-mono text-xs" style={{ opacity: 0.6, wordBreak: 'break-all' }}>
                                {source.path || t('usage.localSourceNoPath', '未检测到路径')}
                              </span>
                            </div>
                          </div>
                        );
                      })()}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          <div className="grid grid-cols-4 gap-4 oc-stagger">
            {statCards.map((card, i) => (
              <div key={i} className={`oc-stat-card oc-stat-card--${card.variant}`}>
                <div className="oc-stat-icon">{card.icon}</div>
                <div className="oc-stat-value">{card.value}</div>
                <div className="oc-stat-label">{card.label}</div>
              </div>
            ))}
          </div>

          {(chartTokenData.length > 0 || chartModelData.length > 0) && (
            <div className="grid grid-cols-2 gap-4">
              <div className="oc-mcp-card">
                <div className="text-xs font-medium opacity-60 mb-3 text-center">
                  {t('usage.timeTokenTrend', '时间 Token 趋势')}
                </div>
                <BarChart data={chartTokenData} height={130} />
              </div>
              <div className="oc-mcp-card flex justify-center">
                <DonutChart
                  data={chartModelData}
                  size={130}
                  centerLabel={t('usage.modelBreakdown', '按模型')}
                  centerValue={formatNumber(totalTokens)}
                />
              </div>
            </div>
          )}

          <div className="flex items-center justify-between flex-wrap gap-2">
            <div className="tabs tabs-boxed w-fit">
              <button className={`tab ${activeTab === 'overview' ? 'tab-active' : ''}`} onClick={() => setActiveTab('overview')}>
                {selectedProviderId ? t('usage.modelBreakdown', '按模型') : t('usage.platformBreakdown', '按平台')}
              </button>
              <button className={`tab ${activeTab === 'projects' ? 'tab-active' : ''}`} onClick={() => setActiveTab('projects')}>{t('usage.projectBreakdown', '按项目')}</button>
              <button className={`tab ${activeTab === 'trend' ? 'tab-active' : ''}`} onClick={() => setActiveTab('trend')}>{t('usage.trend', '趋势')}</button>
              <button className={`tab ${activeTab === 'pricing' ? 'tab-active' : ''}`} onClick={() => setActiveTab('pricing')}>{t('usage.pricing', '定价')}</button>
            </div>
            {activeTab !== 'pricing' && (
              <div className="tabs tabs-boxed tabs-xs">
                {(['24h', '7d', '30d', 'all'] as const).map(p => (
                  <button key={p} className={`tab ${period === p ? 'tab-active' : ''}`}
                    onClick={() => setPeriod(p)}>
                    {periodLabel(p)}
                  </button>
                ))}
              </div>
            )}
          </div>

          <div>
            {activeTab === 'overview' && (
              selectedProviderId ? (
                <div className="space-y-3">
                  <div className="oc-mcp-card">
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex items-center gap-3">
                        <div
                          style={{
                            width: 32,
                            height: 32,
                            borderRadius: 10,
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            background: 'rgba(59,130,246,0.10)',
                            flexShrink: 0,
                          }}
                        >
                          {selectedSourceInfo?.platformId
                            ? renderPlatformIcon(selectedSourceInfo.platformId, 18)
                            : <Terminal size={18} />}
                        </div>
                        <div>
                          <div className="text-sm font-semibold">
                            {selectedSourceInfo?.label || t('usage.modelBreakdown', '按模型')}
                          </div>
                          <div className="text-xs opacity-60">
                            {t('usage.modelBreakdownHint', '当前内容已绑定到本地来源扫描里选中的应用，展示模型用量与缓存占比。')}
                          </div>
                        </div>
                      </div>
                      <div className="badge badge-outline">{t('usage.localEstimateOnly', '仅本地历史估算')}</div>
                    </div>

                    {summary && (
                      <div className="flex gap-4 flex-wrap" style={{ fontSize: 12, marginTop: 12, opacity: 0.72 }}>
                        <span>{t('usage.inputTokensShort', '输入')}: {formatNumber(summary.totalInputTokens)}</span>
                        <span>{t('usage.outputTokensShort', '输出')}: {formatNumber(summary.totalOutputTokens)}</span>
                        {summary.totalCacheCreationTokens > 0 && (
                          <span>{t('usage.cacheWrite', '缓存写入')}: {formatNumber(summary.totalCacheCreationTokens)}</span>
                        )}
                        {summary.totalCacheReadTokens > 0 && (
                          <span>{t('usage.cacheRead', '缓存读取')}: {formatNumber(summary.totalCacheReadTokens)}</span>
                        )}
                        {selectedProviderStats && (
                          <span>{t('usage.requests', '请求数')}: {formatNumber(selectedProviderStats.requestCount)}</span>
                        )}
                      </div>
                    )}

                    <div style={{ marginTop: 12 }}>
                      {renderModelDetails(selectedModelKey)}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="space-y-2 oc-stagger">
                  {totalStats && totalStats.totalTokens > 0 && renderPlatformRow(
                    '__total__', <Globe size={20} />, t('usage.total', '总计'),
                    totalStats, '__total__', toggleTotalDetail, true,
                  )}

                  {mergedPlatforms
                    .sort((a, b) => (b.stats?.totalTokens || 0) - (a.stats?.totalTokens || 0))
                    .map(p => {
                      const icon = p.platformId ? renderPlatformIcon(p.platformId, 20) : <Terminal size={20} />;
                      const name = p.platformId ? getPlatformLabel(p.platformId, t) : p.fallbackName;
                      return renderPlatformRow(
                        p.providerId, icon, name, p.stats, p.providerId,
                        () => togglePlatformDetail(p.providerId),
                      );
                    })}

                  {unknownProviders.map(ps =>
                    renderPlatformRow(
                      ps.providerId, <Terminal size={20} />, ps.providerName || ps.providerId, ps,
                      ps.providerId, () => togglePlatformDetail(ps.providerId),
                    )
                  )}
                </div>
              )
            )}

            {activeTab === 'projects' && (
              <div className="space-y-3">
                <div
                  className="oc-mcp-card"
                  style={{
                    background: 'linear-gradient(135deg, rgba(16,185,129,0.08), rgba(59,130,246,0.03))',
                    borderStyle: 'dashed',
                  }}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex items-start gap-3">
                    <div
                      style={{
                        width: 32,
                        height: 32,
                        borderRadius: 10,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'rgba(16,185,129,0.12)',
                        color: '#10b981',
                        flexShrink: 0,
                      }}
                    >
                      <FolderOpen size={16} />
                    </div>
                    <div>
                      <div className="text-sm font-semibold">{t('usage.projectBreakdown', '按项目')}</div>
                      <div className="text-xs opacity-65" style={{ marginTop: 2 }}>
                        {t(
                          'usage.projectBreakdownHint',
                          '这里只统计当前选中本地来源下已识别项目名的记录；展开项目可继续查看该项目下的模型分布。',
                        )}
                      </div>
                    </div>
                    </div>
                    {selectedSourceInfo && (
                      <div className="badge badge-outline">{selectedSourceInfo.label}</div>
                    )}
                  </div>
                </div>

                {activeProjects.length > 0 && (
                  <div className="flex items-center justify-between gap-3 flex-wrap">
                    <div className="oc-search-wrap" style={{ maxWidth: 320 }}>
                      <Search size={14} className="oc-search-icon" />
                      <input
                        type="text"
                        className="oc-search-input"
                        placeholder={t('usage.searchProject', '搜索项目...')}
                        value={projectSearch}
                        onChange={e => setProjectSearch(e.target.value)}
                      />
                    </div>
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="badge">
                        {t('usage.filteredProjects', '项目')} {filteredProjects.length}/{activeProjects.length}
                      </span>
                      <select
                        className="select select-sm select-bordered"
                        value={projectSort}
                        onChange={e => setProjectSort(e.target.value as typeof projectSort)}
                      >
                        <option value="tokens_desc">{t('usage.sortByTokens', '按 Token 排序')}</option>
                        <option value="cost_desc">{t('usage.sortByCost', '按费用排序')}</option>
                        <option value="requests_desc">{t('usage.sortByRequests', '按请求数排序')}</option>
                        <option value="name_asc">{t('usage.sortByName', '按名称排序')}</option>
                      </select>
                    </div>
                  </div>
                )}

                {activeProjects.length > 0 ? (
                  <div className="space-y-2 oc-stagger">
                    {filteredProjects.map((project) => renderProjectRow(project))}
                    {filteredProjects.length === 0 && (
                      <div className="oc-empty-state">
                        <div className="oc-empty-state-icon"><Search size={28} /></div>
                        <div className="oc-empty-state-title">{t('usage.noProjectSearchResult', '未找到匹配项目')}</div>
                        <div style={{ fontSize: 13, opacity: 0.6, maxWidth: 480, textAlign: 'center' }}>
                          {t('usage.noProjectSearchResultHint', '试试更短的关键词，或切换排序方式查看其它项目。')}
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="oc-empty-state">
                    <div className="oc-empty-state-icon"><FolderOpen size={28} /></div>
                    <div className="oc-empty-state-title">{t('usage.noProjects', '暂无可识别项目')}</div>
                    <div style={{ fontSize: 13, opacity: 0.6, maxWidth: 480, textAlign: 'center' }}>
                      {t(
                        'usage.noProjectsHint',
                        '当前导入记录里还没有可用的项目名。先导入支持项目名的本地日志，或继续使用按平台统计查看总量。',
                      )}
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'trend' && (
              <div className="space-y-4">
                {modelTrendRaw.length > 0 ? (
                  <div className="oc-mcp-card" style={{ padding: '12px 8px' }}>
                    <StackedTrendChart data={modelTrendRaw} height={280} />
                  </div>
                ) : (
                  <div className="oc-empty-state">
                    <div className="oc-empty-state-icon"><BarChart3 size={28} /></div>
                    <div className="oc-empty-state-title">{t('usage.noTrend', '暂无趋势数据')}</div>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'pricing' && (
              <div className="space-y-3">
                {selectedSourceInfo && (
                  <div className="oc-mcp-card">
                    <div className="text-sm font-semibold">{selectedSourceInfo.label} · {t('usage.pricing', '定价')}</div>
                    <div className="text-xs opacity-60" style={{ marginTop: 4 }}>
                      {t('usage.pricingHintBound', '当前仅展示所选本地来源涉及到的模型定价；若模型名未命中定价表，费用仍会偏保守。')}
                    </div>
                  </div>
                )}
                {unpricedModels.length > 0 && (
                  <div className="oc-mcp-card" style={{ background: 'rgba(245,158,11,0.06)', borderColor: 'rgba(245,158,11,0.2)' }}>
                    <div className="flex items-center gap-2">
                      <AlertCircle size={14} style={{ color: '#f59e0b', flexShrink: 0 }} />
                      <span className="text-sm font-medium">{t('usage.unpricedModels', '以下模型未定价（费用为 $0）')}</span>
                    </div>
                    <div className="flex flex-wrap gap-2" style={{ marginTop: 8 }}>
                      {unpricedModels.map(m => (
                        <button
                          key={m.model}
                          className="badge badge-outline gap-1"
                          style={{ cursor: 'pointer', fontSize: 11 }}
                          onClick={() => {
                            setAddingPricing(true);
                            setNewPricingValues(v => ({ ...v, modelId: m.model, displayName: m.model }));
                          }}
                        >
                          <Plus size={10} />
                          {m.model}
                          <span className="opacity-50">({formatNumber(m.totalTokens)})</span>
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="oc-search-wrap" style={{ maxWidth: '240px' }}>
                      <Search size={14} className="oc-search-icon" />
                      <input
                        type="text"
                        className="oc-search-input"
                        placeholder={t('usage.searchModel', '搜索模型...')}
                        value={pricingSearch}
                        onChange={e => setPricingSearch(e.target.value)}
                      />
                    </div>
                    <button className="btn btn-xs btn-primary gap-1" onClick={() => setAddingPricing(true)}>
                      <Plus size={12} />
                      {t('usage.addModel', '添加模型')}
                    </button>
                  </div>
                  <button className="btn btn-xs btn-ghost gap-1" onClick={resetAllPricing} title={t('usage.resetPricing', '重置为默认价格')}>
                    <RotateCcw size={12} />
                    {t('usage.resetDefault', '重置默认')}
                  </button>
                </div>

                {addingPricing && (
                  <div className="oc-mcp-card" style={{ padding: 12 }}>
                    <div className="text-sm font-semibold" style={{ marginBottom: 8 }}>{t('usage.addModelPricing', '添加新模型定价')}</div>
                    <div className="grid grid-cols-3 gap-2" style={{ fontSize: 12 }}>
                      <div>
                        <label className="text-xs opacity-50">{t('usage.modelId', '模型 ID')}</label>
                        <input type="text" className="input input-xs input-bordered w-full font-mono"
                          placeholder="e.g. claude-4.5-opus"
                          value={newPricingValues.modelId}
                          onChange={e => setNewPricingValues(v => ({ ...v, modelId: e.target.value }))} />
                      </div>
                      <div>
                        <label className="text-xs opacity-50">{t('usage.displayName', '显示名称')}</label>
                        <input type="text" className="input input-xs input-bordered w-full"
                          placeholder={t('usage.optional', '可选')}
                          value={newPricingValues.displayName}
                          onChange={e => setNewPricingValues(v => ({ ...v, displayName: e.target.value }))} />
                      </div>
                      <div />
                      <div>
                        <label className="text-xs opacity-50">{t('usage.inputPrice', '输入 $/M')}</label>
                        <input type="text" className="input input-xs input-bordered w-full font-mono text-right"
                          placeholder="0"
                          value={newPricingValues.input}
                          onChange={e => setNewPricingValues(v => ({ ...v, input: e.target.value }))} />
                      </div>
                      <div>
                        <label className="text-xs opacity-50">{t('usage.outputPrice', '输出 $/M')}</label>
                        <input type="text" className="input input-xs input-bordered w-full font-mono text-right"
                          placeholder="0"
                          value={newPricingValues.output}
                          onChange={e => setNewPricingValues(v => ({ ...v, output: e.target.value }))} />
                      </div>
                      <div />
                      <div>
                        <label className="text-xs opacity-50">{t('usage.cacheReadPrice', '缓存读 $/M')}</label>
                        <input type="text" className="input input-xs input-bordered w-full font-mono text-right"
                          placeholder="0"
                          value={newPricingValues.cacheRead}
                          onChange={e => setNewPricingValues(v => ({ ...v, cacheRead: e.target.value }))} />
                      </div>
                      <div>
                        <label className="text-xs opacity-50">{t('usage.cacheWritePrice', '缓存写 $/M')}</label>
                        <input type="text" className="input input-xs input-bordered w-full font-mono text-right"
                          placeholder="0"
                          value={newPricingValues.cacheCreation}
                          onChange={e => setNewPricingValues(v => ({ ...v, cacheCreation: e.target.value }))} />
                      </div>
                      <div className="flex items-end gap-2">
                        <button className="btn btn-xs btn-success" onClick={saveNewPricing} disabled={!newPricingValues.modelId.trim()}>
                          <Check size={12} /> {t('common.save', '保存')}
                        </button>
                        <button className="btn btn-xs btn-ghost" onClick={() => setAddingPricing(false)}>
                          <X size={12} />
                        </button>
                      </div>
                    </div>
                  </div>
                )}
                <div className="overflow-x-auto" style={{ borderRadius: 'var(--radius-lg)', border: '1px solid var(--border)' }}>
                  <table className="table table-sm table-zebra">
                    <thead>
                      <tr>
                        <th>{t('usage.model', '模型')}</th>
                        <th className="text-right">{t('usage.inputPrice', '输入 $/M')}</th>
                        <th className="text-right">{t('usage.outputPrice', '输出 $/M')}</th>
                        <th className="text-right">{t('usage.cacheReadPrice', '缓存读 $/M')}</th>
                        <th className="text-right">{t('usage.cacheWritePrice', '缓存写 $/M')}</th>
                        <th style={{ width: 60 }} />
                      </tr>
                    </thead>
                    <tbody>
                      {filteredPricing.map(p => {
                        const isEditing = editingPricing === p.modelId;
                        const providerOverride = selectedProviderId
                          ? getProviderCustomPrice(selectedProviderId, p.modelId)
                          : undefined;
                        const effectiveInput = providerOverride?.inputCostPerMillion || p.inputCostPerMillion;
                        const effectiveOutput = providerOverride?.outputCostPerMillion || p.outputCostPerMillion;
                        const effectiveCacheRead = providerOverride?.cacheReadCostPerMillion || p.cacheReadCostPerMillion;
                        const effectiveCacheCreation = providerOverride?.cacheCreationCostPerMillion || p.cacheCreationCostPerMillion;
                        if (isEditing) {
                          return (
                            <tr key={p.modelId}>
                              <td className="font-mono text-sm">{p.displayName || p.modelId}</td>
                              <td className="text-right">
                                <input type="text" className="input input-xs input-bordered w-20 text-right font-mono"
                                  value={editValues.input} onChange={e => setEditValues(v => ({ ...v, input: e.target.value }))} />
                              </td>
                              <td className="text-right">
                                <input type="text" className="input input-xs input-bordered w-20 text-right font-mono"
                                  value={editValues.output} onChange={e => setEditValues(v => ({ ...v, output: e.target.value }))} />
                              </td>
                              <td className="text-right">
                                <input type="text" className="input input-xs input-bordered w-20 text-right font-mono"
                                  value={editValues.cacheRead} onChange={e => setEditValues(v => ({ ...v, cacheRead: e.target.value }))} />
                              </td>
                              <td className="text-right">
                                <input type="text" className="input input-xs input-bordered w-20 text-right font-mono"
                                  value={editValues.cacheCreation} onChange={e => setEditValues(v => ({ ...v, cacheCreation: e.target.value }))} />
                              </td>
                              <td>
                                <div className="flex gap-1">
                                  <button className="btn btn-xs btn-ghost text-success" onClick={() => savePricing(p.modelId)}><Check size={12} /></button>
                                  <button className="btn btn-xs btn-ghost" onClick={() => setEditingPricing(null)}><X size={12} /></button>
                                </div>
                              </td>
                            </tr>
                          );
                        }
                        return (
                          <tr key={p.modelId}>
                            <td className="font-mono text-sm">
                              {p.displayName || p.modelId}
                              {providerOverride && <span style={{ fontSize: 10, color: 'var(--primary)', marginLeft: 6 }}>&#9733;</span>}
                            </td>
                            <td className="text-right font-mono">${parseFloat(effectiveInput || '0').toFixed(2)}</td>
                            <td className="text-right font-mono">${parseFloat(effectiveOutput || '0').toFixed(2)}</td>
                            <td className="text-right font-mono">${parseFloat(effectiveCacheRead || '0').toFixed(2)}</td>
                            <td className="text-right font-mono">${parseFloat(effectiveCacheCreation || '0').toFixed(2)}</td>
                            <td>
                              <button className="btn btn-xs btn-ghost opacity-40 hover:opacity-100" onClick={() => startEditPricing(p.modelId, p)}>
                                <Edit2 size={12} />
                              </button>
                            </td>
                          </tr>
                        );
                      })}
                      {filteredPricing.length === 0 && (
                        <tr><td colSpan={6} className="text-center opacity-50">{t('usage.noPricing', '暂无定价数据')}</td></tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

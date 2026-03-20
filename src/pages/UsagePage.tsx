import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  BarChart3, RefreshCw, Trash2, Loader2, DollarSign, Cpu,
  MessageSquare, Upload, Search, Layers, Terminal,
  ChevronDown, ChevronRight, Globe, Edit2, X, Check, RotateCcw,
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
  successRate: number;
}

interface UsageTrend {
  period: string;
  requestCount: number;
  totalCost: number;
  inputTokens: number;
  outputTokens: number;
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

function aggregateModelUsage(trendData: ModelTrendData[]): ModelUsage[] {
  const map = new Map<string, ModelUsage>();
  for (const td of trendData) {
    for (const m of td.models) {
      const existing = map.get(m.model);
      if (existing) {
        existing.inputTokens += m.inputTokens;
        existing.outputTokens += m.outputTokens;
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

export function UsagePage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [trend, setTrend] = useState<UsageTrend[]>([]);
  const [providerStats, setProviderStats] = useState<ProviderStats[]>([]);
  const [pricing, setPricing] = useState<ModelPricing[]>([]);
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [activeTab, setActiveTab] = useState<'overview' | 'trend' | 'pricing'>('overview');
  const [period, setPeriod] = useState<string>('all');
  const [loadError, setLoadError] = useState<string | null>(null);
  const [pricingSearch, setPricingSearch] = useState('');
  const autoImportDone = useRef(false);
  const [expandedPlatform, setExpandedPlatform] = useState<string | null>(null);
  const [modelDetails, setModelDetails] = useState<Record<string, ModelUsage[]>>({});
  const [loadingDetails, setLoadingDetails] = useState<string | null>(null);

  // Custom pricing state
  const [editingPricing, setEditingPricing] = useState<string | null>(null);
  const [editValues, setEditValues] = useState({ input: '', output: '', cacheRead: '', cacheCreation: '' });
  const [providerPricing, setProviderPricing] = useState<ProviderModelPricing[]>([]);

  const loadData = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const [s, tr, ps, pr, pp] = await Promise.all([
        invoke<UsageSummary>('get_proxy_usage_summary', { period }).catch(() => null),
        invoke<UsageTrend[]>('get_proxy_usage_trend', { period }).catch(() => []),
        invoke<ProviderStats[]>('get_provider_stats', { period }).catch(() => []),
        invoke<ModelPricing[]>('get_model_pricing_list').catch(() => []),
        invoke<ProviderModelPricing[]>('get_all_provider_pricing').catch(() => []),
      ]);
      setSummary(s);
      setTrend(Array.isArray(tr) ? tr : []);
      setProviderStats(Array.isArray(ps) ? ps : []);
      setPricing(Array.isArray(pr) ? pr : []);
      setProviderPricing(Array.isArray(pp) ? pp : []);
    } catch (e) {
      setLoadError(String(e));
    } finally { setLoading(false); }
  }, [period]);

  useEffect(() => {
    if (autoImportDone.current) return;
    autoImportDone.current = true;
    setImporting(true);
    loadData();
    invoke('auto_import_local_logs')
      .then(() => loadData())
      .catch(() => {})
      .finally(() => setImporting(false));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => { loadData(); }, [loadData]);

  const handleClear = async () => {
    if (!confirm(t('usage.confirmClear', '确定要清空所有用量数据吗？'))) return;
    try {
      await invoke('clear_proxy_usage_stats');
      setModelDetails({});
      setExpandedPlatform(null);
      await loadData();
      toast.success(t('usage.cleared', '数据已清空'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleAutoImport = async () => {
    if (importing) return;
    setImporting(true);
    try {
      const count = await invoke<number>('auto_import_local_logs');
      await loadData();
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

  useEffect(() => {
    setModelDetails({});
    setExpandedPlatform(null);
  }, [period]);

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

  const formatNumber = (n: number) => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return String(n);
  };

  const totalTokens = summary ? (summary.totalInputTokens + summary.totalOutputTokens) : 0;
  const totalCost = summary ? parseFloat(summary.totalCost) || 0 : 0;

  const trendTokensList = trend.map(t => t.inputTokens + t.outputTokens);
  const maxTrendTokens = trendTokensList.length > 0 ? Math.max(...trendTokensList, 1) : 1;

  const filteredPricing = pricingSearch.trim()
    ? pricing.filter(p =>
        p.modelId.toLowerCase().includes(pricingSearch.toLowerCase()) ||
        p.displayName.toLowerCase().includes(pricingSearch.toLowerCase())
      )
    : pricing;

  const statsMap = new Map(providerStats.map(ps => [ps.providerId, ps]));
  const activePlatformCount = providerStats.filter(ps => ps.totalTokens > 0).length;
  const activeProviders = providerStats.filter(ps => ps.totalTokens > 0);

  const mergedPlatforms = ALL_PLATFORMS.map(platform => ({
    ...platform,
    stats: statsMap.get(platform.providerId) || null,
  }));

  const unknownProviders = providerStats.filter(ps => !KNOWN_PROVIDER_IDS.has(ps.providerId));

  // ─── Chart data ───
  const chartTokenData = useMemo(() => {
    const sorted = activeProviders
      .sort((a, b) => b.totalTokens - a.totalTokens)
      .slice(0, 10);
    return sorted.map((ps, i) => {
      const platform = ALL_PLATFORMS.find(p => p.providerId === ps.providerId);
      return {
        label: platform?.fallbackName || ps.providerName || ps.providerId,
        value: ps.totalTokens,
        color: CHART_COLORS[i % CHART_COLORS.length],
      };
    });
  }, [activeProviders]);

  const chartCostData = useMemo(() => {
    const sorted = activeProviders
      .filter(ps => parseFloat(ps.totalCost) > 0)
      .sort((a, b) => parseFloat(b.totalCost) - parseFloat(a.totalCost))
      .slice(0, 10);
    return sorted.map((ps, i) => {
      const platform = ALL_PLATFORMS.find(p => p.providerId === ps.providerId);
      return {
        label: platform?.fallbackName || ps.providerName || ps.providerId,
        value: parseFloat(ps.totalCost) || 0,
        color: CHART_COLORS[i % CHART_COLORS.length],
      };
    });
  }, [activeProviders]);

  const statCards = [
    { icon: <Cpu size={20} />, label: t('usage.totalTokens', '总 Token 数'), value: formatNumber(totalTokens), variant: 'blue' as const },
    { icon: <DollarSign size={20} />, label: t('usage.totalCost', '总费用'), value: `$${totalCost.toFixed(4)}`, variant: 'green' as const },
    { icon: <MessageSquare size={20} />, label: t('usage.requests', '请求数'), value: formatNumber(summary?.totalRequests || 0), variant: 'purple' as const },
    { icon: <Layers size={20} />, label: t('usage.platforms', '平台数'), value: `${activePlatformCount} / ${ALL_PLATFORMS.length}`, variant: 'orange' as const },
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
              <th className="text-right">Tokens</th>
              <th className="text-right">{t('usage.costLabel', '费用')}</th>
              <th className="text-right">{t('usage.requestsShort', '请求')}</th>
            </tr>
          </thead>
          <tbody>
            {models.map(m => {
              const customPrice = key !== '__total__' ? getProviderCustomPrice(key, m.model) : null;
              return (
                <tr key={m.model}>
                  <td className="font-mono" style={{ paddingLeft: 0, maxWidth: 180, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {m.model}
                    {customPrice && <span style={{ fontSize: 9, color: 'var(--primary)', marginLeft: 4 }}>&#9733;</span>}
                  </td>
                  <td className="text-right font-mono">{formatNumber(m.inputTokens)}</td>
                  <td className="text-right font-mono">{formatNumber(m.outputTokens)}</td>
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

  const totalStats: ProviderStats | null = summary ? {
    providerId: '__total__',
    providerName: t('usage.total', '总计'),
    requestCount: summary.totalRequests,
    totalTokens: summary.totalInputTokens + summary.totalOutputTokens,
    totalInputTokens: summary.totalInputTokens,
    totalOutputTokens: summary.totalOutputTokens,
    totalCacheCreationTokens: summary.totalCacheCreationTokens,
    totalCacheReadTokens: summary.totalCacheReadTokens,
    totalCost: summary.totalCost,
    successRate: summary.successRate,
  } : null;

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <ToastContainer toasts={toast.toasts} />

      <div className="oc-page-header">
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
          <button className="btn btn-sm btn-ghost" onClick={loadData} disabled={loading}><RefreshCw size={14} /></button>
          <button className="btn btn-sm btn-ghost text-error" onClick={handleClear}><Trash2 size={14} /></button>
        </div>
      </div>

      {importing && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg flex-shrink-0" style={{ background: 'rgba(59,130,246,0.08)', border: '1px solid rgba(59,130,246,0.2)' }}>
          <Loader2 size={14} className="animate-spin" style={{ color: 'var(--primary)' }} />
          <span style={{ fontSize: '13px', color: 'var(--primary)' }}>{t('usage.importingHint', '正在扫描本地 AI 工具日志并导入...')}</span>
        </div>
      )}

      {loadError && (
        <div className="p-4 rounded-lg flex-shrink-0" style={{ background: 'rgba(239,68,68,0.1)', border: '1px solid rgba(239,68,68,0.3)' }}>
          <p style={{ fontSize: '13px', color: 'var(--danger)' }}>{loadError}</p>
          <button className="btn btn-sm btn-primary" onClick={loadData} style={{ marginTop: '8px' }}>
            <RefreshCw size={14} /> {t('common.retry', '重试')}
          </button>
        </div>
      )}

      {loading && !importing ? (
        <div className="flex-1 flex justify-center items-center"><Loader2 className="animate-spin" size={32} /></div>
      ) : (
        <>
          <div className="flex items-center justify-end flex-shrink-0">
            <select className="select select-sm select-bordered" value={period} onChange={e => setPeriod(e.target.value)}>
              <option value="24h">{periodLabel('24h')}</option>
              <option value="7d">{periodLabel('7d')}</option>
              <option value="30d">{periodLabel('30d')}</option>
              <option value="all">{periodLabel('all')}</option>
            </select>
          </div>

          <div className="grid grid-cols-4 gap-4 flex-shrink-0 oc-stagger">
            {statCards.map((card, i) => (
              <div key={i} className={`oc-stat-card oc-stat-card--${card.variant}`}>
                <div className="oc-stat-icon">{card.icon}</div>
                <div className="oc-stat-value">{card.value}</div>
                <div className="oc-stat-label">{card.label}</div>
              </div>
            ))}
          </div>

          {/* ─── Charts Section ─── */}
          {activeProviders.length > 0 && (
            <div className="grid grid-cols-2 gap-4 flex-shrink-0">
              <div className="oc-mcp-card">
                <div className="text-xs font-medium opacity-60 mb-3 text-center">{t('usage.tokenDistribution', 'Token 用量分布')}</div>
                <BarChart data={chartTokenData} height={130} />
              </div>
              <div className="oc-mcp-card flex justify-center">
                <DonutChart
                  data={chartCostData}
                  size={130}
                  centerLabel={t('usage.totalCost', '总费用')}
                  centerValue={`$${totalCost.toFixed(2)}`}
                />
              </div>
            </div>
          )}

          <div className="tabs tabs-boxed w-fit self-center">
            <button className={`tab ${activeTab === 'overview' ? 'tab-active' : ''}`} onClick={() => setActiveTab('overview')}>{t('usage.platformBreakdown', '按平台')}</button>
            <button className={`tab ${activeTab === 'trend' ? 'tab-active' : ''}`} onClick={() => setActiveTab('trend')}>{t('usage.trend', '趋势')}</button>
            <button className={`tab ${activeTab === 'pricing' ? 'tab-active' : ''}`} onClick={() => setActiveTab('pricing')}>{t('usage.pricing', '定价')}</button>
          </div>

          <div className="flex-1 min-h-0 overflow-y-auto">
            {activeTab === 'overview' && (
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
            )}

            {activeTab === 'trend' && (
              <div className="space-y-4">
                <div className="space-y-1">
                  {trend.map(day => {
                    const tokens = day.inputTokens + day.outputTokens;
                    return (
                      <div key={day.period} className="oc-trend-row">
                        <span className="oc-trend-date">{day.period.length > 10 ? day.period.slice(11, 16) : day.period.slice(5)}</span>
                        <div className="oc-trend-bar-wrap">
                          <div className="oc-trend-bar" style={{ width: `${Math.max((tokens / maxTrendTokens) * 100, 1)}%` }} />
                        </div>
                        <span className="oc-trend-tokens">{formatNumber(tokens)}</span>
                        <span className="oc-trend-cost">${(day.totalCost || 0).toFixed(3)}</span>
                      </div>
                    );
                  })}
                  {trend.length === 0 && (
                    <div className="oc-empty-state">
                      <div className="oc-empty-state-icon"><BarChart3 size={28} /></div>
                      <div className="oc-empty-state-title">{t('usage.noTrend', '暂无趋势数据')}</div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === 'pricing' && (
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <div className="oc-search-wrap" style={{ maxWidth: '280px' }}>
                    <Search size={14} className="oc-search-icon" />
                    <input
                      type="text"
                      className="oc-search-input"
                      placeholder={t('usage.searchModel', '搜索模型...')}
                      value={pricingSearch}
                      onChange={e => setPricingSearch(e.target.value)}
                    />
                  </div>
                  <button className="btn btn-xs btn-ghost gap-1" onClick={resetAllPricing} title={t('usage.resetPricing', '重置为默认价格')}>
                    <RotateCcw size={12} />
                    {t('usage.resetDefault', '重置默认')}
                  </button>
                </div>
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
                            <td className="font-mono text-sm">{p.displayName || p.modelId}</td>
                            <td className="text-right font-mono">${parseFloat(p.inputCostPerMillion || '0').toFixed(2)}</td>
                            <td className="text-right font-mono">${parseFloat(p.outputCostPerMillion || '0').toFixed(2)}</td>
                            <td className="text-right font-mono">${parseFloat(p.cacheReadCostPerMillion || '0').toFixed(2)}</td>
                            <td className="text-right font-mono">${parseFloat(p.cacheCreationCostPerMillion || '0').toFixed(2)}</td>
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
        </>
      )}
    </div>
  );
}

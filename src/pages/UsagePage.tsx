import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { BarChart3, RefreshCw, Trash2, Loader2, DollarSign, Cpu, MessageSquare, Upload } from 'lucide-react';

interface UsageSummary {
  total_tokens: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cost: number;
  total_requests: number;
  model_breakdown: Record<string, { tokens: number; cost: number; requests: number }>;
}

interface UsageTrend {
  date: string;
  tokens: number;
  cost: number;
  requests: number;
}

interface ModelPricing {
  model_id: string;
  input_price_per_million: number;
  output_price_per_million: number;
}

export function UsagePage() {
  const { t } = useTranslation();
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [trend, setTrend] = useState<UsageTrend[]>([]);
  const [pricing, setPricing] = useState<ModelPricing[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'overview' | 'trend' | 'pricing'>('overview');
  const [trendDays, setTrendDays] = useState(30);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [s, tr, pr] = await Promise.all([
        invoke<UsageSummary>('get_usage_summary').catch(() => null),
        invoke<UsageTrend[]>('get_usage_trend', { days: trendDays }).catch(() => []),
        invoke<ModelPricing[]>('get_model_pricing_list').catch(() => []),
      ]);
      setSummary(s);
      setTrend(tr);
      setPricing(pr);
    } catch (e) { console.error('Load usage failed:', e); }
    finally { setLoading(false); }
  }, [trendDays]);

  useEffect(() => { loadData(); }, [loadData]);

  const handleClear = async () => {
    if (!confirm(t('usage.confirmClear', 'Clear all usage data?'))) return;
    try { await invoke('clear_usage_stats'); await loadData(); } catch (e) { console.error('Clear failed:', e); }
  };

  const handleAutoImport = async () => {
    try { await invoke('auto_import_local_logs'); await loadData(); } catch (e) { console.error('Auto import failed:', e); }
  };

  const formatNumber = (n: number) => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return String(n);
  };

  const maxTrendTokens = Math.max(...trend.map(t => t.tokens), 1);

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-3">
          <BarChart3 size={24} className="text-primary" />
          <h2 className="text-lg font-bold">{t('usage.title', 'Usage Statistics')}</h2>
        </div>
        <div className="flex gap-2">
          <button className="btn btn-sm btn-ghost" onClick={handleAutoImport}><Upload size={14} /> {t('usage.autoImport', 'Auto Import')}</button>
          <button className="btn btn-sm btn-ghost" onClick={loadData} disabled={loading}><RefreshCw size={14} /></button>
          <button className="btn btn-sm btn-ghost text-error" onClick={handleClear}><Trash2 size={14} /></button>
        </div>
      </div>

      {loading ? (
        <div className="flex-1 flex justify-center items-center"><Loader2 className="animate-spin" size={32} /></div>
      ) : (
        <>
          {/* Summary Cards */}
          <div className="grid grid-cols-4 gap-4 flex-shrink-0">
            {[
              { icon: <Cpu size={20} />, label: t('usage.totalTokens', 'Total Tokens'), value: formatNumber(summary?.total_tokens || 0), color: 'text-primary' },
              { icon: <DollarSign size={20} />, label: t('usage.totalCost', 'Total Cost'), value: `$${(summary?.total_cost || 0).toFixed(4)}`, color: 'text-success' },
              { icon: <MessageSquare size={20} />, label: t('usage.requests', 'Requests'), value: formatNumber(summary?.total_requests || 0), color: 'text-info' },
              { icon: <BarChart3 size={20} />, label: t('usage.models', 'Models'), value: String(Object.keys(summary?.model_breakdown || {}).length), color: 'text-secondary' },
            ].map((card, i) => (
              <div key={i} className="card bg-base-200 p-4">
                <div className={`${card.color} mb-2`}>{card.icon}</div>
                <div className="text-2xl font-bold">{card.value}</div>
                <div className="text-xs opacity-60">{card.label}</div>
              </div>
            ))}
          </div>

          {/* Tabs */}
          <div className="tabs tabs-boxed w-fit self-center">
            <button className={`tab ${activeTab === 'overview' ? 'tab-active' : ''}`} onClick={() => setActiveTab('overview')}>{t('usage.modelBreakdown', 'By Model')}</button>
            <button className={`tab ${activeTab === 'trend' ? 'tab-active' : ''}`} onClick={() => setActiveTab('trend')}>{t('usage.trend', 'Trend')}</button>
            <button className={`tab ${activeTab === 'pricing' ? 'tab-active' : ''}`} onClick={() => setActiveTab('pricing')}>{t('usage.pricing', 'Pricing')}</button>
          </div>

          <div className="flex-1 min-h-0 overflow-y-auto">
            {activeTab === 'overview' && summary && (
              <div className="space-y-2">
                {Object.entries(summary.model_breakdown).sort(([, a], [, b]) => b.tokens - a.tokens).map(([model, data]) => (
                  <div key={model} className="card bg-base-200 p-4 flex items-center justify-between">
                    <div>
                      <div className="font-mono text-sm font-medium">{model}</div>
                      <div className="text-xs opacity-50">{data.requests} requests</div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm font-bold">{formatNumber(data.tokens)} tokens</div>
                      <div className="text-xs text-success">${data.cost.toFixed(4)}</div>
                    </div>
                  </div>
                ))}
                {Object.keys(summary.model_breakdown).length === 0 && (
                  <div className="text-center py-8 opacity-50">{t('usage.noData', 'No usage data')}</div>
                )}
              </div>
            )}

            {activeTab === 'trend' && (
              <div className="space-y-4">
                <div className="flex justify-end">
                  <select className="select select-sm select-bordered" value={trendDays} onChange={e => setTrendDays(Number(e.target.value))}>
                    <option value={7}>7 days</option>
                    <option value={14}>14 days</option>
                    <option value={30}>30 days</option>
                    <option value={90}>90 days</option>
                  </select>
                </div>
                <div className="space-y-1">
                  {trend.map(day => (
                    <div key={day.date} className="flex items-center gap-3 text-xs">
                      <span className="w-20 font-mono opacity-60">{day.date.slice(5)}</span>
                      <div className="flex-1 bg-base-300 rounded-full h-5 overflow-hidden">
                        <div className="bg-primary h-full rounded-full transition-all" style={{ width: `${(day.tokens / maxTrendTokens) * 100}%` }} />
                      </div>
                      <span className="w-16 text-right font-mono">{formatNumber(day.tokens)}</span>
                      <span className="w-16 text-right font-mono text-success">${day.cost.toFixed(3)}</span>
                    </div>
                  ))}
                  {trend.length === 0 && <div className="text-center py-8 opacity-50">{t('usage.noTrend', 'No trend data')}</div>}
                </div>
              </div>
            )}

            {activeTab === 'pricing' && (
              <div className="overflow-x-auto">
                <table className="table table-sm">
                  <thead>
                    <tr>
                      <th>{t('usage.model', 'Model')}</th>
                      <th className="text-right">{t('usage.inputPrice', 'Input $/M')}</th>
                      <th className="text-right">{t('usage.outputPrice', 'Output $/M')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {pricing.map(p => (
                      <tr key={p.model_id}>
                        <td className="font-mono text-sm">{p.model_id}</td>
                        <td className="text-right">${p.input_price_per_million.toFixed(2)}</td>
                        <td className="text-right">${p.output_price_per_million.toFixed(2)}</td>
                      </tr>
                    ))}
                    {pricing.length === 0 && (
                      <tr><td colSpan={3} className="text-center opacity-50">{t('usage.noPricing', 'No pricing data')}</td></tr>
                    )}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}

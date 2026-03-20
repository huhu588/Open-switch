import { useState, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Layers,
  Zap,
  Sparkles,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Wand2,
} from 'lucide-react';
import {
  PlatformOverviewTab,
  PlatformOverviewTabsHeader,
} from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformProviderPanel } from '../components/platform/PlatformProviderPanel';
import { useProviderStore, ProviderItem } from '../stores/useProviderStore';

type ProviderStatus = 'active' | 'inactive' | 'loading';

export function OpenCodeAccountsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<PlatformOverviewTab>('overview');
  const store = useProviderStore();
  const [providerStatus, setProviderStatus] = useState<ProviderStatus>('loading');

  const allProviders = useMemo(() => store.providers, [store.providers]);
  const enabledCount = useMemo(() => allProviders.filter((p) => p.enabled).length, [allProviders]);

  useEffect(() => {
    store.loadProviders().then(() => {
      setProviderStatus(enabledCount > 0 ? 'active' : 'inactive');
    });
  }, []);

  useEffect(() => {
    setProviderStatus(enabledCount > 0 ? 'active' : 'inactive');
  }, [enabledCount]);

  const handleRefresh = async () => {
    setProviderStatus('loading');
    await store.loadProviders();
  };

  const providersByType = useMemo(() => {
    const grouped: Record<string, ProviderItem[]> = {};
    for (const p of allProviders) {
      const type = p.model_type || 'claude';
      if (!grouped[type]) grouped[type] = [];
      grouped[type].push(p);
    }
    return grouped;
  }, [allProviders]);

  return (
    <div className="page-container">
      <PlatformOverviewTabsHeader platform="opencode" active={activeTab} onTabChange={setActiveTab} />

      {activeTab === 'overview' && (
        <div className="platform-overview-content" style={{ padding: '24px' }}>
          <div style={{ display: 'grid', gap: '20px', maxWidth: 960, margin: '0 auto' }}>
            <div className="card" style={{ padding: '20px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
                <h3 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>{t('opencode.configStatus', 'OpenCode 配置状态')}</h3>
                <button className="btn btn-secondary icon-only" onClick={handleRefresh} title={t('common.refresh', '刷新')}><RefreshCw size={16} /></button>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
                {providerStatus === 'loading' ? (
                  <Loader2 size={20} className="spin" style={{ color: 'var(--text-secondary)' }} />
                ) : providerStatus === 'active' ? (
                  <CheckCircle2 size={20} style={{ color: 'var(--color-success, #22c55e)' }} />
                ) : (
                  <AlertCircle size={20} style={{ color: 'var(--color-warning, #f59e0b)' }} />
                )}
                <span style={{ fontSize: 14, color: 'var(--text-primary)' }}>
                  {providerStatus === 'loading' ? t('opencode.checking', '检测中...')
                    : providerStatus === 'active' ? t('opencode.configured', '已配置 Provider')
                    : t('opencode.notConfigured', '未配置任何 Provider')}
                </span>
              </div>
              <div style={{ fontSize: 13, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
                {t('opencode.desc', 'OpenCode 是统一的 CLI 工具配置中心，管理 Claude、Codex、Gemini 等多个 AI 工具的 Provider 配置。')}
              </div>
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>{t('opencode.providerDistribution', 'Provider 分布')}</h3>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 12 }}>
                {Object.entries(providersByType).map(([type, providers]) => (
                  <div key={type} style={{ padding: '14px 16px', borderRadius: 8, border: '1px solid var(--border-color)', background: 'var(--card-bg)' }}>
                    <div style={{ fontSize: 13, fontWeight: 600, textTransform: 'capitalize', marginBottom: 8 }}>{type}</div>
                    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 12, color: 'var(--text-secondary)' }}>
                      <span>{providers.length} {t('opencode.providers', 'providers')}</span>
                      <span style={{ color: 'var(--color-success, #22c55e)' }}>{providers.filter((p) => p.enabled).length} {t('opencode.active', '启用')}</span>
                    </div>
                  </div>
                ))}
                {Object.keys(providersByType).length === 0 && (
                  <div style={{ color: 'var(--text-tertiary)', fontSize: 13, gridColumn: '1 / -1', textAlign: 'center', padding: 20 }}>
                    {t('opencode.noProviders', '暂无 Provider，请先添加。')}
                  </div>
                )}
              </div>
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>{t('opencode.quickActions', '快速操作')}</h3>
              <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
                <button className="btn btn-primary" onClick={() => setActiveTab('providers')}>
                  <Layers size={16} />
                  <span>{t('opencode.manageProviders', '管理 Provider')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.dispatchEvent(new CustomEvent('app-request-navigate', { detail: 'mcp' }))}>
                  <Zap size={16} />
                  <span>{t('opencode.manageMcp', '管理 MCP')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.dispatchEvent(new CustomEvent('app-request-navigate', { detail: 'skills' }))}>
                  <Sparkles size={16} />
                  <span>{t('opencode.manageSkills', '管理 Skills')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.dispatchEvent(new CustomEvent('app-request-navigate', { detail: 'ohmy' }))}>
                  <Wand2 size={16} />
                  <span>{t('opencode.ohmy', 'OhMy 配置')}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'providers' && (
        <PlatformProviderPanel modelType="opencode" />
      )}

      {activeTab === 'instances' && (
        <div className="platform-overview-content" style={{ padding: '24px' }}>
          <div className="empty-state" style={{ textAlign: 'center', padding: '60px 20px' }}>
            <Layers size={48} style={{ color: 'var(--text-tertiary)', marginBottom: 16 }} />
            <h3 style={{ color: 'var(--text-secondary)', marginBottom: 8 }}>{t('opencode.instancesComingSoon', '多开实例')}</h3>
            <p style={{ color: 'var(--text-tertiary)', fontSize: 14 }}>{t('opencode.instancesDesc', 'OpenCode 多实例管理功能即将推出。')}</p>
          </div>
        </div>
      )}
    </div>
  );
}

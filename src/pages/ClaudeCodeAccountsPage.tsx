import { useState, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
  Layers,
  Zap,
  Globe,
  Key,
  RefreshCw,
  ExternalLink,
  CheckCircle2,
  AlertCircle,
  Loader2,
} from 'lucide-react';
import {
  PlatformOverviewTab,
  PlatformOverviewTabsHeader,
} from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformProviderPanel } from '../components/platform/PlatformProviderPanel';
import { useProviderStore } from '../stores/useProviderStore';

type ProviderStatus = 'active' | 'inactive' | 'loading';

export function ClaudeCodeAccountsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<PlatformOverviewTab>('overview');
  const store = useProviderStore();
  const [providerStatus, setProviderStatus] = useState<ProviderStatus>('loading');
  const [configPath, setConfigPath] = useState<string>('');

  const claudeProviders = useMemo(
    () => store.providers.filter((p) => (p.model_type || 'claude') === 'claude'),
    [store.providers],
  );

  const activeProvider = useMemo(
    () => claudeProviders.find((p) => p.enabled),
    [claudeProviders],
  );

  useEffect(() => {
    store.loadProviders().then(() => {
      setProviderStatus(activeProvider ? 'active' : 'inactive');
    });
  }, []);

  useEffect(() => {
    setProviderStatus(activeProvider ? 'active' : 'inactive');
  }, [activeProvider]);

  useEffect(() => {
    invoke<string>('get_claude_config_path').then(setConfigPath).catch(() => {});
  }, []);

  const handleRefresh = async () => {
    setProviderStatus('loading');
    await store.loadProviders();
  };

  return (
    <div className="page-container">
      <PlatformOverviewTabsHeader
        platform="claude-code"
        active={activeTab}
        onTabChange={setActiveTab}
      />

      {activeTab === 'overview' && (
        <div className="platform-overview-content" style={{ padding: '24px' }}>
          <div style={{ display: 'grid', gap: '20px', maxWidth: 960, margin: '0 auto' }}>
            <div className="card" style={{ padding: '20px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
                <h3 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>
                  {t('claudeCode.providerStatus', 'Provider 状态')}
                </h3>
                <button className="btn btn-secondary icon-only" onClick={handleRefresh} title={t('common.refresh', '刷新')}>
                  <RefreshCw size={16} />
                </button>
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
                  {providerStatus === 'loading'
                    ? t('claudeCode.checking', '检测中...')
                    : providerStatus === 'active'
                      ? t('claudeCode.providerActive', '已配置并启用')
                      : t('claudeCode.providerInactive', '未配置 Provider')}
                </span>
              </div>
              {activeProvider && (
                <div style={{ display: 'grid', gap: 8 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color: 'var(--text-secondary)' }}>
                    <Layers size={14} />
                    <span>{t('claudeCode.currentProvider', '当前 Provider')}:</span>
                    <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>{activeProvider.name}</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color: 'var(--text-secondary)' }}>
                    <Globe size={14} />
                    <span>Base URL:</span>
                    <span style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: 12 }}>{activeProvider.base_url}</span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color: 'var(--text-secondary)' }}>
                    <Key size={14} />
                    <span>{t('claudeCode.modelCount', '模型数量')}:</span>
                    <span style={{ color: 'var(--text-primary)' }}>{activeProvider.model_count}</span>
                  </div>
                </div>
              )}
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>
                {t('claudeCode.providerSummary', 'Provider 概览')}
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: 28, fontWeight: 700, color: '#D97757' }}>{claudeProviders.length}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 4 }}>{t('claudeCode.totalProviders', '总 Provider 数')}</div>
                </div>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: 28, fontWeight: 700, color: 'var(--color-success, #22c55e)' }}>{claudeProviders.filter((p) => p.enabled).length}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 4 }}>{t('claudeCode.enabledProviders', '已启用')}</div>
                </div>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: 28, fontWeight: 700, color: 'var(--text-primary)' }}>{claudeProviders.reduce((sum, p) => sum + p.model_count, 0)}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 4 }}>{t('claudeCode.totalModels', '总模型数')}</div>
                </div>
              </div>
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>{t('claudeCode.quickActions', '快速操作')}</h3>
              <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
                <button className="btn btn-primary" onClick={() => setActiveTab('providers')}>
                  <Layers size={16} />
                  <span>{t('claudeCode.manageProviders', '管理 Provider')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.dispatchEvent(new CustomEvent('app-request-navigate', { detail: 'mcp' }))}>
                  <Zap size={16} />
                  <span>{t('claudeCode.manageMcp', '管理 MCP')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.open('https://docs.anthropic.com/en/docs/claude-code', '_blank')}>
                  <ExternalLink size={16} />
                  <span>{t('claudeCode.docs', '官方文档')}</span>
                </button>
              </div>
            </div>

            {configPath && (
              <div style={{ fontSize: 12, color: 'var(--text-tertiary)', textAlign: 'center' }}>
                {t('claudeCode.configLocation', '配置路径')}: {configPath}
              </div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'providers' && (
        <PlatformProviderPanel modelType="claude" />
      )}

      {activeTab === 'instances' && (
        <div className="platform-overview-content" style={{ padding: '24px' }}>
          <div className="empty-state" style={{ textAlign: 'center', padding: '60px 20px' }}>
            <Layers size={48} style={{ color: 'var(--text-tertiary)', marginBottom: 16 }} />
            <h3 style={{ color: 'var(--text-secondary)', marginBottom: 8 }}>{t('claudeCode.instancesComingSoon', '多开实例')}</h3>
            <p style={{ color: 'var(--text-tertiary)', fontSize: 14 }}>{t('claudeCode.instancesDesc', 'Claude Code 多实例管理功能即将推出。')}</p>
          </div>
        </div>
      )}
    </div>
  );
}

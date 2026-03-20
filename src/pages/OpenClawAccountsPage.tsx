import { useState, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
  Layers,
  Zap,
  FileText,
  RefreshCw,
  ExternalLink,
  CheckCircle2,
  AlertCircle,
  Loader2,
  FolderOpen,
  PenTool,
} from 'lucide-react';
import {
  PlatformOverviewTab,
  PlatformOverviewTabsHeader,
} from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformProviderPanel } from '../components/platform/PlatformProviderPanel';
import { useProviderStore } from '../stores/useProviderStore';

type ConfigStatus = 'detected' | 'not-found' | 'loading';

export function OpenClawAccountsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<PlatformOverviewTab>('overview');
  const store = useProviderStore();
  const [configStatus, setConfigStatus] = useState<ConfigStatus>('loading');
  const [openclawConfigPath, setOpenclawConfigPath] = useState<string>('');
  const [agentsContent, setAgentsContent] = useState<string>('');

  const openclawProviders = useMemo(
    () => store.providers.filter((p) => p.model_type === 'openclaw'),
    [store.providers],
  );

  useEffect(() => {
    store.loadProviders();
    invoke<string>('get_openclaw_config_path')
      .then((path) => { setOpenclawConfigPath(path); setConfigStatus('detected'); })
      .catch(() => { setConfigStatus('not-found'); });
    invoke<string>('get_openclaw_agents_content').then(setAgentsContent).catch(() => {});
  }, []);

  const handleRefresh = async () => {
    setConfigStatus('loading');
    await store.loadProviders();
    try {
      const path = await invoke<string>('get_openclaw_config_path');
      setOpenclawConfigPath(path);
      setConfigStatus('detected');
    } catch { setConfigStatus('not-found'); }
  };

  return (
    <div className="page-container">
      <PlatformOverviewTabsHeader platform="openclaw" active={activeTab} onTabChange={setActiveTab} />

      {activeTab === 'overview' && (
        <div className="platform-overview-content" style={{ padding: '24px' }}>
          <div style={{ display: 'grid', gap: '20px', maxWidth: 960, margin: '0 auto' }}>
            <div className="card" style={{ padding: '20px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
                <h3 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>{t('openclaw.configStatus', 'OpenClaw 配置状态')}</h3>
                <button className="btn btn-secondary icon-only" onClick={handleRefresh} title={t('common.refresh', '刷新')}><RefreshCw size={16} /></button>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
                {configStatus === 'loading' ? (
                  <Loader2 size={20} className="spin" style={{ color: 'var(--text-secondary)' }} />
                ) : configStatus === 'detected' ? (
                  <CheckCircle2 size={20} style={{ color: 'var(--color-success, #22c55e)' }} />
                ) : (
                  <AlertCircle size={20} style={{ color: 'var(--color-warning, #f59e0b)' }} />
                )}
                <span style={{ fontSize: 14, color: 'var(--text-primary)' }}>
                  {configStatus === 'loading' ? t('openclaw.checking', '检测中...')
                    : configStatus === 'detected' ? t('openclaw.detected', '已检测到 OpenClaw 配置')
                    : t('openclaw.notDetected', '未检测到 OpenClaw 配置')}
                </span>
              </div>
              {openclawConfigPath && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color: 'var(--text-secondary)' }}>
                  <FolderOpen size={14} />
                  <span>{t('openclaw.configPath', '配置目录')}:</span>
                  <span style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: 12 }}>{openclawConfigPath}</span>
                </div>
              )}
              <div style={{ fontSize: 13, color: 'var(--text-secondary)', lineHeight: 1.6, marginTop: 12 }}>
                {t('openclaw.desc', 'OpenClaw 是一个开源的 AI 编码工具。通过此页面管理其 Provider 配置和工作区文件。')}
              </div>
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>{t('openclaw.providerSummary', 'Provider 概览')}</h3>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: 28, fontWeight: 700, color: 'var(--color-primary)' }}>{openclawProviders.length}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 4 }}>{t('openclaw.totalProviders', '总 Provider 数')}</div>
                </div>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: 28, fontWeight: 700, color: 'var(--color-success, #22c55e)' }}>{openclawProviders.filter((p) => p.enabled).length}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 4 }}>{t('openclaw.enabledProviders', '已启用')}</div>
                </div>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: 28, fontWeight: 700, color: 'var(--text-primary)' }}>{openclawProviders.reduce((sum, p) => sum + p.model_count, 0)}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 4 }}>{t('openclaw.totalModels', '总模型数')}</div>
                </div>
              </div>
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>
                <PenTool size={16} style={{ marginRight: 8, verticalAlign: 'middle' }} />
                {t('openclaw.workspaceFiles', '工作区文件')}
              </h3>
              <div style={{ display: 'grid', gap: 8 }}>
                <div style={{ padding: '10px 14px', borderRadius: 6, border: '1px solid var(--border-color)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <FileText size={14} style={{ color: 'var(--text-secondary)' }} />
                    <span style={{ fontSize: 13, fontWeight: 500 }}>AGENTS.md</span>
                  </div>
                  <span style={{ fontSize: 12, color: agentsContent ? 'var(--color-success, #22c55e)' : 'var(--text-tertiary)' }}>
                    {agentsContent ? t('openclaw.fileExists', '已配置') : t('openclaw.fileNotFound', '未找到')}
                  </span>
                </div>
                <div style={{ padding: '10px 14px', borderRadius: 6, border: '1px solid var(--border-color)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <FileText size={14} style={{ color: 'var(--text-secondary)' }} />
                    <span style={{ fontSize: 13, fontWeight: 500 }}>SOUL.md</span>
                  </div>
                  <span style={{ fontSize: 12, color: 'var(--text-tertiary)' }}>{t('openclaw.optional', '可选')}</span>
                </div>
              </div>
            </div>

            <div className="card" style={{ padding: '20px' }}>
              <h3 style={{ margin: '0 0 16px', fontSize: 16, fontWeight: 600 }}>{t('openclaw.quickActions', '快速操作')}</h3>
              <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
                <button className="btn btn-primary" onClick={() => setActiveTab('providers')}>
                  <Layers size={16} />
                  <span>{t('openclaw.manageProviders', '管理 Provider')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.dispatchEvent(new CustomEvent('app-request-navigate', { detail: 'mcp' }))}>
                  <Zap size={16} />
                  <span>{t('openclaw.manageMcp', '管理 MCP')}</span>
                </button>
                <button className="btn btn-secondary" onClick={() => window.open('https://github.com/openclaw-dev/openclaw', '_blank')}>
                  <ExternalLink size={16} />
                  <span>{t('openclaw.docs', '官方文档')}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'providers' && (
        <PlatformProviderPanel modelType="openclaw" />
      )}

      {activeTab === 'instances' && (
        <div className="platform-overview-content" style={{ padding: '24px' }}>
          <div className="empty-state" style={{ textAlign: 'center', padding: '60px 20px' }}>
            <Layers size={48} style={{ color: 'var(--text-tertiary)', marginBottom: 16 }} />
            <h3 style={{ color: 'var(--text-secondary)', marginBottom: 8 }}>{t('openclaw.instancesComingSoon', '多开实例')}</h3>
            <p style={{ color: 'var(--text-tertiary)', fontSize: 14 }}>{t('openclaw.instancesDesc', 'OpenClaw 多实例管理功能即将推出。')}</p>
          </div>
        </div>
      )}
    </div>
  );
}

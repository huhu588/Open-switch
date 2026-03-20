import { useState, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Terminal,
  FileText,
} from 'lucide-react';
import {
  PlatformOverviewTab,
  PlatformOverviewTabsHeader,
} from '../components/platform/PlatformOverviewTabsHeader';
import { PlatformProviderPanel } from '../components/platform/PlatformProviderPanel';
import { useProviderStore } from '../stores/useProviderStore';

type ConfigStatus = 'detected' | 'not-found' | 'loading';

export function WarpAccountsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<PlatformOverviewTab>('overview');
  const store = useProviderStore();
  const [configStatus, setConfigStatus] = useState<ConfigStatus>('loading');
  const [warpVersion, setWarpVersion] = useState<string>('');

  const warpProviders = useMemo(
    () => store.providers.filter((p) => p.model_type === 'warp'),
    [store.providers],
  );

  useEffect(() => {
    store.loadProviders();
    invoke<string>('get_warp_version')
      .then((v) => { setWarpVersion(v); setConfigStatus('detected'); })
      .catch(() => { setConfigStatus('not-found'); });
  }, []);

  const handleRefresh = async () => {
    setConfigStatus('loading');
    await store.loadProviders();
    try {
      const v = await invoke<string>('get_warp_version');
      setWarpVersion(v);
      setConfigStatus('detected');
    } catch { setConfigStatus('not-found'); }
  };

  return (
    <div className="page-container">
      <PlatformOverviewTabsHeader platform="warp" active={activeTab} onTabChange={setActiveTab} />

      {activeTab === 'overview' && (
        <div style={{ padding: '20px 24px' }}>
          <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: 16 }}>
            <button className="btn btn-secondary" onClick={handleRefresh} style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <RefreshCw size={14} /> {t('common.refresh', '刷新')}
            </button>
          </div>

          <div className="card" style={{ padding: 20 }}>
            <h3 style={{ margin: '0 0 16px', fontSize: 15, fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Terminal size={18} />
              Warp {t('platformOverview.status', '状态')}
            </h3>

            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              {configStatus === 'loading' && <Loader2 size={16} className="spin" style={{ color: 'var(--text-tertiary)' }} />}
              {configStatus === 'detected' && (
                <>
                  <CheckCircle2 size={16} style={{ color: 'var(--color-success, #22c55e)' }} />
                  <span style={{ fontSize: 13, color: 'var(--text-primary)' }}>
                    Warp {t('platformOverview.detected', '已检测到')}
                    {warpVersion && <span style={{ color: 'var(--text-tertiary)', marginLeft: 8 }}>v{warpVersion}</span>}
                  </span>
                </>
              )}
              {configStatus === 'not-found' && (
                <>
                  <AlertCircle size={16} style={{ color: 'var(--text-tertiary)' }} />
                  <span style={{ fontSize: 13, color: 'var(--text-tertiary)' }}>
                    Warp {t('platformOverview.notDetected', '未检测到')}
                  </span>
                </>
              )}
            </div>
          </div>

          {warpProviders.length > 0 && (
            <div className="card" style={{ padding: 20, marginTop: 16 }}>
              <h3 style={{ margin: '0 0 12px', fontSize: 15, fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
                <FileText size={18} />
                {t('platformOverview.providerConfig', 'Provider 配置')}
              </h3>
              <p style={{ fontSize: 13, color: 'var(--text-secondary)', margin: 0 }}>
                {t('platformOverview.providerCount', '已配置 {{count}} 个 Provider', { count: warpProviders.length })}
              </p>
            </div>
          )}
        </div>
      )}

      {activeTab === 'providers' && (
        <PlatformProviderPanel platformModelType="warp" />
      )}
    </div>
  );
}

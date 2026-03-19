import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Power, PowerOff, RefreshCw, ExternalLink, Loader2 } from 'lucide-react';

interface Sub2apiStatus {
  running: boolean;
  port: number;
  pid: number | null;
  url: string | null;
}

export function Sub2apiPage() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<Sub2apiStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [iframeLoaded, setIframeLoaded] = useState(false);
  const [iframeError, setIframeError] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const s = await invoke<Sub2apiStatus>('get_sub2api_status');
      setStatus(s);
      return s;
    } catch (error) {
      console.error('Failed to get sub2api status:', error);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleToggle = async () => {
    setActionLoading(true);
    try {
      if (status?.running) {
        await invoke('stop_sub2api');
        setIframeLoaded(false);
        setIframeError(false);
      } else {
        await invoke('start_sub2api');
        setIframeLoaded(false);
        setIframeError(false);
        // 等待一段时间让服务启动
        await new Promise(resolve => setTimeout(resolve, 3000));
      }
      await fetchStatus();
    } catch (error) {
      console.error('Failed to toggle sub2api:', error);
    } finally {
      setActionLoading(false);
    }
  };

  const handleRefreshIframe = () => {
    setIframeLoaded(false);
    setIframeError(false);
    if (iframeRef.current) {
      iframeRef.current.src = iframeRef.current.src;
    }
  };

  const handleOpenExternal = () => {
    if (status?.url) {
      window.open(status.url, '_blank');
    }
  };

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container sub2api-page">
      <div className="page-header sub2api-header">
        <div className="page-header-left">
          <h1 className="page-title">{t('sub2api.title', 'Sub2api 高级网关')}</h1>
          <div className="sub2api-status">
            <span className={`status-dot ${status?.running ? 'status-dot-active' : 'status-dot-inactive'}`} />
            <span className="text-xs">
              {status?.running
                ? t('sub2api.running', '运行中') + ` (PID: ${status.pid || '-'}, Port: ${status.port})`
                : t('sub2api.stopped', '已停止')}
            </span>
          </div>
        </div>
        <div className="page-actions">
          {status?.running && (
            <>
              <button className="btn btn-ghost btn-sm" onClick={handleRefreshIframe}>
                <RefreshCw size={14} />
              </button>
              <button className="btn btn-ghost btn-sm" onClick={handleOpenExternal}>
                <ExternalLink size={14} />
              </button>
            </>
          )}
          <button
            className={`btn btn-sm ${status?.running ? 'btn-error' : 'btn-primary'}`}
            onClick={handleToggle}
            disabled={actionLoading}
          >
            {actionLoading ? (
              <Loader2 size={14} className="animate-spin" />
            ) : status?.running ? (
              <PowerOff size={14} />
            ) : (
              <Power size={14} />
            )}
            <span>
              {status?.running
                ? t('sub2api.stop', '停止')
                : t('sub2api.start', '启动')}
            </span>
          </button>
        </div>
      </div>

      <div className="sub2api-content">
        {!status?.running ? (
          <div className="sub2api-not-running">
            <div className="empty-state">
              <Power size={64} className="empty-icon" />
              <h2>{t('sub2api.notRunning', 'Sub2api 服务未启动')}</h2>
              <p>{t('sub2api.startHint', '点击上方「启动」按钮启动 Sub2api 高级网关服务')}</p>
              <p className="text-xs opacity-50 mt-2">
                {t('sub2api.features', '包含：用户管理、计费系统、订阅管理、API 密钥管理、多平台网关等完整功能')}
              </p>
              <button
                className="btn btn-primary mt-4"
                onClick={handleToggle}
                disabled={actionLoading}
              >
                {actionLoading ? <Loader2 size={14} className="animate-spin" /> : <Power size={14} />}
                <span>{t('sub2api.startService', '启动服务')}</span>
              </button>
            </div>
          </div>
        ) : (
          <div className="sub2api-iframe-container">
            {!iframeLoaded && !iframeError && (
              <div className="sub2api-loading">
                <Loader2 size={32} className="animate-spin" />
                <p>{t('sub2api.loading', '正在加载 Sub2api 界面...')}</p>
              </div>
            )}
            {iframeError && (
              <div className="sub2api-error">
                <p>{t('sub2api.loadError', '加载失败，服务可能还在启动中')}</p>
                <button className="btn btn-primary btn-sm mt-2" onClick={handleRefreshIframe}>
                  <RefreshCw size={14} />
                  <span>{t('sub2api.retry', '重试')}</span>
                </button>
              </div>
            )}
            <iframe
              ref={iframeRef}
              src={status.url || `http://localhost:${status.port}`}
              className={`sub2api-iframe ${iframeLoaded ? 'loaded' : 'hidden'}`}
              onLoad={() => setIframeLoaded(true)}
              onError={() => setIframeError(true)}
              title="Sub2api"
              sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-modals"
            />
          </div>
        )}
      </div>
    </div>
  );
}

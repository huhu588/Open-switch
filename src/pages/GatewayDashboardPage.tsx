import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Power, PowerOff, RefreshCw, Activity, Users, Key, FileText } from 'lucide-react';

interface GatewayStatus {
  running: boolean;
  port: number;
  total_accounts: number;
  active_accounts: number;
  total_api_keys: number;
  total_requests: number;
  uptime_seconds: number | null;
}

interface GatewayConfig {
  enabled: boolean;
  port: number;
  upstream_base_url: string;
  upstream_proxy_url: string | null;
  route_strategy: string;
  auto_start: boolean;
  cors_enabled: boolean;
  max_concurrent_per_account: number;
  cooldown_seconds: number;
}

interface RequestLogSummary {
  total_requests: number;
  success_count: number;
  error_count: number;
  avg_duration_ms: number;
  total_input_tokens: number;
  total_output_tokens: number;
}

export function GatewayDashboardPage() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<GatewayStatus | null>(null);
  const [config, setConfig] = useState<GatewayConfig | null>(null);
  const [summary, setSummary] = useState<RequestLogSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);

  const refreshStatus = useCallback(async () => {
    try {
      const [s, c, sm] = await Promise.all([
        invoke<GatewayStatus>('get_gateway_status'),
        invoke<GatewayConfig>('get_gateway_config'),
        invoke<RequestLogSummary>('get_request_log_summary').catch(() => null),
      ]);
      setStatus(s);
      setConfig(c);
      if (sm) setSummary(sm);
    } catch (error) {
      console.error('Failed to fetch gateway status:', error);
    }
  }, []);

  useEffect(() => {
    setLoading(true);
    refreshStatus().finally(() => setLoading(false));
    const interval = setInterval(refreshStatus, 5000);
    return () => clearInterval(interval);
  }, [refreshStatus]);

  const handleToggleGateway = async () => {
    setActionLoading(true);
    try {
      if (status?.running) {
        await invoke('stop_gateway');
      } else {
        await invoke('start_gateway');
      }
      await refreshStatus();
    } catch (error) {
      console.error('Failed to toggle gateway:', error);
    } finally {
      setActionLoading(false);
    }
  };

  const handleSaveConfig = async (updates: Partial<GatewayConfig>) => {
    if (!config) return;
    const newConfig = { ...config, ...updates };
    try {
      await invoke('save_gateway_config', { config: newConfig });
      setConfig(newConfig);
    } catch (error) {
      console.error('Failed to save config:', error);
    }
  };

  if (loading && !status) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1 className="page-title">{t('gateway.dashboard.title', 'API 网关')}</h1>
        <div className="page-actions">
          <button className="btn btn-ghost btn-sm" onClick={refreshStatus}>
            <RefreshCw size={14} />
          </button>
          <button
            className={`btn btn-sm ${status?.running ? 'btn-error' : 'btn-primary'}`}
            onClick={handleToggleGateway}
            disabled={actionLoading}
          >
            {status?.running ? <PowerOff size={14} /> : <Power size={14} />}
            <span>{status?.running ? t('gateway.stop', '停止网关') : t('gateway.start', '启动网关')}</span>
          </button>
        </div>
      </div>

      <div className="gateway-stats-grid">
        <div className="stat-card">
          <div className="stat-icon"><Activity size={24} /></div>
          <div className="stat-content">
            <div className="stat-value">
              <span className={`status-dot ${status?.running ? 'status-dot-active' : 'status-dot-inactive'}`} />
              {status?.running ? t('gateway.running', '运行中') : t('gateway.stopped', '已停止')}
            </div>
            <div className="stat-label">{t('gateway.status', '服务状态')}</div>
            {status?.running && <div className="stat-sub">:{status.port}</div>}
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon"><Users size={24} /></div>
          <div className="stat-content">
            <div className="stat-value">{status?.active_accounts ?? 0} / {status?.total_accounts ?? 0}</div>
            <div className="stat-label">{t('gateway.accounts', '活跃 / 总账号')}</div>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon"><Key size={24} /></div>
          <div className="stat-content">
            <div className="stat-value">{status?.total_api_keys ?? 0}</div>
            <div className="stat-label">{t('gateway.apiKeys', 'API Keys')}</div>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon"><FileText size={24} /></div>
          <div className="stat-content">
            <div className="stat-value">{summary?.total_requests ?? 0}</div>
            <div className="stat-label">{t('gateway.totalRequests', '总请求数')}</div>
            {summary && summary.total_requests > 0 && (
              <div className="stat-sub">
                {t('gateway.avgLatency', '平均延迟')}: {summary.avg_duration_ms.toFixed(0)}ms
              </div>
            )}
          </div>
        </div>
      </div>

      {summary && summary.total_requests > 0 && (
        <div className="gateway-summary-section">
          <h3>{t('gateway.requestSummary', '请求统计')}</h3>
          <div className="gateway-summary-grid">
            <div className="summary-item">
              <span className="summary-label">{t('gateway.successCount', '成功')}</span>
              <span className="summary-value text-success">{summary.success_count}</span>
            </div>
            <div className="summary-item">
              <span className="summary-label">{t('gateway.errorCount', '失败')}</span>
              <span className="summary-value text-error">{summary.error_count}</span>
            </div>
            <div className="summary-item">
              <span className="summary-label">{t('gateway.inputTokens', '输入 Tokens')}</span>
              <span className="summary-value">{summary.total_input_tokens.toLocaleString()}</span>
            </div>
            <div className="summary-item">
              <span className="summary-label">{t('gateway.outputTokens', '输出 Tokens')}</span>
              <span className="summary-value">{summary.total_output_tokens.toLocaleString()}</span>
            </div>
          </div>
        </div>
      )}

      <div className="gateway-config-section">
        <h3>{t('gateway.config', '网关配置')}</h3>
        {config && (
          <div className="gateway-config-form">
            <div className="form-group">
              <label>{t('gateway.port', '监听端口')}</label>
              <input
                type="number"
                className="input input-bordered input-sm"
                value={config.port}
                onChange={(e) => handleSaveConfig({ port: parseInt(e.target.value) || 48760 })}
                disabled={status?.running}
              />
            </div>
            <div className="form-group">
              <label>{t('gateway.upstreamUrl', '上游地址')}</label>
              <input
                type="text"
                className="input input-bordered input-sm"
                value={config.upstream_base_url}
                onChange={(e) => handleSaveConfig({ upstream_base_url: e.target.value })}
              />
            </div>
            <div className="form-group">
              <label>{t('gateway.proxyUrl', '代理地址')}</label>
              <input
                type="text"
                className="input input-bordered input-sm"
                value={config.upstream_proxy_url || ''}
                placeholder={t('gateway.proxyPlaceholder', '可选，如 socks5://127.0.0.1:1080')}
                onChange={(e) => handleSaveConfig({ upstream_proxy_url: e.target.value || null })}
              />
            </div>
            <div className="form-group">
              <label>{t('gateway.routeStrategy', '路由策略')}</label>
              <select
                className="select select-bordered select-sm"
                value={config.route_strategy}
                onChange={(e) => handleSaveConfig({ route_strategy: e.target.value })}
              >
                <option value="round_robin">{t('gateway.roundRobin', '轮询')}</option>
                <option value="least_used">{t('gateway.leastUsed', '最少使用')}</option>
                <option value="random">{t('gateway.random', '随机')}</option>
                <option value="priority">{t('gateway.priority', '优先级')}</option>
              </select>
            </div>
            <div className="form-group">
              <label className="cursor-pointer label">
                <span>{t('gateway.autoStart', '自动启动')}</span>
                <input
                  type="checkbox"
                  className="toggle toggle-sm toggle-primary"
                  checked={config.auto_start}
                  onChange={(e) => handleSaveConfig({ auto_start: e.target.checked, enabled: e.target.checked })}
                />
              </label>
            </div>
          </div>
        )}
      </div>

      <div className="gateway-endpoint-info">
        <h3>{t('gateway.endpoints', '可用端点')}</h3>
        <div className="endpoint-list">
          <code>POST http://localhost:{config?.port ?? 48760}/v1/chat/completions</code>
          <code>POST http://localhost:{config?.port ?? 48760}/v1/responses</code>
          <code>POST http://localhost:{config?.port ?? 48760}/v1/messages</code>
          <code>GET  http://localhost:{config?.port ?? 48760}/v1/models</code>
        </div>
      </div>
    </div>
  );
}

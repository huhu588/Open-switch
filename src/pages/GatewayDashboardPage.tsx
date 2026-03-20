import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  Power, PowerOff, RefreshCw, Activity, Users, Key, FileText,
  Shield, Clock, Copy, CheckCircle, Settings2, Wifi,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

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

const ROUTE_STRATEGIES = [
  { value: 'round_robin', labelKey: 'gateway.roundRobin', fallback: '轮询' },
  { value: 'least_used', labelKey: 'gateway.leastUsed', fallback: '最少使用' },
  { value: 'random', labelKey: 'gateway.random', fallback: '随机' },
  { value: 'priority', labelKey: 'gateway.priority', fallback: '优先级' },
];

function formatUptime(seconds: number | null | undefined): string {
  if (!seconds || seconds <= 0) return '-';
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function GatewayDashboardPage({ embedded }: { embedded?: boolean } = {}) {
  const { t } = useTranslation();
  const toast = useToast();
  const [status, setStatus] = useState<GatewayStatus | null>(null);
  const [config, setConfig] = useState<GatewayConfig | null>(null);
  const [summary, setSummary] = useState<RequestLogSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);
  const [showConfig, setShowConfig] = useState(false);
  const [copiedEndpoint, setCopiedEndpoint] = useState<string | null>(null);

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
        toast.warning(t('gateway.stopped', '网关已停止'));
      } else {
        await invoke('start_gateway');
        toast.success(t('gateway.started', '网关已启动'));
      }
      await refreshStatus();
    } catch (error) {
      toast.error(String(error));
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
      toast.success(t('gateway.configSaved', '配置已保存'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const copyEndpoint = (endpoint: string) => {
    const url = `http://localhost:${config?.port ?? 48760}${endpoint}`;
    navigator.clipboard.writeText(url);
    setCopiedEndpoint(endpoint);
    toast.success(t('gateway.endpointCopied', '已复制到剪贴板'));
    setTimeout(() => setCopiedEndpoint(null), 2000);
  };

  const successRate = summary && summary.total_requests > 0
    ? Math.round((summary.success_count / summary.total_requests) * 100)
    : 100;

  const healthLevel = successRate >= 95 ? 'good' : successRate >= 80 ? 'warn' : 'bad';
  const healthDot = status?.running
    ? (successRate >= 95 ? 'green' : successRate >= 80 ? 'yellow' : 'red')
    : 'gray';

  if (loading && !status) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  const endpoints = [
    { method: 'POST', path: '/v1/chat/completions' },
    { method: 'POST', path: '/v1/responses' },
    { method: 'POST', path: '/v1/messages' },
    { method: 'GET', path: '/v1/models' },
  ];

  return (
    <div className="page-container">
      <ToastContainer toasts={toast.toasts} />

      {!embedded && (
        <div className="gw-page-header">
          <div className="gw-page-header-left">
            <h1 className="gw-page-title">{t('gateway.dashboard.title', 'API 网关')}</h1>
            <div className="gw-header-status">
              <span className={`gw-health-dot gw-health-dot--${healthDot}`} />
              <span>
                {status?.running
                  ? t('gateway.running', '运行中')
                  : t('gateway.stopped', '已停止')}
              </span>
              {status?.running && status.uptime_seconds != null && (
                <span className="gw-uptime">
                  <Clock size={10} />
                  {formatUptime(status.uptime_seconds)}
                </span>
              )}
            </div>
          </div>
          <div className="gw-page-actions">
            <button className="btn btn-ghost btn-sm" onClick={() => setShowConfig(!showConfig)} title={t('gateway.config', '配置')}>
              <Settings2 size={14} />
            </button>
            <button className="btn btn-ghost btn-sm" onClick={refreshStatus} title={t('common.refresh', '刷新')}>
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
      )}

      {embedded && (
        <div className="gw-embedded-actions">
          <div className="gw-header-status">
            <span className={`gw-health-dot gw-health-dot--${healthDot}`} />
            <span>
              {status?.running
                ? t('gateway.running', '运行中')
                : t('gateway.stopped', '已停止')}
            </span>
            {status?.running && status.uptime_seconds != null && (
              <span className="gw-uptime">
                <Clock size={10} />
                {formatUptime(status.uptime_seconds)}
              </span>
            )}
          </div>
          <div className="gw-page-actions">
            <button className="btn btn-ghost btn-sm" onClick={() => setShowConfig(!showConfig)} title={t('gateway.config', '配置')}>
              <Settings2 size={14} />
            </button>
            <button className="btn btn-ghost btn-sm" onClick={refreshStatus} title={t('common.refresh', '刷新')}>
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
      )}

      {/* Stats Grid */}
      <div className="gateway-stats-grid">
        <div className="gw-stat-card" style={{ animationDelay: '0ms' }}>
          <div className="gw-stat-icon gw-stat-icon--primary">
            <Activity size={22} />
          </div>
          <div className="gw-stat-content">
            <div className="gw-stat-value">
              <span className={`gw-health-dot gw-health-dot--${healthDot}`} />
              {status?.running ? t('gateway.running', '运行中') : t('gateway.stopped', '已停止')}
            </div>
            <div className="gw-stat-label">{t('gateway.status', '服务状态')}</div>
            {status?.running && <div className="gw-stat-sub">:{status.port}</div>}
          </div>
        </div>

        <div className="gw-stat-card" style={{ animationDelay: '50ms' }}>
          <div className="gw-stat-icon gw-stat-icon--success">
            <Users size={22} />
          </div>
          <div className="gw-stat-content">
            <div className="gw-stat-value">{status?.active_accounts ?? 0} / {status?.total_accounts ?? 0}</div>
            <div className="gw-stat-label">{t('gateway.accounts', '活跃 / 总账号')}</div>
          </div>
        </div>

        <div className="gw-stat-card" style={{ animationDelay: '100ms' }}>
          <div className="gw-stat-icon gw-stat-icon--info">
            <Key size={22} />
          </div>
          <div className="gw-stat-content">
            <div className="gw-stat-value">{status?.total_api_keys ?? 0}</div>
            <div className="gw-stat-label">{t('gateway.apiKeys', 'API Keys')}</div>
          </div>
        </div>

        <div className="gw-stat-card" style={{ animationDelay: '150ms' }}>
          <div className="gw-stat-icon gw-stat-icon--warning">
            <FileText size={22} />
          </div>
          <div className="gw-stat-content">
            <div className="gw-stat-value">{summary?.total_requests ?? 0}</div>
            <div className="gw-stat-label">{t('gateway.totalRequests', '总请求数')}</div>
            {summary && summary.total_requests > 0 && (
              <div className="gw-stat-sub">
                {t('gateway.avgLatency', '平均延迟')}: {summary.avg_duration_ms.toFixed(0)}ms
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Health & Request Summary */}
      {summary && summary.total_requests > 0 && (
        <div className="gw-section">
          <div className="gw-section-title">
            <Shield size={16} />
            {t('gateway.healthMonitor', '健康监控')}
          </div>

          <div className="gw-health-bar" style={{ marginBottom: 16 }}>
            <div className="gw-health-bar-track">
              <div
                className={`gw-health-bar-fill gw-health-bar-fill--${healthLevel}`}
                style={{ width: `${successRate}%` }}
              />
            </div>
            <span className="gw-health-bar-pct">{successRate}%</span>
          </div>

          <div className="gw-summary-grid">
            <div className="gw-summary-item">
              <span className="gw-summary-label">{t('gateway.successCount', '成功')}</span>
              <span className="gw-summary-value" style={{ color: 'var(--success)' }}>{summary.success_count}</span>
            </div>
            <div className="gw-summary-item">
              <span className="gw-summary-label">{t('gateway.errorCount', '失败')}</span>
              <span className="gw-summary-value" style={{ color: 'var(--danger)' }}>{summary.error_count}</span>
            </div>
            <div className="gw-summary-item">
              <span className="gw-summary-label">{t('gateway.inputTokens', '输入 Tokens')}</span>
              <span className="gw-summary-value">{summary.total_input_tokens.toLocaleString()}</span>
            </div>
            <div className="gw-summary-item">
              <span className="gw-summary-label">{t('gateway.outputTokens', '输出 Tokens')}</span>
              <span className="gw-summary-value">{summary.total_output_tokens.toLocaleString()}</span>
            </div>
            <div className="gw-summary-item">
              <span className="gw-summary-label">{t('gateway.avgLatency', '平均延迟')}</span>
              <span className="gw-summary-value">{summary.avg_duration_ms.toFixed(0)}ms</span>
            </div>
            <div className="gw-summary-item">
              <span className="gw-summary-label">{t('gateway.successRate', '成功率')}</span>
              <span className="gw-summary-value">{successRate}%</span>
            </div>
          </div>
        </div>
      )}

      {/* Config Section (collapsible) */}
      {showConfig && config && (
        <div className="gw-section">
          <div className="gw-section-title">
            <Settings2 size={16} />
            {t('gateway.config', '网关配置')}
          </div>
          <div className="gw-config-grid">
            <div className="gw-config-field">
              <label className="gw-config-label">{t('gateway.port', '监听端口')}</label>
              <input
                type="number"
                className="input input-bordered input-sm"
                value={config.port}
                onChange={(e) => handleSaveConfig({ port: parseInt(e.target.value) || 48760 })}
                disabled={status?.running}
              />
              <span className="gw-config-hint">{t('gateway.portHint', '运行中无法修改端口')}</span>
            </div>
            <div className="gw-config-field">
              <label className="gw-config-label">{t('gateway.upstreamUrl', '上游地址')}</label>
              <input
                type="text"
                className="input input-bordered input-sm"
                value={config.upstream_base_url}
                onChange={(e) => handleSaveConfig({ upstream_base_url: e.target.value })}
              />
            </div>
            <div className="gw-config-field">
              <label className="gw-config-label">{t('gateway.proxyUrl', '代理地址')}</label>
              <input
                type="text"
                className="input input-bordered input-sm"
                value={config.upstream_proxy_url || ''}
                placeholder={t('gateway.proxyPlaceholder', '可选，如 socks5://127.0.0.1:1080')}
                onChange={(e) => handleSaveConfig({ upstream_proxy_url: e.target.value || null })}
              />
            </div>
            <div className="gw-config-field">
              <label className="gw-config-label">{t('gateway.routeStrategy', '路由策略')}</label>
              <select
                className="select select-bordered select-sm"
                value={config.route_strategy}
                onChange={(e) => handleSaveConfig({ route_strategy: e.target.value })}
              >
                {ROUTE_STRATEGIES.map(s => (
                  <option key={s.value} value={s.value}>{t(s.labelKey, s.fallback)}</option>
                ))}
              </select>
            </div>
            <div className="gw-config-field">
              <label className="gw-config-label">{t('gateway.maxConcurrent', '单账号最大并发')}</label>
              <input
                type="number"
                className="input input-bordered input-sm"
                value={config.max_concurrent_per_account}
                min={1}
                max={100}
                onChange={(e) => handleSaveConfig({ max_concurrent_per_account: parseInt(e.target.value) || 5 })}
              />
            </div>
            <div className="gw-config-field">
              <label className="gw-config-label">{t('gateway.cooldownSeconds', '冷却时间 (秒)')}</label>
              <input
                type="number"
                className="input input-bordered input-sm"
                value={config.cooldown_seconds}
                min={0}
                onChange={(e) => handleSaveConfig({ cooldown_seconds: parseInt(e.target.value) || 0 })}
              />
            </div>
          </div>
          <div style={{ display: 'flex', gap: 16, marginTop: 14 }}>
            <div className="gw-toggle-row" style={{ flex: 1 }}>
              <span className="gw-toggle-row-label">{t('gateway.autoStart', '自动启动')}</span>
              <span
                className={`gw-toggle ${config.auto_start ? 'is-on' : ''}`}
                role="switch"
                aria-checked={config.auto_start}
                onClick={() => handleSaveConfig({ auto_start: !config.auto_start, enabled: !config.auto_start })}
              />
            </div>
            <div className="gw-toggle-row" style={{ flex: 1 }}>
              <span className="gw-toggle-row-label">{t('gateway.cors', 'CORS')}</span>
              <span
                className={`gw-toggle ${config.cors_enabled ? 'is-on' : ''}`}
                role="switch"
                aria-checked={config.cors_enabled}
                onClick={() => handleSaveConfig({ cors_enabled: !config.cors_enabled })}
              />
            </div>
          </div>
        </div>
      )}

      {/* Endpoints */}
      <div className="gw-section">
        <div className="gw-section-title">
          <Wifi size={16} />
          {t('gateway.endpoints', '可用端点')}
        </div>
        <div className="gw-endpoint-list">
          {endpoints.map(ep => (
            <div key={ep.path} className="gw-endpoint-item" onClick={() => copyEndpoint(ep.path)}>
              <span className="gw-endpoint-method">{ep.method}</span>
              <span className="gw-endpoint-path">http://localhost:{config?.port ?? 48760}{ep.path}</span>
              <span className="gw-endpoint-copy">
                {copiedEndpoint === ep.path ? <CheckCircle size={14} /> : <Copy size={14} />}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

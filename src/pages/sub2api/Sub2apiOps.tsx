import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Activity, AlertTriangle, Clock, RefreshCw, Loader2, Wifi, Server } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';

interface OpsStats {
  concurrent_requests?: number;
  total_traffic?: number;
  error_count?: number;
  avg_latency?: number;
  uptime?: number;
  [key: string]: unknown;
}

interface LogEntry {
  id: number;
  level?: string;
  message?: string;
  timestamp?: string;
  [key: string]: unknown;
}

export default function Sub2apiOps() {
  const { t } = useTranslation();
  const [stats, setStats] = useState<OpsStats | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<'overview' | 'logs'>('overview');

  const fetchData = async () => {
    setLoading(true);
    try {
      const [concurrency, traffic, logsData] = await Promise.allSettled([
        sub2apiClient.get<OpsStats>('/admin/ops/concurrency'),
        sub2apiClient.get<OpsStats>('/admin/ops/realtime-traffic'),
        sub2apiClient.get<LogEntry[] | { data?: LogEntry[]; items?: LogEntry[] }>('/admin/ops/system-logs', { page: 1, page_size: 50 }),
      ]);
      const merged: OpsStats = {};
      if (concurrency.status === 'fulfilled' && concurrency.value) Object.assign(merged, concurrency.value);
      if (traffic.status === 'fulfilled' && traffic.value) Object.assign(merged, traffic.value);
      setStats(merged);
      if (logsData.status === 'fulfilled') {
        const ld = logsData.value;
        if (Array.isArray(ld)) setLogs(ld);
        else setLogs((ld as { data?: LogEntry[]; items?: LogEntry[] })?.data || (ld as { items?: LogEntry[] })?.items || []);
      }
    } catch { /* ignore */ }
    finally { setLoading(false); }
  };

  useEffect(() => { fetchData(); }, []);

  if (loading) {
    return <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>;
  }

  return (
    <div>
      <div className="s2a-section-header">
        <h2 className="s2a-page-title">{t('sub2api.ops.title', '运维监控')}</h2>
        <button className="btn btn-ghost btn-sm" onClick={fetchData}><RefreshCw size={14} /></button>
      </div>

      <div className="s2a-tabs">
        <button className={`s2a-tab ${tab === 'overview' ? 'active' : ''}`} onClick={() => setTab('overview')}>
          {t('sub2api.ops.overview', '概览')}
        </button>
        <button className={`s2a-tab ${tab === 'logs' ? 'active' : ''}`} onClick={() => setTab('logs')}>
          {t('sub2api.ops.logs', '日志')}
        </button>
      </div>

      {tab === 'overview' && (
        <div className="s2a-stats-grid">
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.ops.concurrent', '并发请求')}</span>
              <Wifi size={16} className="s2a-stat-card-icon" />
            </div>
            <div className="s2a-stat-card-value">{stats?.concurrent_requests ?? 0}</div>
          </div>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.ops.traffic', '总流量')}</span>
              <Activity size={16} className="s2a-stat-card-icon" />
            </div>
            <div className="s2a-stat-card-value">{stats?.total_traffic ?? 0}</div>
          </div>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.ops.errors', '错误数')}</span>
              <AlertTriangle size={16} className="s2a-stat-card-icon" />
            </div>
            <div className="s2a-stat-card-value">{stats?.error_count ?? 0}</div>
          </div>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.ops.latency', '平均延迟')}</span>
              <Clock size={16} className="s2a-stat-card-icon" />
            </div>
            <div className="s2a-stat-card-value">{stats?.avg_latency ? `${stats.avg_latency}ms` : '-'}</div>
          </div>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.ops.uptime', '运行时间')}</span>
              <Server size={16} className="s2a-stat-card-icon" />
            </div>
            <div className="s2a-stat-card-value">{stats?.uptime ? `${Math.floor(Number(stats.uptime) / 3600)}h` : '-'}</div>
          </div>
        </div>
      )}

      {tab === 'logs' && (
        <div className="s2a-section">
          {logs.length === 0 ? (
            <div className="s2a-empty">
              <div className="s2a-empty-text">{t('sub2api.ops.noLogs', '暂无日志')}</div>
            </div>
          ) : (
            <div style={{ maxHeight: 400, overflowY: 'auto' }}>
              {logs.map((log, i) => (
                <div key={log.id || i} style={{ display: 'flex', gap: 8, padding: '4px 0', borderBottom: '1px solid var(--border-light)', fontSize: '0.68rem' }}>
                  <span style={{ color: 'var(--text-muted)', minWidth: 140, flexShrink: 0 }}>{log.timestamp || '-'}</span>
                  <span className={`s2a-badge s2a-badge--${log.level === 'error' ? 'danger' : log.level === 'warn' ? 'warning' : 'info'}`} style={{ minWidth: 40, textAlign: 'center' }}>
                    {log.level || 'info'}
                  </span>
                  <span style={{ flex: 1, wordBreak: 'break-all' }}>{log.message || '-'}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

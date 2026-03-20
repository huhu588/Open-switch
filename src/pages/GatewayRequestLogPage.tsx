import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  RefreshCw, Trash2, Filter, FileText, ChevronDown, ChevronRight,
  Download, Clock, Zap,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface RequestLogEntry {
  id: number;
  trace_id: string;
  timestamp: number;
  method: string;
  path: string;
  status_code: number;
  duration_ms: number;
  account_email: string | null;
  model: string | null;
  input_tokens: number | null;
  output_tokens: number | null;
  error_message: string | null;
  api_key_prefix: string | null;
}

interface RequestLogQuery {
  limit: number | null;
  offset: number | null;
  status_code: number | null;
  model: string | null;
  start_time: number | null;
  end_time: number | null;
}

export function GatewayRequestLogPage({ embedded }: { embedded?: boolean } = {}) {
  const { t } = useTranslation();
  const toast = useToast();
  const [logs, setLogs] = useState<RequestLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterModel, setFilterModel] = useState('');
  const [filterStatus, setFilterStatus] = useState<string>('');
  const [page, setCurrentPage] = useState(0);
  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const autoRefreshRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const pageSize = 50;

  const fetchLogs = useCallback(async () => {
    try {
      const query: RequestLogQuery = {
        limit: pageSize,
        offset: page * pageSize,
        status_code: filterStatus ? parseInt(filterStatus) : null,
        model: filterModel || null,
        start_time: null,
        end_time: null,
      };
      const data = await invoke<RequestLogEntry[]>('list_request_logs', { query });
      setLogs(data);
    } catch (error) {
      console.error('Failed to fetch request logs:', error);
    } finally {
      setLoading(false);
    }
  }, [page, filterModel, filterStatus]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  useEffect(() => {
    if (autoRefresh) {
      autoRefreshRef.current = setInterval(fetchLogs, 3000);
    }
    return () => {
      if (autoRefreshRef.current) clearInterval(autoRefreshRef.current);
    };
  }, [autoRefresh, fetchLogs]);

  const handleClearLogs = async () => {
    try {
      await invoke('clear_request_logs');
      await fetchLogs();
      toast.success(t('gateway.logsCleared', '日志已清除'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleExportLogs = () => {
    const csv = [
      'Timestamp,Method,Path,Status,Duration(ms),Model,Account,Input Tokens,Output Tokens,Error',
      ...logs.map(l => [
        new Date(l.timestamp * 1000).toISOString(),
        l.method,
        l.path,
        l.status_code,
        l.duration_ms,
        l.model || '',
        l.account_email || '',
        l.input_tokens ?? '',
        l.output_tokens ?? '',
        l.error_message || '',
      ].join(',')),
    ].join('\n');
    navigator.clipboard.writeText(csv);
    toast.success(t('gateway.logsCopied', '日志已复制到剪贴板'));
  };

  const formatTime = (ts: number) => {
    const d = new Date(ts * 1000);
    return d.toLocaleTimeString() + ' ' + d.toLocaleDateString();
  };

  const getStatusStyle = (code: number) => {
    if (code < 300) return 'gw-log-status--ok';
    if (code < 400) return 'gw-log-status--warn';
    return 'gw-log-status--error';
  };

  const getDurationColor = (ms: number) => {
    if (ms < 1000) return 'var(--success)';
    if (ms < 3000) return 'var(--warning)';
    return 'var(--danger)';
  };

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container">
      <ToastContainer toasts={toast.toasts} />

      {!embedded && (
        <div className="gw-page-header">
          <div className="gw-page-header-left">
            <h1 className="gw-page-title">{t('gateway.requestLogs', '请求日志')}</h1>
            <div className="gw-page-subtitle">
              {logs.length > 0 && `${t('gateway.showing', '显示')} ${logs.length} ${t('gateway.entries', '条')}`}
            </div>
          </div>
          <div className="gw-page-actions">
            <button className="btn btn-ghost btn-sm" onClick={handleExportLogs} title={t('gateway.exportLogs', '导出')}>
              <Download size={14} />
            </button>
            <button className="btn btn-ghost btn-sm" onClick={fetchLogs}><RefreshCw size={14} /></button>
            <button className="btn btn-error btn-sm" onClick={handleClearLogs}>
              <Trash2 size={14} /> <span>{t('gateway.clearLogs', '清除')}</span>
            </button>
          </div>
        </div>
      )}

      {embedded && (
        <div className="gw-embedded-actions">
          <div className="gw-page-subtitle" style={{ flex: 1 }}>
            {logs.length > 0 && `${t('gateway.showing', '显示')} ${logs.length} ${t('gateway.entries', '条')}`}
          </div>
          <div className="gw-page-actions">
            <button className="btn btn-ghost btn-sm" onClick={handleExportLogs} title={t('gateway.exportLogs', '导出')}>
              <Download size={14} />
            </button>
            <button className="btn btn-ghost btn-sm" onClick={fetchLogs}><RefreshCw size={14} /></button>
            <button className="btn btn-error btn-sm" onClick={handleClearLogs}>
              <Trash2 size={14} /> <span>{t('gateway.clearLogs', '清除')}</span>
            </button>
          </div>
        </div>
      )}

      {/* Toolbar */}
      <div className="gw-log-toolbar">
        <Filter size={14} style={{ color: 'var(--text-muted)', flexShrink: 0 }} />
        <select
          className="select select-bordered select-sm"
          style={{ minWidth: 130 }}
          value={filterStatus}
          onChange={e => { setFilterStatus(e.target.value); setCurrentPage(0); }}
        >
          <option value="">{t('gateway.allStatus', '全部状态')}</option>
          <option value="200">200 OK</option>
          <option value="400">400 Bad Request</option>
          <option value="401">401 Unauthorized</option>
          <option value="429">429 Rate Limited</option>
          <option value="500">500 Server Error</option>
          <option value="502">502 Bad Gateway</option>
        </select>
        <input
          type="text"
          className="input input-bordered input-sm"
          style={{ maxWidth: 180 }}
          placeholder={t('gateway.filterModel', '筛选模型')}
          value={filterModel}
          onChange={e => { setFilterModel(e.target.value); setCurrentPage(0); }}
        />
        <div className="gw-log-auto-refresh">
          <span>{t('gateway.autoRefresh', '自动刷新')}</span>
          <span
            className={`gw-toggle ${autoRefresh ? 'is-on' : ''}`}
            role="switch"
            aria-checked={autoRefresh}
            onClick={() => setAutoRefresh(!autoRefresh)}
            style={{ width: 32, height: 18 }}
          />
        </div>
      </div>

      {/* Log List */}
      {logs.length === 0 ? (
        <div className="gw-empty">
          <FileText size={56} className="gw-empty-icon" />
          <div className="gw-empty-title">{t('gateway.noLogs', '暂无请求日志')}</div>
          <div className="gw-empty-desc">{t('gateway.noLogsHint', '当网关处理请求时，日志将显示在这里')}</div>
        </div>
      ) : (
        <>
          <div className="gw-list">
            {logs.map((log, i) => {
              const expanded = expandedId === log.id;
              const isError = log.status_code >= 400;
              return (
                <div key={log.id}>
                  <div
                    className={`gw-log-card ${isError ? 'gw-log-card--error' : ''}`}
                    style={{ animationDelay: `${i * 20}ms` }}
                    onClick={() => setExpandedId(expanded ? null : log.id)}
                  >
                    <span className={`gw-log-status ${getStatusStyle(log.status_code)}`}>
                      {log.status_code}
                    </span>
                    <span className="gw-log-method">{log.method}</span>
                    <span className="gw-log-path">{log.path}</span>
                    <div className="gw-log-meta">
                      {log.model && <span>{log.model}</span>}
                      <span style={{ color: getDurationColor(log.duration_ms) }}>
                        <Zap size={10} style={{ display: 'inline', verticalAlign: -1 }} /> {log.duration_ms}ms
                      </span>
                      <span>
                        <Clock size={10} style={{ display: 'inline', verticalAlign: -1 }} /> {new Date(log.timestamp * 1000).toLocaleTimeString()}
                      </span>
                      {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                    </div>
                  </div>

                  {expanded && (
                    <div className="gw-log-detail">
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.traceId', 'Trace ID')}</span>
                        <span className="gw-log-detail-value">{log.trace_id}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.time', '时间')}</span>
                        <span className="gw-log-detail-value">{formatTime(log.timestamp)}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.account', '账号')}</span>
                        <span className="gw-log-detail-value">{log.account_email || '-'}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.apiKey', 'API Key')}</span>
                        <span className="gw-log-detail-value">{log.api_key_prefix ? `${log.api_key_prefix}•••` : '-'}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.tokens', 'Tokens')}</span>
                        <span className="gw-log-detail-value">
                          {log.input_tokens != null && log.output_tokens != null
                            ? `${log.input_tokens.toLocaleString()} → ${log.output_tokens.toLocaleString()}`
                            : '-'}
                        </span>
                      </div>
                      {log.error_message && (
                        <div className="gw-log-detail-item" style={{ gridColumn: '1 / -1' }}>
                          <span className="gw-log-detail-label" style={{ color: 'var(--danger)' }}>{t('gateway.errorMessage', '错误信息')}</span>
                          <span className="gw-log-detail-value" style={{ color: 'var(--danger)' }}>{log.error_message}</span>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          {/* Pagination */}
          <div className="gw-pagination">
            <button className="btn btn-ghost btn-xs" disabled={page === 0} onClick={() => setCurrentPage(page - 1)}>
              {t('common.prev', '上一页')}
            </button>
            <span className="gw-pagination-info">
              {t('gateway.pageInfo', '第 {{page}} 页', { page: page + 1 })}
            </span>
            <button className="btn btn-ghost btn-xs" disabled={logs.length < pageSize} onClick={() => setCurrentPage(page + 1)}>
              {t('common.next', '下一页')}
            </button>
          </div>
        </>
      )}
    </div>
  );
}

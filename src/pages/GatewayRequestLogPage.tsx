import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { RefreshCw, Trash2, Filter, FileText } from 'lucide-react';

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

export function GatewayRequestLogPage() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<RequestLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterModel, setFilterModel] = useState('');
  const [filterStatus, setFilterStatus] = useState<string>('');
  const [page, setCurrentPage] = useState(0);
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

  const handleClearLogs = async () => {
    if (!confirm(t('gateway.confirmClearLogs', '确定清除所有请求日志？'))) return;
    try {
      await invoke('clear_request_logs');
      await fetchLogs();
    } catch (error) {
      console.error('Failed to clear logs:', error);
    }
  };

  const formatTime = (ts: number) => {
    return new Date(ts * 1000).toLocaleString();
  };

  const getStatusClass = (code: number) => {
    if (code < 300) return 'text-success';
    if (code < 400) return 'text-warning';
    return 'text-error';
  };

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1 className="page-title">{t('gateway.requestLogs', '请求日志')}</h1>
        <div className="page-actions">
          <button className="btn btn-ghost btn-sm" onClick={fetchLogs}>
            <RefreshCw size={14} />
          </button>
          <button className="btn btn-error btn-sm" onClick={handleClearLogs}>
            <Trash2 size={14} />
            <span>{t('gateway.clearLogs', '清除')}</span>
          </button>
        </div>
      </div>

      <div className="gateway-log-filters">
        <div className="flex gap-2 items-center">
          <Filter size={14} />
          <select
            className="select select-bordered select-xs"
            value={filterStatus}
            onChange={(e) => { setFilterStatus(e.target.value); setCurrentPage(0); }}
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
            className="input input-bordered input-xs"
            placeholder={t('gateway.filterModel', '筛选模型')}
            value={filterModel}
            onChange={(e) => { setFilterModel(e.target.value); setCurrentPage(0); }}
          />
        </div>
      </div>

      <div className="gateway-log-list">
        {logs.length === 0 ? (
          <div className="empty-state">
            <FileText size={48} className="empty-icon" />
            <p>{t('gateway.noLogs', '暂无请求日志')}</p>
          </div>
        ) : (
          <>
            <table className="table table-compact table-zebra w-full">
              <thead>
                <tr>
                  <th>{t('gateway.time', '时间')}</th>
                  <th>{t('gateway.method', '方法')}</th>
                  <th>{t('gateway.pathLabel', '路径')}</th>
                  <th>{t('gateway.statusCode', '状态码')}</th>
                  <th>{t('gateway.duration', '耗时')}</th>
                  <th>{t('gateway.model', '模型')}</th>
                  <th>{t('gateway.account', '账号')}</th>
                  <th>{t('gateway.tokens', 'Tokens')}</th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <tr key={log.id}>
                    <td className="text-xs opacity-70">{formatTime(log.timestamp)}</td>
                    <td><span className="badge badge-outline badge-xs">{log.method}</span></td>
                    <td className="text-xs font-mono max-w-[200px] truncate">{log.path}</td>
                    <td>
                      <span className={`font-mono font-bold ${getStatusClass(log.status_code)}`}>
                        {log.status_code}
                      </span>
                    </td>
                    <td className="text-xs">{log.duration_ms}ms</td>
                    <td className="text-xs">{log.model || '-'}</td>
                    <td className="text-xs opacity-70">{log.account_email || '-'}</td>
                    <td className="text-xs">
                      {log.input_tokens != null && log.output_tokens != null
                        ? `${log.input_tokens}/${log.output_tokens}`
                        : '-'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            <div className="gateway-log-pagination">
              <button
                className="btn btn-ghost btn-xs"
                disabled={page === 0}
                onClick={() => setCurrentPage(page - 1)}
              >
                {t('common.prev', '上一页')}
              </button>
              <span className="text-xs opacity-70">
                {t('gateway.pageInfo', '第 {{page}} 页', { page: page + 1 })}
              </span>
              <button
                className="btn btn-ghost btn-xs"
                disabled={logs.length < pageSize}
                onClick={() => setCurrentPage(page + 1)}
              >
                {t('common.next', '下一页')}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

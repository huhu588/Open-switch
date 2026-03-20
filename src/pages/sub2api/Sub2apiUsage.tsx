import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { RefreshCw, Loader2, ChevronLeft, ChevronRight, Search, Trash2 } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface UsageRecord {
  id: number;
  user_id?: number;
  username?: string;
  model?: string;
  input_tokens?: number;
  output_tokens?: number;
  quota?: number;
  created_at?: string;
  [key: string]: unknown;
}

export default function Sub2apiUsage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);

  const fetchRecords = useCallback(async () => {
    setLoading(true);
    try {
      const params: Record<string, unknown> = { page, page_size: pageSize };
      if (search) params.keyword = search;
      const resp = await sub2apiClient.get<UsageRecord[] | { data?: UsageRecord[]; total?: number }>('/admin/usage', params);
      if (Array.isArray(resp)) { setRecords(resp); setTotal(resp.length); }
      else if (resp) {
        const items = resp.data || [];
        setRecords(Array.isArray(items) ? items : []);
        setTotal(Number(resp.total ?? items.length));
      }
    } catch (err) { console.error('[Sub2api Usage]', err); }
    finally { setLoading(false); }
  }, [page, pageSize, search]);

  useEffect(() => { fetchRecords(); }, [fetchRecords]);

  const handleClear = async () => {
    try {
      await sub2apiClient.post('/admin/usage/cleanup-tasks', { type: 'all' });
      toast.success(t('sub2api.usage.clearSuccess', '清理任务已创建'));
      fetchRecords();
    } catch (err) { toast.error(String(err)); }
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.usage.title', '使用记录')}</h2>
      <div className="s2a-toolbar">
        <div style={{ position: 'relative', flex: 1, minWidth: 180 }}>
          <Search size={14} style={{ position: 'absolute', left: 8, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
          <input className="s2a-search-input" style={{ paddingLeft: 28 }} placeholder={t('sub2api.usage.search', '搜索...')} value={search} onChange={(e) => { setSearch(e.target.value); setPage(1); }} />
        </div>
        <button className="btn btn-ghost btn-sm" onClick={handleClear}><Trash2 size={14} /> {t('sub2api.usage.clear', '清除')}</button>
        <button className="btn btn-ghost btn-sm" onClick={fetchRecords}><RefreshCw size={14} /></button>
      </div>
      {loading ? (
        <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>
      ) : records.length === 0 ? (
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.usage.empty', '暂无记录')}</div></div>
      ) : (
        <>
          <div className="s2a-table-wrapper">
            <table className="s2a-table">
              <thead><tr><th>ID</th><th>{t('sub2api.usage.user', '用户')}</th><th>{t('sub2api.usage.model', '模型')}</th><th>Input</th><th>Output</th><th>{t('sub2api.usage.quota', '消耗')}</th><th>{t('sub2api.usage.time', '时间')}</th></tr></thead>
              <tbody>
                {records.map((r) => (
                  <tr key={r.id}>
                    <td>{r.id}</td>
                    <td>{r.username || r.user_id || '-'}</td>
                    <td><span className="s2a-badge s2a-badge--info">{r.model || '-'}</span></td>
                    <td>{r.input_tokens?.toLocaleString() ?? '-'}</td>
                    <td>{r.output_tokens?.toLocaleString() ?? '-'}</td>
                    <td>{r.quota ?? '-'}</td>
                    <td style={{ fontSize: '0.65rem', color: 'var(--text-muted)' }}>{r.created_at || '-'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="s2a-pagination">
            <div className="s2a-pagination-info">{t('sub2api.pagination.total', '共')} {total} {t('sub2api.pagination.items', '条')}</div>
            <div className="s2a-pagination-buttons">
              <button className="s2a-pagination-btn" disabled={page <= 1} onClick={() => setPage(p => p - 1)}><ChevronLeft size={14} /></button>
              <span style={{ padding: '0 8px', fontSize: '0.68rem' }}>{page} / {totalPages}</span>
              <button className="s2a-pagination-btn" disabled={page >= totalPages} onClick={() => setPage(p => p + 1)}><ChevronRight size={14} /></button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

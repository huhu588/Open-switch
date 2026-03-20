import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { RefreshCw, Loader2, ChevronLeft, ChevronRight } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';

interface UsageRecord {
  id: number;
  model?: string;
  input_tokens?: number;
  output_tokens?: number;
  quota?: number;
  created_at?: string;
  [key: string]: unknown;
}

export default function UserUsage() {
  const { t } = useTranslation();
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);

  const fetchRecords = useCallback(async () => {
    setLoading(true);
    try {
      const resp = await sub2apiClient.get<UsageRecord[] | { data?: UsageRecord[]; total?: number }>('/usage', { page, page_size: 20 });
      if (Array.isArray(resp)) { setRecords(resp); setTotal(resp.length); }
      else if (resp) { setRecords(resp.data || []); setTotal(Number(resp.total ?? 0)); }
    } catch (err) { console.error('[User Usage]', err); }
    finally { setLoading(false); }
  }, [page]);

  useEffect(() => { fetchRecords(); }, [fetchRecords]);
  const totalPages = Math.max(1, Math.ceil(total / 20));

  return (
    <div>
      <div className="s2a-section-header">
        <h2 className="s2a-page-title">{t('sub2api.userUsage.title', '使用记录')}</h2>
        <button className="btn btn-ghost btn-sm" onClick={fetchRecords}><RefreshCw size={14} /></button>
      </div>
      {loading ? (
        <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>
      ) : records.length === 0 ? (
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.userUsage.empty', '暂无记录')}</div></div>
      ) : (
        <>
          <div className="s2a-table-wrapper">
            <table className="s2a-table">
              <thead><tr><th>{t('sub2api.usage.model', '模型')}</th><th>Input</th><th>Output</th><th>{t('sub2api.usage.quota', '消耗')}</th><th>{t('sub2api.usage.time', '时间')}</th></tr></thead>
              <tbody>
                {records.map((r) => (
                  <tr key={r.id}>
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

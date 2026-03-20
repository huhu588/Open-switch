import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Copy, CheckCircle, Loader2, X, ChevronLeft, ChevronRight } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface ApiKey {
  id: number;
  key?: string;
  name?: string;
  status?: number;
  user_id?: number;
  group?: string;
  used_count?: number;
  remaining_quota?: number;
  created_at?: string;
  [key: string]: unknown;
}

export default function Sub2apiApiKeys() {
  const { t } = useTranslation();
  const toast = useToast();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: '' });
  const [copied, setCopied] = useState<number | null>(null);

  const fetchKeys = useCallback(async () => {
    setLoading(true);
    try {
      const resp = await sub2apiClient.get<ApiKey[] | { data?: ApiKey[]; items?: ApiKey[]; total?: number }>('/keys', { page, page_size: pageSize });
      if (Array.isArray(resp)) {
        setKeys(resp);
        setTotal(resp.length);
      } else if (resp && typeof resp === 'object') {
        const items = resp.data || resp.items || [];
        setKeys(Array.isArray(items) ? items : []);
        setTotal(Number(resp.total ?? items.length));
      }
    } catch (err) { console.error('[Sub2api ApiKeys]', err); }
    finally { setLoading(false); }
  }, [page, pageSize]);

  useEffect(() => { fetchKeys(); }, [fetchKeys]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/keys', form as unknown as Record<string, unknown>);
      toast.success(t('sub2api.apiKeys.addSuccess', 'Key 已创建'));
      setShowAdd(false);
      setForm({ name: '' });
      fetchKeys();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/keys/${id}`);
      toast.success(t('sub2api.apiKeys.deleteSuccess', '已删除'));
      fetchKeys();
    } catch (err) { toast.error(String(err)); }
  };

  const handleCopy = (id: number, key?: string) => {
    if (!key) return;
    navigator.clipboard.writeText(key);
    setCopied(id);
    toast.success(t('sub2api.endpointCopied', '已复制'));
    setTimeout(() => setCopied(null), 2000);
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.apiKeys.title', 'API Key 管理')}</h2>

      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}>
          <Plus size={14} /> {t('common.add', '添加')}
        </button>
        <button className="btn btn-ghost btn-sm" onClick={fetchKeys}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.apiKeys.addTitle', '创建 API Key')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.apiKeys.name', '名称')}</label>
                <input className="s2a-form-input" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder={t('sub2api.apiKeys.namePlaceholder', '可选')} />
              </div>
            </div>
            <div className="s2a-modal-actions">
              <button className="btn btn-ghost btn-sm" onClick={() => setShowAdd(false)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary btn-sm" onClick={handleAdd}>{t('common.confirm', '确定')}</button>
            </div>
          </div>
        </div>
      )}

      {loading ? (
        <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>
      ) : keys.length === 0 ? (
        <div className="s2a-empty">
          <div className="s2a-empty-text">{t('sub2api.apiKeys.empty', '暂无 API Key')}</div>
        </div>
      ) : (
        <>
          <div className="s2a-table-wrapper">
            <table className="s2a-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>{t('sub2api.apiKeys.name', '名称')}</th>
                  <th>Key</th>
                  <th>{t('sub2api.accounts.status', '状态')}</th>
                  <th>{t('sub2api.accounts.actions', '操作')}</th>
                </tr>
              </thead>
              <tbody>
                {keys.map((k) => (
                  <tr key={k.id}>
                    <td>{k.id}</td>
                    <td>{k.name || '-'}</td>
                    <td>
                      <span className="s2a-truncate" style={{ maxWidth: 180, fontFamily: 'monospace', fontSize: '0.65rem' }}>
                        {k.key ? `${k.key.slice(0, 8)}...${k.key.slice(-4)}` : '-'}
                      </span>
                    </td>
                    <td>
                      {k.status === 1
                        ? <span className="s2a-badge s2a-badge--success">{t('common.enabled', '启用')}</span>
                        : <span className="s2a-badge s2a-badge--danger">{t('common.disabled', '禁用')}</span>}
                    </td>
                    <td>
                      <div className="s2a-actions">
                        <button className="s2a-action-btn" onClick={() => handleCopy(k.id, k.key)}>
                          {copied === k.id ? <CheckCircle size={14} /> : <Copy size={14} />}
                        </button>
                        <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(k.id)}><Trash2 size={14} /></button>
                      </div>
                    </td>
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

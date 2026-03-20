import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Copy, CheckCircle, Loader2, X } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

interface UserKey {
  id: number;
  key?: string;
  name?: string;
  status?: number;
  group?: string;
  created_at?: string;
  [key: string]: unknown;
}

export default function UserKeys() {
  const { t } = useTranslation();
  const toast = useToast();
  const [keys, setKeys] = useState<UserKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: '' });
  const [copied, setCopied] = useState<number | null>(null);

  const fetchKeys = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<UserKey[] | { data?: UserKey[] }>('/keys');
      const items = Array.isArray(data) ? data : (data as { data?: UserKey[] })?.data || [];
      setKeys(items);
    } catch (err) { console.error('[User Keys]', err); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchKeys(); }, [fetchKeys]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/keys', form as unknown as Record<string, unknown>);
      toast.success(t('sub2api.userKeys.addSuccess', 'Key 已创建'));
      setShowAdd(false);
      setForm({ name: '' });
      fetchKeys();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/keys/${id}`);
      toast.success(t('sub2api.userKeys.deleteSuccess', '已删除'));
      fetchKeys();
    } catch (err) { toast.error(String(err)); }
  };

  const handleCopy = (id: number, key?: string) => {
    if (!key) return;
    navigator.clipboard.writeText(key);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.userKeys.title', '我的 API Key')}</h2>
      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}><Plus size={14} /> {t('common.add', '添加')}</button>
        <button className="btn btn-ghost btn-sm" onClick={fetchKeys}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.userKeys.addTitle', '创建 API Key')}</div>
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
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.userKeys.empty', '暂无 API Key')}</div></div>
      ) : (
        <div className="s2a-table-wrapper">
          <table className="s2a-table">
            <thead><tr><th>{t('sub2api.apiKeys.name', '名称')}</th><th>Key</th><th>{t('sub2api.accounts.status', '状态')}</th><th>{t('sub2api.accounts.actions', '操作')}</th></tr></thead>
            <tbody>
              {keys.map((k) => (
                <tr key={k.id}>
                  <td>{k.name || '-'}</td>
                  <td><span style={{ fontFamily: 'monospace', fontSize: '0.65rem' }}>{k.key ? `${k.key.slice(0, 8)}...${k.key.slice(-4)}` : '-'}</span></td>
                  <td>{k.status === 1 ? <span className="s2a-badge s2a-badge--success">{t('common.enabled', '启用')}</span> : <span className="s2a-badge s2a-badge--danger">{t('common.disabled', '禁用')}</span>}</td>
                  <td>
                    <div className="s2a-actions">
                      <button className="s2a-action-btn" onClick={() => handleCopy(k.id, k.key)}>{copied === k.id ? <CheckCircle size={14} /> : <Copy size={14} />}</button>
                      <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(k.id)}><Trash2 size={14} /></button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

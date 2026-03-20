import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Loader2, X } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface Proxy {
  id: number;
  name?: string;
  url?: string;
  type?: string;
  status?: number;
  [key: string]: unknown;
}

export default function Sub2apiProxies() {
  const { t } = useTranslation();
  const toast = useToast();
  const [proxies, setProxies] = useState<Proxy[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: '', url: '', type: 'http' });

  const fetchProxies = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<Proxy[] | { data?: Proxy[] }>('/admin/proxies');
      const items = Array.isArray(data) ? data : (data as { data?: Proxy[] })?.data || [];
      setProxies(items);
    } catch (err) { console.error('[Sub2api Proxies]', err); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchProxies(); }, [fetchProxies]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/proxies', form as unknown as Record<string, unknown>);
      toast.success(t('sub2api.proxies.addSuccess', '代理已添加'));
      setShowAdd(false);
      setForm({ name: '', url: '', type: 'http' });
      fetchProxies();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/proxies/${id}`);
      toast.success(t('sub2api.proxies.deleteSuccess', '已删除'));
      fetchProxies();
    } catch (err) { toast.error(String(err)); }
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.proxies.title', '代理管理')}</h2>

      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}>
          <Plus size={14} /> {t('common.add', '添加')}
        </button>
        <button className="btn btn-ghost btn-sm" onClick={fetchProxies}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.proxies.addTitle', '添加代理')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.proxies.name', '名称')}</label>
                <input className="s2a-form-input" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">URL</label>
                <input className="s2a-form-input" value={form.url} onChange={(e) => setForm({ ...form, url: e.target.value })} placeholder="http://proxy:port" />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.proxies.type', '类型')}</label>
                <select className="s2a-form-select" value={form.type} onChange={(e) => setForm({ ...form, type: e.target.value })}>
                  <option value="http">HTTP</option>
                  <option value="socks5">SOCKS5</option>
                </select>
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
      ) : proxies.length === 0 ? (
        <div className="s2a-empty">
          <div className="s2a-empty-text">{t('sub2api.proxies.empty', '暂无代理')}</div>
        </div>
      ) : (
        <div className="s2a-table-wrapper">
          <table className="s2a-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>{t('sub2api.proxies.name', '名称')}</th>
                <th>URL</th>
                <th>{t('sub2api.proxies.type', '类型')}</th>
                <th>{t('sub2api.accounts.actions', '操作')}</th>
              </tr>
            </thead>
            <tbody>
              {proxies.map((p) => (
                <tr key={p.id}>
                  <td>{p.id}</td>
                  <td>{p.name || '-'}</td>
                  <td><span className="s2a-truncate">{p.url || '-'}</span></td>
                  <td><span className="s2a-badge s2a-badge--info">{p.type || 'http'}</span></td>
                  <td>
                    <div className="s2a-actions">
                      <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(p.id)}><Trash2 size={14} /></button>
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

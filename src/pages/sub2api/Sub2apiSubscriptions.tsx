import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Loader2, X } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface Subscription {
  id: number;
  name?: string;
  price?: number;
  quota?: number;
  duration_days?: number;
  models?: string;
  status?: number;
  [key: string]: unknown;
}

export default function Sub2apiSubscriptions() {
  const { t } = useTranslation();
  const toast = useToast();
  const [subs, setSubs] = useState<Subscription[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: '', price: '9.9', quota: '100000', duration_days: '30' });

  const fetchSubs = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<Subscription[] | { data?: Subscription[] }>('/admin/subscriptions');
      const items = Array.isArray(data) ? data : (data as { data?: Subscription[] })?.data || [];
      setSubs(items);
    } catch (err) { console.error('[Sub2api Subscriptions]', err); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchSubs(); }, [fetchSubs]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/subscriptions/assign', {
        name: form.name,
        price: parseFloat(form.price) || 0,
        quota: parseInt(form.quota) || 0,
        duration_days: parseInt(form.duration_days) || 30,
      });
      toast.success(t('sub2api.subscriptions.addSuccess', '订阅计划已创建'));
      setShowAdd(false);
      setForm({ name: '', price: '9.9', quota: '100000', duration_days: '30' });
      fetchSubs();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/subscriptions/${id}`);
      toast.success(t('sub2api.subscriptions.deleteSuccess', '已删除'));
      fetchSubs();
    } catch (err) { toast.error(String(err)); }
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.subscriptions.title', '订阅管理')}</h2>
      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}><Plus size={14} /> {t('common.add', '添加')}</button>
        <button className="btn btn-ghost btn-sm" onClick={fetchSubs}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.subscriptions.addTitle', '创建订阅计划')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.subscriptions.name', '名称')}</label>
                <input className="s2a-form-input" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.subscriptions.price', '价格')}</label>
                <input className="s2a-form-input" type="number" step="0.01" value={form.price} onChange={(e) => setForm({ ...form, price: e.target.value })} />
              </div>
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.subscriptions.quota', '额度')}</label>
                <input className="s2a-form-input" type="number" value={form.quota} onChange={(e) => setForm({ ...form, quota: e.target.value })} />
              </div>
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.subscriptions.duration', '天数')}</label>
                <input className="s2a-form-input" type="number" value={form.duration_days} onChange={(e) => setForm({ ...form, duration_days: e.target.value })} />
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
      ) : subs.length === 0 ? (
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.subscriptions.empty', '暂无订阅计划')}</div></div>
      ) : (
        <div className="s2a-table-wrapper">
          <table className="s2a-table">
            <thead><tr><th>ID</th><th>{t('sub2api.subscriptions.name', '名称')}</th><th>{t('sub2api.subscriptions.price', '价格')}</th><th>{t('sub2api.subscriptions.quota', '额度')}</th><th>{t('sub2api.subscriptions.duration', '天数')}</th><th>{t('sub2api.accounts.actions', '操作')}</th></tr></thead>
            <tbody>
              {subs.map((s) => (
                <tr key={s.id}>
                  <td>{s.id}</td>
                  <td>{s.name || '-'}</td>
                  <td>¥{s.price?.toFixed(2) ?? '-'}</td>
                  <td>{s.quota?.toLocaleString() ?? '-'}</td>
                  <td>{s.duration_days ?? '-'}</td>
                  <td><div className="s2a-actions"><button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(s.id)}><Trash2 size={14} /></button></div></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

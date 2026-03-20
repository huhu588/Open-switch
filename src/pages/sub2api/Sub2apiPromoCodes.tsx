import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Loader2, X } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface PromoCode {
  id: number;
  code?: string;
  discount?: number;
  type?: string;
  max_uses?: number;
  used_count?: number;
  expires_at?: string;
  status?: number;
  [key: string]: unknown;
}

export default function Sub2apiPromoCodes() {
  const { t } = useTranslation();
  const toast = useToast();
  const [codes, setCodes] = useState<PromoCode[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ code: '', discount: '10', type: 'percentage' });

  const fetchCodes = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<PromoCode[] | { data?: PromoCode[] }>('/admin/promo-codes');
      const items = Array.isArray(data) ? data : (data as { data?: PromoCode[] })?.data || [];
      setCodes(items);
    } catch (err) { console.error('[Sub2api PromoCodes]', err); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchCodes(); }, [fetchCodes]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/promo-codes', { code: form.code, discount: parseFloat(form.discount) || 10, type: form.type });
      toast.success(t('sub2api.promoCodes.addSuccess', '优惠码已创建'));
      setShowAdd(false);
      setForm({ code: '', discount: '10', type: 'percentage' });
      fetchCodes();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/promo-codes/${id}`);
      toast.success(t('sub2api.promoCodes.deleteSuccess', '已删除'));
      fetchCodes();
    } catch (err) { toast.error(String(err)); }
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.promoCodes.title', '优惠码管理')}</h2>
      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}><Plus size={14} /> {t('common.add', '添加')}</button>
        <button className="btn btn-ghost btn-sm" onClick={fetchCodes}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.promoCodes.addTitle', '创建优惠码')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.promoCodes.code', '优惠码')}</label>
                <input className="s2a-form-input" value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.promoCodes.discount', '折扣')}</label>
                <input className="s2a-form-input" type="number" value={form.discount} onChange={(e) => setForm({ ...form, discount: e.target.value })} />
              </div>
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.promoCodes.type', '类型')}</label>
                <select className="s2a-form-select" value={form.type} onChange={(e) => setForm({ ...form, type: e.target.value })}>
                  <option value="percentage">{t('sub2api.promoCodes.percentage', '百分比')}</option>
                  <option value="fixed">{t('sub2api.promoCodes.fixed', '固定金额')}</option>
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
      ) : codes.length === 0 ? (
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.promoCodes.empty', '暂无优惠码')}</div></div>
      ) : (
        <div className="s2a-table-wrapper">
          <table className="s2a-table">
            <thead><tr><th>ID</th><th>{t('sub2api.promoCodes.code', '优惠码')}</th><th>{t('sub2api.promoCodes.discount', '折扣')}</th><th>{t('sub2api.promoCodes.type', '类型')}</th><th>{t('sub2api.promoCodes.used', '已用/最大')}</th><th>{t('sub2api.accounts.actions', '操作')}</th></tr></thead>
            <tbody>
              {codes.map((c) => (
                <tr key={c.id}>
                  <td>{c.id}</td>
                  <td style={{ fontFamily: 'monospace' }}>{c.code || '-'}</td>
                  <td>{c.discount ?? '-'}{c.type === 'percentage' ? '%' : ''}</td>
                  <td><span className="s2a-badge s2a-badge--info">{c.type || '-'}</span></td>
                  <td>{c.used_count ?? 0} / {c.max_uses ?? '∞'}</td>
                  <td><div className="s2a-actions"><button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(c.id)}><Trash2 size={14} /></button></div></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

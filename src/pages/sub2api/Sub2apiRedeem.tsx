import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Loader2, X, Copy, CheckCircle } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface RedeemCode {
  id: number;
  code?: string;
  quota?: number;
  count?: number;
  used_count?: number;
  status?: number;
  created_at?: string;
  [key: string]: unknown;
}

export default function Sub2apiRedeem() {
  const { t } = useTranslation();
  const toast = useToast();
  const [codes, setCodes] = useState<RedeemCode[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ quota: '1000', count: '1' });
  const [copied, setCopied] = useState<number | null>(null);

  const fetchCodes = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<RedeemCode[] | { data?: RedeemCode[] }>('/admin/redeem-codes');
      const items = Array.isArray(data) ? data : (data as { data?: RedeemCode[] })?.data || [];
      setCodes(items);
    } catch (err) { console.error('[Sub2api Redeem]', err); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchCodes(); }, [fetchCodes]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/redeem-codes/generate', { quota: parseInt(form.quota) || 1000, count: parseInt(form.count) || 1 });
      toast.success(t('sub2api.redeem.addSuccess', '兑换码已生成'));
      setShowAdd(false);
      fetchCodes();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/redeem-codes/${id}`);
      toast.success(t('sub2api.redeem.deleteSuccess', '已删除'));
      fetchCodes();
    } catch (err) { toast.error(String(err)); }
  };

  const handleCopy = (id: number, code?: string) => {
    if (!code) return;
    navigator.clipboard.writeText(code);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.redeem.title', '兑换码管理')}</h2>
      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}><Plus size={14} /> {t('sub2api.redeem.generate', '生成')}</button>
        <button className="btn btn-ghost btn-sm" onClick={fetchCodes}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.redeem.addTitle', '生成兑换码')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.redeem.quota', '额度')}</label>
                <input className="s2a-form-input" type="number" value={form.quota} onChange={(e) => setForm({ ...form, quota: e.target.value })} />
              </div>
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.redeem.count', '数量')}</label>
                <input className="s2a-form-input" type="number" value={form.count} onChange={(e) => setForm({ ...form, count: e.target.value })} />
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
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.redeem.empty', '暂无兑换码')}</div></div>
      ) : (
        <div className="s2a-table-wrapper">
          <table className="s2a-table">
            <thead><tr><th>ID</th><th>{t('sub2api.redeem.code', '兑换码')}</th><th>{t('sub2api.redeem.quota', '额度')}</th><th>{t('sub2api.redeem.used', '已用/总')}</th><th>{t('sub2api.accounts.actions', '操作')}</th></tr></thead>
            <tbody>
              {codes.map((c) => (
                <tr key={c.id}>
                  <td>{c.id}</td>
                  <td><span style={{ fontFamily: 'monospace', fontSize: '0.65rem' }}>{c.code || '-'}</span></td>
                  <td>{c.quota ?? '-'}</td>
                  <td>{c.used_count ?? 0} / {c.count ?? '-'}</td>
                  <td>
                    <div className="s2a-actions">
                      <button className="s2a-action-btn" onClick={() => handleCopy(c.id, c.code)}>{copied === c.id ? <CheckCircle size={14} /> : <Copy size={14} />}</button>
                      <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(c.id)}><Trash2 size={14} /></button>
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

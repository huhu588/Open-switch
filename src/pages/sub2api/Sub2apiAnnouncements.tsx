import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Loader2, X } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface Announcement {
  id: number;
  title?: string;
  content?: string;
  status?: number;
  created_at?: string;
  [key: string]: unknown;
}

export default function Sub2apiAnnouncements() {
  const { t } = useTranslation();
  const toast = useToast();
  const [items, setItems] = useState<Announcement[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ title: '', content: '' });

  const fetchItems = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<Announcement[] | { data?: Announcement[] }>('/admin/announcements');
      const list = Array.isArray(data) ? data : (data as { data?: Announcement[] })?.data || [];
      setItems(list);
    } catch (err) { console.error('[Sub2api Announcements]', err); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchItems(); }, [fetchItems]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/announcements', form as unknown as Record<string, unknown>);
      toast.success(t('sub2api.announcements.addSuccess', '公告已发布'));
      setShowAdd(false);
      setForm({ title: '', content: '' });
      fetchItems();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/announcements/${id}`);
      toast.success(t('sub2api.announcements.deleteSuccess', '已删除'));
      fetchItems();
    } catch (err) { toast.error(String(err)); }
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.announcements.title', '公告管理')}</h2>
      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}><Plus size={14} /> {t('sub2api.announcements.publish', '发布')}</button>
        <button className="btn btn-ghost btn-sm" onClick={fetchItems}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.announcements.addTitle', '发布公告')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.announcements.titleField', '标题')}</label>
                <input className="s2a-form-input" value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.announcements.content', '内容')}</label>
                <textarea className="s2a-form-input" rows={4} value={form.content} onChange={(e) => setForm({ ...form, content: e.target.value })} style={{ resize: 'vertical' }} />
              </div>
            </div>
            <div className="s2a-modal-actions">
              <button className="btn btn-ghost btn-sm" onClick={() => setShowAdd(false)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary btn-sm" onClick={handleAdd}>{t('sub2api.announcements.publish', '发布')}</button>
            </div>
          </div>
        </div>
      )}

      {loading ? (
        <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>
      ) : items.length === 0 ? (
        <div className="s2a-empty"><div className="s2a-empty-text">{t('sub2api.announcements.empty', '暂无公告')}</div></div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {items.map((item) => (
            <div key={item.id} className="s2a-section">
              <div className="s2a-section-header">
                <div className="s2a-section-title">{item.title || t('sub2api.announcements.untitled', '无标题')}</div>
                <div className="s2a-actions">
                  <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(item.id)}><Trash2 size={14} /></button>
                </div>
              </div>
              <p style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', margin: 0, whiteSpace: 'pre-wrap' }}>{item.content || '-'}</p>
              <div style={{ fontSize: '0.62rem', color: 'var(--text-muted)', marginTop: 8 }}>{item.created_at || '-'}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

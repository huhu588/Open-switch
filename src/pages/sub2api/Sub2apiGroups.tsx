import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Edit, Loader2, X } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface Group {
  id: number;
  name: string;
  ratio?: number;
  models?: string;
  priority?: number;
  accounts_count?: number;
  [key: string]: unknown;
}

export default function Sub2apiGroups() {
  const { t } = useTranslation();
  const toast = useToast();
  const [groups, setGroups] = useState<Group[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [editingGroup, setEditingGroup] = useState<Group | null>(null);
  const [form, setForm] = useState({ name: '', ratio: '1', priority: '0' });

  const fetchGroups = useCallback(async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<Group[] | { data: Group[] }>('/admin/groups');
      const items = Array.isArray(data) ? data : ((data as { data: Group[] })?.data || []);
      setGroups(items);
    } catch (err) {
      console.error('[Sub2api Groups]', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { fetchGroups(); }, [fetchGroups]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/groups', {
        name: form.name,
        ratio: parseFloat(form.ratio) || 1,
        priority: parseInt(form.priority) || 0,
      });
      toast.success(t('sub2api.groups.addSuccess', '分组已创建'));
      setShowAdd(false);
      setForm({ name: '', ratio: '1', priority: '0' });
      fetchGroups();
    } catch (err) { toast.error(String(err)); }
  };

  const handleUpdate = async () => {
    if (!editingGroup) return;
    try {
      await sub2apiClient.put(`/admin/groups/${editingGroup.id}`, {
        name: form.name,
        ratio: parseFloat(form.ratio) || 1,
        priority: parseInt(form.priority) || 0,
      });
      toast.success(t('sub2api.groups.updateSuccess', '已更新'));
      setEditingGroup(null);
      fetchGroups();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/groups/${id}`);
      toast.success(t('sub2api.groups.deleteSuccess', '已删除'));
      fetchGroups();
    } catch (err) { toast.error(String(err)); }
  };

  const openEdit = (g: Group) => {
    setEditingGroup(g);
    setForm({ name: g.name, ratio: String(g.ratio ?? 1), priority: String(g.priority ?? 0) });
  };

  const isModalOpen = showAdd || editingGroup !== null;

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.groups.title', '分组管理')}</h2>

      <div className="s2a-toolbar">
        <button className="btn btn-primary btn-sm" onClick={() => { setShowAdd(true); setForm({ name: '', ratio: '1', priority: '0' }); }}>
          <Plus size={14} /> {t('common.add', '添加')}
        </button>
        <button className="btn btn-ghost btn-sm" onClick={fetchGroups}><RefreshCw size={14} /></button>
      </div>

      {isModalOpen && (
        <div className="s2a-modal-overlay" onClick={() => { setShowAdd(false); setEditingGroup(null); }}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{editingGroup ? t('sub2api.groups.edit', '编辑分组') : t('sub2api.groups.addTitle', '创建分组')}</div>
              <button className="s2a-action-btn" onClick={() => { setShowAdd(false); setEditingGroup(null); }}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.groups.name', '分组名称')}</label>
                <input className="s2a-form-input" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.groups.ratio', '倍率')}</label>
                <input className="s2a-form-input" type="number" step="0.1" value={form.ratio} onChange={(e) => setForm({ ...form, ratio: e.target.value })} />
              </div>
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.groups.priority', '优先级')}</label>
                <input className="s2a-form-input" type="number" value={form.priority} onChange={(e) => setForm({ ...form, priority: e.target.value })} />
              </div>
            </div>
            <div className="s2a-modal-actions">
              <button className="btn btn-ghost btn-sm" onClick={() => { setShowAdd(false); setEditingGroup(null); }}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary btn-sm" onClick={editingGroup ? handleUpdate : handleAdd}>{t('common.confirm', '确定')}</button>
            </div>
          </div>
        </div>
      )}

      {loading ? (
        <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>
      ) : groups.length === 0 ? (
        <div className="s2a-empty">
          <div className="s2a-empty-text">{t('sub2api.groups.empty', '暂无分组')}</div>
        </div>
      ) : (
        <div className="s2a-table-wrapper">
          <table className="s2a-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>{t('sub2api.groups.name', '名称')}</th>
                <th>{t('sub2api.groups.ratio', '倍率')}</th>
                <th>{t('sub2api.groups.priority', '优先级')}</th>
                <th>{t('sub2api.groups.accountsCount', '账号数')}</th>
                <th>{t('sub2api.accounts.actions', '操作')}</th>
              </tr>
            </thead>
            <tbody>
              {groups.map((g) => (
                <tr key={g.id}>
                  <td>{g.id}</td>
                  <td>{g.name}</td>
                  <td>{g.ratio ?? 1}</td>
                  <td>{g.priority ?? 0}</td>
                  <td>{g.accounts_count ?? '-'}</td>
                  <td>
                    <div className="s2a-actions">
                      <button className="s2a-action-btn" onClick={() => openEdit(g)}><Edit size={14} /></button>
                      <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(g.id)}><Trash2 size={14} /></button>
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

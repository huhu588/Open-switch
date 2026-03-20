import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, RefreshCw, Trash2, Loader2, X, ChevronLeft, ChevronRight, Search } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface User {
  id: number;
  username?: string;
  email?: string;
  display_name?: string;
  role?: number;
  status?: number;
  quota?: number;
  used_quota?: number;
  balance?: number;
  created_at?: string;
  [key: string]: unknown;
}

export default function Sub2apiUsers() {
  const { t } = useTranslation();
  const toast = useToast();
  const [users, setUsers] = useState<User[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ username: '', email: '', password: '' });

  const fetchUsers = useCallback(async () => {
    setLoading(true);
    try {
      const params: Record<string, unknown> = { page, page_size: pageSize };
      if (search) params.keyword = search;
      const resp = await sub2apiClient.get<User[] | { data?: User[]; items?: User[]; total?: number }>('/admin/users', params);
      if (Array.isArray(resp)) {
        setUsers(resp);
        setTotal(resp.length);
      } else if (resp && typeof resp === 'object') {
        const items = resp.data || resp.items || [];
        setUsers(Array.isArray(items) ? items : []);
        setTotal(Number(resp.total ?? items.length));
      }
    } catch (err) { console.error('[Sub2api Users]', err); }
    finally { setLoading(false); }
  }, [page, pageSize, search]);

  useEffect(() => { fetchUsers(); }, [fetchUsers]);

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/users', form as unknown as Record<string, unknown>);
      toast.success(t('sub2api.users.addSuccess', '用户已创建'));
      setShowAdd(false);
      setForm({ username: '', email: '', password: '' });
      fetchUsers();
    } catch (err) { toast.error(String(err)); }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/users/${id}`);
      toast.success(t('sub2api.users.deleteSuccess', '已删除'));
      fetchUsers();
    } catch (err) { toast.error(String(err)); }
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  const roleLabel = (r?: number) => {
    if (r === 100) return <span className="s2a-badge s2a-badge--warning">Admin</span>;
    if (r === 10) return <span className="s2a-badge s2a-badge--info">Moderator</span>;
    return <span className="s2a-badge s2a-badge--gray">User</span>;
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.users.title', '用户管理')}</h2>

      <div className="s2a-toolbar">
        <div style={{ position: 'relative', flex: 1, minWidth: 180 }}>
          <Search size={14} style={{ position: 'absolute', left: 8, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
          <input className="s2a-search-input" style={{ paddingLeft: 28 }} placeholder={t('sub2api.users.search', '搜索用户...')} value={search} onChange={(e) => { setSearch(e.target.value); setPage(1); }} />
        </div>
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}>
          <Plus size={14} /> {t('common.add', '添加')}
        </button>
        <button className="btn btn-ghost btn-sm" onClick={fetchUsers}><RefreshCw size={14} /></button>
      </div>

      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div className="s2a-modal-title">{t('sub2api.users.addTitle', '创建用户')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.users.username', '用户名')}</label>
                <input className="s2a-form-input" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">Email</label>
                <input className="s2a-form-input" type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.users.password', '密码')}</label>
                <input className="s2a-form-input" type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />
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
      ) : users.length === 0 ? (
        <div className="s2a-empty">
          <div className="s2a-empty-text">{t('sub2api.users.empty', '暂无用户')}</div>
        </div>
      ) : (
        <>
          <div className="s2a-table-wrapper">
            <table className="s2a-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>{t('sub2api.users.username', '用户名')}</th>
                  <th>Email</th>
                  <th>{t('sub2api.users.role', '角色')}</th>
                  <th>{t('sub2api.users.balance', '余额')}</th>
                  <th>{t('sub2api.accounts.actions', '操作')}</th>
                </tr>
              </thead>
              <tbody>
                {users.map((u) => (
                  <tr key={u.id}>
                    <td>{u.id}</td>
                    <td>{u.username || u.display_name || '-'}</td>
                    <td><span className="s2a-truncate">{u.email || '-'}</span></td>
                    <td>{roleLabel(u.role)}</td>
                    <td>{u.balance != null ? u.balance.toFixed(2) : u.quota ?? '-'}</td>
                    <td>
                      <div className="s2a-actions">
                        <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(u.id)}><Trash2 size={14} /></button>
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

import { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
  Plus, Search, RefreshCw, Trash2, TestTube, Loader2, X,
  ChevronLeft, ChevronRight, Download,
} from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface Account {
  id: number;
  email?: string;
  access_token?: string;
  platform?: string;
  group?: string;
  status?: number;
  weight?: number;
  models?: string;
  created_at?: string;
  test_time?: string;
  [key: string]: unknown;
}

interface ListResponse {
  data?: Account[];
  items?: Account[];
  total?: number;
  page?: number;
  [key: string]: unknown;
}

export default function Sub2apiAccounts() {
  const { t } = useTranslation();
  const toast = useToast();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [showAdd, setShowAdd] = useState(false);
  const [newAccount, setNewAccount] = useState({ email: '', access_token: '', platform: 'openai' });

  const fetchAccounts = useCallback(async () => {
    setLoading(true);
    try {
      const params: Record<string, unknown> = { page, page_size: pageSize };
      if (search) params.keyword = search;
      const resp = await sub2apiClient.get<ListResponse | Account[]>('/admin/accounts', params);
      if (Array.isArray(resp)) {
        setAccounts(resp);
        setTotal(resp.length);
      } else if (resp && typeof resp === 'object') {
        const items = resp.data || resp.items || [];
        setAccounts(Array.isArray(items) ? items : []);
        setTotal(Number(resp.total ?? items.length));
      }
    } catch (err) {
      console.error('[Sub2api Accounts]', err);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, search]);

  useEffect(() => { fetchAccounts(); }, [fetchAccounts]);

  const handleSync = async () => {
    setSyncing(true);
    try {
      await invoke('sync_accounts_to_sub2api');
      toast.success(t('sub2api.accounts.syncSuccess', '账号已同步'));
      fetchAccounts();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setSyncing(false);
    }
  };

  const handleAdd = async () => {
    try {
      await sub2apiClient.post('/admin/accounts', newAccount as unknown as Record<string, unknown>);
      toast.success(t('sub2api.accounts.addSuccess', '账号已添加'));
      setShowAdd(false);
      setNewAccount({ email: '', access_token: '', platform: 'openai' });
      fetchAccounts();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await sub2apiClient.delete(`/admin/accounts/${id}`);
      toast.success(t('sub2api.accounts.deleteSuccess', '已删除'));
      fetchAccounts();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleTest = async (id: number) => {
    try {
      await sub2apiClient.post(`/admin/accounts/${id}/test`);
      toast.success(t('sub2api.accounts.testSuccess', '测试成功'));
      fetchAccounts();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  const statusLabel = (s?: number) => {
    if (s === 1) return <span className="s2a-badge s2a-badge--success">{t('common.enabled', '启用')}</span>;
    if (s === 0 || s === 2) return <span className="s2a-badge s2a-badge--danger">{t('common.disabled', '禁用')}</span>;
    return <span className="s2a-badge s2a-badge--gray">-</span>;
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.accounts.title', '账号管理')}</h2>

      <div className="s2a-toolbar">
        <div style={{ position: 'relative', flex: 1, minWidth: 180 }}>
          <Search size={14} style={{ position: 'absolute', left: 8, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
          <input
            className="s2a-search-input"
            style={{ paddingLeft: 28 }}
            placeholder={t('sub2api.accounts.search', '搜索账号...')}
            value={search}
            onChange={(e) => { setSearch(e.target.value); setPage(1); }}
          />
        </div>
        <button className="btn btn-primary btn-sm" onClick={() => setShowAdd(true)}>
          <Plus size={14} /> {t('common.add', '添加')}
        </button>
        <button className="btn btn-ghost btn-sm" onClick={handleSync} disabled={syncing}>
          {syncing ? <Loader2 size={14} className="gw-spin" /> : <Download size={14} />}
          {t('sub2api.accounts.sync', '同步')}
        </button>
        <button className="btn btn-ghost btn-sm" onClick={fetchAccounts}>
          <RefreshCw size={14} />
        </button>
      </div>

      {/* Add Account Modal */}
      {showAdd && (
        <div className="s2a-modal-overlay" onClick={() => setShowAdd(false)}>
          <div className="s2a-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div className="s2a-modal-title">{t('sub2api.accounts.addTitle', '添加账号')}</div>
              <button className="s2a-action-btn" onClick={() => setShowAdd(false)}><X size={16} /></button>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">Email</label>
                <input className="s2a-form-input" value={newAccount.email} onChange={(e) => setNewAccount({ ...newAccount, email: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">Access Token</label>
                <input className="s2a-form-input" value={newAccount.access_token} onChange={(e) => setNewAccount({ ...newAccount, access_token: e.target.value })} />
              </div>
            </div>
            <div className="s2a-form-row">
              <div className="s2a-form-field">
                <label className="s2a-form-label">{t('sub2api.accounts.platform', '平台')}</label>
                <select className="s2a-form-select" value={newAccount.platform} onChange={(e) => setNewAccount({ ...newAccount, platform: e.target.value })}>
                  <option value="openai">OpenAI</option>
                  <option value="claude">Claude</option>
                  <option value="gemini">Gemini</option>
                  <option value="cursor">Cursor</option>
                  <option value="copilot">GitHub Copilot</option>
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

      {/* Table */}
      {loading ? (
        <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>
      ) : accounts.length === 0 ? (
        <div className="s2a-empty">
          <div className="s2a-empty-text">{t('sub2api.accounts.empty', '暂无账号')}</div>
          <div className="s2a-empty-desc">{t('sub2api.accounts.emptyHint', '点击「添加」或「同步」导入账号')}</div>
        </div>
      ) : (
        <>
          <div className="s2a-table-wrapper">
            <table className="s2a-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Email</th>
                  <th>{t('sub2api.accounts.platform', '平台')}</th>
                  <th>{t('sub2api.accounts.group', '分组')}</th>
                  <th>{t('sub2api.accounts.status', '状态')}</th>
                  <th>{t('sub2api.accounts.actions', '操作')}</th>
                </tr>
              </thead>
              <tbody>
                {accounts.map((acc) => (
                  <tr key={acc.id}>
                    <td>{acc.id}</td>
                    <td><span className="s2a-truncate">{acc.email || '-'}</span></td>
                    <td><span className="s2a-badge s2a-badge--info">{acc.platform || '-'}</span></td>
                    <td>{acc.group || '-'}</td>
                    <td>{statusLabel(acc.status)}</td>
                    <td>
                      <div className="s2a-actions">
                        <button className="s2a-action-btn" onClick={() => handleTest(acc.id)} title={t('sub2api.accounts.test', '测试')}><TestTube size={14} /></button>
                        <button className="s2a-action-btn s2a-action-btn--danger" onClick={() => handleDelete(acc.id)} title={t('common.delete', '删除')}><Trash2 size={14} /></button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="s2a-pagination">
            <div className="s2a-pagination-info">
              {t('sub2api.pagination.total', '共')} {total} {t('sub2api.pagination.items', '条')}
            </div>
            <div className="s2a-pagination-buttons">
              <button className="s2a-pagination-btn" disabled={page <= 1} onClick={() => setPage(p => p - 1)}>
                <ChevronLeft size={14} />
              </button>
              <span style={{ padding: '0 8px', fontSize: '0.68rem' }}>{page} / {totalPages}</span>
              <button className="s2a-pagination-btn" disabled={page >= totalPages} onClick={() => setPage(p => p + 1)}>
                <ChevronRight size={14} />
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

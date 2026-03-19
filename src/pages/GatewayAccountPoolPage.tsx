import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, RefreshCw, Upload, Download, UserCheck, UserX } from 'lucide-react';

interface GatewayAccount {
  id: string;
  email: string;
  access_token: string;
  refresh_token: string | null;
  status: string;
  tags: string | null;
  group_name: string | null;
  proxy_url: string | null;
  created_at: number;
  updated_at: number;
  last_used_at: number | null;
  error_count: number;
}

export function GatewayAccountPoolPage() {
  const { t } = useTranslation();
  const [accounts, setAccounts] = useState<GatewayAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newEmail, setNewEmail] = useState('');
  const [newToken, setNewToken] = useState('');
  const [importText, setImportText] = useState('');
  const [showImport, setShowImport] = useState(false);

  const fetchAccounts = useCallback(async () => {
    try {
      const data = await invoke<GatewayAccount[]>('list_gateway_accounts');
      setAccounts(data);
    } catch (error) {
      console.error('Failed to list gateway accounts:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAccounts();
  }, [fetchAccounts]);

  const handleAddAccount = async () => {
    if (!newEmail.trim() || !newToken.trim()) return;
    try {
      await invoke('add_gateway_account', {
        email: newEmail.trim(),
        accessToken: newToken.trim(),
        refreshToken: null,
        tags: null,
        groupName: null,
        proxyUrl: null,
      });
      setNewEmail('');
      setNewToken('');
      setShowAddForm(false);
      await fetchAccounts();
    } catch (error) {
      console.error('Failed to add account:', error);
    }
  };

  const handleDeleteAccount = async (id: string) => {
    try {
      await invoke('delete_gateway_account', { id });
      await fetchAccounts();
    } catch (error) {
      console.error('Failed to delete account:', error);
    }
  };

  const handleImport = async () => {
    if (!importText.trim()) return;
    try {
      const lines = importText.trim().split('\n').filter((l) => l.trim());
      const accounts = lines.map((line) => {
        const parts = line.split(/[,\t|]/).map((p) => p.trim());
        return {
          email: parts[0] || 'unknown',
          access_token: parts[1] || parts[0],
          refresh_token: parts[2] || null,
          tags: null,
          group_name: null,
          proxy_url: null,
        };
      });
      const count = await invoke<number>('import_gateway_accounts', { accounts });
      setImportText('');
      setShowImport(false);
      await fetchAccounts();
      alert(t('gateway.importSuccess', '成功导入 {{count}} 个账号', { count }));
    } catch (error) {
      console.error('Failed to import accounts:', error);
    }
  };

  const handleExport = async () => {
    try {
      const data = await invoke<GatewayAccount[]>('export_gateway_accounts');
      const text = data.map((a) => `${a.email},${a.access_token}`).join('\n');
      navigator.clipboard.writeText(text);
      alert(t('gateway.exportCopied', '已复制到剪贴板'));
    } catch (error) {
      console.error('Failed to export accounts:', error);
    }
  };

  const formatTime = (ts: number | null) => {
    if (!ts) return '-';
    return new Date(ts * 1000).toLocaleString();
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active': return <span className="badge badge-success badge-sm">{t('gateway.active', '正常')}</span>;
      case 'cooldown': return <span className="badge badge-warning badge-sm">{t('gateway.cooldown', '冷却')}</span>;
      case 'error': return <span className="badge badge-error badge-sm">{t('gateway.error', '错误')}</span>;
      case 'expired': return <span className="badge badge-ghost badge-sm">{t('gateway.expired', '过期')}</span>;
      default: return <span className="badge badge-ghost badge-sm">{status}</span>;
    }
  };

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1 className="page-title">{t('gateway.accountPool', '账号池')}</h1>
        <div className="page-actions">
          <button className="btn btn-ghost btn-sm" onClick={fetchAccounts}>
            <RefreshCw size={14} />
          </button>
          <button className="btn btn-ghost btn-sm" onClick={() => setShowImport(!showImport)}>
            <Upload size={14} />
            <span>{t('gateway.import', '导入')}</span>
          </button>
          <button className="btn btn-ghost btn-sm" onClick={handleExport}>
            <Download size={14} />
            <span>{t('gateway.export', '导出')}</span>
          </button>
          <button className="btn btn-primary btn-sm" onClick={() => setShowAddForm(!showAddForm)}>
            <Plus size={14} />
            <span>{t('gateway.addAccount', '添加账号')}</span>
          </button>
        </div>
      </div>

      {showImport && (
        <div className="gateway-import-form">
          <textarea
            className="textarea textarea-bordered w-full"
            rows={5}
            placeholder={t('gateway.importPlaceholder', '每行一个账号，格式: email,access_token[,refresh_token]')}
            value={importText}
            onChange={(e) => setImportText(e.target.value)}
          />
          <div className="form-actions">
            <button className="btn btn-sm btn-ghost" onClick={() => setShowImport(false)}>{t('common.cancel', '取消')}</button>
            <button className="btn btn-sm btn-primary" onClick={handleImport}>{t('gateway.importBtn', '导入')}</button>
          </div>
        </div>
      )}

      {showAddForm && (
        <div className="gateway-add-form">
          <input
            type="text"
            className="input input-bordered input-sm"
            placeholder="Email"
            value={newEmail}
            onChange={(e) => setNewEmail(e.target.value)}
          />
          <input
            type="text"
            className="input input-bordered input-sm"
            placeholder="Access Token"
            value={newToken}
            onChange={(e) => setNewToken(e.target.value)}
          />
          <div className="form-actions">
            <button className="btn btn-sm btn-ghost" onClick={() => setShowAddForm(false)}>{t('common.cancel', '取消')}</button>
            <button className="btn btn-sm btn-primary" onClick={handleAddAccount}>{t('common.save', '保存')}</button>
          </div>
        </div>
      )}

      <div className="gateway-account-list">
        {accounts.length === 0 ? (
          <div className="empty-state">
            <UserX size={48} className="empty-icon" />
            <p>{t('gateway.noAccounts', '暂无账号，请添加或导入账号')}</p>
          </div>
        ) : (
          <table className="table table-compact w-full">
            <thead>
              <tr>
                <th>{t('gateway.email', '邮箱')}</th>
                <th>{t('gateway.statusLabel', '状态')}</th>
                <th>{t('gateway.errors', '错误')}</th>
                <th>{t('gateway.lastUsed', '最后使用')}</th>
                <th>{t('gateway.actions', '操作')}</th>
              </tr>
            </thead>
            <tbody>
              {accounts.map((account) => (
                <tr key={account.id}>
                  <td>
                    <div className="flex items-center gap-2">
                      <UserCheck size={14} />
                      <span>{account.email}</span>
                      {account.tags && <span className="badge badge-outline badge-xs">{account.tags}</span>}
                    </div>
                  </td>
                  <td>{getStatusBadge(account.status)}</td>
                  <td>{account.error_count > 0 ? <span className="text-error">{account.error_count}</span> : '0'}</td>
                  <td className="text-xs opacity-70">{formatTime(account.last_used_at)}</td>
                  <td>
                    <button
                      className="btn btn-ghost btn-xs text-error"
                      onClick={() => handleDeleteAccount(account.id)}
                    >
                      <Trash2 size={12} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

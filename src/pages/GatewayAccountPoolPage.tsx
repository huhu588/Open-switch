import { useEffect, useState, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  Plus, Trash2, RefreshCw, Upload, Download, UserX,
  Search, AlertTriangle, ChevronDown, ChevronRight, CheckSquare, Square, Layers,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

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
  platform: string | null;
  source: string | null;
}

const AVATAR_COLORS = [
  'linear-gradient(135deg, #1d4ed8, #0ea5a5)',
  'linear-gradient(135deg, #6366f1, #8b5cf6)',
  'linear-gradient(135deg, #ec4899, #f43f5e)',
  'linear-gradient(135deg, #f59e0b, #d97706)',
  'linear-gradient(135deg, #22c55e, #16a34a)',
  'linear-gradient(135deg, #06b6d4, #0891b2)',
];

function getAvatarColor(email: string) {
  let hash = 0;
  for (let i = 0; i < email.length; i++) hash = ((hash << 5) - hash + email.charCodeAt(i)) | 0;
  return AVATAR_COLORS[Math.abs(hash) % AVATAR_COLORS.length];
}

function getInitials(email: string) {
  return email.charAt(0).toUpperCase();
}

export function GatewayAccountPoolPage({ embedded }: { embedded?: boolean } = {}) {
  const { t } = useTranslation();
  const toast = useToast();
  const [accounts, setAccounts] = useState<GatewayAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newEmail, setNewEmail] = useState('');
  const [newToken, setNewToken] = useState('');
  const [newRefreshToken, setNewRefreshToken] = useState('');
  const [newTags, setNewTags] = useState('');
  const [newProxy, setNewProxy] = useState('');
  const [importText, setImportText] = useState('');
  const [showImport, setShowImport] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [addTab, setAddTab] = useState<'manual' | 'sync'>('manual');
  const [syncing, setSyncing] = useState(false);

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

  const filteredAccounts = useMemo(() => {
    if (!searchQuery.trim()) return accounts;
    const q = searchQuery.toLowerCase();
    return accounts.filter(a =>
      a.email.toLowerCase().includes(q) ||
      (a.tags && a.tags.toLowerCase().includes(q)) ||
      (a.group_name && a.group_name.toLowerCase().includes(q)) ||
      a.status.toLowerCase().includes(q)
    );
  }, [accounts, searchQuery]);

  const statusCounts = useMemo(() => {
    const counts = { active: 0, cooldown: 0, error: 0, expired: 0 };
    accounts.forEach(a => {
      if (a.status in counts) counts[a.status as keyof typeof counts]++;
    });
    return counts;
  }, [accounts]);

  const handleAddAccount = async () => {
    if (!newEmail.trim() || !newToken.trim()) return;
    try {
      await invoke('add_gateway_account', {
        email: newEmail.trim(),
        accessToken: newToken.trim(),
        refreshToken: newRefreshToken.trim() || null,
        tags: newTags.trim() || null,
        groupName: null,
        proxyUrl: newProxy.trim() || null,
      });
      setNewEmail('');
      setNewToken('');
      setNewRefreshToken('');
      setNewTags('');
      setNewProxy('');
      setShowAddForm(false);
      await fetchAccounts();
      toast.success(t('gateway.accountAdded', '账号已添加'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleDeleteAccount = async (id: string) => {
    try {
      await invoke('delete_gateway_account', { id });
      await fetchAccounts();
      setSelectedIds(prev => { const s = new Set(prev); s.delete(id); return s; });
      toast.success(t('gateway.accountDeleted', '已删除'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleBulkDelete = async () => {
    if (selectedIds.size === 0) return;
    for (const id of selectedIds) {
      try {
        await invoke('delete_gateway_account', { id });
      } catch { /* continue */ }
    }
    setSelectedIds(new Set());
    await fetchAccounts();
    toast.success(t('gateway.bulkDeleted', '批量删除完成'));
  };

  const handleImport = async () => {
    if (!importText.trim()) return;
    try {
      const lines = importText.trim().split('\n').filter(l => l.trim());
      const importAccounts = lines.map(line => {
        const parts = line.split(/[,\t|]/).map(p => p.trim());
        return {
          email: parts[0] || 'unknown',
          access_token: parts[1] || parts[0],
          refresh_token: parts[2] || null,
          tags: null,
          group_name: null,
          proxy_url: null,
        };
      });
      const count = await invoke<number>('import_gateway_accounts', { accounts: importAccounts });
      setImportText('');
      setShowImport(false);
      await fetchAccounts();
      toast.success(t('gateway.importSuccess', '成功导入 {{count}} 个账号', { count }));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleExport = async () => {
    try {
      const data = await invoke<GatewayAccount[]>('export_gateway_accounts');
      const text = data.map(a => `${a.email},${a.access_token}${a.refresh_token ? ',' + a.refresh_token : ''}`).join('\n');
      navigator.clipboard.writeText(text);
      toast.success(t('gateway.exportCopied', '已复制到剪贴板'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleSyncPlatformAccounts = async () => {
    setSyncing(true);
    try {
      await invoke('sync_accounts_to_gateway');
      await fetchAccounts();
      toast.success(t('gateway.syncSuccess', '平台账号已同步'));
    } catch (error) {
      toast.error(String(error));
    } finally {
      setSyncing(false);
    }
  };

  const toggleSelectAll = () => {
    if (selectedIds.size === filteredAccounts.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(filteredAccounts.map(a => a.id)));
    }
  };

  const toggleSelect = (id: string) => {
    setSelectedIds(prev => {
      const s = new Set(prev);
      s.has(id) ? s.delete(id) : s.add(id);
      return s;
    });
  };

  const formatTime = (ts: number | null) => {
    if (!ts) return '-';
    return new Date(ts * 1000).toLocaleString();
  };

  const getHealthDot = (account: GatewayAccount) => {
    if (account.status === 'active' && account.error_count === 0) return 'green';
    if (account.status === 'cooldown' || (account.error_count > 0 && account.error_count < 3)) return 'yellow';
    if (account.status === 'error' || account.error_count >= 3) return 'red';
    return 'gray';
  };

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container">
      <ToastContainer toasts={toast.toasts} />

      {!embedded && (
        <div className="gw-page-header">
          <div className="gw-page-header-left">
            <h1 className="gw-page-title">{t('gateway.accountPool', '账号池')}</h1>
            <div className="gw-page-subtitle">
              {t('gateway.accountSummary', '{{active}} 活跃 / {{total}} 总计', {
                active: statusCounts.active,
                total: accounts.length,
              })}
              {statusCounts.error > 0 && (
                <span style={{ color: 'var(--danger)', marginLeft: 8 }}>
                  <AlertTriangle size={12} style={{ display: 'inline', verticalAlign: -2 }} /> {statusCounts.error} {t('gateway.errors', '异常')}
                </span>
              )}
            </div>
          </div>
          <div className="gw-page-actions">
            <div className="gw-search">
              <Search size={14} className="gw-search-icon" />
              <input
                className="gw-search-input"
                placeholder={t('gateway.searchAccounts', '搜索账号...')}
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
              />
            </div>
            <button className="btn btn-ghost btn-sm" onClick={fetchAccounts}><RefreshCw size={14} /></button>
            <button className="btn btn-ghost btn-sm" onClick={() => setShowImport(!showImport)}><Upload size={14} /></button>
            <button className="btn btn-ghost btn-sm" onClick={handleExport}><Download size={14} /></button>
            <button className="btn btn-primary btn-sm" onClick={() => setShowAddForm(!showAddForm)}>
              <Plus size={14} /> <span>{t('gateway.addAccount', '添加')}</span>
            </button>
          </div>
        </div>
      )}

      {embedded && (
        <div className="gw-embedded-actions">
          <div className="gw-page-subtitle" style={{ flex: 1 }}>
            {t('gateway.accountSummary', '{{active}} 活跃 / {{total}} 总计', {
              active: statusCounts.active,
              total: accounts.length,
            })}
            {statusCounts.error > 0 && (
              <span style={{ color: 'var(--danger)', marginLeft: 8 }}>
                <AlertTriangle size={12} style={{ display: 'inline', verticalAlign: -2 }} /> {statusCounts.error} {t('gateway.errors', '异常')}
              </span>
            )}
          </div>
          <div className="gw-page-actions">
            <div className="gw-search">
              <Search size={14} className="gw-search-icon" />
              <input
                className="gw-search-input"
                placeholder={t('gateway.searchAccounts', '搜索账号...')}
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
              />
            </div>
            <button className="btn btn-ghost btn-sm" onClick={fetchAccounts}><RefreshCw size={14} /></button>
            <button className="btn btn-ghost btn-sm" onClick={() => setShowImport(!showImport)}><Upload size={14} /></button>
            <button className="btn btn-ghost btn-sm" onClick={handleExport}><Download size={14} /></button>
            <button className="btn btn-primary btn-sm" onClick={() => setShowAddForm(!showAddForm)}>
              <Plus size={14} /> <span>{t('gateway.addAccount', '添加')}</span>
            </button>
          </div>
        </div>
      )}

      {/* Add Form */}
      {showAddForm && (
        <div className="gw-form-panel">
          <div className="gw-tab-bar">
            <button className={`gw-tab ${addTab === 'manual' ? 'is-active' : ''}`} onClick={() => setAddTab('manual')}>
              {t('gateway.manualAdd', '手动添加')}
            </button>
            <button className={`gw-tab ${addTab === 'sync' ? 'is-active' : ''}`} onClick={() => setAddTab('sync')}>
              <Layers size={12} /> {t('gateway.syncFromPlatform', '从平台同步')}
            </button>
          </div>

          {addTab === 'manual' ? (
            <>
              <div className="gw-form-row">
                <input type="text" className="input input-bordered input-sm" placeholder="Email" value={newEmail} onChange={e => setNewEmail(e.target.value)} />
                <input type="text" className="input input-bordered input-sm" placeholder="Access Token" value={newToken} onChange={e => setNewToken(e.target.value)} />
              </div>
              <div className="gw-form-row">
                <input type="text" className="input input-bordered input-sm" placeholder={t('gateway.refreshTokenOptional', 'Refresh Token (可选)')} value={newRefreshToken} onChange={e => setNewRefreshToken(e.target.value)} />
                <input type="text" className="input input-bordered input-sm" placeholder={t('gateway.tagsOptional', '标签 (可选)')} value={newTags} onChange={e => setNewTags(e.target.value)} />
              </div>
              <input type="text" className="input input-bordered input-sm" placeholder={t('gateway.proxyOptional', '代理地址 (可选)')} value={newProxy} onChange={e => setNewProxy(e.target.value)} />
              <div className="gw-form-actions">
                <button className="btn btn-sm btn-ghost" onClick={() => setShowAddForm(false)}>{t('common.cancel', '取消')}</button>
                <button className="btn btn-sm btn-primary" onClick={handleAddAccount}>{t('common.save', '保存')}</button>
              </div>
            </>
          ) : (
            <div style={{ textAlign: 'center', padding: '20px 0' }}>
              <p style={{ fontSize: '0.85rem', marginBottom: 12, color: 'var(--text-secondary)' }}>
                {t('gateway.syncDesc', '将各平台已添加的账号自动同步到网关账号池')}
              </p>
              <button className="gw-sync-btn" onClick={handleSyncPlatformAccounts} disabled={syncing}>
                {syncing ? <RefreshCw size={14} className="gw-spin" /> : <Layers size={14} />}
                {syncing ? t('gateway.syncing', '同步中...') : t('gateway.startSync', '开始同步')}
              </button>
            </div>
          )}
        </div>
      )}

      {/* Import Form */}
      {showImport && (
        <div className="gw-form-panel">
          <textarea
            className="textarea textarea-bordered w-full"
            rows={5}
            placeholder={t('gateway.importPlaceholder', '每行一个账号，格式: email,access_token[,refresh_token]')}
            value={importText}
            onChange={e => setImportText(e.target.value)}
          />
          <div className="gw-form-actions">
            <button className="btn btn-sm btn-ghost" onClick={() => setShowImport(false)}>{t('common.cancel', '取消')}</button>
            <button className="btn btn-sm btn-primary" onClick={handleImport}>{t('gateway.importBtn', '导入')}</button>
          </div>
        </div>
      )}

      {/* Bulk Actions */}
      {selectedIds.size > 0 && (
        <div className="gw-bulk-bar">
          <span className="gw-bulk-count">
            {t('gateway.selectedCount', '已选择 {{count}} 项', { count: selectedIds.size })}
          </span>
          <div className="gw-bulk-actions">
            <button className="btn btn-error btn-xs" onClick={handleBulkDelete}>
              <Trash2 size={12} /> {t('gateway.bulkDelete', '批量删除')}
            </button>
          </div>
        </div>
      )}

      {/* Account List */}
      {filteredAccounts.length === 0 ? (
        <div className="gw-empty">
          <UserX size={56} className="gw-empty-icon" />
          <div className="gw-empty-title">{t('gateway.noAccounts', '暂无账号')}</div>
          <div className="gw-empty-desc">{t('gateway.addHint', '请添加、导入账号或从平台同步')}</div>
        </div>
      ) : (
        <>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, fontSize: '0.75rem', color: 'var(--text-muted)' }}>
            <button className="btn btn-ghost btn-xs" onClick={toggleSelectAll} style={{ padding: '2px 4px' }}>
              {selectedIds.size === filteredAccounts.length ? <CheckSquare size={14} /> : <Square size={14} />}
            </button>
            <span>{t('gateway.totalFiltered', '共 {{count}} 个账号', { count: filteredAccounts.length })}</span>
          </div>
          <div className="gw-list">
            {filteredAccounts.map((account, i) => {
              const expanded = expandedId === account.id;
              return (
                <div key={account.id}>
                  <div
                    className={`gw-account-card gw-account-card--${account.status}`}
                    style={{ animationDelay: `${i * 30}ms` }}
                    onClick={() => setExpandedId(expanded ? null : account.id)}
                  >
                    <button className="btn btn-ghost btn-xs" style={{ padding: '2px 4px' }} onClick={e => { e.stopPropagation(); toggleSelect(account.id); }}>
                      {selectedIds.has(account.id) ? <CheckSquare size={14} /> : <Square size={14} />}
                    </button>
                    <div className="gw-account-avatar" style={{ background: getAvatarColor(account.email) }}>
                      {getInitials(account.email)}
                    </div>
                    <div className="gw-account-info">
                      <div className="gw-account-email">{account.email}</div>
                      <div className="gw-account-meta">
                        <span className={`gw-badge gw-badge--${account.status}`}>
                          <span className={`gw-health-dot gw-health-dot--${getHealthDot(account)}`} style={{ width: 6, height: 6 }} />
                          {account.status === 'active' ? t('gateway.active', '正常')
                            : account.status === 'cooldown' ? t('gateway.cooldown', '冷却')
                            : account.status === 'error' ? t('gateway.error', '错误')
                            : t('gateway.expired', '过期')}
                        </span>
                        {account.error_count > 0 && (
                          <span className="gw-account-meta-item" style={{ color: 'var(--danger)' }}>
                            <AlertTriangle size={10} /> {account.error_count}
                          </span>
                        )}
                        {account.tags && (
                          <span className="gw-account-meta-item">{account.tags}</span>
                        )}
                        {account.platform && (
                          <span className="gw-account-meta-item">{account.platform}</span>
                        )}
                      </div>
                    </div>
                    <div className="gw-account-actions">
                      {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                      <button className="btn btn-ghost btn-xs" style={{ color: 'var(--danger)' }} onClick={e => { e.stopPropagation(); handleDeleteAccount(account.id); }}>
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>

                  {expanded && (
                    <div className="gw-log-detail">
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.lastUsed', '最后使用')}</span>
                        <span className="gw-log-detail-value">{formatTime(account.last_used_at)}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.createdAt', '创建时间')}</span>
                        <span className="gw-log-detail-value">{formatTime(account.created_at)}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.source', '来源')}</span>
                        <span className="gw-log-detail-value">{account.source || '-'}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.proxy', '代理')}</span>
                        <span className="gw-log-detail-value">{account.proxy_url || '-'}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.hasRefreshToken', 'Refresh Token')}</span>
                        <span className="gw-log-detail-value">{account.refresh_token ? '✓' : '✗'}</span>
                      </div>
                      <div className="gw-log-detail-item">
                        <span className="gw-log-detail-label">{t('gateway.errors', '错误次数')}</span>
                        <span className="gw-log-detail-value" style={{ color: account.error_count > 0 ? 'var(--danger)' : undefined }}>
                          {account.error_count}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}

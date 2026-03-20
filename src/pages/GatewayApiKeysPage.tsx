import { useEffect, useState, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  Plus, Trash2, RefreshCw, Copy, Eye, EyeOff, Key,
  Search, ToggleLeft, ToggleRight, CheckCircle, BarChart3,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface GatewayApiKey {
  id: string;
  name: string;
  key_hash: string;
  key_prefix: string;
  allowed_models: string | null;
  enabled: boolean;
  created_at: number;
  last_used_at: number | null;
  usage_count: number;
}

export function GatewayApiKeysPage({ embedded }: { embedded?: boolean } = {}) {
  const { t } = useTranslation();
  const toast = useToast();
  const [apiKeys, setApiKeys] = useState<GatewayApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [newKeyModels, setNewKeyModels] = useState('');
  const [createdKey, setCreatedKey] = useState<string | null>(null);
  const [showKey, setShowKey] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  const fetchKeys = useCallback(async () => {
    try {
      const data = await invoke<GatewayApiKey[]>('list_api_keys');
      setApiKeys(data);
    } catch (error) {
      console.error('Failed to list api keys:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchKeys();
  }, [fetchKeys]);

  const filteredKeys = useMemo(() => {
    if (!searchQuery.trim()) return apiKeys;
    const q = searchQuery.toLowerCase();
    return apiKeys.filter(k =>
      k.name.toLowerCase().includes(q) ||
      k.key_prefix.toLowerCase().includes(q)
    );
  }, [apiKeys, searchQuery]);

  const stats = useMemo(() => {
    const enabled = apiKeys.filter(k => k.enabled).length;
    const totalUsage = apiKeys.reduce((s, k) => s + k.usage_count, 0);
    return { total: apiKeys.length, enabled, totalUsage };
  }, [apiKeys]);

  const handleCreateKey = async () => {
    if (!newKeyName.trim()) return;
    try {
      const allowedModels = newKeyModels.trim()
        ? newKeyModels.split(',').map(m => m.trim()).filter(Boolean)
        : null;

      const [rawKey] = await invoke<[string, GatewayApiKey]>('create_api_key', {
        payload: { name: newKeyName.trim(), allowed_models: allowedModels },
      });
      setCreatedKey(rawKey);
      setNewKeyName('');
      setNewKeyModels('');
      setShowCreateForm(false);
      await fetchKeys();
      toast.success(t('gateway.keyCreatedMsg', 'API Key 已创建'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleDeleteKey = async (id: string) => {
    try {
      await invoke('delete_api_key', { id });
      await fetchKeys();
      toast.success(t('gateway.keyDeleted', 'Key 已删除'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleToggleKey = async (id: string, enabled: boolean) => {
    try {
      await invoke('toggle_api_key', { id, enabled: !enabled });
      await fetchKeys();
      toast.info(enabled ? t('gateway.keyDisabled', 'Key 已禁用') : t('gateway.keyEnabled', 'Key 已启用'));
    } catch (error) {
      toast.error(String(error));
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success(t('gateway.copied', '已复制'));
  };

  const parseModels = (modelsStr: string | null): string[] => {
    if (!modelsStr) return [];
    try { return JSON.parse(modelsStr); } catch { return []; }
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
            <h1 className="gw-page-title">{t('gateway.apiKeyManagement', 'API Key 管理')}</h1>
            <div className="gw-page-subtitle">
              {stats.enabled}/{stats.total} {t('gateway.enabled', '启用')} · {stats.totalUsage.toLocaleString()} {t('gateway.totalUsage', '总调用')}
            </div>
          </div>
          <div className="gw-page-actions">
            <div className="gw-search">
              <Search size={14} className="gw-search-icon" />
              <input
                className="gw-search-input"
                placeholder={t('gateway.searchKeys', '搜索 Key...')}
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
              />
            </div>
            <button className="btn btn-ghost btn-sm" onClick={fetchKeys}><RefreshCw size={14} /></button>
            <button className="btn btn-primary btn-sm" onClick={() => setShowCreateForm(!showCreateForm)}>
              <Plus size={14} /> <span>{t('gateway.createKey', '创建 Key')}</span>
            </button>
          </div>
        </div>
      )}

      {embedded && (
        <div className="gw-embedded-actions">
          <div className="gw-page-subtitle" style={{ flex: 1 }}>
            {stats.enabled}/{stats.total} {t('gateway.enabled', '启用')} · {stats.totalUsage.toLocaleString()} {t('gateway.totalUsage', '总调用')}
          </div>
          <div className="gw-page-actions">
            <div className="gw-search">
              <Search size={14} className="gw-search-icon" />
              <input
                className="gw-search-input"
                placeholder={t('gateway.searchKeys', '搜索 Key...')}
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
              />
            </div>
            <button className="btn btn-ghost btn-sm" onClick={fetchKeys}><RefreshCw size={14} /></button>
            <button className="btn btn-primary btn-sm" onClick={() => setShowCreateForm(!showCreateForm)}>
              <Plus size={14} /> <span>{t('gateway.createKey', '创建 Key')}</span>
            </button>
          </div>
        </div>
      )}

      {/* Created Key Alert */}
      {createdKey && (
        <div className="gw-key-alert">
          <div className="gw-key-alert-icon"><CheckCircle size={18} /></div>
          <div className="gw-key-alert-content">
            <div className="gw-key-alert-title">{t('gateway.keyCreated', 'API Key 已创建，请立即复制保存：')}</div>
            <div className="gw-key-display">
              <span style={{ flex: 1 }}>{showKey ? createdKey : createdKey.substring(0, 12) + '•'.repeat(20)}</span>
              <button className="btn btn-ghost btn-xs" onClick={() => setShowKey(!showKey)}>
                {showKey ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
              <button className="btn btn-ghost btn-xs" onClick={() => copyToClipboard(createdKey)}>
                <Copy size={12} />
              </button>
            </div>
            <button className="btn btn-ghost btn-xs" style={{ marginTop: 8 }} onClick={() => setCreatedKey(null)}>
              {t('common.dismiss', '关闭')}
            </button>
          </div>
        </div>
      )}

      {/* Create Form */}
      {showCreateForm && (
        <div className="gw-form-panel">
          <div className="gw-form-row">
            <input
              type="text"
              className="input input-bordered input-sm"
              placeholder={t('gateway.keyName', 'Key 名称')}
              value={newKeyName}
              onChange={e => setNewKeyName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleCreateKey()}
            />
            <input
              type="text"
              className="input input-bordered input-sm"
              placeholder={t('gateway.allowedModels', '允许的模型（逗号分隔，留空为全部）')}
              value={newKeyModels}
              onChange={e => setNewKeyModels(e.target.value)}
            />
          </div>
          <div className="gw-form-actions">
            <button className="btn btn-sm btn-ghost" onClick={() => setShowCreateForm(false)}>{t('common.cancel', '取消')}</button>
            <button className="btn btn-sm btn-primary" onClick={handleCreateKey}>{t('common.create', '创建')}</button>
          </div>
        </div>
      )}

      {/* Key List */}
      {filteredKeys.length === 0 ? (
        <div className="gw-empty">
          <Key size={56} className="gw-empty-icon" />
          <div className="gw-empty-title">{t('gateway.noApiKeys', '暂无 API Key')}</div>
          <div className="gw-empty-desc">{t('gateway.createKeyHint', '创建一个 API Key 来访问网关服务')}</div>
        </div>
      ) : (
        <div className="gw-list">
          {filteredKeys.map((key, i) => {
            const models = parseModels(key.allowed_models);
            return (
              <div
                key={key.id}
                className={`gw-key-card ${key.enabled ? '' : 'is-disabled'}`}
                style={{ animationDelay: `${i * 40}ms` }}
              >
                <div className="gw-key-icon"><Key size={18} /></div>
                <div className="gw-key-info">
                  <div className="gw-key-name">{key.name}</div>
                  <div className="gw-key-meta">
                    <span className="gw-key-prefix">{key.key_prefix}•••</span>
                    <span className={`gw-badge ${key.enabled ? 'gw-badge--enabled' : 'gw-badge--disabled'}`}>
                      {key.enabled ? t('gateway.enabled', '启用') : t('gateway.disabled', '禁用')}
                    </span>
                    <span className="gw-key-usage">
                      <BarChart3 size={10} />
                      {key.usage_count.toLocaleString()} {t('gateway.calls', '次调用')}
                    </span>
                  </div>
                  {models.length > 0 && (
                    <div className="gw-key-models" style={{ marginTop: 6 }}>
                      {models.map(m => (
                        <span key={m} className="gw-key-model-tag">{m}</span>
                      ))}
                    </div>
                  )}
                  {!models.length && (
                    <div style={{ fontSize: '0.65rem', color: 'var(--text-muted)', marginTop: 4 }}>
                      {t('gateway.allModels', '全部模型')}
                    </div>
                  )}
                </div>
                <div className="gw-key-actions">
                  <button className="btn btn-ghost btn-xs" onClick={() => handleToggleKey(key.id, key.enabled)} title={key.enabled ? t('gateway.disable', '禁用') : t('gateway.enable', '启用')}>
                    {key.enabled ? <ToggleRight size={16} style={{ color: 'var(--success)' }} /> : <ToggleLeft size={16} />}
                  </button>
                  <button className="btn btn-ghost btn-xs" style={{ color: 'var(--danger)' }} onClick={() => handleDeleteKey(key.id)}>
                    <Trash2 size={12} />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

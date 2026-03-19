import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, RefreshCw, Copy, Eye, EyeOff, ToggleLeft, ToggleRight, Key } from 'lucide-react';

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

export function GatewayApiKeysPage() {
  const { t } = useTranslation();
  const [apiKeys, setApiKeys] = useState<GatewayApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [newKeyModels, setNewKeyModels] = useState('');
  const [createdKey, setCreatedKey] = useState<string | null>(null);
  const [showKey, setShowKey] = useState(false);

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

  const handleCreateKey = async () => {
    if (!newKeyName.trim()) return;
    try {
      const allowedModels = newKeyModels.trim()
        ? newKeyModels.split(',').map((m) => m.trim()).filter(Boolean)
        : null;

      const [rawKey] = await invoke<[string, GatewayApiKey]>('create_api_key', {
        payload: {
          name: newKeyName.trim(),
          allowed_models: allowedModels,
        },
      });
      setCreatedKey(rawKey);
      setNewKeyName('');
      setNewKeyModels('');
      await fetchKeys();
    } catch (error) {
      console.error('Failed to create api key:', error);
    }
  };

  const handleDeleteKey = async (id: string) => {
    try {
      await invoke('delete_api_key', { id });
      await fetchKeys();
    } catch (error) {
      console.error('Failed to delete api key:', error);
    }
  };

  const handleToggleKey = async (id: string, enabled: boolean) => {
    try {
      await invoke('toggle_api_key', { id, enabled: !enabled });
      await fetchKeys();
    } catch (error) {
      console.error('Failed to toggle api key:', error);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const formatTime = (ts: number | null) => {
    if (!ts) return '-';
    return new Date(ts * 1000).toLocaleString();
  };

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1 className="page-title">{t('gateway.apiKeyManagement', 'API Key 管理')}</h1>
        <div className="page-actions">
          <button className="btn btn-ghost btn-sm" onClick={fetchKeys}>
            <RefreshCw size={14} />
          </button>
          <button className="btn btn-primary btn-sm" onClick={() => setShowCreateForm(!showCreateForm)}>
            <Plus size={14} />
            <span>{t('gateway.createKey', '创建 Key')}</span>
          </button>
        </div>
      </div>

      {createdKey && (
        <div className="alert alert-success">
          <div className="alert-content">
            <p className="font-semibold">{t('gateway.keyCreated', 'API Key 已创建，请立即复制保存：')}</p>
            <div className="flex items-center gap-2 mt-2">
              <code className="bg-base-300 px-2 py-1 rounded text-sm">
                {showKey ? createdKey : createdKey.substring(0, 12) + '•'.repeat(20)}
              </code>
              <button className="btn btn-ghost btn-xs" onClick={() => setShowKey(!showKey)}>
                {showKey ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
              <button className="btn btn-ghost btn-xs" onClick={() => copyToClipboard(createdKey)}>
                <Copy size={12} />
              </button>
            </div>
            <button className="btn btn-ghost btn-xs mt-2" onClick={() => setCreatedKey(null)}>
              {t('common.dismiss', '关闭')}
            </button>
          </div>
        </div>
      )}

      {showCreateForm && (
        <div className="gateway-create-key-form">
          <input
            type="text"
            className="input input-bordered input-sm"
            placeholder={t('gateway.keyName', 'Key 名称')}
            value={newKeyName}
            onChange={(e) => setNewKeyName(e.target.value)}
          />
          <input
            type="text"
            className="input input-bordered input-sm"
            placeholder={t('gateway.allowedModels', '允许的模型（逗号分隔，留空为全部）')}
            value={newKeyModels}
            onChange={(e) => setNewKeyModels(e.target.value)}
          />
          <div className="form-actions">
            <button className="btn btn-sm btn-ghost" onClick={() => setShowCreateForm(false)}>
              {t('common.cancel', '取消')}
            </button>
            <button className="btn btn-sm btn-primary" onClick={handleCreateKey}>
              {t('common.create', '创建')}
            </button>
          </div>
        </div>
      )}

      <div className="gateway-apikey-list">
        {apiKeys.length === 0 ? (
          <div className="empty-state">
            <Key size={48} className="empty-icon" />
            <p>{t('gateway.noApiKeys', '暂无 API Key，请创建一个')}</p>
          </div>
        ) : (
          <table className="table table-compact w-full">
            <thead>
              <tr>
                <th>{t('gateway.keyName', '名称')}</th>
                <th>{t('gateway.keyPrefix', '前缀')}</th>
                <th>{t('gateway.enabled', '状态')}</th>
                <th>{t('gateway.usageCount', '使用次数')}</th>
                <th>{t('gateway.models', '模型限制')}</th>
                <th>{t('gateway.lastUsed', '最后使用')}</th>
                <th>{t('gateway.actions', '操作')}</th>
              </tr>
            </thead>
            <tbody>
              {apiKeys.map((key) => (
                <tr key={key.id} className={key.enabled ? '' : 'opacity-50'}>
                  <td className="font-medium">{key.name}</td>
                  <td><code className="text-xs">{key.key_prefix}•••</code></td>
                  <td>
                    <span className={`badge badge-sm ${key.enabled ? 'badge-success' : 'badge-ghost'}`}>
                      {key.enabled ? t('gateway.enabled', '启用') : t('gateway.disabled', '禁用')}
                    </span>
                  </td>
                  <td>{key.usage_count.toLocaleString()}</td>
                  <td className="text-xs opacity-70">
                    {key.allowed_models ? (
                      JSON.parse(key.allowed_models).join(', ')
                    ) : (
                      t('gateway.allModels', '全部')
                    )}
                  </td>
                  <td className="text-xs opacity-70">{formatTime(key.last_used_at)}</td>
                  <td>
                    <div className="flex gap-1">
                      <button
                        className="btn btn-ghost btn-xs"
                        onClick={() => handleToggleKey(key.id, key.enabled)}
                      >
                        {key.enabled ? <ToggleRight size={14} /> : <ToggleLeft size={14} />}
                      </button>
                      <button
                        className="btn btn-ghost btn-xs text-error"
                        onClick={() => handleDeleteKey(key.id)}
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
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

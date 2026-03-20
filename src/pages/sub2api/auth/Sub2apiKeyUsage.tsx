import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Search, Loader2, BarChart } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';

interface KeyStats {
  key?: string;
  total_requests?: number;
  total_tokens?: number;
  remaining_quota?: number;
  [key: string]: unknown;
}

export default function Sub2apiKeyUsage() {
  const { t } = useTranslation();
  const [apiKey, setApiKey] = useState('');
  const [stats, setStats] = useState<KeyStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleQuery = async () => {
    if (!apiKey.trim()) return;
    setLoading(true);
    setError(null);
    setStats(null);
    try {
      const data = await sub2apiClient.get<KeyStats>('/usage/stats', { api_key: apiKey.trim() });
      setStats(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2 className="s2a-page-title">{t('sub2api.keyUsage.title', 'Key 用量查询')}</h2>
      <p className="s2a-page-desc">{t('sub2api.keyUsage.desc', '输入 API Key 查询使用情况')}</p>

      <div className="s2a-section" style={{ maxWidth: 500 }}>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">API Key</label>
            <input
              className="s2a-form-input"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
              onKeyDown={(e) => e.key === 'Enter' && handleQuery()}
            />
          </div>
          <button className="btn btn-primary btn-sm" onClick={handleQuery} disabled={loading || !apiKey.trim()}>
            {loading ? <Loader2 size={14} className="gw-spin" /> : <Search size={14} />}
            {t('sub2api.keyUsage.query', '查询')}
          </button>
        </div>
      </div>

      {error && (
        <div className="s2a-section" style={{ marginTop: 16, borderColor: 'var(--danger, #ef4444)' }}>
          <div style={{ color: 'var(--danger, #ef4444)', fontSize: '0.72rem' }}>{error}</div>
        </div>
      )}

      {stats && (
        <div className="s2a-stats-grid" style={{ marginTop: 16, maxWidth: 500 }}>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.keyUsage.totalRequests', '总请求')}</span>
              <BarChart size={16} className="s2a-stat-card-icon" />
            </div>
            <div className="s2a-stat-card-value">{Number(stats.total_requests ?? 0).toLocaleString()}</div>
          </div>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.keyUsage.totalTokens', '总 Token')}</span>
            </div>
            <div className="s2a-stat-card-value">{Number(stats.total_tokens ?? 0).toLocaleString()}</div>
          </div>
          <div className="s2a-stat-card">
            <div className="s2a-stat-card-header">
              <span className="s2a-stat-card-label">{t('sub2api.keyUsage.remaining', '剩余额度')}</span>
            </div>
            <div className="s2a-stat-card-value">{Number(stats.remaining_quota ?? 0).toLocaleString()}</div>
          </div>
        </div>
      )}
    </div>
  );
}

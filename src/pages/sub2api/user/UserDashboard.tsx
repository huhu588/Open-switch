import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BarChart3, Key, Activity, CreditCard, Loader2, RefreshCw } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';

interface UserStats {
  total_requests?: number;
  today_requests?: number;
  total_tokens?: number;
  remaining_quota?: number;
  api_key_count?: number;
  [key: string]: unknown;
}

export default function UserDashboard() {
  const { t } = useTranslation();
  const [stats, setStats] = useState<UserStats | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchData = async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<UserStats>('/usage/dashboard/stats');
      setStats(data);
    } catch (err) { console.error('[User Dashboard]', err); }
    finally { setLoading(false); }
  };

  useEffect(() => { fetchData(); }, []);

  if (loading) return <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>;

  const s = stats || {};

  return (
    <div>
      <div className="s2a-section-header">
        <h2 className="s2a-page-title">{t('sub2api.userDashboard.title', '用户仪表盘')}</h2>
        <button className="btn btn-ghost btn-sm" onClick={fetchData}><RefreshCw size={14} /></button>
      </div>
      <div className="s2a-stats-grid">
        <div className="s2a-stat-card">
          <div className="s2a-stat-card-header">
            <span className="s2a-stat-card-label">{t('sub2api.userDashboard.totalRequests', '总请求')}</span>
            <Activity size={16} className="s2a-stat-card-icon" />
          </div>
          <div className="s2a-stat-card-value">{Number(s.total_requests ?? 0).toLocaleString()}</div>
        </div>
        <div className="s2a-stat-card">
          <div className="s2a-stat-card-header">
            <span className="s2a-stat-card-label">{t('sub2api.userDashboard.todayRequests', '今日请求')}</span>
            <BarChart3 size={16} className="s2a-stat-card-icon" />
          </div>
          <div className="s2a-stat-card-value">{Number(s.today_requests ?? 0).toLocaleString()}</div>
        </div>
        <div className="s2a-stat-card">
          <div className="s2a-stat-card-header">
            <span className="s2a-stat-card-label">{t('sub2api.userDashboard.remainingQuota', '剩余额度')}</span>
            <CreditCard size={16} className="s2a-stat-card-icon" />
          </div>
          <div className="s2a-stat-card-value">{Number(s.remaining_quota ?? 0).toLocaleString()}</div>
        </div>
        <div className="s2a-stat-card">
          <div className="s2a-stat-card-header">
            <span className="s2a-stat-card-label">{t('sub2api.userDashboard.apiKeys', 'API Key')}</span>
            <Key size={16} className="s2a-stat-card-icon" />
          </div>
          <div className="s2a-stat-card-value">{Number(s.api_key_count ?? 0)}</div>
        </div>
      </div>
    </div>
  );
}

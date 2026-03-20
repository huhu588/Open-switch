import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BarChart3, Users, Activity, AlertTriangle, Key, Layers, Loader2, RefreshCw } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';

interface DashboardSnapshot {
  total_requests?: number;
  active_accounts?: number;
  today_usage?: number;
  error_rate?: number;
  total_users?: number;
  total_api_keys?: number;
  total_groups?: number;
  total_accounts?: number;
  active_keys?: number;
  today_requests?: number;
  today_tokens?: number;
  [key: string]: unknown;
}

function StatCard({ icon: Icon, label, value, trend }: {
  icon: React.ComponentType<{ size?: number; className?: string }>;
  label: string;
  value: string | number;
  trend?: string;
}) {
  return (
    <div className="s2a-stat-card">
      <div className="s2a-stat-card-header">
        <span className="s2a-stat-card-label">{label}</span>
        <Icon size={16} className="s2a-stat-card-icon" />
      </div>
      <div className="s2a-stat-card-value">{value}</div>
      {trend && <div className="s2a-stat-card-trend">{trend}</div>}
    </div>
  );
}

export default function Sub2apiDashboard() {
  const { t } = useTranslation();
  const [stats, setStats] = useState<DashboardSnapshot | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchData = async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<DashboardSnapshot>('/admin/dashboard/snapshot-v2');
      setStats(data);
    } catch (err) {
      console.error('[Sub2api Dashboard]', err);
      try {
        const fallback = await sub2apiClient.get<DashboardSnapshot>('/admin/dashboard/stats');
        setStats(fallback);
      } catch {
        /* ignore */
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  if (loading) {
    return (
      <div className="sub2api-page-loading">
        <Loader2 size={24} className="gw-spin" />
      </div>
    );
  }

  const s = stats || {};

  return (
    <div>
      <div className="s2a-section-header">
        <h2 className="s2a-page-title">{t('sub2api.dashboard.title', '仪表盘')}</h2>
        <button className="btn btn-ghost btn-sm" onClick={fetchData}>
          <RefreshCw size={14} />
        </button>
      </div>

      <div className="s2a-stats-grid">
        <StatCard
          icon={Activity}
          label={t('sub2api.dashboard.totalRequests', '总请求数')}
          value={Number(s.total_requests ?? s.today_requests ?? 0).toLocaleString()}
        />
        <StatCard
          icon={Users}
          label={t('sub2api.dashboard.activeAccounts', '活跃账号')}
          value={Number(s.active_accounts ?? s.total_accounts ?? 0).toLocaleString()}
        />
        <StatCard
          icon={BarChart3}
          label={t('sub2api.dashboard.todayUsage', '今日用量')}
          value={Number(s.today_usage ?? s.today_tokens ?? 0).toLocaleString()}
        />
        <StatCard
          icon={AlertTriangle}
          label={t('sub2api.dashboard.errorRate', '错误率')}
          value={s.error_rate != null ? `${(Number(s.error_rate) * 100).toFixed(1)}%` : '0%'}
        />
        <StatCard
          icon={Users}
          label={t('sub2api.dashboard.totalUsers', '总用户数')}
          value={Number(s.total_users ?? 0).toLocaleString()}
        />
        <StatCard
          icon={Key}
          label={t('sub2api.dashboard.totalApiKeys', 'API Key 总数')}
          value={Number(s.total_api_keys ?? s.active_keys ?? 0).toLocaleString()}
        />
        <StatCard
          icon={Layers}
          label={t('sub2api.dashboard.totalGroups', '分组数')}
          value={Number(s.total_groups ?? 0).toLocaleString()}
        />
      </div>

      <div className="s2a-section">
        <div className="s2a-section-title">{t('sub2api.dashboard.overview', '服务概览')}</div>
        <p className="s2a-page-desc">
          {t('sub2api.dashboard.overviewDesc', 'Sub2api 高级网关服务运行正常，可通过左侧导航管理各项功能。')}
        </p>
      </div>
    </div>
  );
}

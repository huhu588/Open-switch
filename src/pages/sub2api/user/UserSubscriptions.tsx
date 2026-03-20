import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { CreditCard, Loader2, RefreshCw } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';

interface UserSub {
  id: number;
  plan_name?: string;
  status?: string;
  quota?: number;
  used_quota?: number;
  expires_at?: string;
  [key: string]: unknown;
}

export default function UserSubscriptions() {
  const { t } = useTranslation();
  const [subs, setSubs] = useState<UserSub[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchSubs = async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<UserSub[] | { data?: UserSub[] }>('/subscriptions');
      const items = Array.isArray(data) ? data : (data as { data?: UserSub[] })?.data || [];
      setSubs(items);
    } catch (err) { console.error('[User Subscriptions]', err); }
    finally { setLoading(false); }
  };

  useEffect(() => { fetchSubs(); }, []);

  if (loading) return <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>;

  return (
    <div>
      <div className="s2a-section-header">
        <h2 className="s2a-page-title">{t('sub2api.userSubs.title', '我的订阅')}</h2>
        <button className="btn btn-ghost btn-sm" onClick={fetchSubs}><RefreshCw size={14} /></button>
      </div>

      {subs.length === 0 ? (
        <div className="s2a-empty">
          <CreditCard size={32} className="s2a-empty-icon" />
          <div className="s2a-empty-text">{t('sub2api.userSubs.empty', '暂无订阅')}</div>
          <div className="s2a-empty-desc">{t('sub2api.userSubs.emptyDesc', '前往「购买订阅」选择合适的计划')}</div>
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 12 }}>
          {subs.map((sub) => (
            <div key={sub.id} className="s2a-stat-card">
              <div className="s2a-stat-card-header">
                <span style={{ fontSize: '0.82rem', fontWeight: 600, color: 'var(--text-primary)' }}>{sub.plan_name || '-'}</span>
                <span className={`s2a-badge s2a-badge--${sub.status === 'active' ? 'success' : 'gray'}`}>{sub.status || '-'}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.68rem', color: 'var(--text-muted)' }}>
                <span>{t('sub2api.userSubs.quota', '额度')}: {sub.used_quota ?? 0} / {sub.quota ?? '-'}</span>
                <span>{t('sub2api.userSubs.expires', '到期')}: {sub.expires_at || '-'}</span>
              </div>
              {sub.quota && sub.used_quota != null && (
                <div style={{ height: 4, background: 'var(--bg-tertiary)', borderRadius: 2, overflow: 'hidden', marginTop: 4 }}>
                  <div style={{ height: '100%', width: `${Math.min(100, (sub.used_quota / sub.quota) * 100)}%`, background: 'var(--primary)', borderRadius: 2 }} />
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

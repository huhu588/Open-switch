import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ShoppingCart, Loader2, Check } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

interface Plan {
  id: number;
  name?: string;
  price?: number;
  quota?: number;
  duration_days?: number;
  description?: string;
  [key: string]: unknown;
}

export default function UserPurchase() {
  const { t } = useTranslation();
  const toast = useToast();
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [purchasing, setPurchasing] = useState<number | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const data = await sub2apiClient.get<Plan[] | { data?: Plan[] }>('/admin/subscriptions');
        const items = Array.isArray(data) ? data : (data as { data?: Plan[] })?.data || [];
        setPlans(items);
      } catch (err) { console.error('[User Purchase]', err); }
      finally { setLoading(false); }
    })();
  }, []);

  const handlePurchase = async (planId: number) => {
    setPurchasing(planId);
    try {
      await sub2apiClient.post('/admin/subscriptions/assign', { plan_id: planId });
      toast.success(t('sub2api.purchase.success', '订阅成功'));
    } catch (err) { toast.error(String(err)); }
    finally { setPurchasing(null); }
  };

  if (loading) return <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>;

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.purchase.title', '购买订阅')}</h2>
      <p className="s2a-page-desc">{t('sub2api.purchase.desc', '选择合适的订阅计划')}</p>

      {plans.length === 0 ? (
        <div className="s2a-empty">
          <ShoppingCart size={32} className="s2a-empty-icon" />
          <div className="s2a-empty-text">{t('sub2api.purchase.empty', '暂无可用计划')}</div>
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))', gap: 16 }}>
          {plans.map((plan) => (
            <div key={plan.id} className="s2a-section" style={{ display: 'flex', flexDirection: 'column', gap: 12, textAlign: 'center' }}>
              <div style={{ fontSize: '0.88rem', fontWeight: 700, color: 'var(--text-primary)' }}>{plan.name || '-'}</div>
              <div style={{ fontSize: '1.5rem', fontWeight: 800, color: 'var(--primary)' }}>¥{plan.price?.toFixed(2) ?? '-'}</div>
              <div style={{ fontSize: '0.68rem', color: 'var(--text-muted)' }}>
                {plan.quota?.toLocaleString() ?? '-'} {t('sub2api.purchase.quota', '额度')} · {plan.duration_days ?? '-'} {t('sub2api.purchase.days', '天')}
              </div>
              {plan.description && <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)' }}>{plan.description}</div>}
              <button
                className="btn btn-primary btn-sm"
                style={{ alignSelf: 'center', marginTop: 8 }}
                onClick={() => handlePurchase(plan.id)}
                disabled={purchasing === plan.id}
              >
                {purchasing === plan.id ? <Loader2 size={14} className="gw-spin" /> : <Check size={14} />}
                {t('sub2api.purchase.buy', '购买')}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

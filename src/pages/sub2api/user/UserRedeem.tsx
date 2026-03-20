import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Gift, Loader2 } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

export default function UserRedeem() {
  const { t } = useTranslation();
  const toast = useToast();
  const [code, setCode] = useState('');
  const [loading, setLoading] = useState(false);

  const handleRedeem = async () => {
    if (!code.trim()) return;
    setLoading(true);
    try {
      await sub2apiClient.post('/redeem', { code: code.trim() });
      toast.success(t('sub2api.userRedeem.success', '兑换成功'));
      setCode('');
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.userRedeem.title', '兑换码')}</h2>

      <div className="s2a-section" style={{ maxWidth: 500 }}>
        <div className="s2a-section-title" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Gift size={16} />
          {t('sub2api.userRedeem.inputLabel', '输入兑换码')}
        </div>
        <div className="s2a-form-row" style={{ marginTop: 12 }}>
          <div className="s2a-form-field">
            <input
              className="s2a-form-input"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              placeholder={t('sub2api.userRedeem.placeholder', '请输入兑换码')}
              onKeyDown={(e) => e.key === 'Enter' && handleRedeem()}
            />
          </div>
          <button className="btn btn-primary btn-sm" onClick={handleRedeem} disabled={loading || !code.trim()}>
            {loading ? <Loader2 size={14} className="gw-spin" /> : <Gift size={14} />}
            {t('sub2api.userRedeem.redeem', '兑换')}
          </button>
        </div>
      </div>
    </div>
  );
}

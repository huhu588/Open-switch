import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { LogIn, Loader2 } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

export default function Sub2apiLogin() {
  const { t } = useTranslation();
  const toast = useToast();
  const [form, setForm] = useState({ email: '', password: '' });
  const [loading, setLoading] = useState(false);

  const handleLogin = async () => {
    if (!form.email || !form.password) return;
    setLoading(true);
    try {
      await sub2apiClient.post('/auth/login', form as unknown as Record<string, unknown>);
      toast.success(t('sub2api.login.success', '登录成功'));
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 60 }}>
      <ToastContainer toasts={toast.toasts} />
      <div className="s2a-section" style={{ width: 360 }}>
        <h2 className="s2a-page-title" style={{ textAlign: 'center', marginBottom: 20 }}>{t('sub2api.login.title', '登录')}</h2>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">Email</label>
            <input className="s2a-form-input" type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} placeholder="user@example.com" />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.login.password', '密码')}</label>
            <input
              className="s2a-form-input"
              type="password"
              value={form.password}
              onChange={(e) => setForm({ ...form, password: e.target.value })}
              onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
            />
          </div>
        </div>
        <div style={{ marginTop: 16 }}>
          <button className="btn btn-primary" style={{ width: '100%' }} onClick={handleLogin} disabled={loading}>
            {loading ? <Loader2 size={14} className="gw-spin" /> : <LogIn size={14} />}
            {t('sub2api.login.submit', '登录')}
          </button>
        </div>
      </div>
    </div>
  );
}

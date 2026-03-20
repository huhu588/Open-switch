import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { UserPlus, Loader2 } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

export default function Sub2apiRegister() {
  const { t } = useTranslation();
  const toast = useToast();
  const [form, setForm] = useState({ username: '', email: '', password: '', confirm: '' });
  const [loading, setLoading] = useState(false);

  const handleRegister = async () => {
    if (!form.email || !form.password) return;
    if (form.password !== form.confirm) {
      toast.error(t('sub2api.register.passwordMismatch', '两次密码不一致'));
      return;
    }
    setLoading(true);
    try {
      await sub2apiClient.post('/auth/register', {
        username: form.username,
        email: form.email,
        password: form.password,
      });
      toast.success(t('sub2api.register.success', '注册成功'));
      setForm({ username: '', email: '', password: '', confirm: '' });
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
        <h2 className="s2a-page-title" style={{ textAlign: 'center', marginBottom: 20 }}>{t('sub2api.register.title', '注册')}</h2>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.register.username', '用户名')}</label>
            <input className="s2a-form-input" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">Email</label>
            <input className="s2a-form-input" type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.register.password', '密码')}</label>
            <input className="s2a-form-input" type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.register.confirm', '确认密码')}</label>
            <input className="s2a-form-input" type="password" value={form.confirm} onChange={(e) => setForm({ ...form, confirm: e.target.value })} />
          </div>
        </div>
        <div style={{ marginTop: 16 }}>
          <button className="btn btn-primary" style={{ width: '100%' }} onClick={handleRegister} disabled={loading}>
            {loading ? <Loader2 size={14} className="gw-spin" /> : <UserPlus size={14} />}
            {t('sub2api.register.submit', '注册')}
          </button>
        </div>
      </div>
    </div>
  );
}

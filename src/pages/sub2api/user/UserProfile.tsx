import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Save, Loader2, UserCircle } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

interface Profile {
  username?: string;
  email?: string;
  display_name?: string;
  [key: string]: unknown;
}

export default function UserProfile() {
  const { t } = useTranslation();
  const toast = useToast();
  const [profile, setProfile] = useState<Profile>({});
  const [password, setPassword] = useState({ current: '', new_password: '', confirm: '' });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const data = await sub2apiClient.get<Profile>('/user/profile');
        setProfile(data || {});
      } catch (err) { console.error('[User Profile]', err); }
      finally { setLoading(false); }
    })();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      await sub2apiClient.put('/user', profile as unknown as Record<string, unknown>);
      toast.success(t('sub2api.profile.saveSuccess', '资料已更新'));
    } catch (err) { toast.error(String(err)); }
    finally { setSaving(false); }
  };

  const handleChangePassword = async () => {
    if (password.new_password !== password.confirm) {
      toast.error(t('sub2api.profile.passwordMismatch', '两次密码不一致'));
      return;
    }
    setSaving(true);
    try {
      await sub2apiClient.put('/user/password', {
        current_password: password.current,
        new_password: password.new_password,
      });
      toast.success(t('sub2api.profile.passwordChanged', '密码已修改'));
      setPassword({ current: '', new_password: '', confirm: '' });
    } catch (err) { toast.error(String(err)); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>;

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.profile.title', '个人资料')}</h2>

      <div className="s2a-section" style={{ maxWidth: 500 }}>
        <div className="s2a-section-title" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <UserCircle size={16} />
          {t('sub2api.profile.basicInfo', '基本信息')}
        </div>
        <div className="s2a-form-row" style={{ marginTop: 12 }}>
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.profile.displayName', '显示名称')}</label>
            <input className="s2a-form-input" value={profile.display_name || ''} onChange={(e) => setProfile({ ...profile, display_name: e.target.value })} />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">Email</label>
            <input className="s2a-form-input" value={profile.email || ''} disabled style={{ opacity: 0.6 }} />
          </div>
        </div>
        <div style={{ marginTop: 12 }}>
          <button className="btn btn-primary btn-sm" onClick={handleSave} disabled={saving}>
            {saving ? <Loader2 size={14} className="gw-spin" /> : <Save size={14} />}
            {t('common.save', '保存')}
          </button>
        </div>
      </div>

      <div className="s2a-section" style={{ maxWidth: 500, marginTop: 16 }}>
        <div className="s2a-section-title">{t('sub2api.profile.changePassword', '修改密码')}</div>
        <div className="s2a-form-row" style={{ marginTop: 12 }}>
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.profile.currentPassword', '当前密码')}</label>
            <input className="s2a-form-input" type="password" value={password.current} onChange={(e) => setPassword({ ...password, current: e.target.value })} />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.profile.newPassword', '新密码')}</label>
            <input className="s2a-form-input" type="password" value={password.new_password} onChange={(e) => setPassword({ ...password, new_password: e.target.value })} />
          </div>
        </div>
        <div className="s2a-form-row">
          <div className="s2a-form-field">
            <label className="s2a-form-label">{t('sub2api.profile.confirmPassword', '确认密码')}</label>
            <input className="s2a-form-input" type="password" value={password.confirm} onChange={(e) => setPassword({ ...password, confirm: e.target.value })} />
          </div>
        </div>
        <div style={{ marginTop: 12 }}>
          <button className="btn btn-primary btn-sm" onClick={handleChangePassword} disabled={saving}>
            {t('sub2api.profile.changePassword', '修改密码')}
          </button>
        </div>
      </div>
    </div>
  );
}

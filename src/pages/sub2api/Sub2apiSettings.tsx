import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Save, RefreshCw, Loader2 } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

interface SystemSettings {
  site_name?: string;
  server_address?: string;
  register_enabled?: boolean;
  email_verification?: boolean;
  turnstile_enabled?: boolean;
  turnstile_site_key?: string;
  turnstile_secret_key?: string;
  smtp_server?: string;
  smtp_port?: number;
  smtp_from?: string;
  smtp_token?: string;
  [key: string]: unknown;
}

export default function Sub2apiSettings() {
  const { t } = useTranslation();
  const toast = useToast();
  const [settings, setSettings] = useState<SystemSettings>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [tab, setTab] = useState<'general' | 'email' | 'security'>('general');

  const fetchSettings = async () => {
    setLoading(true);
    try {
      const data = await sub2apiClient.get<SystemSettings>('/admin/settings');
      setSettings(data || {});
    } catch (err) {
      console.error('[Sub2api Settings]', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchSettings(); }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      await sub2apiClient.put('/admin/settings', settings as unknown as Record<string, unknown>);
      toast.success(t('sub2api.settings.saveSuccess', '设置已保存'));
    } catch (err) {
      toast.error(String(err));
    } finally {
      setSaving(false);
    }
  };

  const updateField = (key: string, value: unknown) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  };

  if (loading) {
    return <div className="sub2api-page-loading"><Loader2 size={24} className="gw-spin" /></div>;
  }

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <div className="s2a-section-header">
        <h2 className="s2a-page-title">{t('sub2api.settings.title', '系统设置')}</h2>
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn-ghost btn-sm" onClick={fetchSettings}><RefreshCw size={14} /></button>
          <button className="btn btn-primary btn-sm" onClick={handleSave} disabled={saving}>
            {saving ? <Loader2 size={14} className="gw-spin" /> : <Save size={14} />}
            {t('common.save', '保存')}
          </button>
        </div>
      </div>

      <div className="s2a-tabs">
        <button className={`s2a-tab ${tab === 'general' ? 'active' : ''}`} onClick={() => setTab('general')}>
          {t('sub2api.settings.general', '通用设置')}
        </button>
        <button className={`s2a-tab ${tab === 'email' ? 'active' : ''}`} onClick={() => setTab('email')}>
          {t('sub2api.settings.email', '邮件设置')}
        </button>
        <button className={`s2a-tab ${tab === 'security' ? 'active' : ''}`} onClick={() => setTab('security')}>
          {t('sub2api.settings.security', '安全设置')}
        </button>
      </div>

      {tab === 'general' && (
        <div className="s2a-settings-group">
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">{t('sub2api.settings.siteName', '站点名称')}</div>
              <div className="s2a-setting-desc">{t('sub2api.settings.siteNameDesc', '显示在页面标题和邮件中')}</div>
            </div>
            <input className="s2a-form-input" style={{ width: 200 }} value={settings.site_name || ''} onChange={(e) => updateField('site_name', e.target.value)} />
          </div>
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">{t('sub2api.settings.serverAddress', '服务器地址')}</div>
              <div className="s2a-setting-desc">{t('sub2api.settings.serverAddressDesc', '外部访问地址')}</div>
            </div>
            <input className="s2a-form-input" style={{ width: 200 }} value={settings.server_address || ''} onChange={(e) => updateField('server_address', e.target.value)} />
          </div>
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">{t('sub2api.settings.registerEnabled', '开放注册')}</div>
              <div className="s2a-setting-desc">{t('sub2api.settings.registerEnabledDesc', '允许新用户注册')}</div>
            </div>
            <button className={`s2a-toggle ${settings.register_enabled ? 'active' : ''}`} onClick={() => updateField('register_enabled', !settings.register_enabled)} />
          </div>
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">{t('sub2api.settings.emailVerification', '邮箱验证')}</div>
              <div className="s2a-setting-desc">{t('sub2api.settings.emailVerificationDesc', '注册时要求邮箱验证')}</div>
            </div>
            <button className={`s2a-toggle ${settings.email_verification ? 'active' : ''}`} onClick={() => updateField('email_verification', !settings.email_verification)} />
          </div>
        </div>
      )}

      {tab === 'email' && (
        <div className="s2a-settings-group">
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">SMTP {t('sub2api.settings.server', '服务器')}</div>
            </div>
            <input className="s2a-form-input" style={{ width: 200 }} value={settings.smtp_server || ''} onChange={(e) => updateField('smtp_server', e.target.value)} placeholder="smtp.example.com" />
          </div>
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">SMTP {t('sub2api.settings.port', '端口')}</div>
            </div>
            <input className="s2a-form-input" style={{ width: 100 }} type="number" value={settings.smtp_port || ''} onChange={(e) => updateField('smtp_port', parseInt(e.target.value) || 0)} placeholder="587" />
          </div>
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">{t('sub2api.settings.smtpFrom', '发件地址')}</div>
            </div>
            <input className="s2a-form-input" style={{ width: 200 }} value={settings.smtp_from || ''} onChange={(e) => updateField('smtp_from', e.target.value)} placeholder="noreply@example.com" />
          </div>
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">SMTP Token</div>
            </div>
            <input className="s2a-form-input" style={{ width: 200 }} type="password" value={settings.smtp_token || ''} onChange={(e) => updateField('smtp_token', e.target.value)} />
          </div>
        </div>
      )}

      {tab === 'security' && (
        <div className="s2a-settings-group">
          <div className="s2a-setting-item">
            <div className="s2a-setting-info">
              <div className="s2a-setting-label">Turnstile {t('sub2api.settings.enabled', '验证')}</div>
              <div className="s2a-setting-desc">{t('sub2api.settings.turnstileDesc', 'Cloudflare Turnstile 人机验证')}</div>
            </div>
            <button className={`s2a-toggle ${settings.turnstile_enabled ? 'active' : ''}`} onClick={() => updateField('turnstile_enabled', !settings.turnstile_enabled)} />
          </div>
          {settings.turnstile_enabled && (
            <>
              <div className="s2a-setting-item">
                <div className="s2a-setting-info"><div className="s2a-setting-label">Site Key</div></div>
                <input className="s2a-form-input" style={{ width: 250 }} value={settings.turnstile_site_key || ''} onChange={(e) => updateField('turnstile_site_key', e.target.value)} />
              </div>
              <div className="s2a-setting-item">
                <div className="s2a-setting-info"><div className="s2a-setting-label">Secret Key</div></div>
                <input className="s2a-form-input" style={{ width: 250 }} type="password" value={settings.turnstile_secret_key || ''} onChange={(e) => updateField('turnstile_secret_key', e.target.value)} />
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}

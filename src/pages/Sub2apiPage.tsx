import { useEffect, useState, useCallback, lazy, Suspense } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  Power, PowerOff, RefreshCw, Loader2, Copy, CheckCircle,
  BarChart3, Users, Layers, UserCog, Key, Globe, Activity,
  Settings, FileText, Gift, Tag, CreditCard,
  LayoutDashboard, UserCircle, ShoppingCart, Sparkles,
  Home, LogIn, UserPlus, BarChart, ExternalLink,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';
import { useSub2apiAdminStore } from '../stores/useSub2apiAdminStore';

const Sub2apiDashboard = lazy(() => import('./sub2api/Sub2apiDashboard'));
const Sub2apiAccounts = lazy(() => import('./sub2api/Sub2apiAccounts'));
const Sub2apiGroups = lazy(() => import('./sub2api/Sub2apiGroups'));
const Sub2apiUsers = lazy(() => import('./sub2api/Sub2apiUsers'));
const Sub2apiApiKeys = lazy(() => import('./sub2api/Sub2apiApiKeys'));
const Sub2apiProxies = lazy(() => import('./sub2api/Sub2apiProxies'));
const Sub2apiOps = lazy(() => import('./sub2api/Sub2apiOps'));
const Sub2apiSettings = lazy(() => import('./sub2api/Sub2apiSettings'));
const Sub2apiUsage = lazy(() => import('./sub2api/Sub2apiUsage'));
const Sub2apiRedeem = lazy(() => import('./sub2api/Sub2apiRedeem'));
const Sub2apiPromoCodes = lazy(() => import('./sub2api/Sub2apiPromoCodes'));
const Sub2apiSubscriptions = lazy(() => import('./sub2api/Sub2apiSubscriptions'));
const Sub2apiBackup = lazy(() => import('./sub2api/Sub2apiBackup'));
const Sub2apiAnnouncements = lazy(() => import('./sub2api/Sub2apiAnnouncements'));
const Sub2apiDataMgmt = lazy(() => import('./sub2api/Sub2apiDataMgmt'));

const UserDashboard = lazy(() => import('./sub2api/user/UserDashboard'));
const UserKeys = lazy(() => import('./sub2api/user/UserKeys'));
const UserUsage = lazy(() => import('./sub2api/user/UserUsage'));
const UserRedeem = lazy(() => import('./sub2api/user/UserRedeem'));
const UserProfile = lazy(() => import('./sub2api/user/UserProfile'));
const UserSubscriptions = lazy(() => import('./sub2api/user/UserSubscriptions'));
const UserPurchase = lazy(() => import('./sub2api/user/UserPurchase'));
const UserSora = lazy(() => import('./sub2api/user/UserSora'));

const Sub2apiHome = lazy(() => import('./sub2api/public/Sub2apiHome'));
const Sub2apiLogin = lazy(() => import('./sub2api/auth/Sub2apiLogin'));
const Sub2apiRegister = lazy(() => import('./sub2api/auth/Sub2apiRegister'));
const Sub2apiKeyUsage = lazy(() => import('./sub2api/auth/Sub2apiKeyUsage'));

interface Sub2apiStatus {
  running: boolean;
  port: number;
  pid: number | null;
  url: string | null;
}

const QUICK_ENDPOINTS = [
  { label: 'OpenAI Chat', path: '/v1/chat/completions' },
  { label: 'Claude Messages', path: '/v1/messages' },
  { label: 'Gemini', path: '/v1beta/' },
  { label: 'Antigravity Claude', path: '/antigravity/v1/messages' },
];

interface NavTab {
  key: string;
  label: string;
  icon: React.ComponentType<{ size?: number }>;
}

interface NavSection {
  title: string;
  tabs: NavTab[];
}

const PAGE_MAP: Record<string, React.LazyExoticComponent<React.ComponentType>> = {
  'dashboard': Sub2apiDashboard,
  'accounts': Sub2apiAccounts,
  'groups': Sub2apiGroups,
  'users': Sub2apiUsers,
  'admin-apikeys': Sub2apiApiKeys,
  'proxies': Sub2apiProxies,
  'ops': Sub2apiOps,
  'settings': Sub2apiSettings,
  'admin-usage': Sub2apiUsage,
  'redeem': Sub2apiRedeem,
  'promo-codes': Sub2apiPromoCodes,
  'subscriptions': Sub2apiSubscriptions,
  'backup': Sub2apiBackup,
  'announcements': Sub2apiAnnouncements,
  'data-mgmt': Sub2apiDataMgmt,

  'user-dashboard': UserDashboard,
  'user-keys': UserKeys,
  'user-usage': UserUsage,
  'user-redeem': UserRedeem,
  'profile': UserProfile,
  'user-subscriptions': UserSubscriptions,
  'purchase': UserPurchase,
  'sora': UserSora,

  'home': Sub2apiHome,
  'login': Sub2apiLogin,
  'register': Sub2apiRegister,
  'key-usage': Sub2apiKeyUsage,
};

export function Sub2apiPage({ embedded }: { embedded?: boolean } = {}) {
  const { t } = useTranslation();
  const toast = useToast();
  const { activeTab, setActiveTab } = useSub2apiAdminStore();
  const [status, setStatus] = useState<Sub2apiStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [copiedEndpoint, setCopiedEndpoint] = useState<string | null>(null);

  const NAV_SECTIONS: NavSection[] = [
    {
      title: t('sub2api.nav.admin', '管理后台'),
      tabs: [
        { key: 'dashboard', label: t('sub2api.nav.dashboard', '仪表盘'), icon: BarChart3 },
        { key: 'accounts', label: t('sub2api.nav.accounts', '账号管理'), icon: Users },
        { key: 'groups', label: t('sub2api.nav.groups', '分组管理'), icon: Layers },
        { key: 'users', label: t('sub2api.nav.users', '用户管理'), icon: UserCog },
        { key: 'admin-apikeys', label: t('sub2api.nav.apiKeys', 'API Key'), icon: Key },
        { key: 'proxies', label: t('sub2api.nav.proxies', '代理管理'), icon: Globe },
        { key: 'ops', label: t('sub2api.nav.ops', '运维监控'), icon: Activity },
        { key: 'settings', label: t('sub2api.nav.settings', '系统设置'), icon: Settings },
        { key: 'admin-usage', label: t('sub2api.nav.usage', '使用记录'), icon: FileText },
        { key: 'redeem', label: t('sub2api.nav.redeem', '兑换码'), icon: Gift },
        { key: 'promo-codes', label: t('sub2api.nav.promoCodes', '优惠码'), icon: Tag },
        { key: 'subscriptions', label: t('sub2api.nav.subscriptions', '订阅管理'), icon: CreditCard },
      ],
    },
    {
      title: t('sub2api.nav.user', '用户端'),
      tabs: [
        { key: 'user-dashboard', label: t('sub2api.nav.userDashboard', '用户仪表盘'), icon: LayoutDashboard },
        { key: 'user-keys', label: t('sub2api.nav.userKeys', 'API Key'), icon: Key },
        { key: 'user-usage', label: t('sub2api.nav.userUsage', '使用记录'), icon: FileText },
        { key: 'user-redeem', label: t('sub2api.nav.userRedeem', '兑换'), icon: Gift },
        { key: 'profile', label: t('sub2api.nav.profile', '个人资料'), icon: UserCircle },
        { key: 'user-subscriptions', label: t('sub2api.nav.userSubs', '我的订阅'), icon: CreditCard },
        { key: 'purchase', label: t('sub2api.nav.purchase', '购买订阅'), icon: ShoppingCart },
        { key: 'sora', label: 'Sora', icon: Sparkles },
      ],
    },
    {
      title: t('sub2api.nav.public', '认证/公开'),
      tabs: [
        { key: 'home', label: t('sub2api.nav.home', '首页'), icon: Home },
        { key: 'login', label: t('sub2api.nav.login', '登录'), icon: LogIn },
        { key: 'register', label: t('sub2api.nav.register', '注册'), icon: UserPlus },
        { key: 'key-usage', label: t('sub2api.nav.keyUsage', 'Key 用量'), icon: BarChart },
      ],
    },
  ];

  const fetchStatus = useCallback(async () => {
    try {
      const s = await invoke<Sub2apiStatus>('get_sub2api_status');
      setStatus(s);
      return s;
    } catch (error) {
      console.error('Failed to get sub2api status:', error);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleToggle = async () => {
    setActionLoading(true);
    try {
      if (status?.running) {
        await invoke('stop_sub2api');
        toast.warning(t('sub2api.stopped', 'Sub2api 已停止'));
      } else {
        await invoke('start_sub2api');
        await new Promise(resolve => setTimeout(resolve, 3000));
        toast.success(t('sub2api.started', 'Sub2api 已启动'));
      }
      await fetchStatus();
    } catch (error) {
      toast.error(String(error));
    } finally {
      setActionLoading(false);
    }
  };

  const handleSyncAccounts = async () => {
    setSyncing(true);
    try {
      await invoke('sync_accounts_to_sub2api');
      toast.success(t('sub2api.syncSuccess', '账号已同步到 Sub2api'));
    } catch (error) {
      toast.error(String(error));
    } finally {
      setSyncing(false);
    }
  };

  const copyEndpoint = (path: string) => {
    const baseUrl = status?.url || `http://localhost:${status?.port || 3456}`;
    navigator.clipboard.writeText(`${baseUrl}${path}`);
    setCopiedEndpoint(path);
    toast.success(t('sub2api.endpointCopied', '已复制'));
    setTimeout(() => setCopiedEndpoint(null), 2000);
  };

  const handleOpenExternal = () => {
    if (status?.url) window.open(status.url, '_blank');
  };

  const ActivePage = PAGE_MAP[activeTab];

  if (loading) {
    return <div className="loading-state">{t('common.loading', '加载中...')}</div>;
  }

  return (
    <div className="page-container sub2api-page">
      <ToastContainer toasts={toast.toasts} />

      {/* Header: 服务控制栏 */}
      {!embedded && (
        <div className="gw-page-header sub2api-header">
          <div className="gw-page-header-left">
            <h1 className="gw-page-title">{t('sub2api.title', 'Sub2api 高级网关')}</h1>
            <div className="gw-header-status">
              <span className={`gw-health-dot gw-health-dot--${status?.running ? 'green' : 'gray'}`} />
              <span>
                {status?.running
                  ? t('sub2api.running', '运行中') + ` (PID: ${status.pid || '-'}, Port: ${status.port})`
                  : t('sub2api.stopped', '已停止')}
              </span>
            </div>
          </div>
          <div className="gw-page-actions">
            {status?.running && (
              <>
                <button className="btn btn-ghost btn-sm" onClick={handleSyncAccounts} disabled={syncing} title={t('sub2api.sync', '同步账号')}>
                  {syncing ? <Loader2 size={14} className="gw-spin" /> : <RefreshCw size={14} />}
                </button>
                <button className="btn btn-ghost btn-sm" onClick={handleOpenExternal} title={t('sub2api.openExternal', '外部打开')}>
                  <ExternalLink size={14} />
                </button>
              </>
            )}
            <button
              className={`btn btn-sm ${status?.running ? 'btn-error' : 'btn-primary'}`}
              onClick={handleToggle}
              disabled={actionLoading}
            >
              {actionLoading ? (
                <Loader2 size={14} className="gw-spin" />
              ) : status?.running ? (
                <PowerOff size={14} />
              ) : (
                <Power size={14} />
              )}
              <span>{status?.running ? t('sub2api.stop', '停止') : t('sub2api.start', '启动')}</span>
            </button>
          </div>
        </div>
      )}

      {embedded && (
        <div className="gw-embedded-actions">
          <div className="gw-header-status">
            <span className={`gw-health-dot gw-health-dot--${status?.running ? 'green' : 'gray'}`} />
            <span>
              {status?.running
                ? t('sub2api.running', '运行中') + ` (Port: ${status.port})`
                : t('sub2api.stopped', '已停止')}
            </span>
          </div>
          <button
            className={`btn btn-sm ${status?.running ? 'btn-error' : 'btn-primary'}`}
            onClick={handleToggle}
            disabled={actionLoading}
          >
            {actionLoading ? <Loader2 size={14} className="gw-spin" /> : status?.running ? <PowerOff size={14} /> : <Power size={14} />}
            <span>{status?.running ? t('sub2api.stop', '停止') : t('sub2api.start', '启动')}</span>
          </button>
        </div>
      )}

      {/* Main Content */}
      <div className="sub2api-content">
        {!status?.running ? (
          <div className="sub2api-not-running">
            <div className="gw-empty" style={{ background: 'transparent', border: 'none' }}>
              <Power size={64} className="gw-empty-icon" />
              <div className="gw-empty-title">{t('sub2api.notRunning', 'Sub2api 服务未启动')}</div>
              <div className="gw-empty-desc">{t('sub2api.startHint', '点击上方「启动」按钮启动 Sub2api 高级网关服务')}</div>
              <p style={{ fontSize: '0.7rem', color: 'var(--text-muted)', marginTop: 8, maxWidth: 400 }}>
                {t('sub2api.features', '包含：用户管理、计费系统、订阅管理、API 密钥管理、多平台网关等完整功能')}
              </p>
              <button className="btn btn-primary" style={{ marginTop: 16 }} onClick={handleToggle} disabled={actionLoading}>
                {actionLoading ? <Loader2 size={14} className="gw-spin" /> : <Power size={14} />}
                <span>{t('sub2api.startService', '启动服务')}</span>
              </button>
            </div>
          </div>
        ) : (
          <div className="sub2api-main-layout">
            {/* Sub-navigation */}
            <div className="sub2api-subnav">
              {/* Quick Endpoints */}
              <div className="sub2api-subnav-section sub2api-endpoints-section">
                <div className="sub2api-subnav-title">{t('sub2api.quickEndpoints', '常用端点')}</div>
                <div className="sub2api-endpoints-list">
                  {QUICK_ENDPOINTS.map(ep => (
                    <div key={ep.path} className="sub2api-endpoint-item" onClick={() => copyEndpoint(ep.path)}>
                      <span className="sub2api-endpoint-label">{ep.label}</span>
                      <span className="sub2api-endpoint-copy">
                        {copiedEndpoint === ep.path ? <CheckCircle size={10} /> : <Copy size={10} />}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

              {NAV_SECTIONS.map((section) => (
                <div key={section.title} className="sub2api-subnav-section">
                  <div className="sub2api-subnav-title">{section.title}</div>
                  {section.tabs.map((tab) => {
                    const Icon = tab.icon;
                    return (
                      <button
                        key={tab.key}
                        className={`sub2api-subnav-item ${activeTab === tab.key ? 'active' : ''}`}
                        onClick={() => setActiveTab(tab.key)}
                      >
                        <Icon size={14} />
                        <span>{tab.label}</span>
                      </button>
                    );
                  })}
                </div>
              ))}
            </div>

            {/* Page Content */}
            <div className="sub2api-page-content">
              <Suspense fallback={
                <div className="sub2api-page-loading">
                  <Loader2 size={24} className="gw-spin" />
                </div>
              }>
                {ActivePage ? <ActivePage /> : <div className="gw-empty"><div className="gw-empty-title">页面不存在</div></div>}
              </Suspense>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

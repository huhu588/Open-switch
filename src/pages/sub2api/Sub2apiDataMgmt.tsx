import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Trash2, Database, AlertTriangle, Loader2 } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

export default function Sub2apiDataMgmt() {
  const { t } = useTranslation();
  const toast = useToast();
  const [loading, setLoading] = useState<string | null>(null);

  const ACTION_ROUTES: Record<string, { method: 'post' | 'delete'; path: string }> = {
    'clear-usage': { method: 'post', path: '/admin/usage/cleanup-tasks' },
    'reset-errors': { method: 'post', path: '/admin/accounts/batch-clear-error' },
    'restart': { method: 'post', path: '/admin/system/restart' },
  };

  const handleAction = async (action: string, label: string) => {
    setLoading(action);
    try {
      const route = ACTION_ROUTES[action];
      if (route) {
        if (route.method === 'delete') {
          await sub2apiClient.delete(route.path);
        } else {
          await sub2apiClient.post(route.path);
        }
      } else {
        toast.error(t('sub2api.dataMgmt.unsupported', '该操作暂不支持'));
        return;
      }
      toast.success(`${label} ${t('common.success', '成功')}`);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(null);
    }
  };

  const actions = [
    { key: 'clear-usage', label: t('sub2api.dataMgmt.clearUsage', '清除用量'), icon: Trash2, desc: t('sub2api.dataMgmt.clearUsageDesc', '创建清理任务清除使用量统计'), danger: true },
    { key: 'reset-errors', label: t('sub2api.dataMgmt.resetErrors', '重置错误'), icon: AlertTriangle, desc: t('sub2api.dataMgmt.resetErrorsDesc', '批量清除所有账号错误状态'), danger: false },
    { key: 'restart', label: t('sub2api.dataMgmt.restart', '重启服务'), icon: Database, desc: t('sub2api.dataMgmt.restartDesc', '重启 Sub2api 后端服务'), danger: true },
  ];

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.dataMgmt.title', '数据管理')}</h2>
      <p className="s2a-page-desc">{t('sub2api.dataMgmt.desc', '管理 Sub2api 服务的数据和日志。危险操作请谨慎执行。')}</p>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: 12 }}>
        {actions.map((action) => {
          const Icon = action.icon;
          return (
            <div key={action.key} className="s2a-stat-card">
              <div className="s2a-stat-card-header">
                <span className="s2a-stat-card-label">{action.label}</span>
                <Icon size={16} className="s2a-stat-card-icon" style={action.danger ? { color: 'var(--danger, #ef4444)' } : undefined} />
              </div>
              <div style={{ fontSize: '0.68rem', color: 'var(--text-muted)', marginBottom: 8 }}>{action.desc}</div>
              <button
                className={`btn btn-sm ${action.danger ? 'btn-error' : 'btn-ghost'}`}
                onClick={() => handleAction(action.key, action.label)}
                disabled={loading === action.key}
                style={{ alignSelf: 'flex-start' }}
              >
                {loading === action.key ? <Loader2 size={14} className="gw-spin" /> : <Icon size={14} />}
                {action.label}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}

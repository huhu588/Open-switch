import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { RefreshCw, Download, Trash2, Loader2, Sparkles } from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface OhMyVersionInfo { current_version: string | null; latest_version: string | null; has_update: boolean; }
interface OhMyStatus { bun_installed: boolean; bun_version: string | null; npm_installed: boolean; ohmy_installed: boolean; config: { agents: Record<string, { model: string }> } | null; version_info: OhMyVersionInfo | null; }
interface AvailableModel { provider_name: string; model_id: string; display_name: string; }
interface AgentInfo { key: string; name: string; description: string; usage: string | null; }

function pickDefaultModel(models: AvailableModel[]) {
  const userModels = models.filter(m => m.provider_name !== 'OpenCode Zen');
  if (userModels.length > 0) {
    const claudeModel = userModels.find(m => m.model_id.toLowerCase().includes('claude'));
    return claudeModel?.display_name || userModels[0].display_name;
  }

  const freeModels = models.filter(m => m.provider_name === 'OpenCode Zen');
  const glm = freeModels.find(m => m.model_id === 'glm-4.7');
  return glm?.display_name || models[0]?.display_name || '';
}

const AGENT_COLORS = [
  { bg: 'linear-gradient(135deg, rgba(139, 92, 246, 0.15), rgba(168, 85, 247, 0.1))', color: '#8b5cf6' },
  { bg: 'linear-gradient(135deg, rgba(59, 130, 246, 0.15), rgba(99, 102, 241, 0.1))', color: '#3b82f6' },
  { bg: 'linear-gradient(135deg, rgba(236, 72, 153, 0.15), rgba(244, 114, 182, 0.1))', color: '#ec4899' },
  { bg: 'linear-gradient(135deg, rgba(249, 115, 22, 0.15), rgba(245, 158, 11, 0.1))', color: '#f97316' },
  { bg: 'linear-gradient(135deg, rgba(16, 185, 129, 0.15), rgba(5, 150, 105, 0.1))', color: '#10b981' },
  { bg: 'linear-gradient(135deg, rgba(14, 165, 233, 0.15), rgba(6, 182, 212, 0.1))', color: '#0ea5e9' },
];

export function OhMyPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [loading, setLoading] = useState(true);
  const [versionLoading, setVersionLoading] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [uninstalling, setUninstalling] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [installLog, setInstallLog] = useState('');
  const [status, setStatus] = useState<OhMyStatus | null>(null);
  const [availableModels, setAvailableModels] = useState<AvailableModel[]>([]);
  const [agentInfos, setAgentInfos] = useState<AgentInfo[]>([]);
  const [agentModels, setAgentModels] = useState<Record<string, string>>({});
  const [showUninstallConfirm, setShowUninstallConfirm] = useState(false);

  const freeModels = useMemo(() => availableModels.filter(m => m.provider_name === 'OpenCode Zen'), [availableModels]);
  const userModels = useMemo(() => availableModels.filter(m => m.provider_name !== 'OpenCode Zen'), [availableModels]);
  // 仅当所有 Agent 使用同一模型时，快速设置下拉框才回显该值。
  const quickSetModel = useMemo(() => {
    if (agentInfos.length === 0) return '';

    const selectedModels = agentInfos
      .map(agent => agentModels[agent.key])
      .filter((model): model is string => Boolean(model));

    if (selectedModels.length !== agentInfos.length) return '';

    const [firstModel, ...restModels] = selectedModels;
    return restModels.every(model => model === firstModel) ? firstModel : '';
  }, [agentInfos, agentModels]);

  const loadVersionInfo = useCallback(async () => {
    setVersionLoading(true);
    try {
      const versionInfo = await invoke<OhMyVersionInfo | null>('get_ohmy_version_info');
      setStatus(prev => (prev ? { ...prev, version_info: versionInfo } : prev));
    } finally {
      setVersionLoading(false);
    }
  }, []);

  const loadStatus = useCallback(async () => {
    setLoading(true);
    try {
      const [s, m, a] = await Promise.all([
        invoke<OhMyStatus>('check_ohmy_status'),
        invoke<AvailableModel[]>('get_available_models'),
        invoke<AgentInfo[]>('get_agent_infos'),
      ]);
      setStatus(s);
      setAvailableModels(m);
      setAgentInfos(a);
      const fallbackModel = pickDefaultModel(m);
      const models: Record<string, string> = {};
      for (const agent of a) {
        models[agent.key] = s?.config?.agents?.[agent.key]?.model || fallbackModel;
      }
      setAgentModels(models);
      if (!installLog.includes('\u274C')) setInstallLog('');
      void loadVersionInfo();
    } catch (e) {
      toast.error(t('ohmy.loadFailed', '加载状态失败'));
    } finally { setLoading(false); }
  }, [installLog, loadVersionInfo, t, toast]);

  const installAndConfigure = async () => {
    setInstalling(true);
    setInstallLog(t('ohmy.startingInstall', '开始安装...') + '\n');
    try {
      await invoke<string>('install_and_configure', { agents: agentModels });
      setInstallLog('');
      toast.success(t('ohmy.installSuccess', '安装成功'));
      await loadStatus();
    } catch (e) {
      setInstallLog(prev => prev + '\n\u274C ' + String(e));
      toast.error(t('ohmy.installFailed', '安装失败'));
    } finally { setInstalling(false); }
  };

  const saveConfig = async () => {
    try {
      await invoke('save_ohmy_config', { agents: agentModels });
      toast.success(t('ohmy.saved', '配置已保存'));
    } catch (e) {
      toast.error(t('ohmy.saveFailed', '保存失败'));
    }
  };

  const doUninstall = async () => {
    setUninstalling(true);
    setShowUninstallConfirm(false);
    try {
      await invoke<string>('uninstall_ohmy');
      toast.success(t('ohmy.uninstallSuccess', '卸载成功'));
      await loadStatus();
    } catch (e) {
      setInstallLog('\u274C ' + String(e));
      toast.error(t('ohmy.uninstallFailed', '卸载失败'));
    } finally { setUninstalling(false); }
  };

  const updateOhmy = async () => {
    setUpdating(true);
    setInstallLog(t('ohmy.startingUpdate', '正在更新...') + '\n');
    try {
      await invoke<string>('update_ohmy');
      setInstallLog('');
      toast.success(t('ohmy.updateSuccess', '更新成功'));
      await loadStatus();
    } catch (e) {
      setInstallLog(prev => prev + '\n\u274C ' + String(e));
      toast.error(t('ohmy.updateFailed', '更新失败'));
    } finally { setUpdating(false); }
  };

  const setAllAgentsModel = (model: string) => {
    const updated: Record<string, string> = {};
    agentInfos.forEach(a => { updated[a.key] = model; });
    setAgentModels(updated);
  };

  useEffect(() => { loadStatus(); }, []);

  const renderModelSelect = (value: string, onChange: (v: string) => void, placeholder?: string) => (
    <select className="select select-sm select-bordered w-full" value={value} onChange={e => onChange(e.target.value)}>
      {placeholder && <option value="">{placeholder}</option>}
      {userModels.length > 0 && (
        <optgroup label={t('ohmy.yourModels', '你的模型')}>
          {userModels.map(m => <option key={m.display_name} value={m.display_name}>{m.display_name}</option>)}
        </optgroup>
      )}
      <optgroup label={t('ohmy.freeModels', '免费模型')}>
        {freeModels.map(m => <option key={m.display_name} value={m.display_name}>{m.display_name}</option>)}
      </optgroup>
    </select>
  );

  const statusItems = [
    { label: 'Bun', ok: status?.bun_installed, version: status?.bun_version, color: '#f7df1e' },
    { label: 'npm', ok: status?.npm_installed, version: null, color: '#cb3837' },
    { label: 'oh-my-opencode', ok: status?.ohmy_installed, version: status?.version_info?.current_version, color: '#a855f7' },
  ];

  return (
    <div className="h-full flex flex-col gap-4 overflow-auto p-4">
      <ToastContainer toasts={toast.toasts} />

      {showUninstallConfirm && (
        <div className="oc-modal-overlay">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('ohmy.uninstallTitle', '确认卸载')}</h3>
            <p className="py-4">{t('ohmy.confirmUninstall', '确定要卸载 oh-my-opencode 吗？')}</p>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowUninstallConfirm(false)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-error" onClick={doUninstall}>{t('ohmy.uninstall', '卸载')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Page Header */}
      <div className="oc-page-header">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--ohmy"><Sparkles size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('ohmy.title', 'oh-my-opencode')}</h2>
            <p className="oc-page-subtitle">{t('ohmy.subtitle', 'Agent 增强工具集')}</p>
          </div>
        </div>
        <div className="oc-page-header-actions">
          {!loading && <button className="btn btn-sm btn-ghost" onClick={loadStatus}><RefreshCw size={14} /></button>}
          {!loading && status?.ohmy_installed && <button className="btn btn-sm btn-primary" onClick={saveConfig}>{t('ohmy.saveConfig', '保存配置')}</button>}
          {!loading && !status?.ohmy_installed && (
            <button className="btn btn-sm btn-primary" style={{ background: 'linear-gradient(135deg, #a855f7, #ec4899)', border: 'none' }} disabled={installing || availableModels.length === 0} onClick={installAndConfigure}>
              {installing ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
              {installing ? t('ohmy.installing', '安装中...') : t('ohmy.installAndConfigure', '安装并配置')}
            </button>
          )}
        </div>
      </div>

      {loading ? (
        <div className="flex-1 flex items-center justify-center"><Loader2 className="animate-spin" size={32} /></div>
      ) : (
        <div className="flex-1 space-y-6">
          {/* Status Cards Grid */}
          <div className="grid grid-cols-3 gap-3 oc-stagger">
            {statusItems.map(item => (
              <div key={item.label} className="oc-stat-card" style={{ background: 'var(--bg-card)' }}>
                <div className="flex items-center gap-3">
                  <div className={`oc-status-dot ${item.ok ? 'is-ok' : 'is-warn'}`} />
                  <div>
                    <div className="font-semibold text-sm">{item.label}</div>
                    <div className="text-xs opacity-50 mt-0.5">
                      {item.ok ? (item.version ? `v${item.version}` : t('ohmy.installed', '已安装')) : t('ohmy.notInstalled', '未安装')}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Version Update */}
          {status?.version_info?.has_update && status.version_info.latest_version && (
            <div className="oc-mcp-card" style={{ borderColor: 'rgba(245, 158, 11, 0.3)', background: 'rgba(245, 158, 11, 0.05)' }}>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-sm">v{status.version_info.current_version}</span>
                  <span className="text-warning">&rarr;</span>
                  <span className="text-sm font-medium text-warning">v{status.version_info.latest_version}</span>
                </div>
                <button className="btn btn-xs btn-warning" disabled={updating} onClick={updateOhmy}>
                  {updating ? <Loader2 className="animate-spin" size={10} /> : <Download size={10} />}
                  {updating ? t('ohmy.updating', '更新中') : t('ohmy.update', '更新')}
                </button>
              </div>
            </div>
          )}

          {versionLoading && !status?.version_info && (
            <div className="oc-mcp-card">
              <div className="flex items-center gap-2 text-sm opacity-70">
                <Loader2 className="animate-spin" size={14} />
                <span>{t('ohmy.checkingVersion', '正在检查版本信息...')}</span>
              </div>
            </div>
          )}

          {installLog && (
            <div className={`oc-mcp-card ${installLog.includes('\u274C') ? '' : ''}`} style={installLog.includes('\u274C') ? { borderColor: 'rgba(239, 68, 68, 0.3)', background: 'rgba(239, 68, 68, 0.05)' } : {}}>
              <h4 className="text-sm font-medium mb-2">{t('ohmy.installLog', '安装日志')}</h4>
              <pre className={`text-xs font-mono whitespace-pre-wrap max-h-48 overflow-auto ${installLog.includes('\u274C') ? 'text-error' : 'opacity-60'}`}>{installLog}</pre>
            </div>
          )}

          {/* Quick Set All */}
          {availableModels.length > 0 && (
            <div className="oc-mcp-card">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <h3 className="font-medium text-sm">{t('ohmy.quickSet', '快速设置全部')}</h3>
                  <p className="text-xs opacity-50">{t('ohmy.quickSetDesc', '为所有 Agent 设置相同的模型')}</p>
                </div>
                <div style={{ minWidth: '200px' }}>
                  {renderModelSelect(quickSetModel, v => setAllAgentsModel(v), t('ohmy.quickSetPlaceholder', '请选择统一模型'))}
                </div>
              </div>
            </div>
          )}

          {/* Agent Cards */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 oc-stagger">
            {agentInfos.map((agent, i) => {
              const colorScheme = AGENT_COLORS[i % AGENT_COLORS.length];
              return (
                <div key={agent.key} className="oc-mcp-card" style={{ transition: 'transform 0.2s, box-shadow 0.2s' }}>
                  <div className="flex items-start gap-3 mb-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl shrink-0" style={{ background: colorScheme.bg, color: colorScheme.color }}>
                      <Sparkles size={20} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-sm">{agent.name}</h3>
                      <p className="text-xs opacity-50 mt-0.5">{agent.description}</p>
                    </div>
                  </div>
                  {agent.usage && (
                    <div className="mb-4 p-3 rounded-lg bg-base-300 text-xs opacity-80" style={{ border: '1px solid var(--border-light)' }}>
                      <span style={{ color: colorScheme.color }} className="font-medium">{t('ohmy.usage', '用法')}:</span> {agent.usage}
                    </div>
                  )}
                  <div>
                    <label className="text-xs opacity-50 mb-1.5 block">{t('ohmy.selectModel', '选择模型')}</label>
                    {renderModelSelect(agentModels[agent.key] || '', v => setAgentModels(prev => ({ ...prev, [agent.key]: v })))}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Uninstall */}
          {status?.ohmy_installed && (
            <div className="pt-4 border-t border-base-content/10">
              <button className="btn btn-sm btn-error btn-outline" disabled={uninstalling} onClick={() => setShowUninstallConfirm(true)}>
                {uninstalling ? <Loader2 className="animate-spin" size={14} /> : <Trash2 size={14} />}
                {uninstalling ? t('ohmy.uninstalling', '卸载中...') : t('ohmy.uninstall', '卸载')}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

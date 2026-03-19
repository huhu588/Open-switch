import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { RefreshCw, Download, Trash2, Loader2, Sparkles } from 'lucide-react';

interface OhMyVersionInfo { current_version: string | null; latest_version: string | null; has_update: boolean; }
interface OhMyStatus { bun_installed: boolean; bun_version: string | null; npm_installed: boolean; ohmy_installed: boolean; config: { agents: Record<string, { model: string }> } | null; version_info: OhMyVersionInfo | null; }
interface AvailableModel { provider_name: string; model_id: string; display_name: string; }
interface AgentInfo { key: string; name: string; description: string; usage: string | null; }

export function OhMyPage() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
  const [installing, setInstalling] = useState(false);
  const [uninstalling, setUninstalling] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [installLog, setInstallLog] = useState('');
  const [status, setStatus] = useState<OhMyStatus | null>(null);
  const [availableModels, setAvailableModels] = useState<AvailableModel[]>([]);
  const [agentInfos, setAgentInfos] = useState<AgentInfo[]>([]);
  const [agentModels, setAgentModels] = useState<Record<string, string>>({});
  const [message, setMessage] = useState('');
  const [messageType, setMessageType] = useState<'success' | 'error'>('success');
  const [showUninstallConfirm, setShowUninstallConfirm] = useState(false);

  const freeModels = useMemo(() => availableModels.filter(m => m.provider_name === 'OpenCode Zen'), [availableModels]);
  const userModels = useMemo(() => availableModels.filter(m => m.provider_name !== 'OpenCode Zen'), [availableModels]);

  const defaultModel = useMemo(() => {
    if (userModels.length > 0) {
      const claudeModel = userModels.find(m => m.model_id.toLowerCase().includes('claude'));
      if (claudeModel) return claudeModel.display_name;
      return userModels[0].display_name;
    }
    const glm = freeModels.find(m => m.model_id === 'glm-4.7');
    return glm?.display_name || availableModels[0]?.display_name || '';
  }, [availableModels, userModels, freeModels]);

  const showMsg = useCallback((msg: string, type: 'success' | 'error') => {
    setMessage(msg);
    setMessageType(type);
    setTimeout(() => setMessage(''), 3000);
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
      const models: Record<string, string> = {};
      for (const agent of a) {
        models[agent.key] = s?.config?.agents?.[agent.key]?.model || defaultModel;
      }
      setAgentModels(models);
      if (!installLog.includes('❌')) setInstallLog('');
    } catch (e) {
      console.error('Failed to load status:', e);
      showMsg(t('ohmy.loadFailed', 'Failed to load'), 'error');
    } finally {
      setLoading(false);
    }
  }, [defaultModel, installLog, showMsg, t]);

  const installAndConfigure = async () => {
    setInstalling(true);
    setInstallLog(t('ohmy.startingInstall', 'Starting installation...') + '\n');
    try {
      await invoke<string>('install_and_configure', { agents: agentModels });
      setInstallLog('');
      showMsg(t('ohmy.installSuccess', 'Installation successful'), 'success');
      await loadStatus();
    } catch (e) {
      setInstallLog(prev => prev + '\n❌ ' + String(e));
      showMsg(t('ohmy.installFailed', 'Installation failed'), 'error');
    } finally {
      setInstalling(false);
    }
  };

  const saveConfig = async () => {
    try {
      await invoke('save_ohmy_config', { agents: agentModels });
      showMsg(t('ohmy.saved', 'Configuration saved'), 'success');
    } catch (e) {
      showMsg(t('ohmy.saveFailed', 'Save failed'), 'error');
    }
  };

  const doUninstall = async () => {
    setUninstalling(true);
    setShowUninstallConfirm(false);
    try {
      await invoke<string>('uninstall_ohmy');
      showMsg(t('ohmy.uninstallSuccess', 'Uninstalled successfully'), 'success');
      await loadStatus();
    } catch (e) {
      setInstallLog('❌ ' + String(e));
      showMsg(t('ohmy.uninstallFailed', 'Uninstall failed'), 'error');
    } finally {
      setUninstalling(false);
    }
  };

  const updateOhmy = async () => {
    setUpdating(true);
    setInstallLog(t('ohmy.startingUpdate', 'Updating...') + '\n');
    try {
      await invoke<string>('update_ohmy');
      setInstallLog('');
      showMsg(t('ohmy.updateSuccess', 'Updated successfully'), 'success');
      await loadStatus();
    } catch (e) {
      setInstallLog(prev => prev + '\n❌ ' + String(e));
      showMsg(t('ohmy.updateFailed', 'Update failed'), 'error');
    } finally {
      setUpdating(false);
    }
  };

  const setAllAgentsModel = (model: string) => {
    const updated: Record<string, string> = {};
    agentInfos.forEach(a => { updated[a.key] = model; });
    setAgentModels(updated);
  };

  useEffect(() => { loadStatus(); }, []);

  const renderModelSelect = (value: string, onChange: (v: string) => void) => (
    <select className="select select-sm select-bordered w-full" value={value} onChange={e => onChange(e.target.value)}>
      {userModels.length > 0 && <optgroup label={t('ohmy.yourModels', 'Your Models')}>{userModels.map(m => <option key={m.display_name} value={m.display_name}>{m.display_name}</option>)}</optgroup>}
      <optgroup label={t('ohmy.freeModels', 'Free Models')}>{freeModels.map(m => <option key={m.display_name} value={m.display_name}>{m.display_name}</option>)}</optgroup>
    </select>
  );

  return (
    <div className="h-full flex flex-col gap-4 overflow-auto p-4">
      {showUninstallConfirm && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('ohmy.uninstallTitle', 'Confirm Uninstall')}</h3>
            <p className="py-4">{t('ohmy.confirmUninstall', 'Are you sure you want to uninstall oh-my-opencode?')}</p>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowUninstallConfirm(false)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-error" onClick={doUninstall}>{t('ohmy.uninstall', 'Uninstall')}</button>
            </div>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 text-white shadow-lg">
            <Sparkles size={24} />
          </div>
          <div>
            <h1 className="text-xl font-bold">{t('ohmy.title', 'oh-my-opencode')}</h1>
            <p className="text-xs opacity-60">{t('ohmy.subtitle', 'Agent enhancement toolkit')}</p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          {message && <div className={`text-sm px-4 py-2 rounded-lg animate-pulse ${messageType === 'success' ? 'bg-success/20 text-success' : 'bg-error/20 text-error'}`}>{message}</div>}
          {!loading && <button className="btn btn-sm btn-ghost" onClick={loadStatus}><RefreshCw size={14} /> {t('ohmy.refresh', 'Refresh')}</button>}
          {!loading && status?.ohmy_installed && <button className="btn btn-sm btn-primary" onClick={saveConfig}>{t('ohmy.saveConfig', 'Save Config')}</button>}
          {!loading && !status?.ohmy_installed && (
            <button className="btn btn-sm btn-primary bg-gradient-to-r from-purple-500 to-pink-500 border-none" disabled={installing || availableModels.length === 0} onClick={installAndConfigure}>
              {installing ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
              {installing ? t('ohmy.installing', 'Installing...') : t('ohmy.installAndConfigure', 'Install & Configure')}
            </button>
          )}
        </div>
      </div>

      {loading ? (
        <div className="flex-1 flex items-center justify-center"><Loader2 className="animate-spin" size={32} /></div>
      ) : (
        <div className="flex-1 space-y-6">
          <div className="card bg-base-200 p-4">
            <div className="flex items-center gap-4 flex-wrap">
              {[
                { label: 'Bun', ok: status?.bun_installed, version: status?.bun_version },
                { label: 'npm', ok: status?.npm_installed },
                { label: 'oh-my-opencode', ok: status?.ohmy_installed },
              ].map(item => (
                <div key={item.label} className="flex items-center gap-2">
                  <div className={`w-3 h-3 rounded-full ${item.ok ? 'bg-success' : 'bg-warning'}`} />
                  <span className="text-sm">{item.label}: {item.ok ? (item.version || t('ohmy.installed', 'Installed')) : t('ohmy.notInstalled', 'Not installed')}</span>
                </div>
              ))}
              {status?.version_info?.current_version && (
                <div className="flex items-center gap-2">
                  <span className="text-sm opacity-60">v{status.version_info.current_version}</span>
                  {status.version_info.has_update && status.version_info.latest_version && (
                    <>
                      <span className="text-xs text-warning">→ v{status.version_info.latest_version}</span>
                      <button className="btn btn-xs btn-primary" disabled={updating} onClick={updateOhmy}>
                        {updating ? <Loader2 className="animate-spin" size={10} /> : <Download size={10} />}
                        {updating ? t('ohmy.updating', 'Updating') : t('ohmy.update', 'Update')}
                      </button>
                    </>
                  )}
                </div>
              )}
            </div>
          </div>

          {installLog && (
            <div className={`card p-4 ${installLog.includes('❌') ? 'bg-error/10 border border-error/30' : 'bg-base-200'}`}>
              <h4 className="text-sm font-medium mb-2">{t('ohmy.installLog', 'Install Log')}</h4>
              <pre className={`text-xs font-mono whitespace-pre-wrap max-h-60 overflow-auto ${installLog.includes('❌') ? 'text-error' : 'opacity-60'}`}>{installLog}</pre>
            </div>
          )}

          {availableModels.length > 0 && (
            <div className="card bg-base-200 p-4">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <h3 className="font-medium">{t('ohmy.quickSet', 'Quick Set All')}</h3>
                  <p className="text-xs opacity-60">{t('ohmy.quickSetDesc', 'Set the same model for all agents')}</p>
                </div>
                {renderModelSelect('', v => setAllAgentsModel(v))}
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {agentInfos.map(agent => (
              <div key={agent.key} className="card bg-base-200 p-5 hover:shadow-md transition-all">
                <div className="flex items-start gap-3 mb-4">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary"><Sparkles size={20} /></div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-semibold">{agent.name}</h3>
                    <p className="text-sm opacity-60">{agent.description}</p>
                  </div>
                </div>
                {agent.usage && (
                  <div className="mb-4 p-3 rounded-lg bg-base-300 text-xs opacity-80">
                    <span className="text-primary font-medium">{t('ohmy.usage', 'Usage')}:</span> {agent.usage}
                  </div>
                )}
                <div>
                  <label className="text-xs opacity-60 mb-1.5 block">{t('ohmy.selectModel', 'Select Model')}</label>
                  {renderModelSelect(agentModels[agent.key] || '', v => setAgentModels(prev => ({ ...prev, [agent.key]: v })))}
                </div>
              </div>
            ))}
          </div>

          {status?.ohmy_installed && (
            <div className="pt-4 border-t border-base-content/10">
              <button className="btn btn-sm btn-error btn-outline" disabled={uninstalling} onClick={() => setShowUninstallConfirm(true)}>
                {uninstalling ? <Loader2 className="animate-spin" size={14} /> : <Trash2 size={14} />}
                {uninstalling ? t('ohmy.uninstalling', 'Uninstalling...') : t('ohmy.uninstall', 'Uninstall')}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { RefreshCw, ChevronDown, X, Loader2, Terminal } from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface VersionManagerInfo {
  name: string;
  installed: boolean;
  version: string | null;
  install_hint: string;
  can_uninstall: boolean;
}

interface RecommendedVersion { version: string; label: string; for_claude: boolean; }

interface DevEnvInfo {
  name: string;
  id: string;
  installed: boolean;
  current_version: string | null;
  installed_versions: string[];
  version_manager: VersionManagerInfo;
  recommended_versions: RecommendedVersion[];
  icon: string;
}

interface EnvState {
  switching: boolean;
  installing: boolean;
  uninstalling: boolean;
  managerInstalling: boolean;
  managerUninstalling: boolean;
  expanded: boolean;
  customVersion: string;
}

const ENV_COLORS: Record<string, string> = {
  nodejs: '#68A063', python: '#3776AB', rust: '#DEA584', go: '#00ADD8',
  java: '#E76F00', cpp: '#00599C', dotnet: '#512BD4', php: '#777BB4',
  kotlin: '#7F52FF', swift: '#F05138',
};

const ENV_LOGOS: Record<string, string> = {
  nodejs: 'N', python: 'Py', rust: 'Rs', go: 'Go', java: 'J',
  cpp: 'C++', dotnet: 'C#', php: 'PHP', kotlin: 'Kt', swift: 'Sw',
};

const DEFAULT_STATE: EnvState = { switching: false, installing: false, uninstalling: false, managerInstalling: false, managerUninstalling: false, expanded: false, customVersion: '' };

export function DevEnvPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [environments, setEnvironments] = useState<DevEnvInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [logs, setLogs] = useState<string[]>([]);
  const [envStates, setEnvStates] = useState<Record<string, EnvState>>({});

  const addLog = useCallback((message: string) => {
    const time = new Date().toLocaleTimeString();
    setLogs(prev => [`[${time}] ${message}`, ...prev].slice(0, 100));
  }, []);

  const getState = useCallback((envId: string): EnvState => envStates[envId] || DEFAULT_STATE, [envStates]);

  const updateState = useCallback((envId: string, patch: Partial<EnvState>) => {
    setEnvStates(prev => ({ ...prev, [envId]: { ...(prev[envId] || DEFAULT_STATE), ...patch } }));
  }, []);

  const detectAll = useCallback(async () => {
    setLoading(true);
    addLog(t('devenv.detecting', '正在检测环境...'));
    try {
      const envs = await invoke<DevEnvInfo[]>('detect_all_dev_envs');
      setEnvironments(envs);
      envs.forEach(env => {
        setEnvStates(prev => prev[env.id] ? prev : { ...prev, [env.id]: DEFAULT_STATE });
      });
      addLog(`Detected ${envs.length} environments, ${envs.filter(e => e.installed).length} installed`);
    } catch (e) {
      addLog(`Detection failed: ${e}`);
      toast.error(String(e));
    } finally { setLoading(false); }
  }, [addLog, t]);

  const refreshEnv = useCallback(async (envId: string) => {
    try {
      const env = await invoke<DevEnvInfo>('detect_single_dev_env', { envName: envId });
      setEnvironments(prev => prev.map(e => e.id === envId ? env : e));
    } catch (e) { addLog(`Refresh failed: ${e}`); }
  }, [addLog]);

  const switchVersion = async (envId: string, version: string) => {
    updateState(envId, { switching: true });
    addLog(`Switching ${envId} to v${version}...`);
    try {
      const result = await invoke<string>('switch_env_version', { envName: envId, version });
      addLog(result);
      await refreshEnv(envId);
      toast.success(t('devenv.switchSuccess', '版本已切换'));
    } catch (e) { addLog(`Switch failed: ${e}`); toast.error(String(e)); }
    finally { updateState(envId, { switching: false }); }
  };

  const installVersion = async (envId: string, version: string) => {
    updateState(envId, { installing: true });
    addLog(`Installing ${envId} v${version}...`);
    try {
      const result = await invoke<string>('install_env_version', { envName: envId, version });
      addLog(result);
      await refreshEnv(envId);
      toast.success(t('devenv.installSuccess', '安装成功'));
    } catch (e) { addLog(`Install failed: ${e}`); toast.error(String(e)); }
    finally { updateState(envId, { installing: false }); }
  };

  const uninstallEnv = async (envId: string, version: string) => {
    updateState(envId, { uninstalling: true });
    addLog(`Uninstalling ${envId} v${version}...`);
    try {
      const result = await invoke<string>('uninstall_env_version', { envName: envId, version });
      addLog(result);
      await refreshEnv(envId);
      toast.success(t('devenv.uninstallSuccess', '已卸载'));
    } catch (e) { addLog(`Uninstall failed: ${e}`); toast.error(String(e)); }
    finally { updateState(envId, { uninstalling: false }); }
  };

  const installManager = async (envId: string) => {
    updateState(envId, { managerInstalling: true });
    addLog(`Installing ${envId} version manager...`);
    try {
      const result = await invoke<string>('install_version_manager', { envName: envId });
      addLog(result);
      await detectAll();
      toast.success(t('devenv.managerInstalled', '版本管理器已安装'));
    } catch (e) { addLog(`Manager install failed: ${e}`); toast.error(String(e)); }
    finally { updateState(envId, { managerInstalling: false }); }
  };

  useEffect(() => { detectAll(); }, [detectAll]);

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <ToastContainer toasts={toast.toasts} />

      {/* Page Header */}
      <div className="oc-page-header">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--devenv"><Terminal size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('devenv.title', '开发环境')}</h2>
            <p className="oc-page-subtitle">{t('devenv.subtitle', '管理编程语言环境版本')}</p>
          </div>
        </div>
        <button className="btn btn-sm btn-primary" onClick={detectAll} disabled={loading}>
          {loading ? <Loader2 className="animate-spin" size={14} /> : <RefreshCw size={14} />}
          {loading ? t('devenv.detecting', '检测中...') : t('devenv.detectAll', '全部检测')}
        </button>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {loading && environments.length === 0 ? (
          <div className="flex flex-col items-center py-12 opacity-60">
            <Loader2 className="animate-spin mb-3" size={32} />
            <span>{t('devenv.detecting', '检测中...')}</span>
          </div>
        ) : (
          <div className="space-y-3 oc-stagger">
            {environments.map(env => {
              const state = getState(env.id);
              const envColor = ENV_COLORS[env.id] || '#6366f1';
              return (
                <div key={env.id} className="oc-mcp-card" style={{ overflow: 'hidden' }}>
                  <div className="flex items-center gap-4">
                    <div
                      className="flex items-center justify-center rounded-xl text-white font-bold text-lg shrink-0 shadow-sm"
                      style={{ backgroundColor: envColor, width: '48px', height: '48px', fontSize: '16px' }}
                    >
                      {ENV_LOGOS[env.id] || env.name.charAt(0)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold">{env.name}</span>
                        <span className={`badge badge-xs ${env.installed ? 'badge-success' : 'badge-ghost'}`}>
                          {env.installed ? t('devenv.installed', '已安装') : t('devenv.notInstalled', '未安装')}
                        </span>
                      </div>
                      <div className="flex items-center gap-4 mt-1 text-xs opacity-50">
                        <span>{t('devenv.version', '版本')}: <span className="font-mono">{env.current_version || '-'}</span></span>
                        <span>
                          {env.version_manager.name}: {env.version_manager.installed ? (
                            env.version_manager.version ? (
                              <span className="text-success">v{env.version_manager.version}</span>
                            ) : (
                              <span className="text-success">{t('devenv.installed', '已安装')}</span>
                            )
                          ) : <span>{t('devenv.notInstalled', '未安装')}</span>}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      {!env.version_manager.installed && (
                        <button className="btn btn-xs btn-warning" disabled={state.managerInstalling} onClick={() => installManager(env.id)}>
                          {state.managerInstalling ? t('devenv.installing', '安装中...') : t('devenv.installManager', '安装管理器')}
                        </button>
                      )}
                      <button className="btn btn-xs btn-ghost" onClick={() => updateState(env.id, { expanded: !state.expanded })}>
                        <ChevronDown size={14} className={`transition-transform ${state.expanded ? 'rotate-180' : ''}`} />
                      </button>
                    </div>
                  </div>

                  {state.expanded && (
                    <div className="oc-mcp-detail" style={{ paddingTop: '14px', marginTop: '14px' }}>
                      <div className="space-y-5">
                        {/* Installed Versions */}
                        {env.installed_versions.length > 0 && (
                          <div>
                            <div className="oc-section-title">{t('devenv.installedVersions', '已安装版本')}</div>
                            <div className="flex flex-wrap gap-2">
                              {env.installed_versions.map(ver => {
                                const isCurrent = ver === env.current_version;
                                return (
                                  <div
                                    key={ver}
                                    className="group flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg border transition-all"
                                    style={isCurrent ? { borderColor: envColor, background: `${envColor}15`, color: envColor, fontWeight: 600 } : { borderColor: 'var(--border)' }}
                                  >
                                    <button disabled={state.switching || isCurrent} onClick={() => switchVersion(env.id, ver)} className="flex items-center gap-1">
                                      <span className="font-mono">v{ver}</span>
                                      {isCurrent && <span>\u2713</span>}
                                    </button>
                                    {!isCurrent && env.installed_versions.length > 1 && (
                                      <button className="ml-1 opacity-0 group-hover:opacity-100 text-error transition-opacity" onClick={() => uninstallEnv(env.id, ver)} disabled={state.uninstalling}>
                                        <X size={12} />
                                      </button>
                                    )}
                                  </div>
                                );
                              })}
                            </div>
                          </div>
                        )}

                        {/* Recommended Versions */}
                        <div>
                          <div className="oc-section-title">{t('devenv.recommendedVersions', '推荐版本')}</div>
                          <div className="space-y-2">
                            {env.recommended_versions.map(rec => (
                              <div key={rec.version} className="flex items-center justify-between p-3 rounded-lg" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-light)' }}>
                                <div className="flex items-center gap-3">
                                  <span className="font-mono text-sm font-medium">v{rec.version}</span>
                                  <span className={`oc-badge-gradient ${rec.for_claude ? 'oc-badge-gradient--claude' : 'oc-badge-gradient--stable'}`}>
                                    {rec.for_claude ? 'For Claude' : 'Stable'}
                                  </span>
                                  <span className="text-xs opacity-50">{rec.label}</span>
                                </div>
                                <button
                                  className={`btn btn-xs ${env.installed_versions.includes(rec.version) ? 'btn-success btn-disabled' : 'btn-primary'}`}
                                  disabled={state.installing || env.installed_versions.includes(rec.version)}
                                  onClick={() => installVersion(env.id, rec.version)}
                                >
                                  {env.installed_versions.includes(rec.version) ? t('devenv.installed', '已安装') : t('devenv.installVersion', '安装')}
                                </button>
                              </div>
                            ))}
                          </div>
                        </div>

                        {/* Custom Version */}
                        <div>
                          <div className="oc-section-title">{t('devenv.customVersion', '自定义版本')}</div>
                          <div className="flex gap-2">
                            <input
                              type="text"
                              className="input input-sm input-bordered flex-1 font-mono"
                              placeholder={t('devenv.customVersionPlaceholder', '如 20.0.0')}
                              value={state.customVersion}
                              onChange={e => updateState(env.id, { customVersion: e.target.value })}
                              onKeyDown={e => { if (e.key === 'Enter' && state.customVersion.trim()) installVersion(env.id, state.customVersion.trim()); }}
                            />
                            <button
                              className="btn btn-sm btn-primary"
                              disabled={state.installing || !state.customVersion?.trim()}
                              onClick={() => { installVersion(env.id, state.customVersion.trim()); updateState(env.id, { customVersion: '' }); }}
                            >
                              {state.installing ? t('devenv.installingVersion', '安装中...') : t('devenv.installVersion', '安装')}
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}

        {/* Operation Log */}
        {logs.length > 0 && (
          <div className="mt-4">
            <div className="flex items-center justify-between mb-2">
              <div className="oc-section-title" style={{ marginBottom: 0 }}>{t('devenv.operationLog', '操作日志')}</div>
              <button className="btn btn-xs btn-ghost" onClick={() => setLogs([])}>{t('devenv.clearLogs', '清除')}</button>
            </div>
            <div className="p-3 max-h-48 overflow-y-auto font-mono text-xs space-y-1" style={{ borderRadius: 'var(--radius-md)', background: 'var(--bg-tertiary)', border: '1px solid var(--border-light)' }}>
              {logs.map((log, i) => <div key={i} className="opacity-60">{log}</div>)}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

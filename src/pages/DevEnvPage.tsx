import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { RefreshCw, ChevronDown, X, Loader2 } from 'lucide-react';

interface VersionManagerInfo {
  name: string;
  installed: boolean;
  version: string | null;
  install_hint: string;
  can_uninstall: boolean;
}

interface RecommendedVersion {
  version: string;
  label: string;
  for_claude: boolean;
}

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

export function DevEnvPage() {
  const { t } = useTranslation();
  const [environments, setEnvironments] = useState<DevEnvInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [logs, setLogs] = useState<string[]>([]);
  const [envStates, setEnvStates] = useState<Record<string, EnvState>>({});

  const addLog = useCallback((message: string) => {
    const time = new Date().toLocaleTimeString();
    setLogs(prev => [`[${time}] ${message}`, ...prev].slice(0, 100));
  }, []);

  const getState = useCallback((envId: string): EnvState => {
    return envStates[envId] || { switching: false, installing: false, uninstalling: false, managerInstalling: false, managerUninstalling: false, expanded: false, customVersion: '' };
  }, [envStates]);

  const updateState = useCallback((envId: string, patch: Partial<EnvState>) => {
    setEnvStates(prev => ({ ...prev, [envId]: { ...prev[envId] || { switching: false, installing: false, uninstalling: false, managerInstalling: false, managerUninstalling: false, expanded: false, customVersion: '' }, ...patch } }));
  }, []);

  const detectAll = useCallback(async () => {
    setLoading(true);
    addLog(t('devenv.detecting', 'Detecting environments...'));
    try {
      const envs = await invoke<DevEnvInfo[]>('detect_all_dev_envs');
      setEnvironments(envs);
      envs.forEach(env => {
        setEnvStates(prev => {
          if (!prev[env.id]) return { ...prev, [env.id]: { switching: false, installing: false, uninstalling: false, managerInstalling: false, managerUninstalling: false, expanded: false, customVersion: '' } };
          return prev;
        });
      });
      addLog(`Detected ${envs.length} environments, ${envs.filter(e => e.installed).length} installed`);
    } catch (e) {
      addLog(`Detection failed: ${e}`);
    } finally {
      setLoading(false);
    }
  }, [addLog, t]);

  const refreshEnv = useCallback(async (envId: string) => {
    try {
      const env = await invoke<DevEnvInfo>('detect_single_dev_env', { envName: envId });
      setEnvironments(prev => prev.map(e => e.id === envId ? env : e));
    } catch (e) {
      addLog(`Refresh failed: ${e}`);
    }
  }, [addLog]);

  const switchVersion = async (envId: string, version: string) => {
    updateState(envId, { switching: true });
    addLog(`Switching ${envId} to v${version}...`);
    try {
      const result = await invoke<string>('switch_env_version', { envName: envId, version });
      addLog(result);
      await refreshEnv(envId);
    } catch (e) { addLog(`Switch failed: ${e}`); }
    finally { updateState(envId, { switching: false }); }
  };

  const installVersion = async (envId: string, version: string) => {
    updateState(envId, { installing: true });
    addLog(`Installing ${envId} v${version}...`);
    try {
      const result = await invoke<string>('install_env_version', { envName: envId, version });
      addLog(result);
      await refreshEnv(envId);
    } catch (e) { addLog(`Install failed: ${e}`); }
    finally { updateState(envId, { installing: false }); }
  };

  const uninstallEnv = async (envId: string, version: string) => {
    updateState(envId, { uninstalling: true });
    addLog(`Uninstalling ${envId} v${version}...`);
    try {
      const result = await invoke<string>('uninstall_env_version', { envName: envId, version });
      addLog(result);
      await refreshEnv(envId);
    } catch (e) { addLog(`Uninstall failed: ${e}`); }
    finally { updateState(envId, { uninstalling: false }); }
  };

  const installManager = async (envId: string) => {
    updateState(envId, { managerInstalling: true });
    addLog(`Installing ${envId} version manager...`);
    try {
      const result = await invoke<string>('install_version_manager', { envName: envId });
      addLog(result);
      await detectAll();
    } catch (e) { addLog(`Manager install failed: ${e}`); }
    finally { updateState(envId, { managerInstalling: false }); }
  };

  useEffect(() => { detectAll(); }, [detectAll]);

  return (
    <div className="max-w-3xl mx-auto p-4">
      <div className="card bg-base-200 shadow-sm">
        <div className="card-body">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h2 className="card-title">{t('devenv.title', 'Development Environments')}</h2>
              <p className="text-xs opacity-60 mt-0.5">{t('devenv.subtitle', 'Manage programming environment versions')}</p>
            </div>
            <button className="btn btn-sm btn-primary" onClick={detectAll} disabled={loading}>
              {loading ? <Loader2 className="animate-spin" size={14} /> : <RefreshCw size={14} />}
              {loading ? t('devenv.detecting', 'Detecting...') : t('devenv.detectAll', 'Detect All')}
            </button>
          </div>

          {loading && environments.length === 0 ? (
            <div className="flex flex-col items-center py-12 opacity-60">
              <Loader2 className="animate-spin mb-3" size={32} />
              <span>{t('devenv.detecting', 'Detecting...')}</span>
            </div>
          ) : (
            <div className="space-y-4">
              {environments.map(env => {
                const state = getState(env.id);
                return (
                  <div key={env.id} className="rounded-lg bg-base-300 overflow-hidden">
                    <div className="p-4 flex items-center gap-4">
                      <div className="flex items-center justify-center w-12 h-12 rounded-xl text-white font-bold text-lg shrink-0 shadow-sm" style={{ backgroundColor: ENV_COLORS[env.id] || '#6366f1' }}>
                        {ENV_LOGOS[env.id] || env.name.charAt(0)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-semibold">{env.name}</span>
                          <span className={`badge badge-xs ${env.installed ? 'badge-success' : 'badge-ghost'}`}>
                            {env.installed ? t('devenv.installed', 'Installed') : t('devenv.notInstalled', 'Not Installed')}
                          </span>
                        </div>
                        <div className="flex items-center gap-4 mt-1 text-xs opacity-60">
                          <span>Version: <span className="font-mono">{env.current_version || '-'}</span></span>
                          <span>{env.version_manager.name}: {env.version_manager.installed ? <span className="text-success">v{env.version_manager.version || '?'}</span> : <span>Not installed</span>}</span>
                        </div>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        {!env.version_manager.installed && (
                          <button className="btn btn-xs btn-warning" disabled={state.managerInstalling} onClick={() => installManager(env.id)}>
                            {state.managerInstalling ? t('devenv.installing', 'Installing...') : t('devenv.installManager', 'Install Manager')}
                          </button>
                        )}
                        <button className="btn btn-xs btn-ghost" onClick={() => updateState(env.id, { expanded: !state.expanded })}>
                          <ChevronDown size={14} className={`transition-transform ${state.expanded ? 'rotate-180' : ''}`} />
                        </button>
                      </div>
                    </div>

                    {state.expanded && (
                      <div className="border-t border-base-content/10 p-4 space-y-4 bg-base-100/30">
                        {env.installed_versions.length > 0 && (
                          <div>
                            <h4 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">{t('devenv.installedVersions', 'Installed Versions')}</h4>
                            <div className="flex flex-wrap gap-2">
                              {env.installed_versions.map(ver => (
                                <div key={ver} className={`group flex items-center gap-1 px-3 py-1.5 text-xs rounded-lg border transition-all ${ver === env.current_version ? 'border-primary bg-primary/10 text-primary font-semibold' : 'border-base-content/20 hover:border-primary/50'}`}>
                                  <button disabled={state.switching || ver === env.current_version} onClick={() => switchVersion(env.id, ver)}>
                                    <span className="font-mono">v{ver}</span>
                                    {ver === env.current_version && <span className="ml-1">✓</span>}
                                  </button>
                                  {ver !== env.current_version && env.installed_versions.length > 1 && (
                                    <button className="ml-1 opacity-0 group-hover:opacity-100 text-error transition-opacity" onClick={() => uninstallEnv(env.id, ver)} disabled={state.uninstalling}>
                                      <X size={12} />
                                    </button>
                                  )}
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                        <div>
                          <h4 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">{t('devenv.recommendedVersions', 'Recommended')}</h4>
                          <div className="space-y-2">
                            {env.recommended_versions.map(rec => (
                              <div key={rec.version} className="flex items-center justify-between p-3 rounded-lg bg-base-300 border border-base-content/10">
                                <div className="flex items-center gap-3">
                                  <span className="font-mono text-sm font-medium">v{rec.version}</span>
                                  <span className={`badge badge-xs ${rec.for_claude ? 'badge-secondary' : 'badge-info'}`}>
                                    {rec.for_claude ? 'For Claude' : 'Stable'}
                                  </span>
                                  <span className="text-xs opacity-60">{rec.label}</span>
                                </div>
                                <button className={`btn btn-xs ${env.installed_versions.includes(rec.version) ? 'btn-success btn-disabled' : 'btn-primary'}`} disabled={state.installing || env.installed_versions.includes(rec.version)} onClick={() => installVersion(env.id, rec.version)}>
                                  {env.installed_versions.includes(rec.version) ? t('devenv.installed', 'Installed') : t('devenv.installVersion', 'Install')}
                                </button>
                              </div>
                            ))}
                          </div>
                        </div>

                        <div>
                          <h4 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">{t('devenv.customVersion', 'Custom Version')}</h4>
                          <div className="flex gap-2">
                            <input type="text" className="input input-sm input-bordered flex-1 font-mono" placeholder={t('devenv.customVersionPlaceholder', 'e.g. 20.0.0')} value={state.customVersion} onChange={e => updateState(env.id, { customVersion: e.target.value })} onKeyDown={e => { if (e.key === 'Enter' && state.customVersion.trim()) installVersion(env.id, state.customVersion.trim()); }} />
                            <button className="btn btn-sm btn-primary" disabled={state.installing || !state.customVersion?.trim()} onClick={() => { installVersion(env.id, state.customVersion.trim()); updateState(env.id, { customVersion: '' }); }}>
                              {state.installing ? t('devenv.installingVersion', 'Installing...') : t('devenv.installVersion', 'Install')}
                            </button>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}

          {logs.length > 0 && (
            <div className="mt-6">
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-xs font-semibold uppercase tracking-wide opacity-60">{t('devenv.operationLog', 'Operation Log')}</h3>
                <button className="btn btn-xs btn-ghost" onClick={() => setLogs([])}>{t('devenv.clearLogs', 'Clear')}</button>
              </div>
              <div className="bg-base-300 rounded-lg p-3 max-h-48 overflow-y-auto font-mono text-xs space-y-1">
                {logs.map((log, i) => <div key={i} className="opacity-60">{log}</div>)}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

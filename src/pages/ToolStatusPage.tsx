import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Activity, CheckCircle, AlertTriangle, Download, Loader2 } from 'lucide-react';

interface CliToolInfo {
  id: string;
  name: string;
  installed: boolean;
  current_version: string | null;
  latest_version: string | null;
  has_update: boolean;
  npm_package: string;
  description: string;
}

interface ConflictSource {
  app: string;
  value: string;
  config_path: string;
}

interface EnvConflict {
  variable: string;
  sources: ConflictSource[];
}

interface AppStatus {
  has_global_config: boolean;
  has_project_config: boolean;
  active_provider: string | null;
  provider_count: number;
  mcp_server_count: number;
  config_paths: {
    global_config_dir: string;
    global_opencode_dir: string;
    project_opencode_dir: string | null;
  };
}

export function ToolStatusPage() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [version, setVersion] = useState('');
  const [loading, setLoading] = useState(true);
  const [cliTools, setCliTools] = useState<CliToolInfo[]>([]);
  const [cliLoading, setCliLoading] = useState(false);
  const [cliUpdating, setCliUpdating] = useState<Record<string, boolean>>({});
  const [cliUpdateMsg, setCliUpdateMsg] = useState<Record<string, string>>({});
  const [envConflicts, setEnvConflicts] = useState<EnvConflict[]>([]);
  const [conflictsLoading, setConflictsLoading] = useState(false);

  const loadStatus = useCallback(async () => {
    setLoading(true);
    try {
      const [s, v] = await Promise.all([
        invoke<AppStatus>('get_status'),
        invoke<string>('get_version'),
      ]);
      setStatus(s);
      setVersion(v);
    } catch (e) {
      console.error('Failed to load status:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadCliTools = useCallback(async () => {
    setCliLoading(true);
    try {
      const tools = await invoke<CliToolInfo[]>('detect_cli_tools');
      setCliTools(tools);
      for (const tool of tools) {
        if (tool.installed) {
          try {
            const latest = await invoke<string>('check_cli_latest_version', { npmPackage: tool.npm_package });
            setCliTools(prev => prev.map(t =>
              t.id === tool.id ? { ...t, latest_version: latest, has_update: !!(t.current_version && latest && t.current_version !== latest) } : t
            ));
          } catch { /* silent */ }
        }
      }
    } catch (e) {
      console.error('Failed to detect CLI tools:', e);
    } finally {
      setCliLoading(false);
    }
  }, []);

  const loadEnvConflicts = useCallback(async () => {
    setConflictsLoading(true);
    try {
      setEnvConflicts(await invoke<EnvConflict[]>('detect_env_conflicts'));
    } catch (e) {
      console.error('Failed to detect env conflicts:', e);
    } finally {
      setConflictsLoading(false);
    }
  }, []);

  const updateCliTool = async (tool: CliToolInfo) => {
    setCliUpdating(prev => ({ ...prev, [tool.id]: true }));
    setCliUpdateMsg(prev => ({ ...prev, [tool.id]: '' }));
    try {
      await invoke<string>('update_cli_tool', { npmPackage: tool.npm_package });
      setCliUpdateMsg(prev => ({ ...prev, [tool.id]: t('status.cliTools.updateSuccess', 'Update successful') }));
      await loadCliTools();
    } catch (e) {
      setCliUpdateMsg(prev => ({ ...prev, [tool.id]: String(e) }));
    } finally {
      setCliUpdating(prev => ({ ...prev, [tool.id]: false }));
    }
  };

  useEffect(() => {
    loadStatus();
    loadCliTools();
    loadEnvConflicts();
  }, [loadStatus, loadCliTools, loadEnvConflicts]);

  return (
    <div className="max-w-2xl mx-auto p-4 space-y-6">
      <div className="card bg-base-200 shadow-sm">
        <div className="card-body">
          <div className="flex items-center gap-3 mb-4">
            <Activity size={28} className="text-primary" />
            <h2 className="card-title">{t('nav.toolStatus', 'Tool Status')}</h2>
          </div>

          {loading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="animate-spin" size={32} />
            </div>
          ) : status && (
            <div className="space-y-6">
              {/* Version & Stats */}
              <div className="grid grid-cols-2 gap-4">
                <div className="stat bg-base-300 rounded-lg p-4">
                  <div className="stat-title text-xs">{t('status.currentVersion', 'Version')}</div>
                  <div className="stat-value text-2xl">v{version}</div>
                </div>
                <div className="stat bg-base-300 rounded-lg p-4">
                  <div className="stat-title text-xs">{t('status.providerCount', 'Providers')}</div>
                  <div className="stat-value text-2xl">{status.provider_count}</div>
                </div>
              </div>

              {/* Config Status */}
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-3">{t('status.configStatus', 'Config Status')}</h3>
                <div className="space-y-2">
                  {[
                    { label: t('status.globalConfig', 'Global Config'), value: t('status.configured', 'Configured'), ok: true },
                    { label: t('status.projectConfig', 'Project Config'), value: status.has_project_config ? t('status.configured', 'Configured') : t('status.notConfigured', 'Not Configured'), ok: status.has_project_config },
                    { label: t('status.currentProvider', 'Active Provider'), value: status.active_provider || '-', ok: !!status.active_provider },
                    { label: t('status.mcpServers', 'MCP Servers'), value: String(status.mcp_server_count), ok: true },
                  ].map((item, i) => (
                    <div key={i} className="flex justify-between items-center py-2 border-b border-base-300 last:border-0">
                      <span className="text-sm">{item.label}</span>
                      <span className={`text-sm font-mono ${item.ok ? 'text-success' : 'opacity-50'}`}>{item.value}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Config Paths */}
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-3">{t('status.configPaths', 'Config Paths')}</h3>
                <div className="space-y-2 text-sm">
                  <div className="flex gap-3"><span className="opacity-60 w-20 shrink-0">{t('status.globalConfig', 'Global')}</span><span className="font-mono text-xs break-all">{status.config_paths.global_config_dir}</span></div>
                  <div className="flex gap-3"><span className="opacity-60 w-20 shrink-0">OpenCode</span><span className="font-mono text-xs break-all">{status.config_paths.global_opencode_dir}</span></div>
                  {status.config_paths.project_opencode_dir && (
                    <div className="flex gap-3"><span className="opacity-60 w-20 shrink-0">{t('status.projectConfig', 'Project')}</span><span className="font-mono text-xs break-all">{status.config_paths.project_opencode_dir}</span></div>
                  )}
                </div>
              </div>

              {/* CLI Tools */}
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-3">{t('status.cliTools.title', 'CLI Tools')}</h3>
                {cliLoading ? (
                  <div className="flex items-center gap-2 text-sm opacity-60"><Loader2 className="animate-spin" size={14} /> Loading...</div>
                ) : (
                  <div className="space-y-3">
                    {cliTools.map(tool => (
                      <div key={tool.id} className="bg-base-300 rounded-lg p-4 flex items-center justify-between gap-4">
                        <div className="flex items-center gap-3 min-w-0">
                          <span className={`w-2.5 h-2.5 rounded-full shrink-0 ${tool.installed ? (tool.has_update ? 'bg-warning' : 'bg-success') : 'bg-base-content/30'}`} />
                          <div className="min-w-0">
                            <div className="font-medium text-sm">{tool.name}</div>
                            <div className="text-xs opacity-60 truncate">
                              {tool.installed ? `v${tool.current_version}${tool.latest_version ? ` · Latest: v${tool.latest_version}` : ''}` : t('status.cliTools.notInstalled', 'Not installed')}
                            </div>
                            {cliUpdateMsg[tool.id] && <div className="text-xs mt-0.5 text-success">{cliUpdateMsg[tool.id]}</div>}
                          </div>
                        </div>
                        <div className="flex items-center gap-2 shrink-0">
                          {tool.installed && tool.has_update && (
                            <button className="btn btn-xs btn-warning" disabled={cliUpdating[tool.id]} onClick={() => updateCliTool(tool)}>
                              {cliUpdating[tool.id] ? <Loader2 className="animate-spin" size={12} /> : <Download size={12} />}
                              {t('status.cliTools.update', 'Update')}
                            </button>
                          )}
                          {tool.installed && !tool.has_update && tool.latest_version && (
                            <span className="text-xs text-success flex items-center gap-1"><CheckCircle size={12} /> {t('status.cliTools.upToDate', 'Up to date')}</span>
                          )}
                          {!tool.installed && (
                            <button className="btn btn-xs btn-primary" disabled={cliUpdating[tool.id]} onClick={() => updateCliTool(tool)}>
                              {cliUpdating[tool.id] ? <Loader2 className="animate-spin" size={12} /> : <Download size={12} />}
                              {t('status.cliTools.install', 'Install')}
                            </button>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Env Conflicts */}
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wide opacity-60 mb-3">{t('status.envConflicts', 'Environment Conflicts')}</h3>
                {conflictsLoading ? (
                  <div className="flex items-center gap-2 text-sm opacity-60"><Loader2 className="animate-spin" size={14} /> Loading...</div>
                ) : envConflicts.length === 0 ? (
                  <div className="flex items-center gap-2 text-success text-sm"><CheckCircle size={16} /> {t('status.noConflicts', 'No conflicts detected')}</div>
                ) : (
                  <div className="space-y-3">
                    <div className="flex items-center gap-2 text-warning text-sm"><AlertTriangle size={16} /> {envConflicts.length} conflicts found</div>
                    {envConflicts.map(conflict => (
                      <div key={conflict.variable} className="bg-warning/10 border border-warning/30 rounded-lg p-3">
                        <div className="font-mono text-sm font-medium text-warning mb-2">{conflict.variable}</div>
                        {conflict.sources.map(source => (
                          <div key={source.app} className="flex justify-between text-xs">
                            <span className="font-medium">{source.app}</span>
                            <span className="font-mono opacity-60">{source.value}</span>
                          </div>
                        ))}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Activity, CheckCircle, AlertTriangle, Download, Loader2, Layers, Plug, FolderOpen } from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

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

interface ConflictSource { app: string; value: string; config_path: string; }
interface EnvConflict { variable: string; sources: ConflictSource[]; }

interface AppStatus {
  has_global_config: boolean;
  has_project_config: boolean;
  active_provider: string | null;
  provider_count: number;
  mcp_server_count: number;
  config_paths: { global_config_dir: string; global_opencode_dir: string; project_opencode_dir: string | null; };
}

export function ToolStatusPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [version, setVersion] = useState('');
  const [loading, setLoading] = useState(true);
  const [cliTools, setCliTools] = useState<CliToolInfo[]>([]);
  const [cliLoading, setCliLoading] = useState(false);
  const [cliUpdating, setCliUpdating] = useState<Record<string, boolean>>({});
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
      toast.error(String(e));
    } finally { setLoading(false); }
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
    } catch { /* silent */ }
    finally { setCliLoading(false); }
  }, []);

  const loadEnvConflicts = useCallback(async () => {
    setConflictsLoading(true);
    try {
      setEnvConflicts(await invoke<EnvConflict[]>('detect_env_conflicts'));
    } catch { /* silent */ }
    finally { setConflictsLoading(false); }
  }, []);

  const updateCliTool = async (tool: CliToolInfo) => {
    setCliUpdating(prev => ({ ...prev, [tool.id]: true }));
    try {
      await invoke<string>('update_cli_tool', { npmPackage: tool.npm_package });
      toast.success(t('status.cliTools.updateSuccess', '更新成功'));
      await loadCliTools();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setCliUpdating(prev => ({ ...prev, [tool.id]: false }));
    }
  };

  useEffect(() => { loadStatus(); loadCliTools(); loadEnvConflicts(); }, [loadStatus, loadCliTools, loadEnvConflicts]);

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <ToastContainer toasts={toast.toasts} />

      {/* Page Header */}
      <div className="oc-page-header">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--toolstatus"><Activity size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('nav.toolStatus', '工具状态')}</h2>
            <p className="oc-page-subtitle">{t('status.pageSubtitle', '查看配置状态、CLI 工具和环境冲突')}</p>
          </div>
        </div>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {loading ? (
          <div className="flex justify-center py-8"><Loader2 className="animate-spin" size={32} /></div>
        ) : status && (
          <div className="space-y-6">
            {/* Dashboard Stats */}
            <div className="grid grid-cols-4 gap-3 oc-stagger">
              <div className="oc-stat-card oc-stat-card--blue">
                <div className="oc-stat-icon"><Activity size={18} /></div>
                <div className="oc-stat-value">v{version}</div>
                <div className="oc-stat-label">{t('status.currentVersion', '当前版本')}</div>
              </div>
              <div className="oc-stat-card oc-stat-card--purple">
                <div className="oc-stat-icon"><Layers size={18} /></div>
                <div className="oc-stat-value">{status.provider_count}</div>
                <div className="oc-stat-label">{t('status.providerCount', 'Providers')}</div>
              </div>
              <div className="oc-stat-card oc-stat-card--green">
                <div className="oc-stat-icon"><Plug size={18} /></div>
                <div className="oc-stat-value">{status.mcp_server_count}</div>
                <div className="oc-stat-label">{t('status.mcpServers', 'MCP Servers')}</div>
              </div>
              <div className="oc-stat-card oc-stat-card--orange">
                <div className="oc-stat-icon"><CheckCircle size={18} /></div>
                <div className="oc-stat-value" style={{ fontSize: '16px' }}>{status.active_provider || '-'}</div>
                <div className="oc-stat-label">{t('status.currentProvider', '当前 Provider')}</div>
              </div>
            </div>

            {/* Config Status */}
            <div>
              <div className="oc-section-title">{t('status.configStatus', '配置状态')}</div>
              <div className="oc-mcp-card">
                <div className="space-y-3">
                  {[
                    { label: t('status.globalConfig', '全局配置'), ok: status.has_global_config },
                    { label: t('status.projectConfig', '项目配置'), ok: status.has_project_config },
                  ].map(item => (
                    <div key={item.label} className="flex justify-between items-center">
                      <div className="flex items-center gap-3">
                        <div className={`oc-status-dot ${item.ok ? 'is-ok' : 'is-idle'}`} />
                        <span className="text-sm">{item.label}</span>
                      </div>
                      <span className={`text-xs font-medium ${item.ok ? 'text-success' : 'opacity-40'}`}>
                        {item.ok ? t('status.configured', '已配置') : t('status.notConfigured', '未配置')}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Config Paths */}
            <div>
              <div className="oc-section-title">{t('status.configPaths', '配置路径')}</div>
              <div className="oc-mcp-card">
                <div className="space-y-3 text-sm">
                  {[
                    { label: t('status.globalConfig', '全局'), path: status.config_paths.global_config_dir },
                    { label: 'OpenCode', path: status.config_paths.global_opencode_dir },
                    ...(status.config_paths.project_opencode_dir ? [{ label: t('status.projectConfig', '项目'), path: status.config_paths.project_opencode_dir }] : []),
                  ].map(item => (
                    <div key={item.label} className="flex items-start gap-3">
                      <FolderOpen size={14} className="opacity-40 shrink-0 mt-0.5" />
                      <div className="min-w-0">
                        <div className="text-xs font-medium opacity-60">{item.label}</div>
                        <div className="font-mono text-xs opacity-80 break-all mt-0.5">{item.path}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* CLI Tools */}
            <div>
              <div className="oc-section-title">{t('status.cliTools.title', 'CLI 工具')}</div>
              {cliLoading ? (
                <div className="flex items-center gap-2 text-sm opacity-60"><Loader2 className="animate-spin" size={14} /> Loading...</div>
              ) : (
                <div className="space-y-3 oc-stagger">
                  {cliTools.map(tool => (
                    <div key={tool.id} className="oc-mcp-card">
                      <div className="flex items-center justify-between gap-4">
                        <div className="flex items-center gap-3 min-w-0">
                          <div className={`oc-status-dot ${tool.installed ? (tool.has_update ? 'is-warn' : 'is-ok') : 'is-idle'}`} />
                          <div className="min-w-0">
                            <div className="font-medium text-sm">{tool.name}</div>
                            <div className="text-xs opacity-50 truncate">
                              {tool.installed
                                ? `v${tool.current_version}${tool.latest_version ? ` \u00b7 Latest: v${tool.latest_version}` : ''}`
                                : t('status.cliTools.notInstalled', '未安装')
                              }
                            </div>
                          </div>
                        </div>
                        <div className="flex items-center gap-2 shrink-0">
                          {tool.installed && tool.has_update && (
                            <button className="btn btn-xs btn-warning" disabled={cliUpdating[tool.id]} onClick={() => updateCliTool(tool)}>
                              {cliUpdating[tool.id] ? <Loader2 className="animate-spin" size={12} /> : <Download size={12} />}
                              {t('status.cliTools.update', '更新')}
                            </button>
                          )}
                          {tool.installed && !tool.has_update && tool.latest_version && (
                            <span className="text-xs text-success flex items-center gap-1"><CheckCircle size={12} /> {t('status.cliTools.upToDate', '已最新')}</span>
                          )}
                          {!tool.installed && (
                            <button className="btn btn-xs btn-primary" disabled={cliUpdating[tool.id]} onClick={() => updateCliTool(tool)}>
                              {cliUpdating[tool.id] ? <Loader2 className="animate-spin" size={12} /> : <Download size={12} />}
                              {t('status.cliTools.install', '安装')}
                            </button>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Env Conflicts */}
            <div>
              <div className="oc-section-title">{t('status.envConflicts', '环境变量冲突')}</div>
              {conflictsLoading ? (
                <div className="flex items-center gap-2 text-sm opacity-60"><Loader2 className="animate-spin" size={14} /> Loading...</div>
              ) : envConflicts.length === 0 ? (
                <div className="oc-mcp-card" style={{ borderColor: 'rgba(34, 197, 94, 0.2)' }}>
                  <div className="flex items-center gap-2 text-success text-sm">
                    <CheckCircle size={16} /> {t('status.noConflicts', '未检测到冲突')}
                  </div>
                </div>
              ) : (
                <div className="space-y-3">
                  <div className="flex items-center gap-2 text-warning text-sm mb-2">
                    <AlertTriangle size={16} /> {envConflicts.length} {t('status.conflictsFound', '个冲突')}
                  </div>
                  {envConflicts.map(conflict => (
                    <div key={conflict.variable} className="oc-mcp-card" style={{ borderColor: 'rgba(245, 158, 11, 0.3)', background: 'rgba(245, 158, 11, 0.03)' }}>
                      <div className="font-mono text-sm font-medium text-warning mb-2">{conflict.variable}</div>
                      <div className="space-y-1">
                        {conflict.sources.map(source => (
                          <div key={source.app} className="flex justify-between text-xs">
                            <span className="font-medium">{source.app}</span>
                            <span className="font-mono opacity-60 truncate ml-4">{source.value}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

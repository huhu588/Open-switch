import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, RefreshCw, Loader2, Plug, FileText, Pencil, ChevronDown, CheckCircle, XCircle, Search } from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface McpServer { name: string; command: string; args?: string[]; env?: Record<string, string>; enabled: boolean; description?: string; }
interface RuleItem { name: string; content?: string; enabled: boolean; path?: string; }

interface McpPreset {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  description: string;
}

const MCP_PRESETS: McpPreset[] = [
  { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '/path/to/dir'], env: {}, description: '文件系统访问' },
  { name: 'brave-search', command: 'npx', args: ['-y', '@modelcontextprotocol/server-brave-search'], env: { BRAVE_API_KEY: '' }, description: 'Brave 搜索引擎' },
  { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], env: { GITHUB_PERSONAL_ACCESS_TOKEN: '' }, description: 'GitHub API 访问' },
  { name: 'sqlite', command: 'npx', args: ['-y', '@modelcontextprotocol/server-sqlite', '/path/to/db.sqlite'], env: {}, description: 'SQLite 数据库' },
  { name: 'memory', command: 'npx', args: ['-y', '@modelcontextprotocol/server-memory'], env: {}, description: '知识图谱记忆' },
  { name: 'puppeteer', command: 'npx', args: ['-y', '@modelcontextprotocol/server-puppeteer'], env: {}, description: '浏览器自动化' },
  { name: 'fetch', command: 'npx', args: ['-y', '@modelcontextprotocol/server-fetch'], env: {}, description: 'URL 内容抓取' },
];

function validateJson(str: string): { valid: boolean; error?: string } {
  if (!str.trim()) return { valid: true };
  try { JSON.parse(str); return { valid: true }; }
  catch (e) { return { valid: false, error: (e as Error).message }; }
}

export function McpPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<'mcp' | 'rules'>('mcp');
  const [servers, setServers] = useState<McpServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState<string | null>(null);
  const [addTab, setAddTab] = useState<'preset' | 'custom'>('preset');
  const [selectedPreset, setSelectedPreset] = useState<McpPreset | null>(null);
  const [addForm, setAddForm] = useState({ name: '', command: '', args: '', env: '', description: '' });
  const [envValidation, setEnvValidation] = useState<{ valid: boolean; error?: string }>({ valid: true });
  const [rules, setRules] = useState<RuleItem[]>([]);
  const [rulesLoading, setRulesLoading] = useState(true);
  const [showRuleDialog, setShowRuleDialog] = useState(false);
  const [ruleForm, setRuleForm] = useState({ name: '', content: '' });
  const [loadError, setLoadError] = useState<string | null>(null);
  const [expandedServer, setExpandedServer] = useState<string | null>(null);
  const [editingServer, setEditingServer] = useState<string | null>(null);
  const [editForm, setEditForm] = useState({ command: '', args: '', env: '', description: '' });
  const [editEnvValidation, setEditEnvValidation] = useState<{ valid: boolean; error?: string }>({ valid: true });
  const [searchQuery, setSearchQuery] = useState('');

  const filteredServers = useMemo(() => {
    if (!searchQuery.trim()) return servers;
    const q = searchQuery.toLowerCase();
    return servers.filter(s => s.name.toLowerCase().includes(q) || s.command.toLowerCase().includes(q) || (s.description || '').toLowerCase().includes(q));
  }, [servers, searchQuery]);

  const loadServers = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const s = await invoke<McpServer[]>('get_mcp_servers');
      setServers(Array.isArray(s) ? s : []);
    } catch (e) {
      setLoadError(String(e));
      setServers([]);
    } finally { setLoading(false); }
  }, []);

  const loadRules = useCallback(async () => {
    setRulesLoading(true);
    try {
      const r = await invoke<RuleItem[]>('get_installed_rules');
      setRules(Array.isArray(r) ? r : []);
    } catch { setRules([]); }
    finally { setRulesLoading(false); }
  }, []);

  useEffect(() => { loadServers(); loadRules(); }, [loadServers, loadRules]);

  const handleAddServer = async () => {
    if (addTab === 'preset' && selectedPreset) {
      try {
        await invoke('add_mcp_server', { input: { name: selectedPreset.name, command: selectedPreset.command, args: selectedPreset.args, env: selectedPreset.env, description: selectedPreset.description } });
        await loadServers();
        setShowAddDialog(false);
        setSelectedPreset(null);
        toast.success(t('mcp.addSuccess', 'MCP 服务器已添加'));
      } catch (e) { toast.error(String(e)); }
      return;
    }

    if (!addForm.name || !addForm.command) return;
    if (!envValidation.valid) return;
    try {
      const args = addForm.args ? addForm.args.split(/[,\n]/).map(a => a.trim()).filter(Boolean) : [];
      let env: Record<string, string> = {};
      if (addForm.env.trim()) { try { env = JSON.parse(addForm.env); } catch { env = {}; } }
      await invoke('add_mcp_server', { input: { name: addForm.name, command: addForm.command, args, env, description: addForm.description || undefined } });
      await loadServers();
      setShowAddDialog(false);
      setAddForm({ name: '', command: '', args: '', env: '', description: '' });
      toast.success(t('mcp.addSuccess', 'MCP 服务器已添加'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleDeleteServer = async () => {
    if (!showDeleteDialog) return;
    try {
      await invoke('delete_mcp_server', { name: showDeleteDialog });
      await loadServers();
      setShowDeleteDialog(null);
      toast.success(t('mcp.deleteSuccess', '已删除'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleToggleServer = async (name: string, enabled: boolean) => {
    await invoke('toggle_mcp_server', { name, enabled });
    await loadServers();
  };

  const handleSyncToApps = async () => {
    try {
      await invoke('sync_mcp_to_apps', {
        input: {
          server_names: [],
          targets: ['opencode', 'claudecode', 'codex', 'gemini', 'cursor'],
        },
      });
      await loadServers();
      toast.success(t('mcp.syncSuccess', '同步完成'));
    } catch (e) { toast.error(String(e)); }
  };

  const openEditServer = (server: McpServer) => {
    setEditForm({
      command: server.command,
      args: (server.args || []).join(', '),
      env: server.env && Object.keys(server.env).length > 0 ? JSON.stringify(server.env, null, 2) : '',
      description: server.description || '',
    });
    setEditEnvValidation({ valid: true });
    setEditingServer(server.name);
  };

  const handleSaveEdit = async () => {
    if (!editingServer || !editEnvValidation.valid) return;
    try {
      await invoke('delete_mcp_server', { name: editingServer });
      const args = editForm.args ? editForm.args.split(/[,\n]/).map(a => a.trim()).filter(Boolean) : [];
      let env: Record<string, string> = {};
      if (editForm.env.trim()) { try { env = JSON.parse(editForm.env); } catch { env = {}; } }
      await invoke('add_mcp_server', { input: { name: editingServer, command: editForm.command, args, env, description: editForm.description || undefined } });
      await loadServers();
      setEditingServer(null);
      toast.success(t('mcp.editSuccess', '修改已保存'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleInstallRule = async () => {
    if (!ruleForm.name || !ruleForm.content) return;
    try {
      await invoke('install_rule', { input: { name: ruleForm.name, content: ruleForm.content } });
      await loadRules();
      setShowRuleDialog(false);
      setRuleForm({ name: '', content: '' });
      toast.success(t('rules.addSuccess', 'Rule 已添加'));
    } catch (e) { toast.error(String(e)); }
  };

  const handleDeleteRule = async (name: string) => {
    await invoke('delete_rule', { name });
    await loadRules();
    toast.success(t('rules.deleted', 'Rule 已删除'));
  };

  const handleToggleRule = async (name: string, enabled: boolean) => {
    await invoke('toggle_rule_enabled', { name, enabled });
    await loadRules();
  };

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <ToastContainer toasts={toast.toasts} />

      {/* Page Header */}
      <div className="oc-page-header">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--mcp"><Plug size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('mcp.pageTitle', 'MCP & Rules')}</h2>
            <p className="oc-page-subtitle">{t('mcp.pageSubtitle', '管理 MCP 服务器和 Rules 规则')}</p>
          </div>
        </div>
      </div>

      {/* Tab Bar */}
      <div className="tabs tabs-boxed w-fit self-center">
        <button className={`tab ${activeTab === 'mcp' ? 'tab-active' : ''}`} onClick={() => setActiveTab('mcp')}>
          <Plug size={14} className="mr-1" /> MCP {t('mcp.servers', 'Servers')} ({servers.length})
        </button>
        <button className={`tab ${activeTab === 'rules' ? 'tab-active' : ''}`} onClick={() => setActiveTab('rules')}>
          <FileText size={14} className="mr-1" /> {t('mcp.rules', 'Rules')} ({rules.length})
        </button>
      </div>

      {loadError && (
        <div className="p-4 rounded-lg" style={{ background: 'rgba(239,68,68,0.1)', border: '1px solid rgba(239,68,68,0.3)' }}>
          <p style={{ fontSize: '13px', color: 'var(--danger)' }}>{loadError}</p>
          <button className="btn btn-sm btn-primary" onClick={() => { loadServers(); loadRules(); }} style={{ marginTop: '8px' }}>
            <RefreshCw size={14} /> {t('common.retry', '重试')}
          </button>
        </div>
      )}

      {activeTab === 'mcp' ? (
        <div className="flex-1 min-h-0 flex flex-col bg-base-200 overflow-hidden" style={{ borderRadius: 'var(--radius-lg)', border: '1px solid var(--border)' }}>
          <div className="p-4 border-b border-base-content/10 flex items-center justify-between gap-3">
            <div className="oc-search-wrap" style={{ flex: 1, maxWidth: '260px' }}>
              <Search size={14} className="oc-search-icon" />
              <input type="text" className="oc-search-input" placeholder={t('mcp.search', '搜索服务器...')} value={searchQuery} onChange={e => setSearchQuery(e.target.value)} />
            </div>
            <div className="flex gap-2">
              <button className="btn btn-sm btn-ghost" onClick={handleSyncToApps}><RefreshCw size={14} /> {t('mcp.sync', '同步')}</button>
              <button className="btn btn-sm btn-primary" onClick={() => { setShowAddDialog(true); setAddTab('preset'); setSelectedPreset(null); }}><Plus size={14} /> {t('mcp.add', '添加')}</button>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-3 oc-stagger">
            {loading ? (
              <div className="flex justify-center py-8"><Loader2 className="animate-spin" size={32} /></div>
            ) : filteredServers.length === 0 ? (
              <div className="oc-empty-state">
                <div className="oc-empty-state-icon"><Plug size={28} /></div>
                <div className="oc-empty-state-title">{t('mcp.empty', '暂无 MCP 服务器')}</div>
                <div className="oc-empty-state-desc">{t('mcp.emptyDesc', '添加 MCP 服务器以扩展 AI 的能力')}</div>
              </div>
            ) : filteredServers.map(s => (
              <div key={s.name} className="oc-mcp-card group">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3 min-w-0">
                    <span
                      className={`oc-toggle ${s.enabled ? 'is-on' : ''}`}
                      role="switch"
                      aria-checked={s.enabled}
                      onClick={() => handleToggleServer(s.name, !s.enabled)}
                    />
                    <div className="min-w-0">
                      <div className="font-medium text-sm">{s.name}</div>
                      <div className="text-xs font-mono opacity-50 truncate">{s.command} {(s.args || []).join(' ')}</div>
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    {s.env && Object.keys(s.env).length > 0 && (
                      <span className="oc-env-tag">{Object.keys(s.env).length} env</span>
                    )}
                    <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button className="btn btn-xs btn-ghost" onClick={() => setExpandedServer(expandedServer === s.name ? null : s.name)} title={t('mcp.details', '详情')}>
                        <ChevronDown size={12} className={`transition-transform ${expandedServer === s.name ? 'rotate-180' : ''}`} />
                      </button>
                      <button className="btn btn-xs btn-ghost" onClick={() => openEditServer(s)} title={t('mcp.edit', '编辑')}>
                        <Pencil size={12} />
                      </button>
                      <button className="btn btn-xs btn-ghost text-error" onClick={() => setShowDeleteDialog(s.name)} title={t('mcp.delete', '删除')}>
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                </div>
                {s.description && <div className="text-xs opacity-40 mt-1.5">{s.description}</div>}

                {expandedServer === s.name && (
                  <div className="oc-mcp-detail">
                    <div className="space-y-2">
                      <div><span className="font-medium opacity-60">Command:</span> <span className="font-mono">{s.command}</span></div>
                      {s.args && s.args.length > 0 && (
                        <div><span className="font-medium opacity-60">Args:</span> <span className="font-mono">{s.args.join(', ')}</span></div>
                      )}
                      {s.env && Object.keys(s.env).length > 0 && (
                        <div>
                          <span className="font-medium opacity-60">Env:</span>
                          <pre className="mt-1 p-2 rounded bg-base-300 font-mono text-xs overflow-auto" style={{ maxHeight: '120px' }}>
                            {JSON.stringify(s.env, null, 2)}
                          </pre>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="flex-1 min-h-0 flex flex-col bg-base-200 overflow-hidden" style={{ borderRadius: 'var(--radius-lg)', border: '1px solid var(--border)' }}>
          <div className="p-4 border-b border-base-content/10 flex items-center justify-between">
            <h3 className="font-semibold">{t('rules.title', 'Rules')}</h3>
            <button className="btn btn-sm btn-primary" onClick={() => setShowRuleDialog(true)}><Plus size={14} /> {t('rules.add', '添加 Rule')}</button>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-3 oc-stagger">
            {rulesLoading ? (
              <div className="flex justify-center py-8"><Loader2 className="animate-spin" size={32} /></div>
            ) : rules.length === 0 ? (
              <div className="oc-empty-state">
                <div className="oc-empty-state-icon"><FileText size={28} /></div>
                <div className="oc-empty-state-title">{t('rules.empty', '暂无 Rules')}</div>
                <div className="oc-empty-state-desc">{t('rules.emptyDesc', '添加 Rule 以自定义 AI 的行为模式')}</div>
              </div>
            ) : rules.map(r => (
              <div key={r.name} className="oc-mcp-card group">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span
                      className={`oc-toggle ${r.enabled ? 'is-on' : ''}`}
                      role="switch"
                      aria-checked={r.enabled}
                      onClick={() => handleToggleRule(r.name, !r.enabled)}
                    />
                    <div>
                      <span className="font-medium text-sm">{r.name}</span>
                      {r.path && <div className="text-xs font-mono opacity-40 mt-0.5 truncate">{r.path}</div>}
                    </div>
                  </div>
                  <button className="btn btn-xs btn-ghost text-error opacity-0 group-hover:opacity-100" onClick={() => handleDeleteRule(r.name)}><Trash2 size={12} /></button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Add MCP Dialog */}
      {showAddDialog && (
        <div className="oc-modal-overlay">
          <div className="modal-box" style={{ maxWidth: '580px' }}>
            <h3 className="font-bold text-lg">{t('mcp.addServer', '添加 MCP 服务器')}</h3>

            <div className="tabs tabs-boxed w-fit mt-4" style={{ marginBottom: '16px' }}>
              <button className={`tab ${addTab === 'preset' ? 'tab-active' : ''}`} onClick={() => setAddTab('preset')}>
                {t('mcp.fromPreset', '从预设')}
              </button>
              <button className={`tab ${addTab === 'custom' ? 'tab-active' : ''}`} onClick={() => setAddTab('custom')}>
                {t('mcp.custom', '自定义')}
              </button>
            </div>

            {addTab === 'preset' ? (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {MCP_PRESETS.map(preset => (
                  <div
                    key={preset.name}
                    className={`oc-preset-item ${selectedPreset?.name === preset.name ? 'is-selected' : ''}`}
                    onClick={() => setSelectedPreset(preset)}
                    style={{ gridColumn: 'span 2' }}
                  >
                    <Plug size={16} style={{ color: 'var(--primary)', flexShrink: 0 }} />
                    <div className="min-w-0 flex-1">
                      <div className="oc-preset-name">{preset.name}</div>
                      <div className="oc-preset-url">{preset.description}</div>
                    </div>
                    {selectedPreset?.name === preset.name && <CheckCircle size={16} style={{ color: 'var(--primary)', flexShrink: 0 }} />}
                  </div>
                ))}
              </div>
            ) : (
              <div className="space-y-3">
                <input type="text" className="input input-bordered w-full" placeholder={t('mcp.serverName', '服务器名称')} value={addForm.name} onChange={e => setAddForm(f => ({ ...f, name: e.target.value }))} />
                <input type="text" className="input input-bordered w-full font-mono" placeholder={t('mcp.command', 'Command (如 npx)')} value={addForm.command} onChange={e => setAddForm(f => ({ ...f, command: e.target.value }))} />
                <input type="text" className="input input-bordered w-full font-mono" placeholder={t('mcp.args', 'Args (逗号分隔)')} value={addForm.args} onChange={e => setAddForm(f => ({ ...f, args: e.target.value }))} />
                <input type="text" className="input input-bordered w-full" placeholder={t('mcp.description', '描述（可选）')} value={addForm.description} onChange={e => setAddForm(f => ({ ...f, description: e.target.value }))} />
                <div>
                  <textarea
                    className={`textarea textarea-bordered w-full font-mono ${envValidation.valid ? (addForm.env.trim() ? 'oc-json-valid' : '') : 'oc-json-invalid'}`}
                    placeholder={t('mcp.env', '环境变量 (JSON 格式)')}
                    value={addForm.env}
                    onChange={e => { setAddForm(f => ({ ...f, env: e.target.value })); setEnvValidation(validateJson(e.target.value)); }}
                    rows={3}
                  />
                  {!envValidation.valid && <div className="oc-json-hint is-invalid"><XCircle size={11} style={{ display: 'inline', verticalAlign: 'middle', marginRight: '4px' }} />{envValidation.error}</div>}
                  {envValidation.valid && addForm.env.trim() && <div className="oc-json-hint is-valid"><CheckCircle size={11} style={{ display: 'inline', verticalAlign: 'middle', marginRight: '4px' }} />JSON 格式正确</div>}
                </div>
              </div>
            )}

            <div className="modal-action">
              <button className="btn" onClick={() => { setShowAddDialog(false); setSelectedPreset(null); }}>{t('common.cancel', '取消')}</button>
              <button
                className="btn btn-primary"
                onClick={handleAddServer}
                disabled={addTab === 'preset' ? !selectedPreset : (!addForm.name || !addForm.command || !envValidation.valid)}
              >
                {t('common.save', '保存')}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit MCP Dialog */}
      {editingServer && (
        <div className="oc-modal-overlay">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('mcp.editServer', '编辑')}: {editingServer}</h3>
            <div className="space-y-3 mt-4">
              <div>
                <label className="text-xs font-medium opacity-60 mb-1 block">Command</label>
                <input type="text" className="input input-bordered w-full font-mono" value={editForm.command} onChange={e => setEditForm(f => ({ ...f, command: e.target.value }))} />
              </div>
              <div>
                <label className="text-xs font-medium opacity-60 mb-1 block">Args</label>
                <input type="text" className="input input-bordered w-full font-mono" value={editForm.args} onChange={e => setEditForm(f => ({ ...f, args: e.target.value }))} />
              </div>
              <div>
                <label className="text-xs font-medium opacity-60 mb-1 block">{t('mcp.description', '描述')}</label>
                <input type="text" className="input input-bordered w-full" value={editForm.description} onChange={e => setEditForm(f => ({ ...f, description: e.target.value }))} />
              </div>
              <div>
                <label className="text-xs font-medium opacity-60 mb-1 block">Env (JSON)</label>
                <textarea
                  className={`textarea textarea-bordered w-full font-mono ${editEnvValidation.valid ? (editForm.env.trim() ? 'oc-json-valid' : '') : 'oc-json-invalid'}`}
                  value={editForm.env}
                  onChange={e => { setEditForm(f => ({ ...f, env: e.target.value })); setEditEnvValidation(validateJson(e.target.value)); }}
                  rows={4}
                />
                {!editEnvValidation.valid && <div className="oc-json-hint is-invalid"><XCircle size={11} style={{ display: 'inline', verticalAlign: 'middle', marginRight: '4px' }} />{editEnvValidation.error}</div>}
              </div>
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setEditingServer(null)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary" onClick={handleSaveEdit} disabled={!editEnvValidation.valid}>{t('common.save', '保存')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirm */}
      {showDeleteDialog && (
        <div className="oc-modal-overlay">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('confirm.deleteTitle', '确认删除')}</h3>
            <p className="py-4">{t('confirm.deleteMcp', `确定要删除 "${showDeleteDialog}"?`)}</p>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowDeleteDialog(null)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-error" onClick={handleDeleteServer}>{t('common.delete', '删除')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Add Rule Dialog */}
      {showRuleDialog && (
        <div className="oc-modal-overlay">
          <div className="modal-box" style={{ maxWidth: '600px' }}>
            <h3 className="font-bold text-lg">{t('rules.addRule', '添加 Rule')}</h3>
            <div className="space-y-3 mt-4">
              <input type="text" className="input input-bordered w-full" placeholder={t('rules.ruleName', 'Rule 名称')} value={ruleForm.name} onChange={e => setRuleForm(f => ({ ...f, name: e.target.value }))} />
              <textarea
                className="textarea textarea-bordered w-full font-mono"
                placeholder={t('rules.ruleContent', 'Rule 内容（支持 Markdown）')}
                value={ruleForm.content}
                onChange={e => setRuleForm(f => ({ ...f, content: e.target.value }))}
                rows={12}
                style={{ minHeight: '200px' }}
              />
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowRuleDialog(false)}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary" onClick={handleInstallRule} disabled={!ruleForm.name || !ruleForm.content}>{t('common.save', '保存')}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

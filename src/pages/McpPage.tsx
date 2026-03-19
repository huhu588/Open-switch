import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, RefreshCw, Loader2, Plug, ToggleLeft, ToggleRight, FileText } from 'lucide-react';

interface McpServer { name: string; command: string; args: string[]; env: Record<string, string>; enabled: boolean; description?: string; }
interface RuleItem { name: string; content?: string; enabled: boolean; path?: string; }

export function McpPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<'mcp' | 'rules'>('mcp');
  const [servers, setServers] = useState<McpServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState<string | null>(null);
  const [addForm, setAddForm] = useState({ name: '', command: '', args: '', env: '' });
  const [rules, setRules] = useState<RuleItem[]>([]);
  const [rulesLoading, setRulesLoading] = useState(true);
  const [showRuleDialog, setShowRuleDialog] = useState(false);
  const [ruleForm, setRuleForm] = useState({ name: '', content: '' });

  const loadServers = useCallback(async () => {
    setLoading(true);
    try {
      const s = await invoke<McpServer[]>('get_mcp_servers');
      setServers(s);
    } catch (e) { console.error('Load MCP servers failed:', e); }
    finally { setLoading(false); }
  }, []);

  const loadRules = useCallback(async () => {
    setRulesLoading(true);
    try {
      setRules(await invoke<RuleItem[]>('get_installed_rules'));
    } catch (e) { console.error('Load rules failed:', e); }
    finally { setRulesLoading(false); }
  }, []);

  useEffect(() => { loadServers(); loadRules(); }, [loadServers, loadRules]);

  const handleAddServer = async () => {
    if (!addForm.name || !addForm.command) return;
    try {
      const args = addForm.args ? addForm.args.split(/[,\n]/).map(a => a.trim()).filter(Boolean) : [];
      let env: Record<string, string> = {};
      if (addForm.env) { try { env = JSON.parse(addForm.env); } catch { env = {}; } }
      await invoke('add_mcp_server', { input: { name: addForm.name, command: addForm.command, args, env } });
      await loadServers();
      setShowAddDialog(false);
      setAddForm({ name: '', command: '', args: '', env: '' });
    } catch (e) { console.error('Add server failed:', e); }
  };

  const handleDeleteServer = async () => {
    if (!showDeleteDialog) return;
    await invoke('delete_mcp_server', { name: showDeleteDialog });
    await loadServers();
    setShowDeleteDialog(null);
  };

  const handleToggleServer = async (name: string, enabled: boolean) => {
    await invoke('toggle_mcp_server', { name, enabled });
    await loadServers();
  };

  const handleSyncToApps = async () => {
    try { await invoke('sync_mcp_to_apps'); await loadServers(); } catch (e) { console.error('Sync failed:', e); }
  };

  const handleInstallRule = async () => {
    if (!ruleForm.name || !ruleForm.content) return;
    try {
      await invoke('install_rule', { input: { name: ruleForm.name, content: ruleForm.content } });
      await loadRules();
      setShowRuleDialog(false);
      setRuleForm({ name: '', content: '' });
    } catch (e) { console.error('Install rule failed:', e); }
  };

  const handleDeleteRule = async (name: string) => {
    await invoke('delete_rule', { name });
    await loadRules();
  };

  const handleToggleRule = async (name: string, enabled: boolean) => {
    await invoke('toggle_rule_enabled', { name, enabled });
    await loadRules();
  };

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      {/* Tab Bar */}
      <div className="tabs tabs-boxed w-fit self-center">
        <button className={`tab ${activeTab === 'mcp' ? 'tab-active' : ''}`} onClick={() => setActiveTab('mcp')}>
          <Plug size={14} className="mr-1" /> MCP {t('mcp.servers', 'Servers')}
        </button>
        <button className={`tab ${activeTab === 'rules' ? 'tab-active' : ''}`} onClick={() => setActiveTab('rules')}>
          <FileText size={14} className="mr-1" /> {t('mcp.rules', 'Rules')}
        </button>
      </div>

      {activeTab === 'mcp' ? (
        <div className="flex-1 min-h-0 flex flex-col bg-base-200 rounded-lg overflow-hidden">
          <div className="p-4 border-b border-base-content/10 flex items-center justify-between">
            <h3 className="font-semibold">{t('mcp.title', 'MCP Servers')}</h3>
            <div className="flex gap-2">
              <button className="btn btn-sm btn-ghost" onClick={handleSyncToApps}><RefreshCw size={14} /> {t('mcp.sync', 'Sync')}</button>
              <button className="btn btn-sm btn-primary" onClick={() => setShowAddDialog(true)}><Plus size={14} /> {t('mcp.add', 'Add')}</button>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-3">
            {loading ? (
              <div className="flex justify-center py-8"><Loader2 className="animate-spin" size={32} /></div>
            ) : servers.length === 0 ? (
              <div className="text-center py-8 opacity-50">{t('mcp.empty', 'No MCP servers configured')}</div>
            ) : servers.map(s => (
              <div key={s.name} className="p-4 rounded-lg bg-base-300 group">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3 min-w-0">
                    <button onClick={() => handleToggleServer(s.name, !s.enabled)}>
                      {s.enabled ? <ToggleRight size={20} className="text-success" /> : <ToggleLeft size={20} className="opacity-40" />}
                    </button>
                    <div className="min-w-0">
                      <div className="font-medium text-sm">{s.name}</div>
                      <div className="text-xs font-mono opacity-50 truncate">{s.command} {s.args.join(' ')}</div>
                      {s.description && <div className="text-xs opacity-40 mt-0.5">{s.description}</div>}
                    </div>
                  </div>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="btn btn-xs btn-ghost text-error" onClick={() => setShowDeleteDialog(s.name)}><Trash2 size={12} /></button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="flex-1 min-h-0 flex flex-col bg-base-200 rounded-lg overflow-hidden">
          <div className="p-4 border-b border-base-content/10 flex items-center justify-between">
            <h3 className="font-semibold">{t('rules.title', 'Rules')}</h3>
            <button className="btn btn-sm btn-primary" onClick={() => setShowRuleDialog(true)}><Plus size={14} /> {t('rules.add', 'Add Rule')}</button>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-3">
            {rulesLoading ? (
              <div className="flex justify-center py-8"><Loader2 className="animate-spin" size={32} /></div>
            ) : rules.length === 0 ? (
              <div className="text-center py-8 opacity-50">{t('rules.empty', 'No rules installed')}</div>
            ) : rules.map(r => (
              <div key={r.name} className="p-4 rounded-lg bg-base-300 group">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <button onClick={() => handleToggleRule(r.name, !r.enabled)}>
                      {r.enabled ? <ToggleRight size={20} className="text-success" /> : <ToggleLeft size={20} className="opacity-40" />}
                    </button>
                    <span className="font-medium text-sm">{r.name}</span>
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
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('mcp.addServer', 'Add MCP Server')}</h3>
            <div className="space-y-3 mt-4">
              <input type="text" className="input input-bordered w-full" placeholder={t('mcp.serverName', 'Server Name')} value={addForm.name} onChange={e => setAddForm(f => ({ ...f, name: e.target.value }))} />
              <input type="text" className="input input-bordered w-full font-mono" placeholder={t('mcp.command', 'Command (e.g. npx)')} value={addForm.command} onChange={e => setAddForm(f => ({ ...f, command: e.target.value }))} />
              <input type="text" className="input input-bordered w-full font-mono" placeholder={t('mcp.args', 'Args (comma separated)')} value={addForm.args} onChange={e => setAddForm(f => ({ ...f, args: e.target.value }))} />
              <textarea className="textarea textarea-bordered w-full font-mono" placeholder={t('mcp.env', 'Environment (JSON)')} value={addForm.env} onChange={e => setAddForm(f => ({ ...f, env: e.target.value }))} rows={3} />
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowAddDialog(false)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-primary" onClick={handleAddServer} disabled={!addForm.name || !addForm.command}>{t('common.save', 'Save')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirm */}
      {showDeleteDialog && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('confirm.deleteTitle', 'Confirm Delete')}</h3>
            <p className="py-4">{t('confirm.deleteMcp', `Delete server "${showDeleteDialog}"?`)}</p>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowDeleteDialog(null)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-error" onClick={handleDeleteServer}>{t('common.delete', 'Delete')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Add Rule Dialog */}
      {showRuleDialog && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('rules.addRule', 'Add Rule')}</h3>
            <div className="space-y-3 mt-4">
              <input type="text" className="input input-bordered w-full" placeholder={t('rules.ruleName', 'Rule Name')} value={ruleForm.name} onChange={e => setRuleForm(f => ({ ...f, name: e.target.value }))} />
              <textarea className="textarea textarea-bordered w-full font-mono" placeholder={t('rules.ruleContent', 'Rule Content')} value={ruleForm.content} onChange={e => setRuleForm(f => ({ ...f, content: e.target.value }))} rows={8} />
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowRuleDialog(false)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-primary" onClick={handleInstallRule} disabled={!ruleForm.name || !ruleForm.content}>{t('common.save', 'Save')}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

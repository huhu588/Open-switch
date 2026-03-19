import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { FolderArchive, Download, Upload, Loader2, CheckCircle, FileText, MessageSquare } from 'lucide-react';

interface BackupPreview {
  providers: string[];
  models: Record<string, string[]>;
  mcp_servers: string[];
  skills: string[];
  rules: string[];
  settings: boolean;
}

interface ChatSource { platform: string; path: string; conversation_count: number; }
interface ConversationItem { id: string; title: string; platform: string; message_count: number; created_at: string; }

export function BackupPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<'backup' | 'migration'>('backup');
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [preview, setPreview] = useState<BackupPreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [messageType, setMessageType] = useState<'success' | 'error'>('success');

  const [chatSources, setChatSources] = useState<ChatSource[]>([]);
  const [conversations, setConversations] = useState<ConversationItem[]>([]);
  const [scanLoading, setScanLoading] = useState(false);
  const [extracting, setExtracting] = useState(false);
  const [selectedPlatform, setSelectedPlatform] = useState('');
  const [selectedConversations, setSelectedConversations] = useState<Set<string>>(new Set());

  const showMsg = useCallback((msg: string, type: 'success' | 'error') => {
    setMessage(msg);
    setMessageType(type);
    setTimeout(() => setMessage(''), 3000);
  }, []);

  const handleExport = async () => {
    setExporting(true);
    try {
      const path = await invoke<string>('export_backup');
      showMsg(t('backup.exportSuccess', `Backup exported to: ${path}`), 'success');
    } catch (e) { showMsg(t('backup.exportFailed', 'Export failed') + ': ' + String(e), 'error'); }
    finally { setExporting(false); }
  };

  const handlePreview = async () => {
    setPreviewLoading(true);
    try {
      setPreview(await invoke<BackupPreview>('preview_backup'));
    } catch (e) { showMsg(t('backup.previewFailed', 'Preview failed') + ': ' + String(e), 'error'); }
    finally { setPreviewLoading(false); }
  };

  const handleImport = async () => {
    setImporting(true);
    try {
      await invoke('import_backup');
      showMsg(t('backup.importSuccess', 'Backup imported successfully'), 'success');
    } catch (e) { showMsg(t('backup.importFailed', 'Import failed') + ': ' + String(e), 'error'); }
    finally { setImporting(false); }
  };

  const handleScanChatSources = async () => {
    setScanLoading(true);
    try {
      const sources = await invoke<ChatSource[]>('scan_chat_sources');
      setChatSources(sources);
      if (sources.length > 0) setSelectedPlatform(sources[0].platform);
    } catch (e) { showMsg('Scan failed: ' + String(e), 'error'); }
    finally { setScanLoading(false); }
  };

  const handleExtractConversations = async () => {
    if (!selectedPlatform) return;
    setExtracting(true);
    try {
      const convs = await invoke<ConversationItem[]>('extract_conversations', { platform: selectedPlatform });
      setConversations(convs);
      setSelectedConversations(new Set(convs.map(c => c.id)));
    } catch (e) { showMsg('Extract failed: ' + String(e), 'error'); }
    finally { setExtracting(false); }
  };

  const handleExportConversations = async () => {
    const ids = Array.from(selectedConversations);
    if (ids.length === 0) return;
    try {
      const path = await invoke<string>('export_conversations', { conversationIds: ids, platform: selectedPlatform });
      showMsg(t('backup.migrationExportSuccess', `Exported to: ${path}`), 'success');
    } catch (e) { showMsg('Export failed: ' + String(e), 'error'); }
  };

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-3">
          <FolderArchive size={24} className="text-primary" />
          <h2 className="text-lg font-bold">{t('backup.title', 'Backup & Recovery')}</h2>
        </div>
        {message && <div className={`text-sm px-4 py-2 rounded-lg ${messageType === 'success' ? 'bg-success/20 text-success' : 'bg-error/20 text-error'}`}>{message}</div>}
      </div>

      <div className="tabs tabs-boxed w-fit self-center">
        <button className={`tab ${activeTab === 'backup' ? 'tab-active' : ''}`} onClick={() => setActiveTab('backup')}>
          <FolderArchive size={14} className="mr-1" /> {t('backup.config', 'Config Backup')}
        </button>
        <button className={`tab ${activeTab === 'migration' ? 'tab-active' : ''}`} onClick={() => setActiveTab('migration')}>
          <MessageSquare size={14} className="mr-1" /> {t('backup.chatMigration', 'Chat Migration')}
        </button>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {activeTab === 'backup' ? (
          <div className="space-y-6">
            <div className="card bg-base-200 p-6">
              <h3 className="font-semibold mb-4">{t('backup.exportTitle', 'Export Configuration')}</h3>
              <p className="text-sm opacity-60 mb-4">{t('backup.exportDesc', 'Export all providers, models, MCP servers, skills, and rules to a backup file.')}</p>
              <div className="flex gap-3">
                <button className="btn btn-primary" onClick={handleExport} disabled={exporting}>
                  {exporting ? <Loader2 className="animate-spin" size={16} /> : <Download size={16} />}
                  {t('backup.export', 'Export Backup')}
                </button>
                <button className="btn btn-ghost" onClick={handlePreview} disabled={previewLoading}>
                  {previewLoading ? <Loader2 className="animate-spin" size={16} /> : <FileText size={16} />}
                  {t('backup.preview', 'Preview')}
                </button>
              </div>

              {preview && (
                <div className="mt-4 p-4 bg-base-300 rounded-lg space-y-2 text-sm">
                  <div className="flex justify-between"><span className="opacity-60">Providers</span><span className="font-medium">{preview.providers.length}</span></div>
                  <div className="flex justify-between"><span className="opacity-60">MCP Servers</span><span className="font-medium">{preview.mcp_servers.length}</span></div>
                  <div className="flex justify-between"><span className="opacity-60">Skills</span><span className="font-medium">{preview.skills.length}</span></div>
                  <div className="flex justify-between"><span className="opacity-60">Rules</span><span className="font-medium">{preview.rules.length}</span></div>
                  <div className="flex justify-between"><span className="opacity-60">Settings</span><span className="font-medium">{preview.settings ? <CheckCircle size={14} className="text-success" /> : '-'}</span></div>
                </div>
              )}
            </div>

            <div className="card bg-base-200 p-6">
              <h3 className="font-semibold mb-4">{t('backup.importTitle', 'Import Configuration')}</h3>
              <p className="text-sm opacity-60 mb-4">{t('backup.importDesc', 'Restore configuration from a backup file.')}</p>
              <button className="btn btn-secondary" onClick={handleImport} disabled={importing}>
                {importing ? <Loader2 className="animate-spin" size={16} /> : <Upload size={16} />}
                {t('backup.import', 'Import Backup')}
              </button>
            </div>
          </div>
        ) : (
          <div className="space-y-6">
            <div className="card bg-base-200 p-6">
              <h3 className="font-semibold mb-4">{t('backup.scanSources', 'Scan Chat Sources')}</h3>
              <p className="text-sm opacity-60 mb-4">{t('backup.scanDesc', 'Scan for conversation data from supported platforms (Cursor, Claude Code, Codex, Windsurf, Trae).')}</p>
              <button className="btn btn-primary" onClick={handleScanChatSources} disabled={scanLoading}>
                {scanLoading ? <Loader2 className="animate-spin" size={16} /> : <MessageSquare size={16} />}
                {t('backup.scan', 'Scan')}
              </button>

              {chatSources.length > 0 && (
                <div className="mt-4 space-y-2">
                  {chatSources.map(source => (
                    <div key={source.platform} className={`p-3 rounded-lg cursor-pointer transition-all ${selectedPlatform === source.platform ? 'bg-primary/10 border border-primary/30' : 'bg-base-300'}`} onClick={() => setSelectedPlatform(source.platform)}>
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-sm">{source.platform}</span>
                        <span className="text-xs opacity-60">{source.conversation_count} conversations</span>
                      </div>
                      <div className="text-xs font-mono opacity-40 truncate">{source.path}</div>
                    </div>
                  ))}

                  <button className="btn btn-sm btn-secondary mt-2" onClick={handleExtractConversations} disabled={extracting || !selectedPlatform}>
                    {extracting ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
                    {t('backup.extract', 'Extract Conversations')}
                  </button>
                </div>
              )}
            </div>

            {conversations.length > 0 && (
              <div className="card bg-base-200 p-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="font-semibold">{t('backup.conversations', 'Conversations')} ({conversations.length})</h3>
                  <div className="flex gap-2">
                    <button className="btn btn-xs btn-ghost" onClick={() => setSelectedConversations(new Set(conversations.map(c => c.id)))}>Select All</button>
                    <button className="btn btn-xs btn-ghost" onClick={() => setSelectedConversations(new Set())}>Deselect</button>
                  </div>
                </div>
                <div className="max-h-64 overflow-y-auto space-y-1">
                  {conversations.map(conv => (
                    <label key={conv.id} className="flex items-center gap-2 p-2 rounded hover:bg-base-300 cursor-pointer">
                      <input type="checkbox" className="checkbox checkbox-sm" checked={selectedConversations.has(conv.id)} onChange={() => setSelectedConversations(prev => { const next = new Set(prev); next.has(conv.id) ? next.delete(conv.id) : next.add(conv.id); return next; })} />
                      <div className="flex-1 min-w-0">
                        <div className="text-sm truncate">{conv.title}</div>
                        <div className="text-xs opacity-50">{conv.message_count} messages · {conv.created_at}</div>
                      </div>
                    </label>
                  ))}
                </div>
                <button className="btn btn-primary mt-4" onClick={handleExportConversations} disabled={selectedConversations.size === 0}>
                  <Download size={14} /> {t('backup.exportSelected', 'Export Selected')} ({selectedConversations.size})
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

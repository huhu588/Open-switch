import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  FolderArchive, Download, Upload, Loader2, CheckCircle, FileText,
  MessageSquare, HardDrive, RefreshCw, Layers, Shield,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

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
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<'backup' | 'migration'>('backup');
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [preview, setPreview] = useState<BackupPreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);

  const [chatSources, setChatSources] = useState<ChatSource[]>([]);
  const [conversations, setConversations] = useState<ConversationItem[]>([]);
  const [scanLoading, setScanLoading] = useState(false);
  const [extracting, setExtracting] = useState(false);
  const [selectedPlatform, setSelectedPlatform] = useState('');
  const [selectedConversations, setSelectedConversations] = useState<Set<string>>(new Set());

  const handleExport = async () => {
    setExporting(true);
    try {
      const path = await invoke<string>('export_backup');
      toast.success(t('backup.exportSuccess', `备份已导出到: ${path}`));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setExporting(false);
    }
  };

  const handlePreview = async () => {
    setPreviewLoading(true);
    try {
      setPreview(await invoke<BackupPreview>('preview_backup'));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setPreviewLoading(false);
    }
  };

  const handleImport = async () => {
    setImporting(true);
    try {
      await invoke('import_backup');
      toast.success(t('backup.importSuccess', '备份已恢复成功'));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setImporting(false);
    }
  };

  const handleScanChatSources = async () => {
    setScanLoading(true);
    try {
      const sources = await invoke<ChatSource[]>('scan_chat_sources');
      setChatSources(sources);
      if (sources.length > 0) setSelectedPlatform(sources[0].platform);
      toast.info(t('backup.scanComplete', `扫描到 ${sources.length} 个来源`));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setScanLoading(false);
    }
  };

  const handleExtractConversations = async () => {
    if (!selectedPlatform) return;
    setExtracting(true);
    try {
      const convs = await invoke<ConversationItem[]>('extract_conversations', { platform: selectedPlatform });
      setConversations(convs);
      setSelectedConversations(new Set(convs.map(c => c.id)));
      toast.success(t('backup.extractSuccess', `提取到 ${convs.length} 个对话`));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setExtracting(false);
    }
  };

  const handleExportConversations = async () => {
    const ids = Array.from(selectedConversations);
    if (ids.length === 0) return;
    try {
      const path = await invoke<string>('export_conversations', { conversationIds: ids, platform: selectedPlatform });
      toast.success(t('backup.migrationExportSuccess', `已导出到: ${path}`));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const previewItems = preview ? [
    { label: 'Providers', count: preview.providers.length, icon: Layers },
    { label: 'MCP Servers', count: preview.mcp_servers.length, icon: HardDrive },
    { label: 'Skills', count: preview.skills.length, icon: Shield },
    { label: 'Rules', count: preview.rules.length, icon: FileText },
  ] : [];

  return (
    <div className="page-container" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <ToastContainer toasts={toast.toasts} />

      {/* Header */}
      <div className="gw-page-header">
        <div className="gw-page-header-left">
          <h1 className="gw-page-title">{t('backup.title', '备份与恢复')}</h1>
          <div className="gw-page-subtitle">
            {t('backup.subtitle', '导出配置、恢复备份、迁移对话记录')}
          </div>
        </div>
      </div>

      {/* Tab Bar */}
      <div className="gw-tab-bar" style={{ alignSelf: 'center' }}>
        <button className={`gw-tab ${activeTab === 'backup' ? 'is-active' : ''}`} onClick={() => setActiveTab('backup')}>
          <FolderArchive size={13} /> {t('backup.config', '配置备份')}
        </button>
        <button className={`gw-tab ${activeTab === 'migration' ? 'is-active' : ''}`} onClick={() => setActiveTab('migration')}>
          <MessageSquare size={13} /> {t('backup.chatMigration', '对话迁移')}
        </button>
      </div>

      {/* Content */}
      <div style={{ flex: 1, minHeight: 0, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 16 }}>
        {activeTab === 'backup' ? (
          <>
            {/* Export Section */}
            <div className="gw-section">
              <div className="gw-section-title">
                <Download size={16} />
                {t('backup.exportTitle', '导出配置')}
              </div>
              <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: 16 }}>
                {t('backup.exportDesc', '将所有 Provider、模型、MCP 服务器、Skills 和规则导出为备份文件。')}
              </p>
              <div style={{ display: 'flex', gap: 10 }}>
                <button className="btn btn-primary btn-sm" onClick={handleExport} disabled={exporting}>
                  {exporting ? <Loader2 size={14} className="gw-spin" /> : <Download size={14} />}
                  {t('backup.export', '导出备份')}
                </button>
                <button className="btn btn-ghost btn-sm" onClick={handlePreview} disabled={previewLoading}>
                  {previewLoading ? <Loader2 size={14} className="gw-spin" /> : <FileText size={14} />}
                  {t('backup.preview', '预览')}
                </button>
              </div>

              {preview && (
                <div style={{ marginTop: 16 }}>
                  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))', gap: 10 }}>
                    {previewItems.map(item => (
                      <div key={item.label} className="gw-summary-item">
                        <span className="gw-summary-label" style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                          <item.icon size={10} /> {item.label}
                        </span>
                        <span className="gw-summary-value">{item.count}</span>
                      </div>
                    ))}
                    <div className="gw-summary-item">
                      <span className="gw-summary-label">Settings</span>
                      <span className="gw-summary-value">
                        {preview.settings ? <CheckCircle size={16} style={{ color: 'var(--success)' }} /> : '-'}
                      </span>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Import Section */}
            <div className="gw-section">
              <div className="gw-section-title">
                <Upload size={16} />
                {t('backup.importTitle', '导入配置')}
              </div>
              <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: 16 }}>
                {t('backup.importDesc', '从备份文件恢复配置。')}
              </p>
              <button className="btn btn-sm" style={{ background: 'var(--gradient-primary)', color: 'white', border: 'none' }} onClick={handleImport} disabled={importing}>
                {importing ? <Loader2 size={14} className="gw-spin" /> : <Upload size={14} />}
                {t('backup.import', '导入备份')}
              </button>
            </div>
          </>
        ) : (
          <>
            {/* Scan Sources */}
            <div className="gw-section">
              <div className="gw-section-title">
                <MessageSquare size={16} />
                {t('backup.scanSources', '扫描对话来源')}
              </div>
              <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: 16 }}>
                {t('backup.scanDesc', '扫描 Cursor、Claude Code、Codex、Windsurf、Trae 等平台的对话数据。')}
              </p>
              <button className="btn btn-primary btn-sm" onClick={handleScanChatSources} disabled={scanLoading}>
                {scanLoading ? <Loader2 size={14} className="gw-spin" /> : <RefreshCw size={14} />}
                {t('backup.scan', '扫描')}
              </button>

              {chatSources.length > 0 && (
                <div style={{ marginTop: 16, display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {chatSources.map(source => (
                    <div
                      key={source.platform}
                      className="gw-account-card"
                      style={{
                        cursor: 'pointer',
                        borderColor: selectedPlatform === source.platform ? 'var(--primary)' : undefined,
                        background: selectedPlatform === source.platform ? 'var(--primary-light)' : undefined,
                      }}
                      onClick={() => setSelectedPlatform(source.platform)}
                    >
                      <div className="gw-account-avatar" style={{ background: 'var(--gradient-primary)', width: 32, height: 32, fontSize: '0.7rem' }}>
                        {source.platform.charAt(0).toUpperCase()}
                      </div>
                      <div className="gw-account-info">
                        <div className="gw-account-email">{source.platform}</div>
                        <div className="gw-account-meta">
                          <span className="gw-account-meta-item">{source.conversation_count} {t('backup.conversations', '对话')}</span>
                        </div>
                      </div>
                    </div>
                  ))}

                  <button className="btn btn-sm" style={{ background: 'var(--gradient-primary)', color: 'white', border: 'none', alignSelf: 'flex-start', marginTop: 4 }} onClick={handleExtractConversations} disabled={extracting || !selectedPlatform}>
                    {extracting ? <Loader2 size={14} className="gw-spin" /> : <Download size={14} />}
                    {t('backup.extract', '提取对话')}
                  </button>
                </div>
              )}
            </div>

            {/* Conversations List */}
            {conversations.length > 0 && (
              <div className="gw-section">
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 14 }}>
                  <div className="gw-section-title" style={{ marginBottom: 0 }}>
                    <FileText size={16} />
                    {t('backup.conversations', '对话')} ({conversations.length})
                  </div>
                  <div style={{ display: 'flex', gap: 6 }}>
                    <button className="btn btn-ghost btn-xs" onClick={() => setSelectedConversations(new Set(conversations.map(c => c.id)))}>
                      {t('backup.selectAll', '全选')}
                    </button>
                    <button className="btn btn-ghost btn-xs" onClick={() => setSelectedConversations(new Set())}>
                      {t('backup.deselectAll', '取消全选')}
                    </button>
                  </div>
                </div>
                <div style={{ maxHeight: 256, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 4 }}>
                  {conversations.map(conv => (
                    <label
                      key={conv.id}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 10,
                        padding: '8px 12px',
                        borderRadius: 'var(--radius-sm)',
                        cursor: 'pointer',
                        transition: 'background 0.15s',
                        background: selectedConversations.has(conv.id) ? 'var(--primary-light)' : 'transparent',
                      }}
                    >
                      <input
                        type="checkbox"
                        className="checkbox checkbox-sm"
                        checked={selectedConversations.has(conv.id)}
                        onChange={() => setSelectedConversations(prev => {
                          const next = new Set(prev);
                          next.has(conv.id) ? next.delete(conv.id) : next.add(conv.id);
                          return next;
                        })}
                      />
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div style={{ fontSize: '0.8rem', fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{conv.title}</div>
                        <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>{conv.message_count} messages · {conv.created_at}</div>
                      </div>
                    </label>
                  ))}
                </div>
                <button
                  className="btn btn-primary btn-sm"
                  style={{ marginTop: 12 }}
                  onClick={handleExportConversations}
                  disabled={selectedConversations.size === 0}
                >
                  <Download size={14} />
                  {t('backup.exportSelected', '导出选中')} ({selectedConversations.size})
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

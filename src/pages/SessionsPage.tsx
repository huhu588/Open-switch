import { useState, useEffect, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Search,
  RefreshCw,
  MessageSquare,
  Clock,
  FolderOpen,
  ChevronRight,
  Loader2,
  FileText,
  Copy,
  X,
  User,
  Bot,
  Wrench,
  Trash2,
  AlertTriangle,
} from 'lucide-react';
import { useSessionStore, SessionInfo, SessionMessage } from '../stores/useSessionStore';

const PLATFORM_COLORS: Record<string, string> = {
  'claude-code': '#D97757',
  codex: '#10A37F',
  gemini: '#3186FF',
  opencode: '#8B5CF6',
  openclaw: '#EC4899',
  antigravity: '#F97316',
  cursor: '#0EA5E9',
  windsurf: '#06B6D4',
  kiro: '#10B981',
  'github-copilot': '#6E40C9',
  codebuddy: '#F59E0B',
  codebuddy_cn: '#EAB308',
  qoder: '#7C3AED',
  trae: '#3B82F6',
  workbuddy: '#14B8A6',
  warp: '#01A4FF',
  augment: '#E11D48',
};

const PLATFORM_LABELS: Record<string, string> = {
  'claude-code': 'Claude Code',
  codex: 'Codex',
  gemini: 'Gemini',
  opencode: 'OpenCode',
  openclaw: 'OpenClaw',
  antigravity: 'Antigravity',
  cursor: 'Cursor',
  windsurf: 'Windsurf',
  kiro: 'Kiro',
  'github-copilot': 'GitHub Copilot',
  codebuddy: 'CodeBuddy',
  codebuddy_cn: 'CodeBuddy CN',
  qoder: 'Qoder',
  trae: 'Trae',
  workbuddy: 'WorkBuddy',
  warp: 'Warp',
  augment: 'Augment',
};

function formatRelativeTime(timestamp: number | null | undefined, t: (key: string, fallback: string) => string): string {
  if (!timestamp) return t('sessions.unknownTime', '未知时间');

  let s = timestamp;
  if (s > 1_000_000_000_000) s = Math.floor(s / 1000);

  const now = Math.floor(Date.now() / 1000);
  const diff = now - s;

  if (diff < 60) return t('sessions.justNow', '刚刚');
  if (diff < 3600) return `${Math.floor(diff / 60)} ${t('sessions.minutesAgo', '分钟前')}`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} ${t('sessions.hoursAgo', '小时前')}`;
  if (diff < 604800) return `${Math.floor(diff / 86400)} ${t('sessions.daysAgo', '天前')}`;

  return new Date(s * 1000).toLocaleDateString();
}

function formatTimestamp(ts: number | null | undefined): string {
  if (!ts) return '';
  const ms = ts > 1_000_000_000_000 ? ts : ts * 1000;
  return new Date(ms).toLocaleString();
}

function getRoleIcon(role: string) {
  const r = role.toLowerCase();
  if (r === 'user' || r === 'human') return <User size={14} />;
  if (r === 'assistant') return <Bot size={14} />;
  if (r === 'tool') return <Wrench size={14} />;
  return <MessageSquare size={14} />;
}

function getRoleLabel(role: string): string {
  const r = role.toLowerCase();
  if (r === 'user' || r === 'human') return 'User';
  if (r === 'assistant') return 'Assistant';
  if (r === 'tool') return 'Tool';
  return role;
}

function getRoleColor(role: string): string {
  const r = role.toLowerCase();
  if (r === 'user' || r === 'human') return 'var(--color-primary, #3b82f6)';
  if (r === 'assistant') return '#22c55e';
  if (r === 'tool') return '#f59e0b';
  return 'var(--text-secondary)';
}

function MessageItem({ message, onCopy }: { message: SessionMessage; onCopy: (text: string) => void }) {
  const isUser = ['user', 'human'].includes(message.role.toLowerCase());

  return (
    <div
      style={{
        padding: '14px 16px',
        borderRadius: 10,
        border: '1px solid var(--border-color)',
        background: isUser ? 'var(--card-bg-alt, rgba(59, 130, 246, 0.04))' : 'var(--card-bg)',
        position: 'relative',
        transition: 'border-color 0.15s',
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLElement).style.borderColor = getRoleColor(message.role);
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLElement).style.borderColor = 'var(--border-color)';
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ color: getRoleColor(message.role), display: 'flex', alignItems: 'center' }}>
            {getRoleIcon(message.role)}
          </span>
          <span style={{ fontSize: 12, fontWeight: 600, color: getRoleColor(message.role) }}>
            {getRoleLabel(message.role)}
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {message.timestamp && (
            <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
              {formatTimestamp(message.timestamp)}
            </span>
          )}
          <button
            onClick={() => onCopy(message.content)}
            style={{
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              width: 24, height: 24, borderRadius: 4,
              border: 'none', background: 'transparent',
              cursor: 'pointer', color: 'var(--text-tertiary)', opacity: 0.6,
              transition: 'opacity 0.15s',
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.opacity = '1'; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.opacity = '0.6'; }}
            title="复制内容"
          >
            <Copy size={13} />
          </button>
        </div>
      </div>
      <div
        style={{
          fontSize: 13, lineHeight: 1.7, color: 'var(--text-primary)',
          whiteSpace: 'pre-wrap', wordBreak: 'break-word',
          maxHeight: 400, overflow: 'auto',
        }}
      >
        {message.content.length > 3000 ? message.content.substring(0, 3000) + '...' : message.content}
      </div>
    </div>
  );
}

export function SessionsPage() {
  const { t } = useTranslation();
  const store = useSessionStore();
  const [localSearch, setLocalSearch] = useState('');
  const [isSearchFocused, setIsSearchFocused] = useState(false);
  const [selectedSession, setSelectedSession] = useState<SessionInfo | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    store.loadSessions();
  }, [store.platformFilter]);

  useEffect(() => {
    if (selectedSession) {
      store.loadMessages(selectedSession.platform, selectedSession.file_path);
    } else {
      store.clearMessages();
    }
  }, [selectedSession]);

  const handleSearch = () => {
    if (localSearch.trim()) {
      store.searchSessions(localSearch.trim(), store.platformFilter);
    } else {
      store.loadSessions();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleSearch();
    if (e.key === 'Escape') {
      setLocalSearch('');
      store.loadSessions();
    }
  };

  const handleSelectSession = (session: SessionInfo) => {
    setSelectedSession(session);
  };

  const handleCopy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // ignore
    }
  };

  const handleDeleteSession = async () => {
    if (!selectedSession) return;
    setDeleting(true);
    const ok = await store.deleteSession(
      selectedSession.platform,
      selectedSession.id,
      selectedSession.file_path,
    );
    setDeleting(false);
    setShowDeleteConfirm(false);
    if (ok) {
      setSelectedSession(null);
    }
  };

  const platformFilters = useMemo(
    () => [
      { id: null, label: t('sessions.allPlatforms', '全部') },
      { id: 'claude-code', label: 'Claude Code' },
      { id: 'codex', label: 'Codex' },
      { id: 'gemini', label: 'Gemini' },
      { id: 'opencode', label: 'OpenCode' },
      { id: 'openclaw', label: 'OpenClaw' },
      { id: 'cursor', label: 'Cursor' },
      { id: 'windsurf', label: 'Windsurf' },
      { id: 'kiro', label: 'Kiro' },
      { id: 'antigravity', label: 'Antigravity' },
      { id: 'github-copilot', label: 'GitHub Copilot' },
      { id: 'codebuddy', label: 'CodeBuddy' },
      { id: 'codebuddy_cn', label: 'CodeBuddy CN' },
      { id: 'qoder', label: 'Qoder' },
      { id: 'trae', label: 'Trae' },
      { id: 'workbuddy', label: 'WorkBuddy' },
      { id: 'warp', label: 'Warp' },
      { id: 'augment', label: 'Augment' },
    ],
    [t],
  );

  const selectedKey = selectedSession
    ? `${selectedSession.platform}:${selectedSession.id}`
    : null;

  return (
    <div className="page-container" style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* 页面头部 */}
      <div className="page-header" style={{ flexShrink: 0, padding: '20px 24px 8px' }}>
        <div className="page-title">
          <MessageSquare size={22} style={{ marginRight: 8, verticalAlign: 'middle' }} />
          {t('sessions.title', '会话管理')}
        </div>
        <div className="page-subtitle">
          {t('sessions.subtitle', '浏览和管理各 CLI 工具的对话历史记录。')}
        </div>
      </div>

      {/* 搜索和筛选栏 */}
      <div style={{ padding: '4px 24px 12px', display: 'flex', gap: 10, alignItems: 'center', flexWrap: 'wrap', flexShrink: 0 }}>
        <div style={{ display: 'flex', gap: 8, flex: 1, minWidth: 200, maxWidth: 400 }}>
          <div
            style={{
              position: 'relative', flex: 1, borderRadius: 8,
              border: `1px solid ${isSearchFocused ? 'var(--color-primary, #3b82f6)' : 'var(--border-color)'}`,
              background: 'var(--input-bg)', transition: 'border-color 0.15s',
              display: 'flex', alignItems: 'center',
            }}
          >
            <Search
              size={15}
              style={{
                position: 'absolute', left: 10,
                color: isSearchFocused ? 'var(--color-primary, #3b82f6)' : 'var(--text-tertiary)',
                transition: 'color 0.15s',
              }}
            />
            <input
              type="text"
              placeholder={t('sessions.searchPlaceholder', '搜索会话...')}
              value={localSearch}
              onChange={(e) => setLocalSearch(e.target.value)}
              onKeyDown={handleKeyDown}
              onFocus={() => setIsSearchFocused(true)}
              onBlur={() => setIsSearchFocused(false)}
              style={{
                width: '100%', paddingLeft: 32,
                paddingRight: localSearch ? 28 : 8,
                height: 34, border: 'none', background: 'transparent',
                color: 'var(--text-primary)', fontSize: 13, outline: 'none',
              }}
            />
            {localSearch && (
              <button
                onClick={() => { setLocalSearch(''); store.loadSessions(); }}
                style={{
                  position: 'absolute', right: 6,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  width: 20, height: 20, borderRadius: 4,
                  border: 'none', background: 'var(--bg-secondary, rgba(0,0,0,0.06))',
                  cursor: 'pointer', color: 'var(--text-tertiary)', padding: 0,
                }}
              >
                <X size={12} />
              </button>
            )}
          </div>
          <button
            className="btn btn-secondary icon-only"
            onClick={() => store.loadSessions()}
            title={t('common.refresh', '刷新')}
            style={{ height: 34, width: 34, flexShrink: 0 }}
          >
            <RefreshCw size={15} />
          </button>
        </div>

        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', maxHeight: 68, overflowY: 'auto' }}>
          {platformFilters.map((pf) => {
            const isActive = store.platformFilter === pf.id;
            return (
              <button
                key={pf.id ?? 'all'}
                className="btn btn-secondary"
                style={{
                  fontSize: 12, padding: '4px 10px', borderRadius: 6,
                  height: 28, lineHeight: '20px', transition: 'all 0.15s',
                  ...(isActive ? {
                    background: PLATFORM_COLORS[pf.id ?? ''] || 'var(--color-primary)',
                    color: '#fff', borderColor: 'transparent',
                  } : {}),
                }}
                onClick={() => store.setPlatformFilter(pf.id)}
              >
                {pf.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* 主内容区域 - 左右分栏 */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden', padding: '0 24px 16px', gap: 16, minHeight: 0 }}>
        {/* 左侧会话列表 */}
        <div
          style={{
            width: 360, minWidth: 300, flexShrink: 0,
            display: 'flex', flexDirection: 'column',
            borderRadius: 10, border: '1px solid var(--border-color)',
            background: 'var(--card-bg)', overflow: 'hidden',
          }}
        >
          <div
            style={{
              padding: '10px 14px', borderBottom: '1px solid var(--border-color)',
              fontSize: 12, color: 'var(--text-tertiary)', fontWeight: 500,
              display: 'flex', justifyContent: 'space-between', alignItems: 'center',
            }}
          >
            <span>{t('sessions.sessionList', '会话列表')}</span>
            <span
              style={{
                background: 'var(--bg-secondary, rgba(0,0,0,0.06))',
                padding: '1px 8px', borderRadius: 10, fontSize: 11, fontWeight: 600,
              }}
            >
              {store.sessions.length}
            </span>
          </div>

          <div style={{ flex: 1, overflow: 'auto' }}>
            {store.loading ? (
              <div style={{ textAlign: 'center', padding: '48px 0' }}>
                <Loader2 size={28} className="spin" style={{ color: 'var(--text-tertiary)' }} />
                <p style={{ color: 'var(--text-secondary)', marginTop: 12, fontSize: 13 }}>
                  {t('sessions.loading', '加载中...')}
                </p>
              </div>
            ) : store.sessions.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '48px 16px' }}>
                <FileText size={36} style={{ color: 'var(--text-tertiary)', marginBottom: 12 }} />
                <p style={{ color: 'var(--text-secondary)', fontSize: 13, margin: 0 }}>
                  {t('sessions.noSessions', '暂无会话记录')}
                </p>
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column' }}>
                {store.sessions.map((session) => {
                  const key = `${session.platform}:${session.id}`;
                  const isSelected = selectedKey === key;
                  const platformColor = PLATFORM_COLORS[session.platform] || '#6b7280';

                  return (
                    <button
                      key={`${session.platform}-${session.id}-${session.file_path}`}
                      onClick={() => handleSelectSession(session)}
                      style={{
                        display: 'flex', alignItems: 'flex-start', gap: 10,
                        padding: '10px 14px', border: 'none',
                        borderBottom: '1px solid var(--border-color)',
                        background: isSelected ? `${platformColor}12` : 'transparent',
                        cursor: 'pointer', textAlign: 'left', width: '100%',
                        transition: 'background 0.12s',
                        borderLeft: isSelected ? `3px solid ${platformColor}` : '3px solid transparent',
                      }}
                      onMouseEnter={(e) => {
                        if (!isSelected) (e.currentTarget as HTMLElement).style.background = 'var(--hover-bg, rgba(0,0,0,0.03))';
                      }}
                      onMouseLeave={(e) => {
                        if (!isSelected) (e.currentTarget as HTMLElement).style.background = 'transparent';
                      }}
                    >
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
                          <span
                            style={{
                              fontSize: 10, padding: '1px 6px', borderRadius: 3,
                              fontWeight: 600, color: '#fff', background: platformColor, flexShrink: 0,
                            }}
                          >
                            {PLATFORM_LABELS[session.platform] || session.platform}
                          </span>
                        </div>
                        <div
                          style={{
                            fontSize: 13, fontWeight: 500, color: 'var(--text-primary)',
                            overflow: 'hidden', textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap', marginBottom: 4,
                          }}
                        >
                          {session.title || session.id}
                        </div>
                        {session.summary && session.summary !== session.title && (
                          <div
                            style={{
                              fontSize: 12, color: 'var(--text-tertiary)',
                              overflow: 'hidden', textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap', marginBottom: 4,
                            }}
                          >
                            {session.summary}
                          </div>
                        )}
                        <div style={{ display: 'flex', alignItems: 'center', gap: 10, fontSize: 11, color: 'var(--text-tertiary)' }}>
                          <span style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
                            <Clock size={11} />
                            {formatRelativeTime(session.updated_at, t)}
                          </span>
                          {session.working_directory && (
                            <span
                              style={{
                                display: 'flex', alignItems: 'center', gap: 3,
                                overflow: 'hidden', textOverflow: 'ellipsis',
                                whiteSpace: 'nowrap', maxWidth: 160,
                              }}
                              title={session.working_directory}
                            >
                              <FolderOpen size={11} style={{ flexShrink: 0 }} />
                              {session.working_directory.split(/[/\\]/).pop() || session.working_directory}
                            </span>
                          )}
                        </div>
                      </div>
                      <ChevronRight
                        size={14}
                        style={{
                          color: isSelected ? platformColor : 'var(--text-tertiary)',
                          flexShrink: 0, marginTop: 4, opacity: 0.6,
                        }}
                      />
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </div>

        {/* 右侧会话详情 */}
        <div
          style={{
            flex: 1, display: 'flex', flexDirection: 'column',
            borderRadius: 10, border: '1px solid var(--border-color)',
            background: 'var(--card-bg)', overflow: 'hidden', minWidth: 0,
            position: 'relative',
          }}
        >
          {!selectedSession ? (
            <div
              style={{
                flex: 1, display: 'flex', alignItems: 'center',
                justifyContent: 'center', flexDirection: 'column', gap: 12,
              }}
            >
              <MessageSquare size={40} style={{ color: 'var(--text-tertiary)', opacity: 0.4 }} />
              <p style={{ color: 'var(--text-tertiary)', fontSize: 14, margin: 0 }}>
                {t('sessions.selectSession', '选择一个会话查看详情')}
              </p>
            </div>
          ) : (
            <>
              {/* 删除确认弹窗 */}
              {showDeleteConfirm && (
                <div
                  style={{
                    position: 'absolute', inset: 0, zIndex: 50,
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    background: 'rgba(0, 0, 0, 0.4)', backdropFilter: 'blur(2px)',
                  }}
                  onClick={() => !deleting && setShowDeleteConfirm(false)}
                >
                  <div
                    style={{
                      background: 'var(--card-bg, #fff)', borderRadius: 12,
                      padding: '24px', maxWidth: 380, width: '90%',
                      boxShadow: '0 8px 32px rgba(0,0,0,0.18)',
                      border: '1px solid var(--border-color)',
                    }}
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 14 }}>
                      <div
                        style={{
                          width: 36, height: 36, borderRadius: 8,
                          background: 'rgba(239, 68, 68, 0.1)',
                          display: 'flex', alignItems: 'center', justifyContent: 'center',
                          flexShrink: 0,
                        }}
                      >
                        <AlertTriangle size={20} style={{ color: 'var(--color-error, #ef4444)' }} />
                      </div>
                      <div>
                        <div style={{ fontSize: 15, fontWeight: 600, color: 'var(--text-primary)' }}>
                          {t('sessions.confirmDeleteTitle', '确认删除会话')}
                        </div>
                        <div style={{ fontSize: 12, color: 'var(--text-tertiary)', marginTop: 2 }}>
                          {t('sessions.confirmDeleteDesc', '此操作不可撤销，会话文件将被永久删除。')}
                        </div>
                      </div>
                    </div>

                    <div
                      style={{
                        padding: '10px 12px', borderRadius: 8,
                        background: 'var(--bg-secondary, rgba(0,0,0,0.04))',
                        fontSize: 12, color: 'var(--text-secondary)',
                        marginBottom: 18, lineHeight: 1.5,
                      }}
                    >
                      <div style={{ fontWeight: 500, marginBottom: 2 }}>
                        {selectedSession.title || selectedSession.id}
                      </div>
                      <div style={{ color: 'var(--text-tertiary)', fontSize: 11 }}>
                        {PLATFORM_LABELS[selectedSession.platform] || selectedSession.platform}
                        {selectedSession.working_directory && ` · ${selectedSession.working_directory}`}
                      </div>
                    </div>

                    <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end' }}>
                      <button
                        onClick={() => setShowDeleteConfirm(false)}
                        disabled={deleting}
                        className="btn btn-secondary"
                        style={{ fontSize: 13, padding: '6px 16px', borderRadius: 6 }}
                      >
                        {t('common.cancel', '取消')}
                      </button>
                      <button
                        onClick={handleDeleteSession}
                        disabled={deleting}
                        style={{
                          display: 'flex', alignItems: 'center', gap: 5,
                          fontSize: 13, padding: '6px 16px', borderRadius: 6,
                          border: 'none', background: 'var(--color-error, #ef4444)',
                          color: '#fff', cursor: deleting ? 'wait' : 'pointer',
                          fontWeight: 500, opacity: deleting ? 0.7 : 1,
                        }}
                      >
                        {deleting ? (
                          <Loader2 size={14} className="spin" />
                        ) : (
                          <Trash2 size={13} />
                        )}
                        {deleting
                          ? t('sessions.deleting', '删除中...')
                          : t('sessions.confirmDelete', '确认删除')}
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {/* 详情头部 */}
              <div style={{ padding: '14px 18px', borderBottom: '1px solid var(--border-color)', flexShrink: 0 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 6 }}>
                  <span
                    style={{
                      display: 'inline-block', padding: '2px 8px', borderRadius: 4,
                      fontSize: 11, fontWeight: 600, color: '#fff',
                      background: PLATFORM_COLORS[selectedSession.platform] || '#6b7280',
                    }}
                  >
                    {PLATFORM_LABELS[selectedSession.platform] || selectedSession.platform}
                  </span>
                  <span style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-primary)', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {selectedSession.title || selectedSession.id}
                  </span>
                  <button
                    onClick={() => setShowDeleteConfirm(true)}
                    style={{
                      display: 'flex', alignItems: 'center', gap: 5,
                      padding: '5px 12px', borderRadius: 6,
                      border: '1px solid var(--color-error, #ef4444)',
                      background: 'var(--color-error, #ef4444)',
                      color: '#fff', cursor: 'pointer', fontSize: 12,
                      fontWeight: 500, flexShrink: 0,
                      transition: 'opacity 0.15s',
                    }}
                    onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.opacity = '0.85'; }}
                    onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.opacity = '1'; }}
                  >
                    <Trash2 size={13} />
                    {t('sessions.deleteSession', '删除会话')}
                  </button>
                </div>
                <div
                  style={{
                    display: 'flex', alignItems: 'center', gap: 14,
                    fontSize: 12, color: 'var(--text-tertiary)', flexWrap: 'wrap',
                  }}
                >
                  <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                    <MessageSquare size={12} />
                    {store.messages.length} {t('sessions.messages', '条消息')}
                  </span>
                  <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                    <Clock size={12} />
                    {formatTimestamp(selectedSession.updated_at)}
                  </span>
                  {selectedSession.working_directory && (
                    <button
                      onClick={() => handleCopy(selectedSession.working_directory || '')}
                      style={{
                        display: 'flex', alignItems: 'center', gap: 4,
                        border: 'none', background: 'none', cursor: 'pointer',
                        color: 'var(--text-tertiary)', fontSize: 12, padding: 0,
                      }}
                      title={selectedSession.working_directory}
                    >
                      <FolderOpen size={12} />
                      {selectedSession.working_directory}
                    </button>
                  )}
                  {selectedSession.resume_command && (
                    <button
                      onClick={() => handleCopy(selectedSession.resume_command || '')}
                      style={{
                        display: 'flex', alignItems: 'center', gap: 4,
                        border: '1px solid var(--border-color)', borderRadius: 4,
                        background: 'var(--bg-secondary, rgba(0,0,0,0.04))',
                        cursor: 'pointer', color: 'var(--text-secondary)',
                        fontSize: 11, padding: '2px 8px', fontFamily: 'monospace',
                      }}
                      title={t('sessions.copyResumeCommand', '复制恢复命令')}
                    >
                      <Copy size={11} />
                      {selectedSession.resume_command}
                    </button>
                  )}
                </div>
              </div>

              {/* 消息列表 */}
              <div style={{ flex: 1, overflow: 'auto', padding: '14px 18px' }}>
                {store.messagesLoading ? (
                  <div style={{ textAlign: 'center', padding: '48px 0' }}>
                    <Loader2 size={28} className="spin" style={{ color: 'var(--text-tertiary)' }} />
                    <p style={{ color: 'var(--text-secondary)', marginTop: 12, fontSize: 13 }}>
                      {t('sessions.loadingMessages', '加载消息中...')}
                    </p>
                  </div>
                ) : store.messagesError ? (
                  <div style={{ textAlign: 'center', padding: '48px 0', color: 'var(--text-tertiary)' }}>
                    <FileText size={36} style={{ marginBottom: 12, opacity: 0.5 }} />
                    <p style={{ fontSize: 14, margin: 0, color: 'var(--text-secondary)' }}>
                      {t('sessions.parseError', '解析会话消息失败')}
                    </p>
                    <p style={{ fontSize: 12, marginTop: 8, color: 'var(--color-error, #ef4444)', maxWidth: 400, margin: '8px auto 0', wordBreak: 'break-word' }}>
                      {store.messagesError}
                    </p>
                  </div>
                ) : store.messages.length === 0 ? (
                  <div style={{ textAlign: 'center', padding: '48px 0', color: 'var(--text-tertiary)' }}>
                    <FileText size={36} style={{ marginBottom: 12, opacity: 0.5 }} />
                    <p style={{ fontSize: 14, margin: 0 }}>
                      {t('sessions.noMessages', '此会话没有消息记录')}
                    </p>
                  </div>
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                    {store.messages.map((msg, idx) => (
                      <MessageItem key={idx} message={msg} onCopy={handleCopy} />
                    ))}
                    <div ref={messagesEndRef} />
                  </div>
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

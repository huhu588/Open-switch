import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Trash2, Download, Loader2, Sparkles, RefreshCw, Eye, Search, Link, FolderOpen } from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface SkillItem { name: string; description?: string; path?: string; enabled?: boolean; }
interface RecommendedSkill { name: string; description: string; repo_url?: string; }

export function SkillsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [installedSkills, setInstalledSkills] = useState<SkillItem[]>([]);
  const [recommendedSkills, setRecommendedSkills] = useState<RecommendedSkill[]>([]);
  const [loading, setLoading] = useState(true);
  const [installing, setInstalling] = useState<Record<string, boolean>>({});
  const [activeTab, setActiveTab] = useState<'installed' | 'recommended'>('installed');
  const [viewContent, setViewContent] = useState<{ name: string; content: string } | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showUrlDialog, setShowUrlDialog] = useState(false);
  const [urlInput, setUrlInput] = useState('');
  const [urlInstalling, setUrlInstalling] = useState(false);

  const filteredInstalled = useMemo(() => {
    if (!searchQuery.trim()) return installedSkills;
    const q = searchQuery.toLowerCase();
    return installedSkills.filter(s => s.name.toLowerCase().includes(q) || (s.description || '').toLowerCase().includes(q));
  }, [installedSkills, searchQuery]);

  const filteredRecommended = useMemo(() => {
    if (!searchQuery.trim()) return recommendedSkills;
    const q = searchQuery.toLowerCase();
    return recommendedSkills.filter(s => s.name.toLowerCase().includes(q) || s.description.toLowerCase().includes(q));
  }, [recommendedSkills, searchQuery]);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [rawInstalled, recommended] = await Promise.all([
        invoke<SkillItem[]>('get_installed_skills'),
        invoke<RecommendedSkill[]>('get_recommended_skills').catch(() => []),
      ]);
      const seen = new Set<string>();
      const installed = rawInstalled.filter(s => { if (seen.has(s.name)) return false; seen.add(s.name); return true; });
      setInstalledSkills(installed);
      setRecommendedSkills(recommended);
    } catch (e) {
      toast.error(t('skills.loadFailed', '加载失败: ') + String(e));
    } finally { setLoading(false); }
  }, []);

  useEffect(() => { loadData(); }, [loadData]);

  const handleInstall = async (skillNames: string[]) => {
    const key = skillNames.join(',');
    setInstalling(prev => ({ ...prev, [key]: true }));
    try {
      await invoke('install_skills', { names: skillNames });
      await loadData();
      toast.success(t('skills.installSuccess', '安装成功'));
    } catch (e) {
      toast.error(t('skills.installFailed', '安装失败: ') + String(e));
    } finally { setInstalling(prev => ({ ...prev, [key]: false })); }
  };

  const handleDelete = async (name: string) => {
    try {
      await invoke('delete_skills', { names: [name] });
      await loadData();
      toast.success(t('skills.deleteSuccess', '已删除'));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleViewContent = async (name: string) => {
    try {
      const content = await invoke<string>('read_skills_content', { name });
      setViewContent({ name, content });
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleUrlInstall = async () => {
    if (!urlInput.trim()) return;
    setUrlInstalling(true);
    try {
      await invoke('install_skills', { names: [urlInput.trim()] });
      await loadData();
      setShowUrlDialog(false);
      setUrlInput('');
      toast.success(t('skills.installSuccess', '安装成功'));
    } catch (e) {
      toast.error(t('skills.installFailed', '安装失败: ') + String(e));
    } finally { setUrlInstalling(false); }
  };

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <ToastContainer toasts={toast.toasts} />

      {/* Page Header */}
      <div className="oc-page-header">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--skills"><Sparkles size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('skills.pageTitle', 'Skills 管理')}</h2>
            <p className="oc-page-subtitle">{t('skills.pageSubtitle', '安装和管理 AI 技能扩展')}</p>
          </div>
        </div>
        <div className="oc-page-header-actions">
          <button className="btn btn-sm btn-ghost" onClick={() => setShowUrlDialog(true)}>
            <Link size={14} /> {t('skills.fromUrl', '从 URL 安装')}
          </button>
          <button className="btn btn-sm btn-ghost" onClick={loadData} disabled={loading}>
            <RefreshCw size={14} />
          </button>
        </div>
      </div>

      {/* Search + Tabs */}
      <div className="flex items-center justify-between gap-4 flex-shrink-0">
        <div className="oc-search-wrap" style={{ maxWidth: '280px', flex: 1 }}>
          <Search size={14} className="oc-search-icon" />
          <input
            type="text"
            className="oc-search-input"
            placeholder={t('skills.search', '搜索 Skills...')}
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
          />
        </div>
        <div className="tabs tabs-boxed w-fit">
          <button className={`tab ${activeTab === 'installed' ? 'tab-active' : ''}`} onClick={() => setActiveTab('installed')}>
            {t('skills.installed', '已安装')} ({installedSkills.length})
          </button>
          <button className={`tab ${activeTab === 'recommended' ? 'tab-active' : ''}`} onClick={() => setActiveTab('recommended')}>
            {t('skills.recommended', '推荐')} ({recommendedSkills.length})
          </button>
        </div>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {loading ? (
          <div className="flex justify-center py-12"><Loader2 className="animate-spin" size={32} /></div>
        ) : activeTab === 'installed' ? (
          <div className="space-y-3 oc-stagger">
            {filteredInstalled.length === 0 ? (
              <div className="oc-empty-state">
                <div className="oc-empty-state-icon"><Sparkles size={28} /></div>
                <div className="oc-empty-state-title">{searchQuery ? t('skills.noSearchResult', '未找到匹配的 Skill') : t('skills.noInstalled', '暂无已安装的 Skill')}</div>
                <div className="oc-empty-state-desc">{searchQuery ? t('skills.tryOther', '试试其他关键词') : t('skills.noInstalledDesc', '从推荐列表安装或通过 URL 添加')}</div>
              </div>
            ) : filteredInstalled.map(skill => (
              <div key={skill.name} className="oc-mcp-card group">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl shrink-0" style={{ background: 'linear-gradient(135deg, rgba(249, 115, 22, 0.15), rgba(245, 158, 11, 0.15))', color: '#f59e0b' }}>
                      <Sparkles size={20} />
                    </div>
                    <div className="min-w-0">
                      <div className="font-medium text-sm">{skill.name}</div>
                      {skill.description && <div className="text-xs opacity-50 truncate mt-0.5">{skill.description}</div>}
                      {skill.path && (
                        <div className="flex items-center gap-1 text-xs opacity-30 mt-0.5 font-mono truncate">
                          <FolderOpen size={10} /> {skill.path}
                        </div>
                      )}
                    </div>
                  </div>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="btn btn-xs btn-ghost" onClick={() => handleViewContent(skill.name)} title={t('skills.viewContent', '查看内容')}>
                      <Eye size={12} />
                    </button>
                    <button className="btn btn-xs btn-ghost text-error" onClick={() => handleDelete(skill.name)} title={t('skills.delete', '删除')}>
                      <Trash2 size={12} />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-3 oc-stagger">
            {filteredRecommended.length === 0 ? (
              <div className="oc-empty-state">
                <div className="oc-empty-state-icon"><Download size={28} /></div>
                <div className="oc-empty-state-title">{searchQuery ? t('skills.noSearchResult', '未找到匹配的 Skill') : t('skills.noRecommended', '暂无推荐')}</div>
              </div>
            ) : filteredRecommended.map(skill => {
              const isInstalled = installedSkills.some(s => s.name === skill.name);
              return (
                <div key={skill.name} className="oc-mcp-card">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <div className="flex h-10 w-10 items-center justify-center rounded-xl shrink-0" style={{ background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.15), rgba(168, 85, 247, 0.15))', color: '#8b5cf6' }}>
                        <Download size={20} />
                      </div>
                      <div className="min-w-0">
                        <div className="font-medium text-sm">{skill.name}</div>
                        <div className="text-xs opacity-50 truncate mt-0.5">{skill.description}</div>
                      </div>
                    </div>
                    <button
                      className={`btn btn-sm ${isInstalled ? 'btn-success btn-disabled' : 'btn-primary'}`}
                      disabled={isInstalled || installing[skill.name]}
                      onClick={() => handleInstall([skill.name])}
                    >
                      {installing[skill.name] ? <Loader2 className="animate-spin" size={14} /> : isInstalled ? t('skills.installed', '已安装') : t('skills.install', '安装')}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* View Content Modal */}
      {viewContent && (
        <div className="oc-modal-overlay">
          <div className="modal-box" style={{ maxWidth: '700px', maxHeight: '85vh' }}>
            <h3 className="font-bold text-lg">{viewContent.name}</h3>
            <pre className="mt-4 bg-base-300 rounded-lg p-4 text-xs font-mono overflow-auto whitespace-pre-wrap" style={{ maxHeight: '60vh', border: '1px solid var(--border-light)' }}>
              {viewContent.content}
            </pre>
            <div className="modal-action">
              <button className="btn" onClick={() => setViewContent(null)}>{t('common.close', '关闭')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Install from URL Dialog */}
      {showUrlDialog && (
        <div className="oc-modal-overlay">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('skills.installFromUrl', '从 URL 安装 Skill')}</h3>
            <p className="text-xs opacity-50 mt-1">{t('skills.urlDesc', '输入 GitHub 仓库 URL 或 Skill 名称')}</p>
            <div className="mt-4">
              <input
                type="text"
                className="input input-bordered w-full font-mono"
                placeholder="https://github.com/user/skill-repo 或 skill-name"
                value={urlInput}
                onChange={e => setUrlInput(e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter') handleUrlInstall(); }}
              />
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => { setShowUrlDialog(false); setUrlInput(''); }}>{t('common.cancel', '取消')}</button>
              <button className="btn btn-primary" onClick={handleUrlInstall} disabled={!urlInput.trim() || urlInstalling}>
                {urlInstalling ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
                {urlInstalling ? t('skills.installing', '安装中...') : t('skills.install', '安装')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

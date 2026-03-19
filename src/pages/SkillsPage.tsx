import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Trash2, Download, Loader2, Sparkles, RefreshCw, Eye } from 'lucide-react';

interface SkillItem { name: string; description?: string; path?: string; enabled?: boolean; }
interface RecommendedSkill { name: string; description: string; repo_url?: string; }

export function SkillsPage() {
  const { t } = useTranslation();
  const [installedSkills, setInstalledSkills] = useState<SkillItem[]>([]);
  const [recommendedSkills, setRecommendedSkills] = useState<RecommendedSkill[]>([]);
  const [loading, setLoading] = useState(true);
  const [installing, setInstalling] = useState<Record<string, boolean>>({});
  const [activeTab, setActiveTab] = useState<'installed' | 'recommended'>('installed');
  const [viewContent, setViewContent] = useState<{ name: string; content: string } | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [installed, recommended] = await Promise.all([
        invoke<SkillItem[]>('get_installed_skills'),
        invoke<RecommendedSkill[]>('get_recommended_skills').catch(() => []),
      ]);
      setInstalledSkills(installed);
      setRecommendedSkills(recommended);
    } catch (e) { console.error('Load skills failed:', e); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { loadData(); }, [loadData]);

  const handleInstall = async (skillNames: string[]) => {
    const key = skillNames.join(',');
    setInstalling(prev => ({ ...prev, [key]: true }));
    try {
      await invoke('install_skills', { names: skillNames });
      await loadData();
    } catch (e) { console.error('Install failed:', e); }
    finally { setInstalling(prev => ({ ...prev, [key]: false })); }
  };

  const handleDelete = async (name: string) => {
    try {
      await invoke('delete_skills', { names: [name] });
      await loadData();
    } catch (e) { console.error('Delete failed:', e); }
  };


  const handleViewContent = async (name: string) => {
    try {
      const content = await invoke<string>('read_skills_content', { name });
      setViewContent({ name, content });
    } catch (e) { console.error('Read content failed:', e); }
  };

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-3">
          <Sparkles size={24} className="text-primary" />
          <h2 className="text-lg font-bold">{t('skills.title', 'Skills Management')}</h2>
        </div>
        <button className="btn btn-sm btn-ghost" onClick={loadData} disabled={loading}><RefreshCw size={14} /></button>
      </div>

      <div className="tabs tabs-boxed w-fit self-center">
        <button className={`tab ${activeTab === 'installed' ? 'tab-active' : ''}`} onClick={() => setActiveTab('installed')}>
          {t('skills.installed', 'Installed')} ({installedSkills.length})
        </button>
        <button className={`tab ${activeTab === 'recommended' ? 'tab-active' : ''}`} onClick={() => setActiveTab('recommended')}>
          {t('skills.recommended', 'Recommended')} ({recommendedSkills.length})
        </button>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {loading ? (
          <div className="flex justify-center py-12"><Loader2 className="animate-spin" size={32} /></div>
        ) : activeTab === 'installed' ? (
          <div className="space-y-3">
            {installedSkills.length === 0 ? (
              <div className="text-center py-12 opacity-50">{t('skills.noInstalled', 'No skills installed')}</div>
            ) : installedSkills.map(skill => (
              <div key={skill.name} className="card bg-base-200 p-4 group">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 text-primary shrink-0"><Sparkles size={18} /></div>
                    <div className="min-w-0">
                      <div className="font-medium text-sm">{skill.name}</div>
                      {skill.description && <div className="text-xs opacity-50 truncate">{skill.description}</div>}
                    </div>
                  </div>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="btn btn-xs btn-ghost" onClick={() => handleViewContent(skill.name)}><Eye size={12} /></button>
                    <button className="btn btn-xs btn-ghost text-error" onClick={() => handleDelete(skill.name)}><Trash2 size={12} /></button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-3">
            {recommendedSkills.length === 0 ? (
              <div className="text-center py-12 opacity-50">{t('skills.noRecommended', 'No recommended skills')}</div>
            ) : recommendedSkills.map(skill => {
              const isInstalled = installedSkills.some(s => s.name === skill.name);
              return (
                <div key={skill.name} className="card bg-base-200 p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3 min-w-0">
                      <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-secondary/10 text-secondary shrink-0"><Download size={18} /></div>
                      <div className="min-w-0">
                        <div className="font-medium text-sm">{skill.name}</div>
                        <div className="text-xs opacity-50 truncate">{skill.description}</div>
                      </div>
                    </div>
                    <button className={`btn btn-sm ${isInstalled ? 'btn-success btn-disabled' : 'btn-primary'}`} disabled={isInstalled || installing[skill.name]} onClick={() => handleInstall([skill.name])}>
                      {installing[skill.name] ? <Loader2 className="animate-spin" size={14} /> : isInstalled ? t('skills.installed', 'Installed') : t('skills.install', 'Install')}
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
        <div className="modal modal-open">
          <div className="modal-box max-w-2xl max-h-[80vh]">
            <h3 className="font-bold text-lg">{viewContent.name}</h3>
            <pre className="mt-4 bg-base-300 rounded-lg p-4 text-xs font-mono overflow-auto max-h-96 whitespace-pre-wrap">{viewContent.content}</pre>
            <div className="modal-action">
              <button className="btn" onClick={() => setViewContent(null)}>{t('common.close', 'Close')}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

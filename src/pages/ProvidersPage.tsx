import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, Zap, Loader2, Globe, ToggleLeft, ToggleRight, Upload } from 'lucide-react';
import { useProviderStore } from '../stores/useProviderStore';

type ModelType = 'claude' | 'codex' | 'gemini';
const MODEL_TYPES: { id: ModelType; name: string; color: string }[] = [
  { id: 'claude', name: 'Claude', color: '#D97757' },
  { id: 'codex', name: 'Codex', color: '#10A37F' },
  { id: 'gemini', name: 'Gemini', color: '#3186FF' },
];

export function ProvidersPage() {
  const { t } = useTranslation();
  const store = useProviderStore();
  const [selectedModelType, setSelectedModelType] = useState<ModelType>('claude');
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState<string | null>(null);
  const [addForm, setAddForm] = useState({ name: '', api_key: '', base_url: '', description: '' });
  const [addModelId, setAddModelId] = useState('');
  const [fetchingModels, setFetchingModels] = useState(false);
  const [fetchedModels, setFetchedModels] = useState<string[]>([]);
  const [selectedFetchedModels, setSelectedFetchedModels] = useState<Set<string>>(new Set());
  const [showFetchDialog, setShowFetchDialog] = useState(false);

  const filteredProviders = useMemo(() => {
    return store.providers.filter(p => (p.model_type || 'claude') === selectedModelType);
  }, [store.providers, selectedModelType]);

  useEffect(() => {
    store.loadProviders();
    loadDeployedTools();
  }, []);

  useEffect(() => {
    store.selectProvider(filteredProviders[0]?.name || '');
  }, [selectedModelType]);

  const loadDeployedTools = async () => {
    try {
      await store.loadAllDeployedProviders();
    } catch { /* silent */ }
  };

  const handleAddProvider = async () => {
    if (!addForm.name || !addForm.base_url) return;
    try {
      await invoke('add_provider', { input: { ...addForm, model_type: selectedModelType } });
      await store.loadProviders();
      store.selectProvider(addForm.name);
      setShowAddDialog(false);
      setAddForm({ name: '', api_key: '', base_url: '', description: '' });
    } catch (e) { console.error('Add provider failed:', e); }
  };

  const handleDeleteProvider = async () => {
    if (!showDeleteDialog) return;
    await store.deleteProvider(showDeleteDialog);
    setShowDeleteDialog(null);
  };

  const handleAddModel = async () => {
    if (!addModelId.trim() || !store.selectedProvider) return;
    try {
      await invoke('add_model', { providerName: store.selectedProvider, input: { id: addModelId.trim() } });
      await store.loadModels();
      await store.loadProviders();
      setAddModelId('');
    } catch (e) { console.error('Add model failed:', e); }
  };

  const handleDeleteModel = async (modelId: string) => {
    if (!store.selectedProvider) return;
    await invoke('delete_model', { providerName: store.selectedProvider, modelId });
    await store.loadModels();
    await store.loadProviders();
  };

  const handleFetchModels = async () => {
    setFetchingModels(true);
    try {
      const models = await store.fetchSiteModels();
      setFetchedModels(models);
      setSelectedFetchedModels(new Set(models));
      setShowFetchDialog(true);
    } catch (e) { console.error('Fetch models failed:', e); }
    finally { setFetchingModels(false); }
  };

  const handleAddFetchedModels = async () => {
    const ids = Array.from(selectedFetchedModels);
    if (ids.length === 0) return;
    await store.addModelsBatch(ids);
    setShowFetchDialog(false);
    setFetchedModels([]);
    setSelectedFetchedModels(new Set());
  };

  const handleApplyConfig = async (toGlobal: boolean, toProject: boolean) => {
    const enabledNames = filteredProviders.filter(p => p.enabled).map(p => p.name);
    if (enabledNames.length === 0) return;
    try {
      await invoke('apply_config', { input: { provider_names: enabledNames, apply_to_global: toGlobal, apply_to_project: toProject } });
      await store.loadProviders();
    } catch (e) { console.error('Apply config failed:', e); }
  };

  const handleSpeedTest = async (providerName: string) => {
    const provider = store.providers.find(p => p.name === providerName);
    if (!provider) return;
    try {
      const detail = await invoke<any>('get_provider', { name: providerName });
      const urls = provider.base_urls?.map(u => u.url) || [provider.base_url];
      await invoke('test_and_auto_select_fastest', { providerName, urls, apiKey: detail?.options?.api_key, modelType: provider.model_type });
      await store.loadProviders();
    } catch (e) { console.error('Speed test failed:', e); }
  };

  return (
    <div className="h-full flex flex-col gap-4 p-4">
      {/* Model Type Selector */}
      <div className="flex justify-center gap-2 flex-shrink-0">
        {MODEL_TYPES.map(mt => (
          <button key={mt.id} className={`btn btn-sm ${selectedModelType === mt.id ? 'btn-primary' : 'btn-ghost'}`} onClick={() => setSelectedModelType(mt.id)} style={selectedModelType === mt.id ? { backgroundColor: mt.color, borderColor: mt.color } : {}}>
            {mt.name}
          </button>
        ))}
      </div>

      <div className="flex-1 min-h-0 flex gap-4">
        {/* Provider List */}
        <div className="w-72 flex flex-col bg-base-200 rounded-lg overflow-hidden">
          <div className="p-3 border-b border-base-content/10 flex items-center justify-between">
            <h3 className="font-semibold text-sm">{t('providers.title', 'Providers')}</h3>
            <div className="flex gap-1">
              <button className="btn btn-xs btn-primary" onClick={() => setShowAddDialog(true)}><Plus size={12} /></button>
              <button className="btn btn-xs btn-ghost" onClick={() => handleApplyConfig(true, false)} title={t('providers.apply', 'Apply')}><Upload size={12} /></button>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto">
            {filteredProviders.map(p => (
              <div key={p.name} className={`group p-3 cursor-pointer border-b border-base-content/5 hover:bg-base-300 transition-colors ${store.selectedProvider === p.name ? 'bg-base-300 border-l-2 border-l-primary' : ''}`} onClick={() => store.selectProvider(p.name)}>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 min-w-0">
                    <button className="shrink-0" onClick={e => { e.stopPropagation(); store.toggleProvider(p.name, !p.enabled); }}>
                      {p.enabled ? <ToggleRight size={18} className="text-success" /> : <ToggleLeft size={18} className="opacity-40" />}
                    </button>
                    <span className={`text-sm font-medium truncate ${!p.enabled ? 'opacity-50' : ''}`}>{p.name}</span>
                  </div>
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button className="btn btn-xs btn-ghost" onClick={e => { e.stopPropagation(); handleSpeedTest(p.name); }}><Zap size={12} /></button>
                    <button className="btn btn-xs btn-ghost text-error" onClick={e => { e.stopPropagation(); setShowDeleteDialog(p.name); }}><Trash2 size={12} /></button>
                  </div>
                </div>
                <div className="text-xs opacity-50 mt-1 truncate font-mono">{p.base_url}</div>
                <div className="text-xs opacity-40 mt-0.5">{p.model_count} models</div>
              </div>
            ))}
            {filteredProviders.length === 0 && (
              <div className="p-8 text-center opacity-50 text-sm">{t('providers.empty', 'No providers')}</div>
            )}
          </div>
        </div>

        {/* Model Detail */}
        <div className="flex-1 bg-base-200 rounded-lg overflow-hidden flex flex-col">
          {store.selectedProvider ? (
            <>
              <div className="p-4 border-b border-base-content/10">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold">{store.selectedProvider} — {t('providers.models', 'Models')}</h3>
                  <button className="btn btn-sm btn-ghost" onClick={handleFetchModels} disabled={fetchingModels}>
                    {fetchingModels ? <Loader2 className="animate-spin" size={14} /> : <Globe size={14} />}
                    {t('providers.fetchModels', 'Fetch Models')}
                  </button>
                </div>
              </div>
              <div className="flex-1 overflow-y-auto p-4 space-y-2">
                {store.models.map(m => (
                  <div key={m.id} className="flex items-center justify-between p-3 rounded-lg bg-base-300 group">
                    <div>
                      <span className="font-mono text-sm">{m.id}</span>
                      {m.name && m.name !== m.id && <span className="text-xs opacity-50 ml-2">({m.name})</span>}
                    </div>
                    <button className="btn btn-xs btn-ghost text-error opacity-0 group-hover:opacity-100" onClick={() => handleDeleteModel(m.id)}><Trash2 size={12} /></button>
                  </div>
                ))}
                {store.models.length === 0 && <div className="text-center py-8 opacity-50 text-sm">{t('providers.noModels', 'No models')}</div>}
              </div>
              <div className="p-3 border-t border-base-content/10 flex gap-2">
                <input type="text" className="input input-sm input-bordered flex-1 font-mono" placeholder={t('providers.addModelPlaceholder', 'Model ID')} value={addModelId} onChange={e => setAddModelId(e.target.value)} onKeyDown={e => { if (e.key === 'Enter') handleAddModel(); }} />
                <button className="btn btn-sm btn-primary" onClick={handleAddModel} disabled={!addModelId.trim()}><Plus size={14} /></button>
              </div>
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center opacity-50">{t('providers.selectProvider', 'Select a provider')}</div>
          )}
        </div>
      </div>

      {/* Add Provider Dialog */}
      {showAddDialog && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('providers.addProvider', 'Add Provider')}</h3>
            <div className="space-y-3 mt-4">
              <input type="text" className="input input-bordered w-full" placeholder={t('providers.name', 'Name')} value={addForm.name} onChange={e => setAddForm(f => ({ ...f, name: e.target.value }))} />
              <input type="text" className="input input-bordered w-full font-mono" placeholder={t('providers.baseUrl', 'Base URL')} value={addForm.base_url} onChange={e => setAddForm(f => ({ ...f, base_url: e.target.value }))} />
              <input type="password" className="input input-bordered w-full font-mono" placeholder={t('providers.apiKey', 'API Key')} value={addForm.api_key} onChange={e => setAddForm(f => ({ ...f, api_key: e.target.value }))} />
              <input type="text" className="input input-bordered w-full" placeholder={t('providers.description', 'Description (optional)')} value={addForm.description} onChange={e => setAddForm(f => ({ ...f, description: e.target.value }))} />
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowAddDialog(false)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-primary" onClick={handleAddProvider} disabled={!addForm.name || !addForm.base_url}>{t('common.save', 'Save')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirm Dialog */}
      {showDeleteDialog && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg">{t('confirm.deleteTitle', 'Confirm Delete')}</h3>
            <p className="py-4">{t('confirm.deleteProvider', `Are you sure you want to delete "${showDeleteDialog}"?`)}</p>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowDeleteDialog(null)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-error" onClick={handleDeleteProvider}>{t('common.delete', 'Delete')}</button>
            </div>
          </div>
        </div>
      )}

      {/* Fetch Models Dialog */}
      {showFetchDialog && (
        <div className="modal modal-open">
          <div className="modal-box max-w-lg">
            <h3 className="font-bold text-lg">{t('providers.fetchedModels', 'Available Models')}</h3>
            <div className="mt-4 max-h-64 overflow-y-auto space-y-1">
              {fetchedModels.map(m => (
                <label key={m} className="flex items-center gap-2 p-2 rounded hover:bg-base-200 cursor-pointer">
                  <input type="checkbox" className="checkbox checkbox-sm" checked={selectedFetchedModels.has(m)} onChange={() => setSelectedFetchedModels(prev => { const next = new Set(prev); next.has(m) ? next.delete(m) : next.add(m); return next; })} />
                  <span className="font-mono text-sm">{m}</span>
                </label>
              ))}
              {fetchedModels.length === 0 && <div className="text-center py-4 opacity-50">{t('providers.noModelsFound', 'No models found')}</div>}
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setShowFetchDialog(false)}>{t('common.cancel', 'Cancel')}</button>
              <button className="btn btn-primary" onClick={handleAddFetchedModels} disabled={selectedFetchedModels.size === 0}>
                {t('providers.addSelected', 'Add Selected')} ({selectedFetchedModels.size})
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

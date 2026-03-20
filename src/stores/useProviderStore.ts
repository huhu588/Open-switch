import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface BaseUrlItem {
  url: string;
  latency_ms: number | null;
  last_tested: string | null;
  quality: 'excellent' | 'good' | 'fair' | 'poor' | 'failed' | 'untested';
}

export interface ProviderItem {
  name: string;
  base_url: string;
  base_urls: BaseUrlItem[];
  model_count: number;
  description: string | null;
  model_type: string;
  enabled: boolean;
}

export interface ModelItem {
  id: string;
  name: string;
  reasoning_effort?: string;
  thinking_budget?: number | null;
}

export interface DeployedProviderItem {
  name: string;
  base_url: string;
  api_key?: string;
  model_count: number;
  source: string;
  inferred_model_type?: string;
  tool?: string;
  current_model?: string;
}

interface ProviderState {
  providers: ProviderItem[];
  selectedProvider: string | null;
  models: ModelItem[];
  loading: boolean;
  error: string | null;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  loadProviders: () => Promise<void>;
  loadModels: () => Promise<void>;
  selectProvider: (name: string) => Promise<void>;
  deleteProvider: (name: string) => Promise<void>;
  toggleProvider: (name: string, enabled: boolean) => Promise<void>;
  updateProvider: (name: string, data: { base_url?: string; api_key?: string; description?: string }) => Promise<void>;
  loadAllDeployedProviders: () => Promise<DeployedProviderItem[]>;
  fetchSiteModels: () => Promise<string[]>;
  addModelsBatch: (modelIds: string[]) => Promise<void>;
}

export const useProviderStore = create<ProviderState>((set, get) => ({
  providers: [],
  selectedProvider: null,
  models: [],
  loading: false,
  error: null,
  searchQuery: '',
  setSearchQuery: (query: string) => set({ searchQuery: query }),

  loadProviders: async () => {
    set({ loading: true, error: null });
    try {
      const providers = await invoke<ProviderItem[]>('get_providers');
      set({ providers });
      if (providers.length > 0 && !get().selectedProvider) {
        const first = providers[0].name;
        set({ selectedProvider: first });
        const models = await invoke<ModelItem[]>('get_models', { providerName: first });
        set({ models });
      }
      try { await invoke('refresh_tray_menu'); } catch { /* silent */ }
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  loadModels: async () => {
    const { selectedProvider } = get();
    if (!selectedProvider) { set({ models: [] }); return; }
    try {
      const models = await invoke<ModelItem[]>('get_models', { providerName: selectedProvider });
      set({ models });
    } catch (e) {
      console.error('Failed to load models:', e);
      set({ models: [] });
    }
  },

  selectProvider: async (name: string) => {
    set({ selectedProvider: name });
    try {
      const models = await invoke<ModelItem[]>('get_models', { providerName: name });
      set({ models });
    } catch { set({ models: [] }); }
  },

  deleteProvider: async (name: string) => {
    await invoke('delete_provider', { name });
    const { selectedProvider } = get();
    if (selectedProvider === name) set({ selectedProvider: null, models: [] });
    await get().loadProviders();
  },

  toggleProvider: async (name: string, enabled: boolean) => {
    await invoke('toggle_provider', { name, enabled });
    await get().loadProviders();
  },

  updateProvider: async (name: string, data: { base_url?: string; api_key?: string; description?: string }) => {
    await invoke('update_provider', { name, input: data });
    await get().loadProviders();
    if (get().selectedProvider === name) {
      await get().loadModels();
    }
  },

  loadAllDeployedProviders: async () => {
    const all: DeployedProviderItem[] = [];
    try { const p = await invoke<DeployedProviderItem[]>('get_deployed_providers'); all.push(...p.map(x => ({ ...x, tool: 'opencode' }))); } catch { /* silent */ }
    try {
      const s = await invoke<{ is_configured: boolean; has_api_key: boolean }>('get_claude_code_status');
      if (s.is_configured && s.has_api_key) {
        const settings = await invoke<{ env?: Record<string, string>; model?: string }>('get_claude_code_settings');
        all.push({ name: 'Claude Code', base_url: settings.env?.['ANTHROPIC_BASE_URL'] || 'https://api.anthropic.com', model_count: -1, source: 'claude_code', tool: 'claude_code', inferred_model_type: 'claude', current_model: settings.model });
      }
    } catch { /* silent */ }
    try {
      const s = await invoke<{ is_configured: boolean; provider_count: number }>('get_codex_status');
      if (s.is_configured && s.provider_count > 0) {
        const providers = await invoke<Record<string, { name?: string; base_url?: string }>>('get_codex_providers');
        Object.entries(providers).forEach(([name, p]) => {
          all.push({ name: name || 'Codex Provider', base_url: p.base_url || 'https://api.openai.com/v1', model_count: -1, source: 'codex', tool: 'codex', inferred_model_type: 'codex' });
        });
      }
    } catch { /* silent */ }
    return all;
  },

  fetchSiteModels: async () => {
    const { selectedProvider } = get();
    if (!selectedProvider) return [];
    return invoke<string[]>('fetch_site_models', { providerName: selectedProvider });
  },

  addModelsBatch: async (modelIds: string[]) => {
    const { selectedProvider } = get();
    if (!selectedProvider) return;
    await invoke('add_models_batch', { providerName: selectedProvider, modelIds });
    await get().loadModels();
    await get().loadProviders();
  },
}));

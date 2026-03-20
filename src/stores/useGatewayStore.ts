import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface GatewayStatus {
  running: boolean;
  port: number;
  total_accounts: number;
  active_accounts: number;
  total_api_keys: number;
  total_requests: number;
  synced_accounts?: number;
  platform_stats?: Record<string, number>;
}

interface GatewayConfig {
  enabled: boolean;
  port: number;
  upstream_base_url: string;
  upstream_proxy_url: string | null;
  route_strategy: string;
  auto_start: boolean;
  cors_enabled: boolean;
  max_concurrent_per_account: number;
  cooldown_seconds: number;
  platform_upstreams: Record<string, string> | null;
  enable_account_bridge: boolean;
}

interface SyncResult {
  total_platforms: number;
  total_accounts: number;
  added: number;
  updated: number;
  skipped: number;
}

interface Sub2apiSyncResult {
  total_accounts: number;
  synced: number;
  failed: number;
  errors: string[];
}

interface GatewayState {
  status: GatewayStatus | null;
  config: GatewayConfig | null;
  loading: boolean;
  syncing: boolean;
  platformStats: Record<string, number> | null;
  fetchStatus: () => Promise<void>;
  fetchConfig: () => Promise<void>;
  updateConfig: (config: Partial<GatewayConfig>) => Promise<void>;
  startGateway: () => Promise<void>;
  stopGateway: () => Promise<void>;
  syncAccountsFromPlatforms: () => Promise<SyncResult>;
  syncAccountsToSub2api: () => Promise<Sub2apiSyncResult>;
  fetchPlatformStats: () => Promise<void>;
}

export const useGatewayStore = create<GatewayState>((set, get) => ({
  status: null,
  config: null,
  loading: false,
  syncing: false,
  platformStats: null,

  fetchStatus: async () => {
    try {
      const status = await invoke<GatewayStatus>('get_gateway_status');
      set({ status });
    } catch (error) {
      console.error('[GatewayStore] fetchStatus failed:', error);
    }
  },

  fetchConfig: async () => {
    try {
      const config = await invoke<GatewayConfig>('get_gateway_config');
      set({ config });
    } catch (error) {
      console.error('[GatewayStore] fetchConfig failed:', error);
    }
  },

  updateConfig: async (updates) => {
    const current = get().config;
    if (!current) return;
    const newConfig = { ...current, ...updates };
    try {
      await invoke('save_gateway_config', { config: newConfig });
      set({ config: newConfig });
    } catch (error) {
      console.error('[GatewayStore] updateConfig failed:', error);
    }
  },

  startGateway: async () => {
    set({ loading: true });
    try {
      await invoke('start_gateway');
      await get().fetchStatus();
    } catch (error) {
      console.error('[GatewayStore] startGateway failed:', error);
      throw error;
    } finally {
      set({ loading: false });
    }
  },

  stopGateway: async () => {
    set({ loading: true });
    try {
      await invoke('stop_gateway');
      await get().fetchStatus();
    } catch (error) {
      console.error('[GatewayStore] stopGateway failed:', error);
      throw error;
    } finally {
      set({ loading: false });
    }
  },

  syncAccountsFromPlatforms: async () => {
    set({ syncing: true });
    try {
      const result = await invoke<SyncResult>('sync_accounts_to_gateway');
      await get().fetchStatus();
      await get().fetchPlatformStats();
      return result;
    } catch (error) {
      console.error('[GatewayStore] syncAccountsFromPlatforms failed:', error);
      throw error;
    } finally {
      set({ syncing: false });
    }
  },

  syncAccountsToSub2api: async () => {
    set({ syncing: true });
    try {
      const result = await invoke<Sub2apiSyncResult>('sync_accounts_to_sub2api');
      return result;
    } catch (error) {
      console.error('[GatewayStore] syncAccountsToSub2api failed:', error);
      throw error;
    } finally {
      set({ syncing: false });
    }
  },

  fetchPlatformStats: async () => {
    try {
      const stats = await invoke<Record<string, number>>('get_platform_account_stats');
      set({ platformStats: stats });
    } catch (error) {
      console.error('[GatewayStore] fetchPlatformStats failed:', error);
    }
  },
}));

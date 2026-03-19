import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface GatewayStatus {
  running: boolean;
  port: number;
  total_accounts: number;
  active_accounts: number;
  total_api_keys: number;
  total_requests: number;
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
}

interface GatewayState {
  status: GatewayStatus | null;
  config: GatewayConfig | null;
  loading: boolean;
  fetchStatus: () => Promise<void>;
  fetchConfig: () => Promise<void>;
  updateConfig: (config: Partial<GatewayConfig>) => Promise<void>;
  startGateway: () => Promise<void>;
  stopGateway: () => Promise<void>;
}

export const useGatewayStore = create<GatewayState>((set, get) => ({
  status: null,
  config: null,
  loading: false,

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
}));

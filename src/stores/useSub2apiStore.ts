import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface Sub2apiStatus {
  running: boolean;
  port: number;
  pid: number | null;
  url: string | null;
}

interface Sub2apiState {
  status: Sub2apiStatus | null;
  loading: boolean;
  fetchStatus: () => Promise<void>;
  startSub2api: () => Promise<void>;
  stopSub2api: () => Promise<void>;
}

export const useSub2apiStore = create<Sub2apiState>((set, get) => ({
  status: null,
  loading: false,

  fetchStatus: async () => {
    try {
      const status = await invoke<Sub2apiStatus>('get_sub2api_status');
      set({ status });
    } catch (error) {
      console.error('[Sub2apiStore] fetchStatus failed:', error);
    }
  },

  startSub2api: async () => {
    set({ loading: true });
    try {
      const status = await invoke<Sub2apiStatus>('start_sub2api');
      set({ status });
    } catch (error) {
      console.error('[Sub2apiStore] startSub2api failed:', error);
      throw error;
    } finally {
      set({ loading: false });
    }
  },

  stopSub2api: async () => {
    set({ loading: true });
    try {
      await invoke('stop_sub2api');
      await get().fetchStatus();
    } catch (error) {
      console.error('[Sub2apiStore] stopSub2api failed:', error);
      throw error;
    } finally {
      set({ loading: false });
    }
  },
}));

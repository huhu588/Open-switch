import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface SessionInfo {
  id: string;
  platform: string;
  title: string | null;
  summary: string | null;
  working_directory: string | null;
  created_at: number | null;
  updated_at: number | null;
  message_count: number;
  file_path: string;
  resume_command: string | null;
}

export interface SessionMessage {
  role: string;
  content: string;
  timestamp: number | null;
}

interface SessionState {
  sessions: SessionInfo[];
  messages: SessionMessage[];
  loading: boolean;
  messagesLoading: boolean;
  messagesError: string | null;
  searchQuery: string;
  platformFilter: string | null;
  error: string | null;

  setSearchQuery: (query: string) => void;
  setPlatformFilter: (platform: string | null) => void;
  loadSessions: (forceRefresh?: boolean) => Promise<void>;
  searchSessions: (query: string, platform?: string | null, forceRefresh?: boolean) => Promise<void>;
  loadMessages: (platform: string, sourcePath: string) => Promise<void>;
  clearMessages: () => void;
  deleteSession: (platform: string, sessionId: string, sourcePath: string) => Promise<boolean>;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: [],
  messages: [],
  loading: false,
  messagesLoading: false,
  messagesError: null,
  searchQuery: '',
  platformFilter: null,
  error: null,

  setSearchQuery: (query) => set({ searchQuery: query }),
  setPlatformFilter: (platform) => set({ platformFilter: platform }),

  loadSessions: async (forceRefresh = false) => {
    set({ loading: true, error: null });
    try {
      const platform = get().platformFilter;
      const sessions = await invoke<SessionInfo[]>('list_sessions', {
        platform: platform ?? undefined,
        forceRefresh: forceRefresh || undefined,
      });
      set({ sessions, loading: false });
    } catch (e) {
      console.error('[SessionStore] loadSessions failed:', e);
      set({ error: String(e), loading: false });
    }
  },

  searchSessions: async (query, platform, forceRefresh = false) => {
    set({ loading: true, error: null });
    try {
      const sessions = await invoke<SessionInfo[]>('search_sessions', {
        query,
        platform: platform ?? get().platformFilter ?? undefined,
        forceRefresh: forceRefresh || undefined,
      });
      set({ sessions, loading: false });
    } catch (e) {
      console.error('[SessionStore] searchSessions failed:', e);
      set({ error: String(e), loading: false });
    }
  },

  loadMessages: async (platform: string, sourcePath: string) => {
    set({ messagesLoading: true, messagesError: null, messages: [] });
    try {
      const messages = await invoke<SessionMessage[]>('get_session_messages', {
        platform,
        sourcePath,
      });
      set({ messages, messagesLoading: false });
    } catch (e) {
      console.error('[SessionStore] loadMessages failed:', e);
      set({ messagesError: String(e), messagesLoading: false });
    }
  },

  clearMessages: () => set({ messages: [], messagesError: null }),

  deleteSession: async (platform: string, sessionId: string, sourcePath: string) => {
    try {
      await invoke<boolean>('delete_session', { platform, sessionId, sourcePath });
      set((state) => ({
        sessions: state.sessions.filter((s) => s.id !== sessionId),
      }));
      return true;
    } catch (e) {
      console.error('[SessionStore] deleteSession failed:', e);
      return false;
    }
  },
}));

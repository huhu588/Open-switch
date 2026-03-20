import { create } from 'zustand';
import { sub2apiClient } from '../services/sub2apiClient';

export interface Sub2apiDashboardStats {
  total_requests: number;
  active_accounts: number;
  today_usage: number;
  error_rate: number;
  total_users: number;
  total_api_keys: number;
  total_groups: number;
  [key: string]: unknown;
}

export interface Sub2apiAccount {
  id: number;
  email?: string;
  access_token?: string;
  platform?: string;
  group?: string;
  status?: string;
  created_at?: string;
  [key: string]: unknown;
}

export interface Sub2apiGroup {
  id: number;
  name: string;
  ratio?: number;
  capacity?: number;
  [key: string]: unknown;
}

export interface Sub2apiUser {
  id: number;
  username?: string;
  email?: string;
  role?: string;
  balance?: number;
  status?: string;
  created_at?: string;
  [key: string]: unknown;
}

export interface Sub2apiApiKey {
  id: number;
  key?: string;
  name?: string;
  status?: string;
  user_id?: number;
  group?: string;
  created_at?: string;
  [key: string]: unknown;
}

export interface PaginatedList<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

interface Sub2apiAdminState {
  activeTab: string;
  dashboardStats: Sub2apiDashboardStats | null;
  dashboardLoading: boolean;

  accounts: PaginatedList<Sub2apiAccount>;
  accountsLoading: boolean;

  groups: Sub2apiGroup[];
  groupsLoading: boolean;

  users: PaginatedList<Sub2apiUser>;
  usersLoading: boolean;

  apiKeys: PaginatedList<Sub2apiApiKey>;
  apiKeysLoading: boolean;

  setActiveTab: (tab: string) => void;

  fetchDashboard: () => Promise<void>;

  fetchAccounts: (page?: number, pageSize?: number, search?: string) => Promise<void>;
  createAccount: (data: Record<string, unknown>) => Promise<void>;
  deleteAccount: (id: number) => Promise<void>;

  fetchGroups: () => Promise<void>;
  createGroup: (data: Record<string, unknown>) => Promise<void>;
  deleteGroup: (id: number) => Promise<void>;

  fetchUsers: (page?: number, pageSize?: number) => Promise<void>;
  createUser: (data: Record<string, unknown>) => Promise<void>;
  deleteUser: (id: number) => Promise<void>;

  fetchApiKeys: (page?: number, pageSize?: number) => Promise<void>;
  createApiKey: (data: Record<string, unknown>) => Promise<void>;
  deleteApiKey: (id: number) => Promise<void>;
}

export const useSub2apiAdminStore = create<Sub2apiAdminState>((set) => ({
  activeTab: 'dashboard',
  dashboardStats: null,
  dashboardLoading: false,

  accounts: { items: [], total: 0, page: 1, page_size: 20 },
  accountsLoading: false,

  groups: [],
  groupsLoading: false,

  users: { items: [], total: 0, page: 1, page_size: 20 },
  usersLoading: false,

  apiKeys: { items: [], total: 0, page: 1, page_size: 20 },
  apiKeysLoading: false,

  setActiveTab: (tab) => set({ activeTab: tab }),

  fetchDashboard: async () => {
    set({ dashboardLoading: true });
    try {
      const stats = await sub2apiClient.get<Sub2apiDashboardStats>('/admin/dashboard/snapshot-v2');
      set({ dashboardStats: stats });
    } catch (error) {
      console.error('[Sub2api] fetchDashboard failed:', error);
    } finally {
      set({ dashboardLoading: false });
    }
  },

  fetchAccounts: async (page = 1, pageSize = 20, search?: string) => {
    set({ accountsLoading: true });
    try {
      const params: Record<string, unknown> = { page, page_size: pageSize };
      if (search) params.search = search;
      const data = await sub2apiClient.get<PaginatedList<Sub2apiAccount>>('/admin/accounts', params);
      if (data && typeof data === 'object') {
        if (Array.isArray(data)) {
          set({ accounts: { items: data, total: data.length, page, page_size: pageSize } });
        } else {
          set({ accounts: { ...data, page, page_size: pageSize } });
        }
      }
    } catch (error) {
      console.error('[Sub2api] fetchAccounts failed:', error);
    } finally {
      set({ accountsLoading: false });
    }
  },

  createAccount: async (data) => {
    await sub2apiClient.post('/admin/accounts', data);
  },

  deleteAccount: async (id) => {
    await sub2apiClient.delete(`/admin/accounts/${id}`);
  },

  fetchGroups: async () => {
    set({ groupsLoading: true });
    try {
      const data = await sub2apiClient.get<Sub2apiGroup[]>('/admin/groups');
      set({ groups: Array.isArray(data) ? data : [] });
    } catch (error) {
      console.error('[Sub2api] fetchGroups failed:', error);
    } finally {
      set({ groupsLoading: false });
    }
  },

  createGroup: async (data) => {
    await sub2apiClient.post('/admin/groups', data);
  },

  deleteGroup: async (id) => {
    await sub2apiClient.delete(`/admin/groups/${id}`);
  },

  fetchUsers: async (page = 1, pageSize = 20) => {
    set({ usersLoading: true });
    try {
      const data = await sub2apiClient.get<PaginatedList<Sub2apiUser>>('/admin/users', { page, page_size: pageSize });
      if (data && typeof data === 'object') {
        if (Array.isArray(data)) {
          set({ users: { items: data, total: data.length, page, page_size: pageSize } });
        } else {
          set({ users: { ...data, page, page_size: pageSize } });
        }
      }
    } catch (error) {
      console.error('[Sub2api] fetchUsers failed:', error);
    } finally {
      set({ usersLoading: false });
    }
  },

  createUser: async (data) => {
    await sub2apiClient.post('/admin/users', data);
  },

  deleteUser: async (id) => {
    await sub2apiClient.delete(`/admin/users/${id}`);
  },

  fetchApiKeys: async (page = 1, pageSize = 20) => {
    set({ apiKeysLoading: true });
    try {
      const data = await sub2apiClient.get<PaginatedList<Sub2apiApiKey>>('/keys', { page, page_size: pageSize });
      if (data && typeof data === 'object') {
        if (Array.isArray(data)) {
          set({ apiKeys: { items: data, total: data.length, page, page_size: pageSize } });
        } else {
          set({ apiKeys: { ...data, page, page_size: pageSize } });
        }
      }
    } catch (error) {
      console.error('[Sub2api] fetchApiKeys failed:', error);
    } finally {
      set({ apiKeysLoading: false });
    }
  },

  createApiKey: async (data) => {
    await sub2apiClient.post('/keys', data);
  },

  deleteApiKey: async (id) => {
    await sub2apiClient.delete(`/keys/${id}`);
  },
}));

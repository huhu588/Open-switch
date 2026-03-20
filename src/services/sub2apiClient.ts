import { invoke } from '@tauri-apps/api/core';

export interface Sub2apiResponse<T = unknown> {
  success: boolean;
  message?: string;
  data?: T;
}

function buildQuery(params?: Record<string, unknown>): string {
  if (!params) return '';
  const parts: string[] = [];
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
    }
  }
  return parts.join('&');
}

async function request<T = unknown>(
  method: string,
  path: string,
  data?: Record<string, unknown> | null,
  params?: Record<string, unknown>,
): Promise<T> {
  const query = buildQuery(params);
  const body = data ? JSON.stringify(data) : undefined;

  const raw = await invoke<string>('sub2api_proxy', {
    method,
    path,
    body: body ?? null,
    query: query || null,
  });

  try {
    const parsed = JSON.parse(raw);
    if (parsed.data !== undefined) {
      return parsed.data as T;
    }
    return parsed as T;
  } catch {
    return raw as unknown as T;
  }
}

export const sub2apiClient = {
  get: <T = unknown>(path: string, params?: Record<string, unknown>) =>
    request<T>('GET', path, null, params),

  post: <T = unknown>(path: string, data?: Record<string, unknown>) =>
    request<T>('POST', path, data),

  put: <T = unknown>(path: string, data?: Record<string, unknown>) =>
    request<T>('PUT', path, data),

  patch: <T = unknown>(path: string, data?: Record<string, unknown>) =>
    request<T>('PATCH', path, data),

  delete: <T = unknown>(path: string, params?: Record<string, unknown>) =>
    request<T>('DELETE', path, null, params),

  login: () => invoke<string>('sub2api_login'),

  clearAuth: () => invoke<void>('sub2api_clear_auth'),
};

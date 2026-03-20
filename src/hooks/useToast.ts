import { useState, useCallback, useRef } from 'react';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
  leaving: boolean;
}

export function useToast(duration = 3000) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const idRef = useRef(0);

  const dismiss = useCallback((id: number) => {
    setToasts(prev => prev.map(t => t.id === id ? { ...t, leaving: true } : t));
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, 250);
  }, []);

  const show = useCallback((message: string, type: ToastType = 'info') => {
    const id = idRef.current++;
    setToasts(prev => [...prev, { id, message, type, leaving: false }]);
    setTimeout(() => dismiss(id), duration);
    return id;
  }, [duration, dismiss]);

  const success = useCallback((msg: string) => show(msg, 'success'), [show]);
  const error = useCallback((msg: string) => show(msg, 'error'), [show]);
  const warning = useCallback((msg: string) => show(msg, 'warning'), [show]);
  const info = useCallback((msg: string) => show(msg, 'info'), [show]);

  return { toasts, show, success, error, warning, info, dismiss };
}

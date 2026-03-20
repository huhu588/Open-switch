import { CheckCircle, AlertTriangle, XCircle, Info } from 'lucide-react';
import type { ToastItem } from '../hooks/useToast';

const ICONS = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
} as const;

export function ToastContainer({ toasts }: { toasts: ToastItem[] }) {
  if (toasts.length === 0) return null;
  return (
    <div className="oc-toast-container">
      {toasts.map(t => {
        const Icon = ICONS[t.type];
        return (
          <div key={t.id} className={`oc-toast oc-toast--${t.type}${t.leaving ? ' is-leaving' : ''}`}>
            <Icon size={16} />
            <span>{t.message}</span>
          </div>
        );
      })}
    </div>
  );
}

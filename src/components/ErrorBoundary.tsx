import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallbackMessage?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error('[ErrorBoundary] Caught rendering error:', error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '300px',
          gap: '16px',
          padding: '32px',
          color: 'var(--text-secondary)',
        }}>
          <AlertTriangle size={40} style={{ color: 'var(--warning)' }} />
          <p style={{ fontSize: '15px', fontWeight: 600, color: 'var(--text-primary)' }}>
            {this.props.fallbackMessage || '页面渲染出错'}
          </p>
          <p style={{ fontSize: '13px', maxWidth: '400px', textAlign: 'center', opacity: 0.7 }}>
            {this.state.error?.message || '未知错误'}
          </p>
          <button
            onClick={this.handleRetry}
            className="btn btn-sm btn-primary"
            style={{ marginTop: '8px', display: 'inline-flex', alignItems: 'center', gap: '6px' }}
          >
            <RefreshCw size={14} />
            重试
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

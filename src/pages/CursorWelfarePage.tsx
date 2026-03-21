import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Power, PowerOff, RefreshCw, Loader2, Copy, CheckCircle,
  Settings, Zap, BarChart3, ExternalLink, Share2, Download,
  AlertTriangle, Globe, Key, Hash, Clock, Bug,
} from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { ToastContainer } from '../components/Toast';

interface CursorWelfareStatus {
  running: boolean;
  port: number;
  pid: number | null;
  url: string | null;
}

interface CursorWelfareConfig {
  port: number;
  apiKey: string;
  models: string;
  timeout: number;
  debug: boolean;
  autoStart: boolean;
  scriptUrl: string;
  userAgent: string;
}

interface BinaryCheckResult {
  available: boolean;
  path: string | null;
  error: string | null;
  downloadUrl: string;
}

interface ApplyToolError { tool: string; error: string; }
interface ApplyToToolsResult { success: string[]; failed: ApplyToolError[]; }
interface ShareResult { success: boolean; shareUrl: string | null; message: string; }

const TOOL_LABELS: Record<string, string> = {
  claude: 'Claude Code',
  codex: 'Codex',
  gemini: 'Gemini',
  opencode: 'OpenCode',
  openclaw: 'OpenClaw',
};

const DEFAULT_CONFIG: CursorWelfareConfig = {
  port: 8002,
  apiKey: '0000',
  models: 'claude-sonnet-4.6',
  timeout: 60,
  debug: false,
  autoStart: false,
  scriptUrl: 'https://cursor.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/a-4-a/c.js?i=0&v=3&h=cursor.com',
  userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36',
};

export function CursorWelfarePage() {
  const toast = useToast();

  const [status, setStatus] = useState<CursorWelfareStatus>({
    running: false, port: 0, pid: null, url: null,
  });
  const [config, setConfig] = useState<CursorWelfareConfig>(DEFAULT_CONFIG);
  const [binaryCheck, setBinaryCheck] = useState<BinaryCheckResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [saving, setSaving] = useState(false);
  const [applying, setApplying] = useState(false);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [selectedTools, setSelectedTools] = useState<string[]>(Object.keys(TOOL_LABELS));
  const [proxyPort, setProxyPort] = useState(15721);
  const [sharing, setSharing] = useState(false);
  const [shareResult, setShareResult] = useState<ShareResult | null>(null);
  const [activeTab, setActiveTab] = useState<'service' | 'tools' | 'share'>('service');

  const refreshStatus = useCallback(async () => {
    try {
      const s = await invoke<CursorWelfareStatus>('get_cursor_welfare_status');
      setStatus(s);
    } catch (e) {
      console.error('获取状态失败:', e);
    }
  }, []);

  const refreshConfig = useCallback(async () => {
    try {
      const c = await invoke<CursorWelfareConfig>('get_cursor_welfare_config');
      setConfig(c);
    } catch { /* use defaults */ }
  }, []);

  const checkBinary = useCallback(async () => {
    try {
      const result = await invoke<BinaryCheckResult>('check_cursor_welfare_binary');
      setBinaryCheck(result);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    const init = async () => {
      setLoading(true);
      await Promise.all([refreshStatus(), refreshConfig(), checkBinary()]);
      try {
        const proxyConfig = await invoke<{ listenPort: number }>('get_proxy_config');
        setProxyPort(proxyConfig.listenPort);
      } catch { /* use default */ }
      setLoading(false);
    };
    init();
    const interval = setInterval(refreshStatus, 5000);
    return () => clearInterval(interval);
  }, [refreshStatus, refreshConfig, checkBinary]);

  const handleStart = async () => {
    setStarting(true);
    try {
      const s = await invoke<CursorWelfareStatus>('start_cursor_welfare');
      setStatus(s);
      toast.success('Cursor 福利服务已启动');
    } catch (e) {
      toast.error(`启动失败: ${e}`);
    } finally {
      setStarting(false);
    }
  };

  const handleStop = async () => {
    setStopping(true);
    try {
      await invoke('stop_cursor_welfare');
      setStatus({ running: false, port: 0, pid: null, url: null });
      toast.success('服务已停止');
    } catch (e) {
      toast.error(`停止失败: ${e}`);
    } finally {
      setStopping(false);
    }
  };

  const handleSaveConfig = async () => {
    setSaving(true);
    try {
      await invoke('save_cursor_welfare_config', { config });
      toast.success('配置已保存');
    } catch (e) {
      toast.error(`保存失败: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleApplyToTools = async () => {
    setApplying(true);
    try {
      const result = await invoke<ApplyToToolsResult>('apply_cursor_welfare_to_tools', {
        input: { apiKey: config.apiKey, proxyPort, tools: selectedTools },
      });
      if (result.success.length > 0) {
        toast.success(`已应用到: ${result.success.map(t => TOOL_LABELS[t] || t).join(', ')}`);
      }
      if (result.failed.length > 0) {
        toast.error(result.failed.map(f => `${TOOL_LABELS[f.tool] || f.tool}: ${f.error}`).join('\n'));
      }
    } catch (e) {
      toast.error(`应用失败: ${e}`);
    } finally {
      setApplying(false);
    }
  };

  const handleShareToSub2api = async () => {
    setSharing(true);
    try {
      const result = await invoke<ShareResult>('share_cursor_welfare_to_sub2api');
      setShareResult(result);
      if (result.success) toast.success(result.message);
      else toast.error(result.message);
    } catch (e) {
      toast.error(`共享失败: ${e}`);
    } finally {
      setSharing(false);
    }
  };

  const copyToClipboard = async (text: string, field: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    } catch { toast.error('复制失败'); }
  };

  const toggleTool = (tool: string) => {
    setSelectedTools(prev =>
      prev.includes(tool) ? prev.filter(t => t !== tool) : [...prev, tool]
    );
  };

  if (loading) {
    return (
      <div className="cw-page">
        <div className="cw-loading"><Loader2 className="animate-spin" size={24} /><span>加载中...</span></div>
      </div>
    );
  }

  const proxyBase = `http://localhost:${proxyPort}`;
  const binaryOk = binaryCheck?.available ?? false;

  return (
    <div className="cw-page">
      <ToastContainer toasts={toast.toasts} />

      {/* Header */}
      <div className="cw-header">
        <div className="cw-header-title">
          <div className="cw-logo"><Zap size={20} /></div>
          <div>
            <h1>Cursor 福利</h1>
            <p>基于 cursor2api-go · Cursor 免费额度复用</p>
          </div>
        </div>
        <div className="cw-header-right">
          <div className={`cw-status-dot ${status.running ? 'running' : 'stopped'}`} />
          <span className="cw-status-text">{status.running ? '运行中' : '已停止'}</span>
          <button className="cw-icon-btn" onClick={refreshStatus} title="刷新"><RefreshCw size={15} /></button>
        </div>
      </div>

      {/* Binary missing banner */}
      {!binaryOk && (
        <div className="cw-banner cw-banner-warn">
          <AlertTriangle size={18} />
          <div className="cw-banner-content">
            <strong>缺少 cursor2api-go 可执行文件</strong>
            <span>请从 GitHub Releases 下载对应平台的二进制文件并放置到应用目录。</span>
          </div>
          <a
            className="cw-banner-action"
            href={binaryCheck?.downloadUrl || 'https://github.com/libaxuan/cursor2api-go/releases'}
            target="_blank"
            rel="noopener noreferrer"
          >
            <Download size={14} /> 前往下载
          </a>
        </div>
      )}

      {/* Tabs */}
      <div className="cw-tabs">
        <button className={`cw-tab ${activeTab === 'service' ? 'active' : ''}`} onClick={() => setActiveTab('service')}>
          <Settings size={15} /> 服务管理
        </button>
        <button className={`cw-tab ${activeTab === 'tools' ? 'active' : ''}`} onClick={() => setActiveTab('tools')}>
          <Zap size={15} /> 快速配置
        </button>
        <button className={`cw-tab ${activeTab === 'share' ? 'active' : ''}`} onClick={() => setActiveTab('share')}>
          <Share2 size={15} /> 共享服务
        </button>
      </div>

      {/* Tab: Service Management */}
      {activeTab === 'service' && (
        <div className="cw-tab-content">
          {/* Top row: Status + Control */}
          <div className="cw-grid-2">
            {/* Service Control */}
            <div className="cw-card">
              <div className="cw-card-title">启停控制</div>
              <div className="cw-control-area">
                {!status.running ? (
                  <button className="cw-btn cw-btn-start" onClick={handleStart} disabled={starting || !binaryOk}>
                    {starting ? <Loader2 className="animate-spin" size={18} /> : <Power size={18} />}
                    {starting ? '启动中...' : '启动服务'}
                  </button>
                ) : (
                  <button className="cw-btn cw-btn-stop" onClick={handleStop} disabled={stopping}>
                    {stopping ? <Loader2 className="animate-spin" size={18} /> : <PowerOff size={18} />}
                    {stopping ? '停止中...' : '停止服务'}
                  </button>
                )}
                {status.running && (
                  <div className="cw-run-info">
                    <span>PID {status.pid ?? '-'}</span>
                    <span>·</span>
                    <span>端口 {status.port}</span>
                  </div>
                )}
                {!binaryOk && !status.running && (
                  <div className="cw-hint-text">需要先安装 cursor2api-go</div>
                )}
              </div>
            </div>

            {/* Running endpoints */}
            <div className="cw-card">
              <div className="cw-card-title">接入地址</div>
              {status.running ? (
                <div className="cw-endpoints">
                  {[
                    { label: 'OpenAI (Codex)', path: '/cursor-welfare/v1/chat/completions' },
                    { label: 'Claude', path: '/cursor-welfare/v1/messages' },
                    { label: 'Gemini', path: '/cursor-welfare/v1beta/' },
                  ].map(ep => (
                    <div key={ep.path} className="cw-endpoint-row">
                      <span className="cw-ep-label">{ep.label}</span>
                      <div className="cw-ep-url">
                        <code>{proxyBase}{ep.path}</code>
                        <button className="cw-copy-btn" onClick={() => copyToClipboard(`${proxyBase}${ep.path}`, ep.path)}>
                          {copiedField === ep.path ? <CheckCircle size={13} /> : <Copy size={13} />}
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="cw-empty-state">
                  <Globe size={28} />
                  <span>启动服务后显示代理入口</span>
                </div>
              )}
            </div>
          </div>

          {/* Config */}
          <div className="cw-card">
            <div className="cw-card-title">服务配置</div>
            <div className="cw-config-grid">
              <div className="cw-field">
                <label><Hash size={13} /> 端口</label>
                <input type="number" value={config.port} onChange={e => setConfig({ ...config, port: parseInt(e.target.value) || 8002 })} disabled={status.running} />
              </div>
              <div className="cw-field">
                <label><Key size={13} /> API Key</label>
                <input type="text" value={config.apiKey} onChange={e => setConfig({ ...config, apiKey: e.target.value })} disabled={status.running} />
              </div>
              <div className="cw-field cw-field-wide">
                <label><BarChart3 size={13} /> 模型</label>
                <input type="text" value={config.models} onChange={e => setConfig({ ...config, models: e.target.value })} disabled={status.running} placeholder="claude-sonnet-4.6,claude-sonnet-4.6-thinking" />
              </div>
              <div className="cw-field">
                <label><Clock size={13} /> 超时 (秒)</label>
                <input type="number" value={config.timeout} onChange={e => setConfig({ ...config, timeout: parseInt(e.target.value) || 60 })} disabled={status.running} />
              </div>
              <div className="cw-field">
                <label><Bug size={13} /> 调试</label>
                <div className="cw-toggle-wrap">
                  <button className={`cw-toggle ${config.debug ? 'on' : ''}`} onClick={() => !status.running && setConfig({ ...config, debug: !config.debug })} disabled={status.running}>
                    <span className="cw-toggle-knob" />
                  </button>
                  <span>{config.debug ? '开' : '关'}</span>
                </div>
              </div>
              <div className="cw-field cw-field-wide">
                <label><Globe size={13} /> Script URL</label>
                <input type="text" value={config.scriptUrl} onChange={e => setConfig({ ...config, scriptUrl: e.target.value })} disabled={status.running} placeholder="cursor.com 的脚本地址" />
              </div>
              <div className="cw-field cw-field-wide">
                <label><Globe size={13} /> User-Agent</label>
                <input type="text" value={config.userAgent} onChange={e => setConfig({ ...config, userAgent: e.target.value })} disabled={status.running} />
              </div>
            </div>
            <div className="cw-config-footer">
              <button className="cw-btn cw-btn-secondary" onClick={handleSaveConfig} disabled={saving || status.running}>
                {saving ? <Loader2 className="animate-spin" size={14} /> : null}
                {saving ? '保存中...' : '保存配置'}
              </button>
              {status.running && <span className="cw-hint-text">停止服务后方可修改</span>}
            </div>
          </div>
        </div>
      )}

      {/* Tab: Quick Config */}
      {activeTab === 'tools' && (
        <div className="cw-tab-content">
          <div className="cw-card">
            <div className="cw-card-title">一键快速配置</div>
            <p className="cw-card-desc">
              将 Cursor 福利的连接信息（API Key + 代理地址）自动写入所选工具的配置文件。
              需要先启动 Cursor 福利服务和 Ai Switch 代理。
            </p>
            <div className="cw-tool-grid">
              {Object.entries(TOOL_LABELS).map(([key, label]) => (
                <label key={key} className={`cw-tool-chip ${selectedTools.includes(key) ? 'selected' : ''}`}>
                  <input type="checkbox" checked={selectedTools.includes(key)} onChange={() => toggleTool(key)} />
                  <span>{label}</span>
                </label>
              ))}
            </div>
            <button className="cw-btn cw-btn-primary" onClick={handleApplyToTools} disabled={applying || selectedTools.length === 0}>
              {applying ? <Loader2 className="animate-spin" size={16} /> : <Zap size={16} />}
              {applying ? '应用中...' : `应用到 ${selectedTools.length} 个工具`}
            </button>
          </div>

          {/* Endpoints for manual copy */}
          <div className="cw-card">
            <div className="cw-card-title">手动配置参考</div>
            <div className="cw-manual-info">
              <div className="cw-info-row">
                <span className="cw-info-label">API Key</span>
                <div className="cw-ep-url">
                  <code>{config.apiKey}</code>
                  <button className="cw-copy-btn" onClick={() => copyToClipboard(config.apiKey, 'apikey')}>
                    {copiedField === 'apikey' ? <CheckCircle size={13} /> : <Copy size={13} />}
                  </button>
                </div>
              </div>
              <div className="cw-info-row">
                <span className="cw-info-label">Base URL (OpenAI)</span>
                <div className="cw-ep-url">
                  <code>{proxyBase}/cursor-welfare</code>
                  <button className="cw-copy-btn" onClick={() => copyToClipboard(`${proxyBase}/cursor-welfare`, 'base-url')}>
                    {copiedField === 'base-url' ? <CheckCircle size={13} /> : <Copy size={13} />}
                  </button>
                </div>
              </div>
              <div className="cw-info-row">
                <span className="cw-info-label">模型</span>
                <div className="cw-ep-url">
                  <code>{config.models}</code>
                  <button className="cw-copy-btn" onClick={() => copyToClipboard(config.models, 'models')}>
                    {copiedField === 'models' ? <CheckCircle size={13} /> : <Copy size={13} />}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab: Share */}
      {activeTab === 'share' && (
        <div className="cw-tab-content">
          <div className="cw-card">
            <div className="cw-card-title">Sub2api 共享</div>
            <p className="cw-card-desc">
              将 Cursor 福利注册为 Sub2api 的上游代理，其他用户可通过 Sub2api 统一入口使用。
              需要先启动 Cursor 福利服务和 Sub2api。
            </p>
            <button className="cw-btn cw-btn-primary" onClick={handleShareToSub2api} disabled={sharing || !status.running}>
              {sharing ? <Loader2 className="animate-spin" size={16} /> : <Share2 size={16} />}
              {sharing ? '注册中...' : '注册到 Sub2api'}
            </button>
            {!status.running && <p className="cw-hint-text" style={{ marginTop: 8 }}>需要先启动 Cursor 福利服务</p>}
            {shareResult?.success && shareResult.shareUrl && (
              <div className="cw-share-result">
                <span className="cw-info-label">可分享地址</span>
                <div className="cw-ep-url">
                  <code>{shareResult.shareUrl}</code>
                  <button className="cw-copy-btn" onClick={() => copyToClipboard(shareResult.shareUrl || '', 'share-url')}>
                    {copiedField === 'share-url' ? <CheckCircle size={13} /> : <Copy size={13} />}
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Footer license */}
      <div className="cw-footer">
        <ExternalLink size={13} />
        <span>cursor2api-go 采用 PolyForm Noncommercial 1.0.0 许可证，仅限非商业用途。</span>
        <a href="https://github.com/libaxuan/cursor2api-go" target="_blank" rel="noopener noreferrer">GitHub</a>
      </div>
    </div>
  );
}

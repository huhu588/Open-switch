import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Sparkles, Loader2, Image } from 'lucide-react';
import { sub2apiClient } from '../../../services/sub2apiClient';
import { useToast } from '../../../hooks/useToast';
import { ToastContainer } from '../../../components/Toast';

export default function UserSora() {
  const { t } = useTranslation();
  const toast = useToast();
  const [prompt, setPrompt] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);

  const handleGenerate = async () => {
    if (!prompt.trim()) return;
    setLoading(true);
    setResult(null);
    try {
      const data = await sub2apiClient.post<{ url?: string; result?: string }>('/sora/generate', { prompt: prompt.trim() });
      if (data && typeof data === 'object') {
        setResult((data as { url?: string; result?: string }).url || (data as { url?: string; result?: string }).result || JSON.stringify(data));
      } else {
        setResult(String(data));
      }
      toast.success(t('sub2api.sora.generateSuccess', '生成完成'));
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.sora.title', 'Sora')}</h2>
      <p className="s2a-page-desc">{t('sub2api.sora.desc', '使用 Sora 生成图像和视频')}</p>

      <div className="s2a-section" style={{ maxWidth: 600 }}>
        <div className="s2a-section-title" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Sparkles size={16} />
          {t('sub2api.sora.prompt', '输入提示词')}
        </div>
        <div style={{ marginTop: 12 }}>
          <textarea
            className="s2a-form-input"
            rows={4}
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder={t('sub2api.sora.placeholder', '描述你想生成的图像或视频...')}
            style={{ width: '100%', resize: 'vertical' }}
          />
        </div>
        <div style={{ marginTop: 12 }}>
          <button className="btn btn-primary btn-sm" onClick={handleGenerate} disabled={loading || !prompt.trim()}>
            {loading ? <Loader2 size={14} className="gw-spin" /> : <Sparkles size={14} />}
            {t('sub2api.sora.generate', '生成')}
          </button>
        </div>
      </div>

      {result && (
        <div className="s2a-section" style={{ marginTop: 16 }}>
          <div className="s2a-section-title" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <Image size={16} />
            {t('sub2api.sora.result', '生成结果')}
          </div>
          {result.startsWith('http') ? (
            <img src={result} alt="Generated" style={{ maxWidth: '100%', borderRadius: 'var(--radius-md)', marginTop: 12 }} />
          ) : (
            <div className="s2a-code-block" style={{ marginTop: 12 }}>{result}</div>
          )}
        </div>
      )}
    </div>
  );
}

import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Layers } from 'lucide-react';
import { PlatformProviderPanel, ProviderModelType } from '../components/platform/PlatformProviderPanel';

const MODEL_TYPES: { id: ProviderModelType; name: string; color: string; gradient: string }[] = [
  { id: 'claude', name: 'Claude', color: '#D97757', gradient: 'linear-gradient(135deg, #D97757, #c2603e)' },
  { id: 'codex', name: 'Codex', color: '#10A37F', gradient: 'linear-gradient(135deg, #10A37F, #0d8a6a)' },
  { id: 'gemini', name: 'Gemini', color: '#3186FF', gradient: 'linear-gradient(135deg, #3186FF, #1a6de0)' },
  { id: 'opencode', name: 'OpenCode', color: '#8B5CF6', gradient: 'linear-gradient(135deg, #8B5CF6, #7c3aed)' },
  { id: 'openclaw', name: 'OpenClaw', color: '#EC4899', gradient: 'linear-gradient(135deg, #EC4899, #db2777)' },
];

export function ProvidersPage() {
  const { t } = useTranslation();
  const [selectedModelType, setSelectedModelType] = useState<ProviderModelType>('claude');

  return (
    <div className="h-full flex flex-col gap-4 p-4">
      <div className="oc-page-header">
        <div className="oc-page-header-left">
          <div className="oc-page-icon oc-page-icon--providers"><Layers size={22} /></div>
          <div>
            <h2 className="oc-page-title">{t('providers.pageTitle', 'Provider 管理')}</h2>
            <p className="oc-page-subtitle">{t('providers.pageSubtitle', '管理 API 提供商，配置模型和端点')}</p>
          </div>
        </div>
      </div>

      <div className="oc-model-type-selector">
        {MODEL_TYPES.map(mt => (
          <button
            key={mt.id}
            className={`oc-model-type-pill ${selectedModelType === mt.id ? 'is-active' : ''}`}
            onClick={() => setSelectedModelType(mt.id)}
            style={selectedModelType === mt.id ? { background: mt.gradient } : {}}
          >
            {mt.name}
          </button>
        ))}
      </div>

      <div className="flex-1 min-h-0">
        <PlatformProviderPanel key={selectedModelType} modelType={selectedModelType} />
      </div>
    </div>
  );
}

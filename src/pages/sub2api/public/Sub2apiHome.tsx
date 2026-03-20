import { useTranslation } from 'react-i18next';
import { Globe, Key, BarChart3, Shield } from 'lucide-react';

export default function Sub2apiHome() {
  const { t } = useTranslation();

  const features = [
    { icon: Globe, title: t('sub2api.home.multiPlatform', '多平台网关'), desc: t('sub2api.home.multiPlatformDesc', '统一接入 OpenAI、Claude、Gemini 等主流 AI 平台') },
    { icon: Key, title: t('sub2api.home.apiManagement', 'API 管理'), desc: t('sub2api.home.apiManagementDesc', '灵活的 API Key 管理与分配系统') },
    { icon: BarChart3, title: t('sub2api.home.usageTracking', '用量追踪'), desc: t('sub2api.home.usageTrackingDesc', '实时监控使用情况，精细化成本控制') },
    { icon: Shield, title: t('sub2api.home.security', '安全可靠'), desc: t('sub2api.home.securityDesc', '本地部署，数据不出境，安全有保障') },
  ];

  return (
    <div>
      <div style={{ textAlign: 'center', padding: '40px 20px' }}>
        <h1 style={{ fontSize: '1.5rem', fontWeight: 800, color: 'var(--text-primary)', marginBottom: 8 }}>
          Sub2api
        </h1>
        <p style={{ fontSize: '0.82rem', color: 'var(--text-muted)', maxWidth: 400, margin: '0 auto' }}>
          {t('sub2api.home.subtitle', '高性能 AI API 网关，一站式管理多平台 AI 服务')}
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))', gap: 16, maxWidth: 800, margin: '0 auto' }}>
        {features.map((f, i) => {
          const Icon = f.icon;
          return (
            <div key={i} className="s2a-stat-card" style={{ textAlign: 'center', padding: 20 }}>
              <Icon size={28} style={{ color: 'var(--primary)', margin: '0 auto 8px' }} />
              <div style={{ fontSize: '0.82rem', fontWeight: 600, color: 'var(--text-primary)', marginBottom: 4 }}>{f.title}</div>
              <div style={{ fontSize: '0.68rem', color: 'var(--text-muted)' }}>{f.desc}</div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

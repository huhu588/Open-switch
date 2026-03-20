import { ReactNode, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Layers, HelpCircle, Router, Settings } from 'lucide-react';
import { CodexIcon } from './icons/CodexIcon';
import { PlatformId } from '../types/platform';
import {
  findGroupByPlatform,
  resolveGroupChildName,
  usePlatformLayoutStore,
} from '../stores/usePlatformLayoutStore';
import { getPlatformLabel } from '../utils/platformMeta';
import { PlatformGroupSwitcher } from './platform/PlatformGroupSwitcher';

export type CodexTab = 'overview' | 'instances' | 'providers' | 'gateway';

interface CodexOverviewTabsHeaderProps {
  active: CodexTab;
  onTabChange?: (tab: CodexTab) => void;
}

interface TabSpec {
  key: CodexTab;
  label: string;
  icon: ReactNode;
}

export function CodexOverviewTabsHeader({
  active,
  onTabChange,
}: CodexOverviewTabsHeaderProps) {
  const { t } = useTranslation();
  const { platformGroups } = usePlatformLayoutStore();
  const currentPlatformId: PlatformId = 'codex';
  const currentGroup = useMemo(
    () => findGroupByPlatform(platformGroups, currentPlatformId),
    [platformGroups, currentPlatformId],
  );
  const switchablePlatforms = currentGroup ? currentGroup.platformIds : [currentPlatformId];
  const currentPlatformLabel = getPlatformLabel(currentPlatformId, t);
  const currentDisplayName = useMemo(
    () =>
      currentGroup
        ? resolveGroupChildName(currentGroup, currentPlatformId, currentPlatformLabel || 'Codex')
        : currentPlatformLabel || 'Codex',
    [currentGroup, currentPlatformId, currentPlatformLabel],
  );
  const switchOptions = useMemo(
    () =>
      switchablePlatforms.map((platformId) => {
        const platformName = currentGroup
          ? resolveGroupChildName(currentGroup, platformId, getPlatformLabel(platformId, t))
          : getPlatformLabel(platformId, t);
        return {
          platformId,
          label: platformName,
        };
      }),
    [switchablePlatforms, currentGroup, t],
  );
  const headerTitle = `Codex ${t('settings.general.accountManagement', '账号管理')}`;
  const tabs: TabSpec[] = [
    {
      key: 'overview',
      label: t('overview.title', '账号总览'),
      icon: <CodexIcon className="tab-icon" />,
    },
    {
      key: 'instances',
      label: t('instances.title', '多开实例'),
      icon: <Layers className="tab-icon" />,
    },
    {
      key: 'providers',
      label: t('providers.tabTitle', 'Provider 配置'),
      icon: <Settings className="tab-icon" />,
    },
    {
      key: 'gateway',
      label: t('codex.tab.gateway', 'API 网关'),
      icon: <Router className="tab-icon" />,
    },
  ];

  const subtitle =
    active === 'instances'
      ? t('instances.subtitle', '多实例独立配置，多账号并行运行。')
      : active === 'providers'
        ? t('providers.tabSubtitle', '管理此平台的 API 提供商和模型配置。')
        : active === 'gateway'
          ? t('codex.tab.gatewaySubtitle', '管理 API 网关服务、账号池、密钥和请求日志。')
          : t('overview.subtitle', '实时监控所有账号的配额状态。');

  return (
    <>
      <div className="page-header">
        <div className="platform-header-title">
          <div className="page-title">{headerTitle}</div>
          <button
            className="btn btn-secondary icon-only platform-header-help"
            onClick={() => window.dispatchEvent(new CustomEvent('app-request-navigate', { detail: 'manual' }))}
            title={t('manual.navTitle', '功能使用手册')}
          >
            <HelpCircle size={18} />
          </button>
        </div>
        <div className="page-subtitle">{subtitle}</div>
      </div>
      <div className="page-tabs-row page-tabs-center page-tabs-row-with-leading">
        <div className="page-tabs-leading">
          <PlatformGroupSwitcher
            currentPlatformId={currentPlatformId}
            currentLabel={currentDisplayName}
            options={switchOptions}
            currentGroupId={currentGroup?.id ?? null}
          />
        </div>
        <div className="page-tabs filter-tabs">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              className={`filter-tab${active === tab.key ? ' active' : ''}`}
              onClick={() => onTabChange?.(tab.key)}
            >
              {tab.icon}
              <span>{tab.label}</span>
            </button>
          ))}
        </div>
      </div>
    </>
  );
}

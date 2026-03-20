import { ReactNode } from 'react';
import { Github } from 'lucide-react';
import { TFunction } from 'i18next';
import { PlatformId } from '../types/platform';
import { RobotIcon } from '../components/icons/RobotIcon';
import { CodexIcon } from '../components/icons/CodexIcon';
import { WindsurfIcon } from '../components/icons/WindsurfIcon';
import { KiroIcon } from '../components/icons/KiroIcon';
import { CursorIcon } from '../components/icons/CursorIcon';
import { GeminiIcon } from '../components/icons/GeminiIcon';
import { CodebuddyIcon } from '../components/icons/CodebuddyIcon';
import { QoderIcon } from '../components/icons/QoderIcon';
import { TraeIcon } from '../components/icons/TraeIcon';
import { WorkbuddyIcon } from '../components/icons/WorkbuddyIcon';
import { ClaudeCodeIcon } from '../components/icons/ClaudeCodeIcon';
import { OpenCodeIcon } from '../components/icons/OpenCodeIcon';
import { OpenClawIcon } from '../components/icons/OpenClawIcon';
import { WarpIcon } from '../components/icons/WarpIcon';
import { AugmentIcon } from '../components/icons/AugmentIcon';

export function getPlatformLabel(platformId: PlatformId, _t: TFunction): string {
  switch (platformId) {
    case 'antigravity':
      return 'Antigravity';
    case 'codex':
      return 'Codex';
    case 'github-copilot':
      return 'GitHub Copilot';
    case 'windsurf':
      return 'Windsurf';
    case 'kiro':
      return 'Kiro';
    case 'cursor':
      return 'Cursor';
    case 'gemini':
      return 'Gemini';
    case 'codebuddy':
      return 'CodeBuddy';
    case 'codebuddy_cn':
      return _t('nav.codebuddyCn', 'CodeBuddy CN');
    case 'qoder':
      return _t('nav.qoder', 'Qoder');
    case 'trae':
      return _t('nav.trae', 'Trae');
    case 'workbuddy':
      return 'WorkBuddy';
    case 'claude-code':
      return 'Claude Code';
    case 'opencode':
      return 'OpenCode';
    case 'openclaw':
      return 'OpenClaw';
    case 'warp':
      return 'Warp';
    case 'augment':
      return 'Augment';
    default:
      return platformId;
  }
}

export function renderPlatformIcon(platformId: PlatformId, size = 20): ReactNode {
  switch (platformId) {
    case 'antigravity':
      return <RobotIcon style={{ width: size, height: size }} />;
    case 'codex':
      return <CodexIcon size={size} />;
    case 'github-copilot':
      return <Github size={size} />;
    case 'windsurf':
      return <WindsurfIcon style={{ width: size, height: size }} />;
    case 'kiro':
      return <KiroIcon style={{ width: size, height: size }} />;
    case 'cursor':
      return <CursorIcon style={{ width: size, height: size }} />;
    case 'gemini':
      return <GeminiIcon style={{ width: size, height: size }} />;
    case 'codebuddy':
      return <CodebuddyIcon style={{ width: size, height: size }} />;
    case 'codebuddy_cn':
      return <CodebuddyIcon style={{ width: size, height: size }} />;
    case 'qoder':
      return <QoderIcon style={{ width: size, height: size }} />;
    case 'trae':
      return <TraeIcon style={{ width: size, height: size }} />;
    case 'workbuddy':
      return <WorkbuddyIcon style={{ width: size, height: size }} />;
    case 'claude-code':
      return <ClaudeCodeIcon style={{ width: size, height: size }} />;
    case 'opencode':
      return <OpenCodeIcon style={{ width: size, height: size }} />;
    case 'openclaw':
      return <OpenClawIcon style={{ width: size, height: size }} />;
    case 'warp':
      return <WarpIcon style={{ width: size, height: size }} />;
    case 'augment':
      return <AugmentIcon style={{ width: size, height: size }} />;
    default:
      return null;
  }
}

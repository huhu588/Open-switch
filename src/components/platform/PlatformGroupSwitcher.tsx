import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { Check, ChevronDown, Pencil } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ALL_PLATFORM_IDS, PLATFORM_PAGE_MAP, PlatformId } from '../../types/platform';
import { getPlatformLabel, renderPlatformIcon } from '../../utils/platformMeta';
import {
  findGroupByPlatform,
  resolveGroupChildName,
  usePlatformLayoutStore,
  PlatformLayoutGroup,
} from '../../stores/usePlatformLayoutStore';

export interface PlatformGroupSwitcherOption {
  platformId: PlatformId;
  label: string;
}

interface PlatformGroupSwitcherProps {
  currentPlatformId: PlatformId;
  currentLabel: string;
  options?: PlatformGroupSwitcherOption[];
  currentGroupId?: string | null;
}

interface SwitcherEntry {
  type: 'platform';
  platformId: PlatformId;
  label: string;
}

interface SwitcherGroupEntry {
  type: 'group';
  group: PlatformLayoutGroup;
  children: { platformId: PlatformId; label: string }[];
}

type SwitcherItem = SwitcherEntry | SwitcherGroupEntry;

export function PlatformGroupSwitcher({
  currentPlatformId,
  currentLabel,
  currentGroupId = null,
}: PlatformGroupSwitcherProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const dropdownRef = useRef<HTMLDivElement | null>(null);
  const [pos, setPos] = useState({ top: 0, left: 0, maxHeight: 0 });

  const { platformGroups } = usePlatformLayoutStore();

  const switcherItems = useMemo<SwitcherItem[]>(() => {
    const visited = new Set<PlatformId>();
    const items: SwitcherItem[] = [];

    for (const pid of ALL_PLATFORM_IDS) {
      if (visited.has(pid)) continue;
      const group = findGroupByPlatform(platformGroups, pid);
      if (group && group.platformIds.length > 1) {
        for (const gp of group.platformIds) visited.add(gp);
        items.push({
          type: 'group',
          group,
          children: group.platformIds.map((gpId) => ({
            platformId: gpId,
            label: resolveGroupChildName(group, gpId, getPlatformLabel(gpId, t)),
          })),
        });
      } else {
        visited.add(pid);
        items.push({
          type: 'platform',
          platformId: pid,
          label: getPlatformLabel(pid, t),
        });
      }
    }
    return items;
  }, [platformGroups, t]);

  const reposition = useCallback(() => {
    const el = triggerRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const dd = dropdownRef.current;
    const dw = dd?.offsetWidth ?? 240;
    const dh = dd?.offsetHeight ?? 400;
    const viewH = window.innerHeight;
    const viewW = window.innerWidth;
    const gap = 8;
    const spaceBelow = viewH - r.bottom - gap;
    const spaceAbove = r.top - gap;
    let top: number;
    if (spaceBelow >= dh || spaceBelow >= spaceAbove) {
      top = Math.round(r.bottom + gap);
    } else {
      top = Math.round(Math.max(gap, r.top - dh - gap));
    }
    const maxH = Math.max(200, (top <= r.top ? spaceAbove : spaceBelow) - gap);
    setPos({
      top,
      left: Math.round(Math.max(gap, Math.min(r.left, viewW - dw - gap))),
      maxHeight: maxH,
    });
  }, []);

  useEffect(() => {
    if (!open) return;

    reposition();
    const frame = requestAnimationFrame(reposition);
    const onResize = () => reposition();
    window.addEventListener('resize', onResize);
    window.addEventListener('scroll', onResize, true);

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    document.addEventListener('keydown', onKey);

    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener('resize', onResize);
      window.removeEventListener('scroll', onResize, true);
      document.removeEventListener('keydown', onKey);
    };
  }, [open, reposition]);

  const close = useCallback(() => setOpen(false), []);

  const handleSwitch = useCallback(
    (nextPlatform: PlatformId) => {
      setOpen(false);
      if (nextPlatform === currentPlatformId) return;
      window.dispatchEvent(
        new CustomEvent('app-request-navigate', { detail: PLATFORM_PAGE_MAP[nextPlatform] }),
      );
    },
    [currentPlatformId],
  );

  const handleManage = useCallback(() => {
    setOpen(false);
    window.dispatchEvent(
      new CustomEvent('app-open-platform-layout', { detail: { groupId: currentGroupId } }),
    );
  }, [currentGroupId]);

  const renderOption = (pid: PlatformId, label: string, isChild = false) => {
    const active = pid === currentPlatformId;
    return (
      <button
        key={pid}
        type="button"
        className={`platform-group-switcher-option${isChild ? ' platform-group-switcher-option-child' : ''}${active ? ' is-active' : ''}`}
        role="option"
        aria-selected={active}
        onClick={() => handleSwitch(pid)}
      >
        <span className="platform-group-switcher-option-icon">
          {renderPlatformIcon(pid, 18)}
        </span>
        <span className="platform-group-switcher-option-label">{label}</span>
        <span className="platform-group-switcher-option-check">
          {active ? <Check size={16} /> : null}
        </span>
      </button>
    );
  };

  return (
    <div className="platform-group-switcher">
      <button
        type="button"
        className={`platform-group-switcher-trigger ${open ? 'is-open' : ''}`}
        ref={triggerRef}
        onClick={() => setOpen((v) => !v)}
        aria-label={t('platformLayout.groupSwitchLabel', '切换平台')}
        aria-haspopup="listbox"
        aria-expanded={open}
      >
        <span className="platform-group-switcher-trigger-icon">
          {renderPlatformIcon(currentPlatformId, 16)}
        </span>
        <span className="platform-group-switcher-trigger-label">{currentLabel}</span>
        <ChevronDown size={16} className="platform-group-switcher-trigger-caret" />
      </button>

      {open &&
        createPortal(
          <>
            {/* backdrop captures outside clicks */}
            <div
              className="platform-group-switcher-backdrop"
              onClick={close}
              onMouseDown={(e) => e.preventDefault()}
            />
            <div
              className="platform-group-switcher-dropdown"
              role="listbox"
              ref={dropdownRef}
              style={{ top: pos.top, left: pos.left, maxHeight: pos.maxHeight > 0 ? pos.maxHeight : undefined }}
            >
              <div className="platform-group-switcher-section-label">
                {t('platformLayout.morePlatforms', '更多平台')}
              </div>

              {switcherItems.map((item) => {
                if (item.type === 'platform') {
                  return renderOption(item.platformId, item.label);
                }
                const grp = item.group;
                const iconPid =
                  grp.iconPlatformId && grp.platformIds.includes(grp.iconPlatformId)
                    ? grp.iconPlatformId
                    : grp.defaultPlatformId;
                const groupActive = grp.platformIds.includes(currentPlatformId);
                return (
                  <div key={`g-${grp.id}`} className="platform-group-switcher-group">
                    <div
                      className={`platform-group-switcher-group-header${groupActive ? ' is-group-active' : ''}`}
                    >
                      <span className="platform-group-switcher-option-icon">
                        {renderPlatformIcon(iconPid, 18)}
                      </span>
                      <span className="platform-group-switcher-group-name">{grp.name}</span>
                    </div>
                    {item.children.map((c) => renderOption(c.platformId, c.label, true))}
                  </div>
                );
              })}

              <div className="platform-group-switcher-divider" />
              <button
                type="button"
                className="platform-group-switcher-action"
                onClick={handleManage}
              >
                <span className="platform-group-switcher-action-icon">
                  <Pencil size={16} />
                </span>
                <span className="platform-group-switcher-action-label">
                  {t('accounts.groups.manageTitle', '管理平台布局')}
                </span>
              </button>
            </div>
          </>,
          document.body,
        )}
    </div>
  );
}

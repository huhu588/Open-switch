import { CSSProperties } from 'react';

type OpenCodeIconProps = {
  className?: string;
  style?: CSSProperties;
};

export function OpenCodeIcon({ className = 'nav-item-icon', style }: OpenCodeIconProps) {
  return (
    <svg
      className={className}
      style={style}
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <path
        d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0L19.2 12l-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"
        fill="currentColor"
      />
      <rect x="10.5" y="3" width="2.5" height="18" rx="1.25" transform="rotate(15 12 12)" fill="currentColor" opacity="0.5" />
    </svg>
  );
}

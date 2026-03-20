import { CSSProperties } from 'react';

type ClaudeCodeIconProps = {
  className?: string;
  style?: CSSProperties;
};

export function ClaudeCodeIcon({ className = 'nav-item-icon', style }: ClaudeCodeIconProps) {
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
        d="M15.483 6.33l-3.69 12.14h-2.656L12.828 6.33h2.655zm-6.478 0L5.314 18.47H2.67L6.35 6.33h2.655zM21.33 6.33l-3.69 12.14h-2.655L18.675 6.33h2.655z"
        fill="#D97757"
      />
    </svg>
  );
}

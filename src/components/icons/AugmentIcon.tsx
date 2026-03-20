import { CSSProperties } from 'react';

type AugmentIconProps = {
  className?: string;
  style?: CSSProperties;
};

export function AugmentIcon({ className = 'nav-item-icon', style }: AugmentIconProps) {
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
      <circle cx="12" cy="12" r="8" stroke="#E11D48" strokeWidth="1.5" opacity="0.3" />
      <circle cx="12" cy="12" r="4" fill="#E11D48" opacity="0.6" />
      <path
        d="M12 2v4M12 18v4M2 12h4M18 12h4"
        stroke="#E11D48"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
      <path
        d="M5.64 5.64l2.83 2.83M15.54 15.54l2.83 2.83M5.64 18.36l2.83-2.83M15.54 8.46l2.83-2.83"
        stroke="#E11D48"
        strokeWidth="1"
        strokeLinecap="round"
        opacity="0.5"
      />
    </svg>
  );
}

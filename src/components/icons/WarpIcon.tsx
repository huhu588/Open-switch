import { CSSProperties } from 'react';

type WarpIconProps = {
  className?: string;
  style?: CSSProperties;
};

export function WarpIcon({ className = 'nav-item-icon', style }: WarpIconProps) {
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
        d="M4 4h16v12a4 4 0 01-4 4H8a4 4 0 01-4-4V4z"
        fill="#01A4FF"
        opacity="0.15"
      />
      <path
        d="M7 8h10M7 12h7"
        stroke="#01A4FF"
        strokeWidth="2"
        strokeLinecap="round"
      />
      <path
        d="M4 4h16v12a4 4 0 01-4 4H8a4 4 0 01-4-4V4z"
        stroke="#01A4FF"
        strokeWidth="1.5"
        fill="none"
      />
      <path
        d="M7 16l1.5-1.5L10 16"
        stroke="#01A4FF"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

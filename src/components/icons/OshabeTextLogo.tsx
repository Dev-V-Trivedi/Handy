import React from "react";

const BRAND_NAME = "Oshabe";

/**
 * Oshabe logo lockup: speech-bubble symbol plus lowercase wordmark.
 * Colors follow the theme tokens (see docs/BRAND.md). The spark stays amber
 * in both themes; the bubble and bars flip with --color-logo-primary and
 * --color-logo-accent.
 */
const OshabeTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="190 190 1720 640"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label={BRAND_NAME}
    >
      <g style={{ fill: "var(--color-logo-primary)" }}>
        <circle cx="490" cy="505" r="255" />
        <path d="M312 660 L238 805 Q232 818 247 815 L408 740 Z" />
      </g>
      <g style={{ fill: "var(--color-logo-accent)" }}>
        <rect x="324" y="449" width="60" height="120" rx="30" />
        <rect x="460" y="355" width="60" height="300" rx="30" />
        <rect x="596" y="429" width="60" height="160" rx="30" />
      </g>
      <circle cx="742" cy="227" r="36" fill="#FFB224" />
      <text
        x="800"
        y="640"
        fontSize="340"
        fontWeight="600"
        fontFamily="Fredoka, ui-rounded, system-ui, sans-serif"
        style={{ fill: "var(--color-logo-primary)" }}
      >
        {BRAND_NAME.toLowerCase()}
      </text>
    </svg>
  );
};

export default OshabeTextLogo;

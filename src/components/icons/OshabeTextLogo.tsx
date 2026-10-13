import React from "react";

const BRAND_NAME = "Oshabe";

/**
 * Interim text wordmark. Replace with the final Oshabe logo when it is
 * designed (see docs/REBRAND-NOTES.md).
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
      viewBox="0 0 360 100"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label={BRAND_NAME}
    >
      <text
        x="0"
        y="76"
        fontSize="84"
        fontWeight="800"
        fontFamily="ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif"
        letterSpacing="-2"
        style={{ fill: "var(--color-logo-primary)" }}
      >
        {BRAND_NAME}
      </text>
    </svg>
  );
};

export default OshabeTextLogo;

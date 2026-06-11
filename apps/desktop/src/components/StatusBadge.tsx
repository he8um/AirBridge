export type BadgeStatus = "idle" | "connected" | "warning" | "error";

interface StatusBadgeProps {
  status: BadgeStatus;
  label: string;
}

const DOT_COLORS: Record<BadgeStatus, string> = {
  idle: "var(--color-idle)",
  connected: "var(--color-success)",
  warning: "var(--color-warning)",
  error: "var(--color-danger)",
};

const BADGE_STYLES: Record<BadgeStatus, React.CSSProperties> = {
  idle: {
    backgroundColor: "var(--color-idle-light)",
    color: "var(--color-idle)",
    border: "1px solid var(--color-border)",
  },
  connected: {
    backgroundColor: "var(--color-success-light)",
    color: "var(--color-success)",
    border: "1px solid #bbf7d0",
  },
  warning: {
    backgroundColor: "var(--color-warning-light)",
    color: "var(--color-warning)",
    border: "1px solid #fde68a",
  },
  error: {
    backgroundColor: "var(--color-danger-light)",
    color: "var(--color-danger)",
    border: "1px solid #fecaca",
  },
};

export function StatusBadge({ status, label }: StatusBadgeProps) {
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "var(--space-2)",
        padding: "2px var(--space-2)",
        borderRadius: "var(--radius-full)",
        fontSize: "var(--text-xs)",
        fontWeight: 500,
        ...BADGE_STYLES[status],
      }}
      aria-label={`Status: ${label}`}
    >
      <span
        style={{
          width: 6,
          height: 6,
          borderRadius: "50%",
          backgroundColor: DOT_COLORS[status],
          flexShrink: 0,
        }}
        aria-hidden="true"
      />
      {label}
    </span>
  );
}

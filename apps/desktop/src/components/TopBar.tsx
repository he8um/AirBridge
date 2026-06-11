import type { NavItem } from "../app/navigation";

interface TopBarProps {
  item: NavItem;
}

export function TopBar({ item }: TopBarProps) {
  return (
    <header
      style={{
        height: "var(--topbar-height)",
        backgroundColor: "var(--color-surface)",
        borderBottom: "1px solid var(--color-border)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0 var(--space-8)",
        flexShrink: 0,
      }}
      aria-label="Page header"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
        <h1 style={{ fontSize: "var(--text-md)", fontWeight: 600 }}>{item.label}</h1>
        <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
          {item.description}
        </p>
      </div>

      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--space-2)",
          padding: "var(--space-2) var(--space-3)",
          borderRadius: "var(--radius-full)",
          backgroundColor: "var(--color-accent-light)",
          border: "1px solid #bfdbfe",
          fontSize: "var(--text-xs)",
          color: "var(--color-accent-text)",
          fontWeight: 500,
          userSelect: "none",
        }}
        title="All operations run entirely on your device"
      >
        {/* Lock icon */}
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0110 0v4" />
        </svg>
        All data stays on your device
      </div>
    </header>
  );
}

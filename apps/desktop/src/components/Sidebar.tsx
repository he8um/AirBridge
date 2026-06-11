import { NAVIGATION } from "../app/navigation";
import type { PageId } from "../app/navigation";

interface SidebarProps {
  activePage: PageId;
  onNavigate: (page: PageId) => void;
}

export function Sidebar({ activePage, onNavigate }: SidebarProps) {
  return (
    <aside
      style={{
        width: "var(--sidebar-width)",
        minWidth: "var(--sidebar-width)",
        backgroundColor: "var(--color-sidebar-bg)",
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        overflow: "hidden",
        flexShrink: 0,
      }}
      aria-label="Main navigation"
    >
      {/* Wordmark */}
      <div
        style={{
          height: "var(--topbar-height)",
          display: "flex",
          alignItems: "center",
          padding: "0 var(--space-5)",
          borderBottom: "1px solid #334155",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: "var(--text-md)",
            fontWeight: 700,
            color: "#f1f5f9",
            letterSpacing: "-0.02em",
            display: "flex",
            alignItems: "center",
            gap: "var(--space-2)",
          }}
        >
          {/* Bridge / transfer icon */}
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="var(--color-accent)"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
          </svg>
          AirBridge
        </span>
      </div>

      {/* Nav items */}
      <nav
        style={{ flex: 1, overflowY: "auto", padding: "var(--space-3) var(--space-2)" }}
        aria-label="Primary navigation"
      >
        <ul style={{ listStyle: "none", display: "flex", flexDirection: "column", gap: 2 }}>
          {NAVIGATION.map((item) => {
            const isActive = item.id === activePage;
            return (
              <li key={item.id}>
                <button
                  type="button"
                  onClick={() => onNavigate(item.id)}
                  aria-current={isActive ? "page" : undefined}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--space-3)",
                    width: "100%",
                    padding: "var(--space-2) var(--space-3)",
                    borderRadius: "var(--radius-md)",
                    border: "none",
                    cursor: "pointer",
                    fontSize: "var(--text-sm)",
                    fontWeight: isActive ? 600 : 400,
                    fontFamily: "var(--font-sans)",
                    color: isActive
                      ? "var(--color-sidebar-active-text)"
                      : "var(--color-sidebar-text)",
                    backgroundColor: isActive ? "var(--color-sidebar-active-bg)" : "transparent",
                    textAlign: "left",
                    transition: "background-color 0.12s ease, color 0.12s ease",
                  }}
                  onMouseEnter={(e) => {
                    if (!isActive) {
                      (e.currentTarget as HTMLButtonElement).style.backgroundColor =
                        "var(--color-sidebar-hover-bg)";
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (!isActive) {
                      (e.currentTarget as HTMLButtonElement).style.backgroundColor = "transparent";
                    }
                  }}
                >
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                    style={{ flexShrink: 0, opacity: isActive ? 1 : 0.7 }}
                  >
                    <path d={item.icon} />
                  </svg>
                  {item.label}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Footer */}
      <div
        style={{
          padding: "var(--space-4) var(--space-5)",
          borderTop: "1px solid #334155",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-sidebar-text-muted)",
          }}
        >
          v0.1.0
        </span>
      </div>
    </aside>
  );
}

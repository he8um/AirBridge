interface StatCardProps {
  label: string;
  value: string | number;
  note?: string;
}

export function StatCard({ label, value, note }: StatCardProps) {
  return (
    <div
      className="card"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
    >
      <span
        style={{
          fontSize: "var(--text-xs)",
          fontWeight: 500,
          color: "var(--color-text-muted)",
          textTransform: "uppercase",
          letterSpacing: "0.06em",
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontSize: "var(--text-2xl)",
          fontWeight: 700,
          color: "var(--color-text)",
          lineHeight: 1,
        }}
      >
        {value}
      </span>
      {note && (
        <span
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-text-subtle)",
          }}
        >
          {note}
        </span>
      )}
    </div>
  );
}

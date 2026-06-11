interface SectionAction {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}

interface SectionHeaderProps {
  title: string;
  action?: SectionAction;
}

export function SectionHeader({ title, action }: SectionHeaderProps) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        marginBottom: "var(--space-4)",
      }}
    >
      <h2>{title}</h2>
      {action && (
        <button
          className="btn btn-secondary btn-sm"
          onClick={action.onClick}
          disabled={action.disabled}
          type="button"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}

import type { PermissionCheck, PermissionCheckStatus } from "../../domain/connection";
import { StatusBadge } from "../../components/StatusBadge";
import type { BadgeStatus } from "../../components/StatusBadge";

interface PermissionCheckListProps {
  checks: PermissionCheck[];
  title?: string;
}

function toBadgeStatus(status: PermissionCheckStatus): BadgeStatus {
  switch (status) {
    case "passed":
      return "connected";
    case "failed":
      return "error";
    case "checking":
      return "warning";
    default:
      return "idle";
  }
}

function toBadgeLabel(status: PermissionCheckStatus): string {
  switch (status) {
    case "passed":
      return "Passed";
    case "failed":
      return "Failed";
    case "checking":
      return "Checking…";
    default:
      return "—";
  }
}

export function PermissionCheckList({ checks, title }: PermissionCheckListProps) {
  return (
    <div>
      {title && <p className="permission-check-list-title">{title}</p>}
      <ul role="list" aria-label={title ?? "Permission checks"} className="permission-check-list">
        {checks.map((check) => (
          <li key={check.key} className="permission-check-item">
            <span className="permission-check-label">{check.label}</span>
            <StatusBadge status={toBadgeStatus(check.status)} label={toBadgeLabel(check.status)} />
          </li>
        ))}
      </ul>
    </div>
  );
}

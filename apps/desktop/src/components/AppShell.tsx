import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { NAVIGATION } from "../app/navigation";
import type { PageId } from "../app/navigation";
import { DashboardPage } from "../pages/DashboardPage";
import { ConnectionsPage } from "../pages/ConnectionsPage";
import { BackupsPage } from "../pages/BackupsPage";
import { RestorePage } from "../pages/RestorePage";
import { ReportsPage } from "../pages/ReportsPage";
import { SettingsPage } from "../pages/SettingsPage";
import { LogsPage } from "../pages/LogsPage";

interface AppShellProps {
  activePage: PageId;
  onNavigate: (page: PageId) => void;
}

const PAGE_COMPONENTS: Record<PageId, React.ComponentType> = {
  dashboard: DashboardPage,
  connections: ConnectionsPage,
  backups: BackupsPage,
  restore: RestorePage,
  reports: ReportsPage,
  settings: SettingsPage,
  logs: LogsPage,
};

export function AppShell({ activePage, onNavigate }: AppShellProps) {
  const ActivePage = PAGE_COMPONENTS[activePage];
  const navItem = NAVIGATION.find((n) => n.id === activePage)!;

  return (
    <div
      style={{
        display: "flex",
        height: "100vh",
        overflow: "hidden",
        backgroundColor: "var(--color-bg)",
      }}
    >
      <Sidebar activePage={activePage} onNavigate={onNavigate} />

      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          minWidth: 0,
        }}
      >
        <TopBar item={navItem} />
        <main
          style={{
            flex: 1,
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
          aria-label={`${navItem.label} content`}
        >
          <ActivePage />
        </main>
      </div>
    </div>
  );
}

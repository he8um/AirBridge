import { useState } from "react";
import { AppShell } from "../components/AppShell";
import type { PageId } from "./navigation";

export function App() {
  const [activePage, setActivePage] = useState<PageId>("dashboard");

  return <AppShell activePage={activePage} onNavigate={setActivePage} />;
}

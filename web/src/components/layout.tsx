import type { ReactNode } from "react";

import { Sidebar } from "@/components/sidebar";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "./theme-toggle";

export function Layout({ children }: { children: ReactNode }) {
  return (
    <SidebarProvider>
      <Sidebar />
      <main className="relative flex-1 min-h-screen">
        <SidebarTrigger className="absolute top-2 left-2 size-9" />
        <ThemeToggle className="absolute top-2 right-2" />

        {children}
      </main>
    </SidebarProvider>
  );
}

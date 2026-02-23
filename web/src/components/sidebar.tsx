import { Link } from "@tanstack/react-router";
import { CatIcon, HashIcon } from "lucide-react";

import Logo from "@/assets/logo.png";
import {
  Sidebar as ScnSidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";

export function Sidebar() {
  return (
    <ScnSidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex items-center">
          <img
            src={Logo}
            alt="Hermes"
            className="w-12 h-12 transition-all group-data-[collapsible=icon]:w-8 group-data-[collapsible=icon]:h-8"
          />

          <div className="ml-2 flex flex-col whitespace-nowrap group-data-[collapsible=icon]:hidden">
            <span className="font-serif text-lg font-black leading-none tracking-tight">
              Hermes
            </span>
            <span className="text-xs font-light">with ❤️ by Bryan Bush</span>
          </div>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Games</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem key="boop">
                <SidebarMenuButton asChild>
                  <Link to="/games/boop">
                    <CatIcon />
                    <span>Boop</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem key="tic-tac-toe">
                <SidebarMenuButton asChild>
                  <Link to="/games/tic-tac-toe">
                    <HashIcon />
                    <span>Tic-Tac-Toe</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </ScnSidebar>
  );
}

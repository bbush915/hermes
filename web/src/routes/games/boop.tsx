import { createFileRoute } from "@tanstack/react-router";

import { Boop } from "@/components/boop";

export const Route = createFileRoute("/games/boop")({
  component: RouteComponent,
});

function RouteComponent() {
  return <Boop />;
}

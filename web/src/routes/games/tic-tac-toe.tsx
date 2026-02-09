import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/games/tic-tac-toe")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/games/tic-tac-toe"!</div>;
}

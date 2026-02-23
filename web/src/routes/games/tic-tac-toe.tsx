import { createFileRoute } from "@tanstack/react-router";

import { TicTacToe } from "@/components/tic-tac-toe";

export const Route = createFileRoute("/games/tic-tac-toe")({
  component: RouteComponent,
});

function RouteComponent() {
  return <TicTacToe />;
}

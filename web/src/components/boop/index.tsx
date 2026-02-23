import { useEffect, useRef, useState } from "react";

import {
  PlayerKind,
  Boop as WasmBoop,
  BoopPhase as Phase,
  BoopSquare as Square,
  Outcome,
  Turn,
} from "@/wasm/hermes_wasm";

import { Board } from "./board";

type PlaceAction = { type: "place"; square: number; is_cat: boolean };
type GraduateAction = { type: "graduate"; squares: number[] };
type BoopAction = PlaceAction | GraduateAction;

export function Boop() {
  const gameRef = useRef<WasmBoop | null>(null);

  const [board, setBoard] = useState<Square[]>([]);
  const [phase, setPhase] = useState<Phase>(Phase.Place);
  const [possibleActions, setPossibleActions] = useState<BoopAction[]>([]);
  const [isCat, setIsCat] = useState(false);
  const [result, setResult] = useState<string | null>(null);

  function syncState() {
    const game = gameRef.current!;
    setBoard([...game.board]);
    setPhase(game.phase);
  }

  useEffect(() => {
    const game = new WasmBoop();
    game.player_1 = PlayerKind.Manual;
    game.player_2 = PlayerKind.Manual;
    gameRef.current = game;

    syncState();
    setPossibleActions(game.get_possible_actions() as BoopAction[]);

    return () => game.free();
  }, []);

  function isManualTurn(): boolean {
    const game = gameRef.current!;
    const kind =
      game.turn === Turn.Player1 ? game.player_1_kind : game.player_2_kind;
    return kind === PlayerKind.Manual;
  }

  function winnerLabel(): string {
    return gameRef.current!.turn === Turn.Player1
      ? "Player 1 wins!"
      : "Player 2 wins!";
  }

  function stepAI() {
    const game = gameRef.current!;

    while (!isManualTurn()) {
      const outcome = game.step();

      if (outcome !== Outcome.InProgress) {
        setResult(outcome === Outcome.Win ? winnerLabel() : "Draw!");
        setBoard([...game.board]);
        return;
      }
    }

    syncState();
    setPossibleActions(game.get_possible_actions() as BoopAction[]);
  }

  function handleSquareClick(index: number) {
    const game = gameRef.current!;
    if (result || !isManualTurn() || game.phase !== Phase.Place) return;

    const action = possibleActions.find(
      (a): a is PlaceAction =>
        a.type === "place" && a.square === index && a.is_cat === isCat,
    );
    if (!action) return;

    game.queue_action(action);
    const outcome = game.step();

    if (outcome !== Outcome.InProgress) {
      setResult(outcome === Outcome.Win ? winnerLabel() : "Draw!");
      setBoard([...game.board]);
      return;
    }

    syncState();

    stepAI();
  }

  function handleGraduateAction(action: GraduateAction) {
    const game = gameRef.current!;

    game.queue_action(action);
    const outcome = game.step();

    if (outcome !== Outcome.InProgress) {
      setResult(outcome === Outcome.Win ? winnerLabel() : "Draw!");
      setBoard([...game.board]);
      return;
    }

    stepAI();
  }

  const validSquares =
    phase === Phase.Place
      ? new Set(
          possibleActions
            .filter(
              (a): a is PlaceAction => a.type === "place" && a.is_cat === isCat,
            )
            .map((a) => a.square),
        )
      : new Set<number>();

  const graduateActions = possibleActions.filter(
    (a): a is GraduateAction => a.type === "graduate",
  );

  return (
    <div className="flex gap-8 p-16 items-start">
      <Board
        board={board}
        validSquares={validSquares}
        onSquareClick={handleSquareClick}
      />

      <div className="flex flex-col gap-4 pt-2">
        {result ? (
          <div className="text-lg font-semibold">{result}</div>
        ) : phase === Phase.Graduate ? (
          <div className="flex flex-col gap-2">
            <div className="text-sm font-medium text-muted-foreground">
              Choose pieces to graduate:
            </div>
            {graduateActions.map((action, i) => (
              <button
                key={i}
                onClick={() => handleGraduateAction(action)}
                className="px-3 py-1.5 rounded bg-secondary hover:bg-secondary/80 text-sm text-left"
              >
                {action.squares
                  .map((s) => `(${Math.floor(s / 6) + 1}, ${(s % 6) + 1})`)
                  .join("  ")}
              </button>
            ))}
          </div>
        ) : (
          <div className="flex flex-col gap-2">
            <div className="text-sm font-medium text-muted-foreground">
              Place a piece:
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => setIsCat(false)}
                className={`px-3 py-1.5 rounded text-sm ${!isCat ? "bg-primary text-primary-foreground" : "bg-secondary hover:bg-secondary/80"}`}
              >
                Kitten
              </button>
              <button
                onClick={() => setIsCat(true)}
                className={`px-3 py-1.5 rounded text-sm ${isCat ? "bg-primary text-primary-foreground" : "bg-secondary hover:bg-secondary/80"}`}
              >
                Cat
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

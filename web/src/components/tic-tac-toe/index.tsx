import { useEffect, useRef, useState } from "react";

import {
  Outcome,
  PlayerKind,
  TicTacToe as WasmTicTacToe,
  TicTacToeSquare,
  Turn,
} from "@/wasm/hermes_wasm";

type PlaceAction = { type: "place"; index: number };

const PLAYER_LABELS: Record<PlayerKind, string> = {
  [PlayerKind.Manual]: "Human",
  [PlayerKind.Random]: "Random",
  [PlayerKind.Minimax]: "Minimax",
};

const ALL_KINDS = [PlayerKind.Manual, PlayerKind.Random, PlayerKind.Minimax];

export function TicTacToe() {
  const gameRef = useRef<WasmTicTacToe | null>(null);
  const aiTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [board, setBoard] = useState<TicTacToeSquare[]>([]);
  const [possibleActions, setPossibleActions] = useState<PlaceAction[]>([]);
  const [result, setResult] = useState<string | null>(null);
  const [p1Kind, setP1Kind] = useState<PlayerKind>(PlayerKind.Manual);
  const [p2Kind, setP2Kind] = useState<PlayerKind>(PlayerKind.Minimax);

  function isManualTurn(game: WasmTicTacToe): boolean {
    const kind = game.turn === Turn.Player1 ? game.player_1_kind : game.player_2_kind;
    return kind === PlayerKind.Manual;
  }

  function winnerLabel(game: WasmTicTacToe): string {
    return game.turn === Turn.Player1 ? "Player 1 wins!" : "Player 2 wins!";
  }

  function stepAI(game: WasmTicTacToe) {
    if (isManualTurn(game) || game.outcome !== Outcome.InProgress) return;

    const outcome = game.step();

    if (outcome !== Outcome.InProgress) {
      setResult(outcome === Outcome.Win ? winnerLabel(game) : "Draw!");
      setBoard([...game.board]);
      return;
    }

    setBoard([...game.board]);
    setPossibleActions(game.get_possible_actions() as PlaceAction[]);

    if (!isManualTurn(game)) {
      aiTimerRef.current = setTimeout(() => stepAI(game), 500);
    }
  }

  function startGame(p1: PlayerKind, p2: PlayerKind) {
    if (aiTimerRef.current) clearTimeout(aiTimerRef.current);
    gameRef.current?.free();
    gameRef.current = null;

    const game = new WasmTicTacToe();
    game.player_1 = p1;
    game.player_2 = p2;
    gameRef.current = game;

    setBoard([...game.board]);
    setPossibleActions(game.get_possible_actions() as PlaceAction[]);
    setResult(null);

    if (!isManualTurn(game)) {
      aiTimerRef.current = setTimeout(() => stepAI(game), 500);
    }
  }

  useEffect(() => {
    startGame(p1Kind, p2Kind);
    return () => {
      if (aiTimerRef.current) clearTimeout(aiTimerRef.current);
      gameRef.current?.free();
      gameRef.current = null;
    };
  }, []);

  function handleSquareClick(index: number) {
    const game = gameRef.current!;
    if (result || !isManualTurn(game)) return;

    const action = possibleActions.find((a) => a.index === index);
    if (!action) return;

    game.queue_action(action);
    const outcome = game.step();

    if (outcome !== Outcome.InProgress) {
      setResult(outcome === Outcome.Win ? winnerLabel(game) : "Draw!");
      setBoard([...game.board]);
      return;
    }

    setBoard([...game.board]);
    setPossibleActions(game.get_possible_actions() as PlaceAction[]);

    if (!isManualTurn(game)) {
      aiTimerRef.current = setTimeout(() => stepAI(game), 500);
    }
  }

  function handleNewGame() {
    startGame(p1Kind, p2Kind);
  }

  const validIndices = new Set(possibleActions.map((a) => a.index));

  return (
    <div className="flex gap-8 p-16 items-start">
      <div className="grid grid-cols-3 grid-rows-3 w-72 h-72 bg-card rounded-xl shadow-lg">
        {board.map((square, i) => (
          <div
            key={i}
            onClick={() => handleSquareClick(i)}
            className={[
              "flex items-center justify-center text-5xl font-bold transition-colors",
              i % 3 !== 2 ? "border-r-2 border-border" : "",
              i < 6 ? "border-b-2 border-border" : "",
              validIndices.has(i) && !result ? "cursor-pointer hover:bg-muted" : "",
            ].join(" ")}
          >
            {square === TicTacToeSquare.Player1 && (
              <span className="text-orange-400">X</span>
            )}
            {square === TicTacToeSquare.Player2 && (
              <span className="text-slate-400">O</span>
            )}
          </div>
        ))}
      </div>

      <div className="flex flex-col gap-4 pt-2">
        <div className="flex flex-col gap-1">
          <span className="text-xs font-medium text-muted-foreground">Player 1 (X)</span>
          <div className="flex gap-1">
            {ALL_KINDS.map((kind) => (
              <button
                key={kind}
                onClick={() => setP1Kind(kind)}
                className={`px-2 py-1 rounded text-xs ${p1Kind === kind ? "bg-primary text-primary-foreground" : "bg-secondary hover:bg-secondary/80"}`}
              >
                {PLAYER_LABELS[kind]}
              </button>
            ))}
          </div>
        </div>

        <div className="flex flex-col gap-1">
          <span className="text-xs font-medium text-muted-foreground">Player 2 (O)</span>
          <div className="flex gap-1">
            {ALL_KINDS.map((kind) => (
              <button
                key={kind}
                onClick={() => setP2Kind(kind)}
                className={`px-2 py-1 rounded text-xs ${p2Kind === kind ? "bg-primary text-primary-foreground" : "bg-secondary hover:bg-secondary/80"}`}
              >
                {PLAYER_LABELS[kind]}
              </button>
            ))}
          </div>
        </div>

        <button
          onClick={handleNewGame}
          className="px-3 py-1.5 rounded bg-secondary hover:bg-secondary/80 text-sm"
        >
          New Game
        </button>

        {result && <div className="text-lg font-semibold">{result}</div>}
      </div>
    </div>
  );
}

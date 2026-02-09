import { Board } from "./board";
import { Player } from "./player";

export function Boop() {
  return (
    <div className="grid grid-cols-3">
      <Player />
      <Board />
      <Player />
    </div>
  );
}

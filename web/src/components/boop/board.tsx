import { BoopSquare } from "@/wasm/hermes_wasm";

import Background from "@/assets/boop/background.png";
import GrayCat from "@/assets/boop/gray-cat.png";
import GrayKitten from "@/assets/boop/gray-kitten.png";
import OrangeCat from "@/assets/boop/orange-cat.png";
import OrangeKitten from "@/assets/boop/orange-kitten.png";

interface BoardProps {
  board: BoopSquare[];
  validSquares: Set<number>;
  onSquareClick: (index: number) => void;
}

export function Board({ board, validSquares, onSquareClick }: BoardProps) {
  function getPiece(square: BoopSquare) {
    switch (square) {
      case BoopSquare.Player1Cat:
        return <img src={OrangeCat} alt="Orange Cat" />;
      case BoopSquare.Player1Kitten:
        return (
          <img
            className="scale-75 translate-y-3"
            src={OrangeKitten}
            alt="Orange Kitten"
          />
        );
      case BoopSquare.Player2Cat:
        return <img src={GrayCat} alt="Gray Cat" />;
      case BoopSquare.Player2Kitten:
        return (
          <img
            className="scale-75 translate-y-3"
            src={GrayKitten}
            alt="Gray Kitten"
          />
        );
      default:
        return null;
    }
  }

  return (
    <div
      className="grid w-[600px] h-[600px] grid-cols-6 grid-rows-6 gap-2 rounded-lg p-4"
      style={{
        backgroundImage: `url(${Background})`,
        backgroundSize: "cover",
        backgroundPosition: "center",
      }}
    >
      {board.map((square, i) => (
        <div
          key={i}
          onClick={() => onSquareClick(i)}
          className={`flex items-center justify-center rounded-lg h-full w-full border-2 cursor-pointer transition-opacity ${
            validSquares.has(i)
              ? "border-white border-solid"
              : "border-white border-dashed opacity-50"
          }`}
        >
          {getPiece(square)}
        </div>
      ))}
    </div>
  );
}

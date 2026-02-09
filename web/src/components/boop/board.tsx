import OrangeCat from "@/assets/orange-cat.png";
import OrangeKitten from "@/assets/orange-kitten.png";
import GrayCat from "@/assets/gray-cat.png";
import GrayKitten from "@/assets/gray-kitten.png";

export function Board() {
  const images = [OrangeCat, OrangeKitten, GrayCat, GrayKitten];

  return (
    <div className="grid w-full h-full grid-cols-6 grid-rows-6 gap-2">
      {Array.from({ length: 36 }).map((_, i) => (
        <div
          key={i}
          className="flex items-center justify-center bg-gray-200 rounded-lg"
        >
          <img
            src={images[Math.floor(Math.random() * images.length)]}
            alt="cat"
            className="object-cover w-32 h-32 rounded-lg"
          />
        </div>
      ))}
    </div>
  );
}

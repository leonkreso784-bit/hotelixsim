// screens/HotelScreen.tsx — privremeno; Task 5 ga zamjenjuje cijelog
import type { HotelBody } from "../api/types";

export function HotelScreen({ me }: { me: HotelBody }) {
  return <h1 className="text-3xl font-extrabold">Moj hotel · {me.hotel.name}</h1>;
}

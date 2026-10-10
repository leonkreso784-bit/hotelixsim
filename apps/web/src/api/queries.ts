import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { navigate } from "../router";
import { useApi } from "./ApiContext";
import { unwrap } from "./errors";
import { poll } from "./poll";
import type { GameBody, HotelBody } from "./types";

export const keys = {
  me: ["me"] as const,
  game: (gameId: string) => ["game", gameId] as const,
  situations: (hotelId: string) => ["situations", hotelId] as const,
  conversation: (hotelId: string, code: string) => ["conversation", hotelId, code] as const,
  feed: (hotelId: string) => ["feed", hotelId] as const,
};

export function useMe() {
  const api = useApi();
  return useQuery<HotelBody>({
    queryKey: keys.me,
    queryFn: async () => unwrap(await api.GET("/v1/me/hotel")),
  });
}

/** Igra; dok se mjesec obračunava, ispituje svakih `poll.roundMs`. */
export function useGame(gameId: string) {
  const api = useApi();
  return useQuery<GameBody>({
    queryKey: keys.game(gameId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/games/{game_id}", { params: { path: { game_id: gameId } } })),
    refetchInterval: (q) => (q.state.data?.current?.status === "closing" ? poll.roundMs : false),
  });
}

/** Nova demo igra (i gost, ako ga nema). Nakon nje sve se čita ispočetka. */
export function useStartDemo() {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => unwrap(await api.POST("/v1/demo")),
    // resetQueries vrati sve upite u početno stanje i ponovno dohvati aktivne (stara igra ne smije ostati na ekranu).
    onSuccess: () => {
      navigate("/");
      return qc.resetQueries();
    },
  });
}

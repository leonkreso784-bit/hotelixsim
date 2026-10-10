import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { navigate } from "../router";
import { useApi } from "./ApiContext";
import { unwrap } from "./errors";
import { poll } from "./poll";
import type {
  ConversationBody,
  FeedItemBody,
  GameBody,
  HotelBody,
  MonthCode,
  PlanBody,
  SituationsBody,
} from "./types";

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

export function useSituations(hotelId: string) {
  const api = useApi();
  return useQuery<SituationsBody>({
    queryKey: keys.situations(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/situations", { params: { path: { hotel_id: hotelId } } })),
    refetchInterval: (q) =>
      q.state.data?.items.some((i) => i.response?.status === "pending") ? poll.verdictMs : false,
  });
}

export function useCloseMonth(gameId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (month: MonthCode) =>
      unwrap(
        await api.POST("/v1/games/{game_id}/rounds/{month}/close", {
          params: { path: { game_id: gameId, month } },
        }),
      ),
    // Navigacija je ovdje, ne u mutate(): ekran se nakon osvježenja zamijeni i per-call onSuccess bi se izgubio.
    onSuccess: (_data, month) => {
      navigate(`/mjesec/${month}`);
      return qc.invalidateQueries();
    },
  });
}

/** Razgovor s likom; dok lik još odgovara, ispituje svakih `poll.conversationMs`. */
export function useConversation(hotelId: string, code: string, enabled: boolean) {
  const api = useApi();
  return useQuery<ConversationBody>({
    queryKey: keys.conversation(hotelId, code),
    enabled,
    queryFn: async () =>
      unwrap(
        await api.GET("/v1/hotels/{hotel_id}/situations/{code}/conversation", {
          params: { path: { hotel_id: hotelId, code } },
        }),
      ),
    refetchInterval: (q) => (q.state.data?.awaiting_reply ? poll.conversationMs : false),
  });
}

export function useSendMessage(hotelId: string, code: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (text: string) =>
      unwrap(
        await api.POST("/v1/hotels/{hotel_id}/situations/{code}/messages", {
          params: { path: { hotel_id: hotelId, code } },
          body: { text },
        }),
      ),
    onSuccess: (conv) => qc.setQueryData(keys.conversation(hotelId, code), conv),
  });
}

export function useAnswer(hotelId: string, code: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (answer: number) =>
      unwrap(
        await api.POST("/v1/hotels/{hotel_id}/situations/{code}/answer", {
          params: { path: { hotel_id: hotelId, code } },
          body: { answer },
        }),
      ),
    onSuccess: (body) => {
      qc.setQueryData(keys.situations(hotelId), body);
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}

export function useSubmitResponse(hotelId: string, code: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (text: string) =>
      unwrap(
        await api.POST("/v1/hotels/{hotel_id}/situations/{code}/response", {
          params: { path: { hotel_id: hotelId, code } },
          body: { text },
        }),
      ),
    onSuccess: (body) => {
      qc.setQueryData(keys.situations(hotelId), body);
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}

export function useSavePlan(hotelId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ month, plan }: { month: MonthCode; plan: PlanBody }) =>
      unwrap(
        await api.PUT("/v1/hotels/{hotel_id}/plans/{month}", {
          params: { path: { hotel_id: hotelId, month } },
          body: plan,
        }),
      ),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: keys.me });
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}

/** Igra za ekran „Mjesec završen“: ispituje samo dok je taj mjesec trenutni a moj hotel ga još nema obračunatog. */
export function useGameUntilResolved(gameId: string, hotelId: string, month: MonthCode) {
  const api = useApi();
  return useQuery<GameBody>({
    queryKey: keys.game(gameId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/games/{game_id}", { params: { path: { game_id: gameId } } })),
    refetchInterval: (q) => {
      const g = q.state.data;
      const mine = g?.hotels.find((h) => h.hotel_id === hotelId);
      if (mine?.months.some((m) => m.month === month)) return false;
      return g?.current?.month === month ? poll.roundMs : false;
    },
  });
}

/** Dnevnik hotela; uz `untilMentorFor` ispituje dok mentor za taj mjesec ne stigne (najviše `poll.maxFeedPolls` puta). */
export function useFeed(hotelId: string, opts: { untilMentorFor?: MonthCode } = {}) {
  const api = useApi();
  return useQuery<FeedItemBody[]>({
    queryKey: keys.feed(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/feed", { params: { path: { hotel_id: hotelId } } })).items,
    refetchInterval: (q) => {
      const m = opts.untilMentorFor;
      if (!m) return false;
      const has = q.state.data?.some((i) => i.kind === "mentor" && i.month === m);
      return has || q.state.dataUpdateCount >= poll.maxFeedPolls ? false : poll.feedMs;
    },
  });
}

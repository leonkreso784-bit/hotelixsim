import { type QueryClient, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef } from "react";
import { navigate, roundPath } from "../router";
import { useApi } from "./ApiContext";
import { unwrap } from "./errors";
import { poll } from "./poll";
import type {
  ConversationBody,
  ExplanationBody,
  FeedItemBody,
  GameBody,
  HotelBody,
  KpiBody,
  NewGameBody,
  PlanBody,
  SituationsBody,
} from "./types";

export const keys = {
  me: ["me"] as const,
  game: (gameId: string) => ["game", gameId] as const,
  situations: (hotelId: string) => ["situations", hotelId] as const,
  conversation: (hotelId: string, code: string) => ["conversation", hotelId, code] as const,
  feed: (hotelId: string) => ["feed", hotelId] as const,
  explanation: (hotelId: string, round: string) => ["explanation", hotelId, round] as const,
  kpi: (hotelId: string) => ["kpi", hotelId] as const,
};

export function useMe() {
  const api = useApi();
  return useQuery<HotelBody>({
    queryKey: keys.me,
    queryFn: async () => unwrap(await api.GET("/v1/me/hotel")),
  });
}

type Api = ReturnType<typeof useApi>;

/**
 * Jedino mjesto koje dohvaća igru. Krug (mjesec ili tjedan) se obračuna u pozadini, nakon što je „Zatvori“ već vratio 202 i
 * invalidirao upite (a radnik može biti brži od tog dohvata, pa se „closing“ nikad ne vidi). Zato se sve što ovisi
 * o obračunu (moj hotel: indeksi, gotovina, zadnji krug; situacije; dnevnik) ponovno dohvaća kad god se krug
 * ILI stanje runde razlikuje od spremljene igre (i kad se sezona završi). Prvi dohvat ne invalidira: podaci su svježi.
 */
async function fetchGame(api: Api, qc: QueryClient, gameId: string): Promise<GameBody> {
  const before = qc.getQueryData<GameBody>(keys.game(gameId));
  const next = unwrap(await api.GET("/v1/games/{game_id}", { params: { path: { game_id: gameId } } }));
  if (
    before &&
    (before.current?.round.code !== next.current?.round.code ||
      before.current?.status !== next.current?.status)
  ) {
    void qc.invalidateQueries({ queryKey: keys.me });
    void qc.invalidateQueries({ queryKey: ["situations"] });
    void qc.invalidateQueries({ queryKey: ["feed"] });
    void qc.invalidateQueries({ queryKey: ["kpi"] });
  }
  return next;
}

/** Igra; dok se mjesec obračunava, ispituje svakih `poll.roundMs`. */
export function useGame(gameId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useQuery<GameBody>({
    queryKey: keys.game(gameId),
    queryFn: () => fetchGame(api, qc, gameId),
    refetchInterval: (q) => (q.state.data?.current?.status === "closing" ? poll.roundMs : false),
  });
}

/**
 * Nova igra (i gost, ako ga nema). Bez postavki ide prazan zahtjev (demo model, kao prije); postavke profesora
 * (`model`, `round_kind`) idu u tijelu. Nakon nje sve se čita ispočetka.
 */
export function useStartDemo() {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (settings?: NewGameBody) =>
      unwrap(await (settings ? api.POST("/v1/demo", { body: settings }) : api.POST("/v1/demo"))),
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

/** Zatvara krug po kodu (mjesec „jun“ ili tjedan „t05“). */
export function useCloseRound(gameId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (round: string) =>
      unwrap(
        await api.POST("/v1/games/{game_id}/rounds/{month}/close", {
          params: { path: { game_id: gameId, month: round } },
        }),
      ),
    // Navigacija je ovdje, ne u mutate(): ekran se nakon osvježenja zamijeni i per-call onSuccess bi se izgubio.
    onSuccess: (_data, round) => {
      navigate(roundPath(round));
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
    mutationFn: async ({ round, plan }: { round: string; plan: PlanBody }) =>
      unwrap(
        await api.PUT("/v1/hotels/{hotel_id}/plans/{month}", {
          params: { path: { hotel_id: hotelId, month: round } },
          body: plan,
        }),
      ),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: keys.me });
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}

/** Igra za ekran „Krug završen“: ispituje samo dok je taj krug trenutni a moj hotel ga još nema obračunatog. */
export function useGameUntilResolved(gameId: string, hotelId: string, round: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useQuery<GameBody>({
    queryKey: keys.game(gameId),
    queryFn: () => fetchGame(api, qc, gameId),
    refetchInterval: (q) => {
      const g = q.state.data;
      const mine = g?.hotels.find((h) => h.hotel_id === hotelId);
      if (mine?.months.some((m) => m.round === round)) return false;
      return g?.current?.round.code === round ? poll.roundMs : false;
    },
  });
}

/** Dnevnik hotela; uz `untilMentorFor` ispituje dok mentor za taj mjesec ne stigne (najviše `poll.maxFeedPolls` puta). */
export function useFeed(hotelId: string, opts: { untilMentorFor?: string | undefined } = {}) {
  const api = useApi();
  const qc = useQueryClient();
  // Budžet se računa „od sada“: dnevnik je mogao biti dohvaćen i prije nego što je ovaj ekran montiran.
  const startCount = useRef(qc.getQueryState(keys.feed(hotelId))?.dataUpdateCount ?? 0);
  return useQuery<FeedItemBody[]>({
    queryKey: keys.feed(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/feed", { params: { path: { hotel_id: hotelId } } })).items,
    refetchInterval: (q) => {
      const m = opts.untilMentorFor;
      if (!m) return false;
      const has = q.state.data?.some((i) => i.kind === "mentor" && i.round === m);
      return has || q.state.dataUpdateCount - startCount.current >= poll.maxFeedPolls ? false : poll.feedMs;
    },
  });
}

/** Zašto su se indeksi obračunatog kruga pomaknuli; obračunat krug se više ne mijenja, pa nema ispitivanja. */
export function useExplanation(hotelId: string, round: string) {
  const api = useApi();
  return useQuery<ExplanationBody>({
    queryKey: keys.explanation(hotelId, round),
    queryFn: async () =>
      unwrap(
        await api.GET("/v1/hotels/{hotel_id}/months/{month}/explanation", {
          params: { path: { hotel_id: hotelId, month: round } },
        }),
      ),
  });
}

/** Hotelski pokazatelji obračunatih mjeseci (ADR, RevPAR, GOPPAR…); računa ih poslužitelj, web samo prikazuje. */
export function useKpi(hotelId: string) {
  const api = useApi();
  return useQuery<KpiBody>({
    queryKey: keys.kpi(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/kpi", { params: { path: { hotel_id: hotelId } } })),
  });
}

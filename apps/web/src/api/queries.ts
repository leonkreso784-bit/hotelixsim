import {
  keepPreviousData,
  type QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useRef } from "react";
import { navigate, roundPath } from "../router";
import { useApi } from "./ApiContext";
import { ApiError, unwrap } from "./errors";
import { poll } from "./poll";
import type {
  ConversationBody,
  DepartmentDetailBody,
  DepartmentsBody,
  ExplanationBody,
  FeedItemBody,
  GameBody,
  HotelBody,
  KpiBody,
  NewGameBody,
  PlanBody,
  PlBody,
  PlScopeCode,
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
  departments: (hotelId: string) => ["departments", hotelId] as const,
  department: (hotelId: string, code: string) => ["departments", hotelId, code] as const,
  pl: (hotelId: string, code: string | null, scope: PlScopeCode) => ["pl", hotelId, code, scope] as const,
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
    void qc.invalidateQueries({ queryKey: ["departments"] });
    void qc.invalidateQueries({ queryKey: ["pl"] });
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

/**
 * Spremanje plana kruga, uvijek kao cijeli plan. Prije slanja se dohvati svježi važeći plan kruga (moj hotel) i na
 * njega se primijene izmjene ekrana (`patch`); sve ostalo ostaje kakvo je na poslužitelju (npr. odluke drugih odjela
 * ili hotela). Krug koji više nije otvoren je 409 `round_not_open`, a otvoren novi krug 409 `round_changed` (samo
 * web), oboje bez slanja; tada se moj hotel osvježi, da ekran pokaže novi krug. Spremljeni plan iz odgovora odmah
 * osvježava moj hotel.
 */
export function useSavePlan(hotelId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ round, patch }: { round: string; patch: (current: PlanBody) => PlanBody }) => {
      const state = unwrap(await api.GET("/v1/me/hotel")).private?.plan;
      if (!state) throw new ApiError(409, "round_not_open");
      if (state.round !== round) throw new ApiError(409, "round_changed");
      return unwrap(
        await api.PUT("/v1/hotels/{hotel_id}/plans/{month}", {
          params: { path: { hotel_id: hotelId, month: round } },
          body: patch(state.plan),
        }),
      );
    },
    onSuccess: (saved) => {
      qc.setQueryData<HotelBody>(keys.me, (me) =>
        me?.private?.plan
          ? { ...me, private: { ...me.private, plan: { ...me.private.plan, plan: saved } } }
          : me,
      );
      void qc.invalidateQueries({ queryKey: keys.me });
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
    // Krug se promijenio ili se obračunava: osvježi moj hotel i igru. Igra tada vidi „closing“ i ispituje dok se ne
    // otvori novi krug, pa se plan sam vrati (UI krug 3, M-2); bez igre bi plan zapeo na „mijenja se dok je otvoren“.
    onError: (error) => {
      if (error instanceof ApiError && error.status === 409) {
        void qc.invalidateQueries({ queryKey: keys.me });
        void qc.invalidateQueries({ queryKey: ["game"] });
      }
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

/** Odjeli hotela (v2): stanje, zadovoljstvo i rezultat zadnjeg obračunatog kruga. U v1 igri poslužitelj vraća 409 `model_v1`. */
export function useDepartments(hotelId: string) {
  const api = useApi();
  return useQuery<DepartmentsBody>({
    queryKey: keys.departments(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/departments", { params: { path: { hotel_id: hotelId } } })),
  });
}

/** Jedan odjel s rezultatima po obračunatim krugovima. */
export function useDepartment(hotelId: string, code: string) {
  const api = useApi();
  return useQuery<DepartmentDetailBody>({
    queryKey: keys.department(hotelId, code),
    queryFn: async () =>
      unwrap(
        await api.GET("/v1/hotels/{hotel_id}/departments/{code}", {
          params: { path: { hotel_id: hotelId, code } },
        }),
      ),
  });
}

/** Račun dobiti (v2) za krug ili sezonu do sada; bez `code` poslužitelj uzima zadnji obračunati krug. */
export function usePl(hotelId: string, code: string | null, scope: PlScopeCode) {
  const api = useApi();
  return useQuery<PlBody>({
    queryKey: keys.pl(hotelId, code, scope),
    // Dok stiže drugi krug ili sezona, prethodni prikaz ostaje (zamućen), da izbornik ne treperi.
    placeholderData: keepPreviousData,
    queryFn: async () =>
      unwrap(
        await api.GET("/v1/hotels/{hotel_id}/pl", {
          params: { path: { hotel_id: hotelId }, query: { scope, ...(code ? { round: code } : {}) } },
        }),
      ),
  });
}

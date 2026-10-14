// test/situationsV2.ts — situacije tjedna u v2 igri (W-1): odjel, obvezne, prepuštene, voditelj.
import type { ResponseBody, SituationBody, SituationsBody } from "../api/types";
import { situations, v2Api } from "./fixtures";

/** Primjerak situacije bez lika, s ponuđenim odgovorima; zadano neobvezan, neodgovoren, voditelj = vlastiti odjel. */
export function instance(
  code: string,
  department: string | null,
  title: string,
  over: Partial<SituationBody> = {},
): SituationBody {
  return {
    code,
    title,
    answers: ["Riješiti odmah i javiti se gostu", "Pričekati do sutra", "Ne raditi ništa"],
    chosen: null,
    free_text: false,
    character: null,
    response: null,
    department,
    kind: "complaint",
    weight: 3,
    description: null,
    roles: [],
    mandatory: false,
    delegated: false,
    leader: department ?? "admin",
    ...over,
  };
}

/** Predan slobodan odgovor (AI ga je ocijenio). */
export const submitted: ResponseBody = {
  text: "Premjestit ću goste i poslati kuhara da se ispriča.",
  status: "judged",
  quality: 4,
  rationale: "Brzo i konkretno.",
  criteria: [],
};

export const LIFT = "P-LIFT.t01";
export const LIFT_DESCRIPTION =
  "Dizalo je stalo između drugog i trećeg kata s dvoje gostiju unutra. Serviser stiže za sat vremena.";

/**
 * Pet situacija tjedna t01 redom kako ih šalje poslužitelj (po važnosti): tri obvezne (odgovorena cijelog hotela,
 * neodgovorena smještaja, predana restorana), pa prepuštena (lobby bar) i neodgovorena neobvezna (održavanje).
 * Redoslijed se namjerno ne poklapa ni s abecedom šifri ni naslova. Riješeno: 3 od 5; bez odgovora: 2.
 */
export function v2Items(): SituationBody[] {
  return [
    instance("K-GRUPA.t01", null, "Agencija traži 30 soba za vikend", {
      mandatory: true,
      weight: 5,
      chosen: 1,
      kind: "opportunity",
      leader: "sales",
    }),
    instance("P-KLIMA.t01", "rooms", "Klima ne radi u tri sobe", {
      mandatory: true,
      weight: 4,
      kind: "breakdown",
    }),
    instance("P-HRANA.t01", "restaurant", "Gost se žali na hladnu večeru", {
      mandatory: true,
      weight: 4,
      free_text: true,
      response: submitted,
    }),
    instance("P-BAR.t01", "lobby_bar", "Konobar kasni treći dan zaredom", {
      weight: 2,
      kind: "staff",
      delegated: true,
    }),
    instance(LIFT, "maintenance", "Dizalo stane između katova", {
      weight: 1,
      kind: "breakdown",
      description: LIFT_DESCRIPTION,
    }),
  ];
}

/** Tijelo kao s poslužitelja: `unanswered` ne broji prepuštene (A-1). */
export function v2Situations(
  items: SituationBody[] = v2Items(),
  over: Partial<SituationsBody> = {},
): SituationsBody {
  const unanswered = items.filter((i) => i.chosen === null && i.response === null && !i.delegated).length;
  return situations(items, { month: "jun", round: "t01", unanswered, ...over });
}

/** v2 igra (tjedan t01, odjeli iz `departmentsBody()`) s gornjim situacijama. */
export function v2SituationsApi(items: SituationBody[] = v2Items(), over: Partial<SituationsBody> = {}) {
  return v2Api().on("GET", "/v1/hotels/:hotel/situations", v2Situations(items, over));
}

/** Iste situacije nakon prepuštanja `code`: tako izgleda tijelo odgovora na `POST …/delegate`. */
export function afterDelegate(code: string, items: SituationBody[] = v2Items()): SituationsBody {
  return v2Situations(items.map((i) => (i.code === code ? { ...i, delegated: true } : i)));
}

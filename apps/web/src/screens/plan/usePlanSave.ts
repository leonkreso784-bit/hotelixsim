// Jedno spremanje plana kruga (L1-13b dio 2, ADR-042 t. 4): izmjene hotela (Plan) i odluke SVIH odjela idu jednim
// objedinjenim PUT-om (`useSavePlan`), s Plana i sa stranice odjela. Čita nacrte iz `drafts` (jedini izvor istine), pa
// radi isto s oba ekrana. Nacrti se brišu tek nakon uspjeha, i to samo poslano (`settleDraft`); pri neuspjehu i pri
// 409 `round_changed` ostaju svi (kao prije po ekranu).
import { useEffect, useRef, useState } from "react";
import { ApiError } from "../../api/errors";
import { useDepartments, useSavePlan } from "../../api/queries";
import type { PlanBody, PlanStateBody } from "../../api/types";
import {
  differs,
  draftKey,
  readDraft,
  roundDrafts,
  settleDraft,
  useDraftsVersion,
  writeDraft,
} from "../../drafts";
import { type Draft, hotelPatch, toDraft, touchedOnly } from "./hotelDraft";

type Plans = NonNullable<PlanBody["departments"]>;
type Entry = Plans[string];
type Problem = "invalid" | "tooMany" | "error";

/** Ukupno sezonskih radnika u planu (poslužitelj ga drži unutar `seasonal_max` hotela). */
export function seasonalTotal(plans: Plans): number {
  return Object.values(plans).reduce((sum, d) => sum + d.seasonal_target, 0);
}

export const roundChanged = (e: unknown) => e instanceof ApiError && e.code === "round_changed";

export function usePlanSave(hotelId: string, state: PlanStateBody | null) {
  const save = useSavePlan(hotelId);
  const inFlight = useRef(false);
  const version = useDraftsVersion();
  const [attempt, setAttempt] = useState<{ sig: string; problem: Problem } | null>(null);
  const round = state?.round ?? "-";
  const entries = state?.plan.departments ?? null;
  const list = useDepartments(hotelId, entries !== null);

  const planKey = draftKey("plan", hotelId, round);
  const planChanges = readDraft<Draft>(planKey);
  const saved = state ? toDraft(state.plan) : null;
  const drafted = roundDrafts(hotelId, round).depts;
  // Odjeli s pravim nacrtom; nacrt odjela kojeg više nema u planu ne šalje se (čisti se niže).
  const depts = entries
    ? drafted
        .filter((code) => entries[code])
        .map((code) => ({ code, mine: readDraft<Entry>(draftKey("dept", hotelId, round, code)) }))
        .filter((d) => differs(entries[d.code] as Entry, d.mine))
    : [];
  const dirty = (saved !== null && differs(saved, planChanges)) || depts.length > 0;
  const sig = JSON.stringify([planChanges, depts]);

  // Nacrt odjela kojeg u planu više nema: ostao bi zauvijek i lažno upozoravao pri zatvaranju kruga.
  // biome-ignore lint/correctness/useExhaustiveDependencies: `version` pokreće provjeru pri svakoj promjeni nacrta.
  useEffect(() => {
    if (!entries) return;
    for (const code of roundDrafts(hotelId, round).depts)
      if (!entries[code]) writeDraft(draftKey("dept", hotelId, round, code), null);
  }, [entries, hotelId, round, version]);

  const waitingForList = depts.length > 0 && !list.isSuccess;
  const enabled = state !== null && dirty && !save.isPending && !waitingForList;

  function run() {
    if (!state || !saved || !enabled || inFlight.current) return;
    const l = state.limits;
    const all = hotelPatch({ ...saved, ...planChanges }, l, entries !== null && !!l.departments);
    if (!all) {
      setAttempt({ sig, problem: "invalid" });
      return;
    }
    if (entries) {
      const merged: Plans = { ...entries };
      for (const d of depts) merged[d.code] = { ...(entries[d.code] as Entry), ...d.mine };
      if (seasonalTotal(merged) > l.seasonal_max) {
        setAttempt({ sig, problem: "tooMany" });
        return;
      }
    }
    inFlight.current = true;
    setAttempt({ sig, problem: "error" });
    const hotel = touchedOnly(all, Object.keys(planChanges) as (keyof Draft)[]);
    const sentPlan = { ...planChanges };
    const sentDepts = depts.map((d) => ({ ...d, mine: { ...d.mine } }));
    save
      .mutateAsync({
        round: state.round,
        patch: (cur) => {
          const next: PlanBody = { ...cur, ...hotel };
          if (sentDepts.length === 0) return next;
          const departments: Plans = { ...(cur.departments ?? {}) };
          for (const d of sentDepts) {
            const fresh = departments[d.code] ?? (entries?.[d.code] as Entry);
            departments[d.code] = { ...fresh, ...d.mine };
          }
          // Ukupni sezonski radnici i održavanje Smještaja izvode se iz odjela, kao na poslužitelju.
          return {
            ...next,
            departments,
            seasonal_target: seasonalTotal(departments),
            maintenance: departments.rooms?.maintenance ?? next.maintenance,
          };
        },
      })
      .then(() => {
        // Promise, ne opcije mutate: nacrti se brišu i kad se ekran napusti prije odgovora (inače bi zatvaranje
        // kruga lažno upozorilo na nespremljeno).
        settleDraft<Draft>(planKey, sentPlan);
        for (const d of sentDepts) settleDraft<Entry>(draftKey("dept", hotelId, round, d.code), d.mine);
      })
      .catch(() => {
        // Greška je u `save.error` i prikazuje se u traci; nacrti ostaju.
      })
      .finally(() => {
        inFlight.current = false;
      });
  }

  const shown = attempt?.sig === sig ? attempt.problem : null;
  const problem: Problem | null = shown === "error" ? (save.isError ? "error" : null) : shown;
  return {
    save,
    run,
    dirty,
    dirtyDepts: depts.map((d) => d.code),
    hotelDirty: saved !== null && differs(saved, planChanges),
    list,
    /** Popis odjela nije stigao a postoje nacrti odjela: spremanje čeka, pa traka kaže zašto. */
    listFailed: waitingForList && list.isError,
    enabled,
    problem,
    done: save.isSuccess && !dirty,
  };
}

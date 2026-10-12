// Uvod u odluke odjela nabraja samo odluke koje se na ekranu stvarno nude (UI M-4: „je li odjel otvoren“ samo kad se
// smije zatvoriti). Bez Reacta.
import { t } from "@hotelixsim/i18n";

export type Offered = { close: boolean; seasonal: boolean; price: boolean; upkeep: boolean; lease: boolean };

/** Nudi li odjel ijednu odluku. */
export function anyDecision(o: Offered): boolean {
  return o.close || o.seasonal || o.price || o.upkeep;
}

export function decisionsIntro(o: Offered): string {
  if (o.lease) return t("dept.decide.intro.lease");
  const parts = [
    ...(o.close ? [t("dept.decide.part.open")] : []),
    ...(o.seasonal ? [t("dept.decide.part.seasonal")] : []),
    ...(o.price ? [t("dept.decide.part.price")] : []),
    ...(o.upkeep ? [t("dept.decide.part.upkeep")] : []),
  ];
  const last = parts.pop() ?? "";
  const list = parts.length > 0 ? `${parts.join(", ")} i ${last}` : last;
  const intro = t("dept.decide.intro", { parts: list });
  return o.seasonal ? `${intro} ${t("dept.decide.intro.step")}` : intro;
}

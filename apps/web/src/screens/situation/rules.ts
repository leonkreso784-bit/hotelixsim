// Pravila situacije koja dijele popis, detalj i zatvaranje kruga (W-1, Leon P4).
import type { SituationBody } from "../../api/types";

/** Riješena: izabran odgovor, predan slobodan odgovor (i dok ga AI ocjenjuje) ili prepuštena voditelju. */
export function isSettled(s: SituationBody): boolean {
  return s.chosen !== null || s.response !== null || s.delegated;
}

/**
 * Smije li se prepustiti voditelju: krug je otvoren, situacija nije obvezna ni riješena i ima voditelja
 * (`leader: null` = stara igra bez voditelja, tada se ne nudi).
 */
export function canDelegate(s: SituationBody, open: boolean): boolean {
  return open && !s.mandatory && s.leader !== null && !isSettled(s);
}

/**
 * Prikazuje li se „Obvezna“ (značka i rečenica o prve tri). Poslužitelj šalje `mandatory: true` i kad voditelja
 * nema (v1 igra, v2 bez otvorenog odjela), a tada „obvezna“ ne znači ništa — student ionako rješava sve sam.
 */
export function showMandatory(s: SituationBody): boolean {
  return s.mandatory && s.leader !== null;
}

/**
 * Redak odjela na kartici: odjel situacije (ikona i ime), ili „Cijeli hotel“ za predložak razine hotela
 * (`department: null`, vrsta postoji). Sidra i v1 nemaju ni odjel ni vrstu — bez retka (voditelj sidra je
 * odjel s najviše ljudi, ne odjel situacije).
 */
export function deptLine(s: SituationBody): "dept" | "hotel" | null {
  if (s.department !== null) return "dept";
  return s.kind !== null ? "hotel" : null;
}

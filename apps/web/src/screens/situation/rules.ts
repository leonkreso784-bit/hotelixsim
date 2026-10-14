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

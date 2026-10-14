// Imena odjela za situacije: API situacije daje samo kod odjela, ime stiže iz `GET /departments` (postojeći upit).
import { useDepartments } from "../../api/queries";

/**
 * Ime odjela po kodu, ili `null` dok imena ne stignu (ili odjela nema) — kod se nikad ne prikazuje (UI_PRAVILA §3.4).
 * `enabled: false` kad nijedna situacija nema odjel ni voditelja (stara igra), da se odjeli ne traže uzalud.
 */
export function useDeptNames(hotelId: string, enabled: boolean): (code: string | null) => string | null {
  const list = useDepartments(hotelId, enabled);
  return (code) => {
    if (code === null) return null;
    return list.data?.departments.find((d) => d.code === code)?.name ?? null;
  };
}

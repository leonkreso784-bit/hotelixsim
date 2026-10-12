// Obrazac plana hotela kao nizovi (kako ih upisuje student) i pretvorba u izmjenu plana kruga. Bez Reacta.
import type { PlanBody, PlanLimitsBody } from "../../api/types";

export type Draft = {
  price: string;
  marketing: string;
  seasonal: string;
  wage: string;
  training: PlanBody["training"];
  maintenance: PlanBody["maintenance"];
  renovation: boolean;
  reply: boolean;
};

export const toDraft = (p: PlanBody): Draft => ({
  price: String(p.price_cents / 100),
  marketing: String(Math.round(p.marketing_cents / 100)),
  seasonal: String(p.seasonal_target),
  wage: String(p.wage_pct),
  training: p.training,
  maintenance: p.maintenance,
  renovation: p.renovation,
  reply: p.reply_reviews,
});

/** Cijeli broj u granicama ili null (prazno, decimalno, izvan raspona). */
function int(s: string, min: number, max: number): number | null {
  if (!/^\d+$/.test(s.trim())) return null;
  const n = Number(s.trim());
  return n >= min && n <= max ? n : null;
}

/**
 * Odluke hotela iz obrasca kao izmjena plana kruga; null kad neki broj nije u rasponu. U v2 se sezonski radnici,
 * održavanje i odjeli ne diraju: ostaju iz svježeg plana kruga (odlučuju se na ekranu odjela).
 */
export function hotelPatch(d: Draft, l: PlanLimitsBody, v2: boolean): Partial<PlanBody> | null {
  const price = int(d.price, l.price_min_cents / 100, l.price_max_cents / 100);
  const marketing = int(d.marketing, 0, l.marketing_max_cents / 100);
  const wage = int(d.wage, l.wage_min_pct, l.wage_max_pct);
  const seasonal = v2 ? 0 : int(d.seasonal, l.seasonal_min, l.seasonal_max);
  if (price === null || marketing === null || seasonal === null || wage === null) return null;
  const hotel: Partial<PlanBody> = {
    price_cents: price * 100,
    marketing_cents: marketing * 100,
    wage_pct: wage,
    training: d.training,
    renovation: d.renovation,
    reply_reviews: d.reply,
  };
  return v2 ? hotel : { ...hotel, seasonal_target: seasonal, maintenance: d.maintenance };
}

/** Polje plana kruga koje puni svako polje obrasca. */
const FIELD: Record<keyof Draft, keyof PlanBody> = {
  price: "price_cents",
  marketing: "marketing_cents",
  seasonal: "seasonal_target",
  wage: "wage_pct",
  training: "training",
  maintenance: "maintenance",
  renovation: "renovation",
  reply: "reply_reviews",
};

/**
 * Samo polja plana koja je student dirao (minor pregleda 2): ostala ostaju iz svježeg plana poslužitelja, pa spremanje
 * plana hotela ne pregazi ono što je u međuvremenu spremljeno drugdje.
 */
export function touchedOnly(patch: Partial<PlanBody>, touched: (keyof Draft)[]): Partial<PlanBody> {
  const fields = new Set(touched.map((k) => FIELD[k]));
  return Object.fromEntries(Object.entries(patch).filter(([k]) => fields.has(k as keyof PlanBody)));
}

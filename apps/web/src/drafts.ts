// Nespremljene izmjene obrasca (plan hotela, odluke odjela) koje prežive odlazak s ekrana i povratak (UI I1). Čuva se
// samo ono što je student promijenio (`changes`), a ekran ih slaže preko svježeg plana kruga: polja koja nije dirao
// prate plan poslužitelja. Ključ nosi hotel i krug, pa izmjene starog kruga ne prelaze u novi. Spremište je
// `sessionStorage` (samo ova kartica preglednika); ako nije dostupno (privatni prozor, blokirano), izmjene žive u memoriji
// dok je stranica otvorena.
import { useState } from "react";

const PREFIX = "hotelixsim:nacrt:";
const memory = new Map<string, string>();

function storage(): Storage | null {
  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}

function load(key: string): string | null {
  try {
    const stored = storage()?.getItem(PREFIX + key);
    if (stored != null) return stored;
  } catch {
    // Spremište ne radi: vrijedi memorija.
  }
  return memory.get(key) ?? null;
}

function save(key: string, value: string | null) {
  if (value === null) memory.delete(key);
  else memory.set(key, value);
  try {
    const s = storage();
    if (!s) return;
    if (value === null) s.removeItem(PREFIX + key);
    else s.setItem(PREFIX + key, value);
  } catch {
    // Puno ili blokirano spremište: izmjena ostaje u memoriji.
  }
}

/** Ključ nacrta: vrsta obrasca, hotel i krug (npr. `plan:h1:t01`, `dept:h1:t01:restaurant`). */
export function draftKey(...parts: string[]): string {
  return parts.join(":");
}

/** Spremljene izmjene za ključ; neispravan zapis (ručno mijenjan, stara inačica) se zanemaruje. */
export function readDraft<T extends object>(key: string): Partial<T> {
  const raw = load(key);
  if (!raw) return {};
  try {
    const value: unknown = JSON.parse(raw);
    return typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Partial<T>) : {};
  } catch {
    return {};
  }
}

export function writeDraft<T extends object>(key: string, changes: Partial<T> | null) {
  save(key, changes && Object.keys(changes).length > 0 ? JSON.stringify(changes) : null);
}

/** Za testove: briše sve nacrte (memoriju i spremište). */
export function clearDrafts() {
  memory.clear();
  try {
    const s = storage();
    if (!s) return;
    for (const k of Object.keys(s)) if (k.startsWith(PREFIX)) s.removeItem(k);
  } catch {
    // Nema spremišta: memorija je već prazna.
  }
}

/**
 * Izmjene obrasca pod ključem: `changes` (samo dirana polja), `update` dodaje izmjenu, `clear` ih briše (nakon
 * spremanja). Kad se ključ promijeni (novi krug), vrijede izmjene novog ključa.
 */
export function useDraft<T extends object>(key: string) {
  const [entry, setEntry] = useState(() => ({ key, changes: readDraft<T>(key) }));
  const changes = entry.key === key ? entry.changes : readDraft<T>(key);
  return {
    changes,
    update(patch: Partial<T>) {
      const next = { ...changes, ...patch };
      writeDraft(key, next);
      setEntry({ key, changes: next });
    },
    clear() {
      writeDraft(key, null);
      setEntry({ key, changes: {} });
    },
  };
}

/**
 * Je li se dok je obrazac otvoren otvorio novi krug (svježi plan stigne za drugi krug nego na početku): ekran tada kaže
 * da odluke vrijede za novi krug i da izmjene prošlog nisu prenesene (minor pregleda 2).
 */
export function useRoundChanged(round: string | null): boolean {
  const [first] = useState(round);
  return first !== null && round !== null && round !== first;
}

/** Razlikuje li se nacrt od plana: neko dirano polje ima drugu vrijednost nego plan poslužitelja. */
export function differs<T extends object>(base: T, changes: Partial<T>): boolean {
  return (Object.keys(changes) as (keyof T)[]).some((k) => changes[k] !== base[k]);
}

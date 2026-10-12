// Nespremljene izmjene obrasca (plan hotela, odluke odjela) koje prežive odlazak s ekrana i povratak (UI I1). Čuva se
// samo ono što je student promijenio i što se razlikuje od plana (`changes`), a ekran ih slaže preko svježeg plana kruga:
// polja koja nije dirao prate plan poslužitelja. Ključ nosi hotel i krug, pa izmjene starog kruga ne prelaze u novi (ekran
// kaže da nisu prenesene i briše ih). Spremište je `sessionStorage` (samo ova kartica preglednika); memorija stranice je
// uvijek najnovija (i kad spremište odbije upis ili brisanje), spremište služi da izmjene prežive osvježavanje.
import { useEffect, useState } from "react";

const PREFIX = "hotelixsim:nacrt:";
/** Najnovije stanje ključa u ovoj stranici; `null` znači obrisano (i kad spremište brisanje odbije). */
const memory = new Map<string, string | null>();

function storage(): Storage | null {
  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}

function load(key: string): string | null {
  if (memory.has(key)) return memory.get(key) ?? null;
  try {
    return storage()?.getItem(PREFIX + key) ?? null;
  } catch {
    return null;
  }
}

function save(key: string, value: string | null) {
  memory.set(key, value);
  try {
    const s = storage();
    if (!s) return;
    if (value === null) s.removeItem(PREFIX + key);
    else s.setItem(PREFIX + key, value);
  } catch {
    // Puno ili blokirano spremište: vrijedi memorija.
  }
}

/** Svi ključevi s nacrtom (spremište i memorija, bez obrisanih). */
function keys(): string[] {
  const all = new Set<string>();
  try {
    const s = storage();
    if (s)
      for (let i = 0; i < s.length; i++) {
        const k = s.key(i);
        if (k?.startsWith(PREFIX)) all.add(k.slice(PREFIX.length));
      }
  } catch {
    // Nema spremišta: samo memorija.
  }
  for (const [k, v] of memory) if (v !== null) all.add(k);
  return [...all].filter((k) => load(k) !== null);
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

/** Nacrti hotela za krug: ima li ga plan i koji ga odjeli imaju (za potvrdu zatvaranja i sažetak u planu). */
export function roundDrafts(hotelId: string, round: string): { plan: boolean; depts: string[] } {
  const all = keys();
  return {
    plan: all.includes(draftKey("plan", hotelId, round)),
    depts: all
      .filter((k) => k.startsWith(`${draftKey("dept", hotelId, round)}:`))
      .map((k) => k.split(":").slice(3).join(":")),
  };
}

/** Nacrti vrste obrasca za hotel iz nekog drugog kruga (stari, više se ne mogu spremiti). */
function staleKeys(scope: "plan" | "dept", hotelId: string, round: string): string[] {
  const head = `${draftKey(scope, hotelId)}:`;
  return keys().filter((k) => k.startsWith(head) && k.slice(head.length).split(":")[0] !== round);
}

/**
 * Izmjene obrasca pod ključem, prema planu `base`. `update` dodaje izmjenu, a polje vraćeno na vrijednost iz plana
 * izbacuje (da ne pregazi kasnije spremljeno drugdje). `settle(sent)` nakon spremanja briše samo poslane izmjene:
 * ono što je upisano dok je zahtjev trajao ostaje. Kad se ključ promijeni (novi krug), vrijede izmjene novog ključa.
 */
export function useDraft<T extends object>(key: string, base: T) {
  const [entry, setEntry] = useState(() => ({ key, changes: readDraft<T>(key) }));
  const changes = entry.key === key ? entry.changes : readDraft<T>(key);
  const store = (next: Partial<T>) => {
    writeDraft(key, next);
    setEntry({ key, changes: next });
  };
  return {
    changes,
    update(patch: Partial<T>) {
      const next: Partial<T> = { ...readDraft<T>(key), ...patch };
      for (const k of Object.keys(next) as (keyof T)[]) if (next[k] === base[k]) delete next[k];
      store(next);
    },
    settle(sent: Partial<T>) {
      const next: Partial<T> = { ...readDraft<T>(key) };
      for (const k of Object.keys(sent) as (keyof T)[]) if (next[k] === sent[k]) delete next[k];
      store(next);
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

/**
 * Nacrt iz prošlog kruga za ovaj obrazac (krug se otvorio dok je student bio na drugom ekranu): ekran kaže da izmjene nisu
 * prenesene, a nacrti se brišu (minor pregleda 3). Odgovor ostaje „da“ za taj krug i nakon brisanja.
 */
export function useStaleDrafts(scope: "plan" | "dept", hotelId: string, round: string | null): boolean {
  const [found, setFound] = useState<Record<string, true>>({});
  const stale = round !== null && (found[round] ?? staleKeys(scope, hotelId, round).length > 0);
  useEffect(() => {
    if (round === null) return;
    const old = staleKeys(scope, hotelId, round);
    if (old.length === 0) return;
    for (const k of old) writeDraft(k, null);
    setFound((f) => ({ ...f, [round]: true }));
  }, [scope, hotelId, round]);
  return stale;
}

/** Razlikuje li se nacrt od plana: neko dirano polje ima drugu vrijednost nego plan poslužitelja. */
export function differs<T extends object>(base: T, changes: Partial<T>): boolean {
  return (Object.keys(changes) as (keyof T)[]).some((k) => changes[k] !== base[k]);
}

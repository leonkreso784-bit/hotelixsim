### Task A7: Kratke grane, jedna integracija i ledger kao stanje

**Veličina:** S, 2–3 h · **Ovisi o:** A1 spojen (situacije u `sloj1`) · **Paralelno s:** A2–A6 · **Model:** sonnet ·
**Prevodi Rust:** ne

**Cilj:** Nijedna grana cigle ne živi dulje od dana, a dvije grane koje diraju napredak više se ne balansiraju odvojeno.
Pravilo je zapisano na jednom mjestu ([AGENTI.md](../../workflow/AGENTI.md)) i provjerljivo: skripta pokazuje
koliko je koja lokalna grana iza integracijske i koliko dugo živi. Voditelj i Leon iz jedne tablice vide stanje plana.

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #14 i izvještaj 06 nalazi 2–5, 11: `model-v2/sloj1` 157 commita i 4 dana
odvojen od `main`; obnova i situacije balansirane odvojeno (B6); dijeljena baza i `target` (B8, B9); 31 mrtva oznaka
grane i 6 napuštenih mapa radnih stabala; ledger od 1080 redaka sa sedam odjeljaka „vjeruj ovome".

**Files:**
- Modify: `docs/workflow/AGENTI.md` §1 t. 3–5 i §2 (pravila ispod), `README.md` → „Lokalni razvoj" (jedan redak:
  naredba `corepack pnpm grane`)
- Create: `scripts/grane.mjs`, `scripts/grane.test.mjs`; `package.json` → skripta `"grane": "node scripts/grane.mjs"`
- Ne dirati: kod igre, CI, `scripts/check-*.mjs`

**Pravila koja se upisuju u AGENTI.md (doslovno, prilagođeno stilu dokumenta):**
1. **Integracijska grana** je `model-v2/sloj1` (do L1-15), poslije `main`. Grana cigle se otvara s nje i **spaja se isti
   dan** kad provjeravatelj kaže „da". Nijedna grana cigle ne živi dulje od ~1 dana. Ako cigla ne stane u dan, dijeli se.
2. **Balans nakon svakog spajanja** na integracijskoj grani: `cargo run -p balance --release -- --model v2 --round week
   --seeds 1000 --check` i isto za `--round month` (točne naredbe iz A1). Pad brane modela = spajanje se vraća.
3. **Dvije cigle koje diraju napredak** (jezgra, sadržaj s učinkom, balans) ne rade paralelno u dva stabla. Paralelno
   smiju samo cigle s odvojenim datotekama i bez utjecaja na napredak (npr. web i AI uputa).
4. **Svako radno stablo ima svoju bazu i svoj `CARGO_TARGET_DIR`** kad pokreće poslužitelj ili mjeri osnovicu (B8, B9).
5. **Ledger je stanje, ne dnevnik:** na vrhu `progress.md` je jedna tablica „cigla | stanje | grana | commit | sljedeći
   korak", koju voditelj prepisuje; povijest ide ispod jednog naslova „Povijest" i ne čita se na početku sesije.
6. **Kopija na kraju faze:** `git bundle create ../hotelixsim-kopije/<datum>/hotelixsim-sve-grane.bundle --all` i
   `tar -czf …/superpowers.tar.gz .superpowers` ([ADR-043](../../records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md) t. 6).

**Interfaces (Produces):** `node scripts/grane.mjs [--integracija <grana>]` (zadano `model-v2/sloj1`) ispisuje tablicu
lokalnih grana: ime, commita ispred i iza integracije, datum prvog commita koji integracija nema, „spojena" (sve je u
integraciji) i upozorenje `STARA` kad grana ima nespojene commite starije od 24 h. Izlazni kod 0 uvijek (alat, ne brana).
Izvezena čista funkcija `procijeni(grane, sada)` → niz redaka, za test bez gita.

**Testovi čuvara** (`scripts/grane.test.mjs`, `node --test`, isti oblik kao `scripts/check-kod.test.mjs`):
```js
import { test } from "node:test";
import assert from "node:assert/strict";
import { procijeni } from "./grane.mjs";

const SADA = Date.parse("2026-10-15T12:00:00Z");
test("spojena grana nije stara ni kad je davno otvorena", () => {
  const [r] = procijeni([{ ime: "model-v2/sloj1-t1", ispred: 0, iza: 40, prviNespojen: null }], SADA);
  assert.equal(r.stanje, "spojena");
});
test("nespojen rad stariji od 24 h je STARA", () => {
  const [r] = procijeni([{ ime: "popravak/a2", ispred: 3, iza: 0, prviNespojen: "2026-10-14T10:00:00Z" }], SADA);
  assert.equal(r.stanje, "STARA");
});
test("nespojen rad mlađi od 24 h je u radu", () => {
  const [r] = procijeni([{ ime: "popravak/a3", ispred: 1, iza: 0, prviNespojen: "2026-10-15T09:00:00Z" }], SADA);
  assert.equal(r.stanje, "u radu");
});
```
Danas crveni jer `scripts/grane.mjs` ne postoji (`ERR_MODULE_NOT_FOUND`).

**Koraci graditelja:**
1. `scripts/grane.mjs`: `git for-each-ref refs/heads --format=%(refname:short)`; za svaku granu
   `git rev-list --left-right --count <grana>...<integracija>` i `git log --reverse --format=%cI <integracija>..<grana> -1`;
   pozivi kroz `spawnSync("git", [...])` **bez shella** (zamka `^` u cmd.exe); zatim `procijeni` i ispis tablice.
2. Pravila 1–6 u AGENTI.md (§1 t. 3 uz voditelja, §2 za ledger i kopiju, §4 za paralelu), redak u README-u.
3. Prepisati vrh ledgera `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` u tablicu stanja (pravilo 5).

**Provjere:** `node --test scripts/grane.test.mjs` (3 prolaza), `node scripts/grane.mjs` (ispis, izlaz 0),
`corepack pnpm check:docs` (0), `corepack pnpm lint` (0).

**Leonov dio:** OK za brisanje 31 spojene oznake grane (`git branch -d`, samo spojene, `-d` odbija nespojene) i 6
napuštenih mapa radnih stabala (`../hotelixsim-s1-fr`, `-fw`, `-ot`, `-t23`, `-zv`, `../hotelixsim-dok-t6`: bez `.git`,
samo `node_modules`/`target`). Brisanje je odvojen korak voditelja nakon OK-a, ne dio ove cigle.

# Raspored dokumentacije — granice veličine, podindeksi, planovi kao mape, ADR po datoteci — plan

**Status:** ⏸️ PAUZIRAN (čeka spajanje Sloja 1; Taskovi 11–13) · **Područje:** dokumentacija · **Otvoren:** 2026-10-10 ·
**Grana:** `dokumentacija/raspored` (od `main`) u radnom stablu `C:/Users/leonk/Documents/hotelixsim-dokumentacija`

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (preporučeno) ili
> superpowers:executing-plans. Koraci su kućice (`- [ ]`). Plan je jedna datoteka (≤300 redaka) pa `task-brief` radi kao i dosad.

**Goal:** Nijedan dokument nije predug, svaki ima jedno mjesto u rasporedu, a brana `check:docs` u CI-ju to čuva — uz
sesiju koda (`model-v2/sloj1`) koja radi paralelno i ne smije se pokvariti.
**Leon (10.10.):** *„Neke md datoteke imaju previše informacija … Na nekim mjestima ima preko 3000 linija — to je JAKO
JAKO loše. Sve treba biti savršeno raspoređeno u dokumentacijama jer to je srce naših operacija."*
**Architecture:** Velik dokument postaje **indeks + mapa istog imena** (`ARHITEKTURA.md` + `ARHITEKTURA/`), pa nijedna
postojeća poveznica ne umire. Granice redaka su u jednom modulu (`scripts/docs-granice.mjs`); `check:docs` ih provodi.
Razdvajanje planova i ADR-ova radi skripta (ponovljivo, s provjerom da se ništa ne izgubi), ne ruka.
**Tech Stack:** Node 24 ESM `.mjs`, `node:test`, bez novih ovisnosti · Git Bash · `corepack pnpm`.
**Spec:** Leonove riječi gore + odluke u „Ciljni raspored" ispod; zapisuju se kao ADR-030 🟨 (Task 10).

## Prompt za početak (Leon zalijepi u NOVI prozor Claude Codea)

```text
/newsession

Ova sesija NE radi po docs/SLJEDECA_SESIJA.md (to je predaja sesije koda, koja paralelno gradi Sloj 1 na grani
model-v2/sloj1 — njezine datoteke ne diraj). Radiš plan dokumentacije:
C:/Users/leonk/Documents/hotelixsim/docs/plan/2026-10-10-dokumentacija.md — pročitaj ga cijeloga, počni od Taska 0
(radno stablo ../hotelixsim-dokumentacija, grana dokumentacija/raspored od main). Subagent-driven, valovi iz plana,
neovisni taskovi paralelno. Poštuj "Protokol paralelnih sesija". Na kraju NE pokrećeš cijeli /kraj: CLAUDE.md
"## Stanje" i SLJEDECA_SESIJA.md pripadaju sesiji koda. Push i spajanje u main samo uz moj OK.
```

## Ciljni raspored (odluke 🟨, ADR-030)

1. **Indeks + mapa istog imena.** Dokument `X.md` koji preraste granicu postaje kratak indeks (svrha, tablica
   „§ → datoteka"), a dijelovi idu u mapu `X/` pokraj njega. `docs/README.md` ostaje **jedini ulaz**: navodi `X.md`,
   a `X.md` navodi svoje dijelove. Samo jedna razina: `docs/<uloga>/X/Y/*.md` je zabranjen. Zašto: stare poveznice
   (`ARHITEKTURA.md`, `DECISIONS.md#adr-012`, `../archive/2026-10-10-ekrani.md`) i navodi u kodu („ARHITEKTURA §7.2",
   „DEMO §11") ostaju točni — § brojevi se čuvaju u naslovima dijelova.
2. **Granice veličine** (redaka, `wc -l`; mjeri `scripts/docs-granice.mjs`, provodi `check:docs`):

| vrsta | gdje | granica | zašto |
|---|---|---|---|
| predaja | `docs/SLJEDECA_SESIJA.md` | 60 | već pravilo `/kraj`, sad i brana |
| ADR | `docs/records/DECISIONS/*.md` | 120 | jedna odluka; najveći danas 63 (ADR-015) |
| indeks | `docs/README.md` i svaki `X.md` uz mapu `X/` (izvan arhive) | 200 | indeks vodi, ne objašnjava |
| živi dokument, plan, task | sve ostalo pod `docs/` osim `archive/` | 300 | čita se cijeli u jednom potezu; najveći današnji odjeljak (MODEL_V2 §3) ima 194; task od 300 tjera plan da navodi datoteke i potpise umjesto da lijepi cijeli kod |
| arhiva | `docs/archive/**` | 1000 | zamrznuto; 13.10. s 1100 na 1000 (Leon, tvrda granica, ADR-030) — ekrani T4 podijeljen na 4 i 4b |
| `CLAUDE.md` | korijen | 12 000 znakova | postojeći budžet |

3. **Planovi.** Plan ≤300 redaka smije biti jedna datoteka. Veći je **pregled** `docs/plan/<datum>-<ime>.md` (zaglavlje,
   Goal, Global Constraints, Review Focus, graf, tablica taskova) + mapa `docs/plan/<datum>-<ime>/task-NN-<slug>.md`,
   svaki task počinje s `### Task N: naslov`. Ledger ostaje `.superpowers/sdd/<datum>-<ime>/` (ime pregleda se ne mijenja).
   **Brief = task datoteka**: izvođač dobiva putanju task datoteke i pregleda. Ako treba `task-brief`, uvijek s trećim
   argumentom: `task-brief <mapa>/task-NN-x.md N "$(sdd-workspace <pregled>.md)/task-N-brief.md"` (inače bi radni prostor
   dobio ime taska). Plan se piše kao jedna datoteka pa se prije aktivacije razdvaja: `corepack pnpm docs:plan <plan.md>`.
4. **Jedan aktivni plan po području.** Zaglavlje nosi `**Područje:** kod` ili `**Područje:** dokumentacija` (bez retka =
   kod). Dvije sesije smiju raditi paralelno samo na različitim područjima.
5. **ADR po datoteci:** `docs/records/DECISIONS/ADR-0NN-<slug>.md` (naslov `# ADR-0NN — …`, isti redak Datum/Status/Vezano).
   `docs/records/DECISIONS.md` je **generirani** indeks (`corepack pnpm docs:adr`): tablica broj · naslov · status · datum,
   najnoviji gore, svaki redak sa sidrom `<a id="adr-0NN"></a>` (stare vanjske poveznice i commit poruke vode na pravi redak).
   Ručno se ne uređuje; `check:docs` pada ako indeks nije svjež. Poveznice u dokumentima idu ravno na datoteku ADR-a.
6. **Arhiva** se razdvaja istim alatom (pregled + mapa taskova); sadržaj taskova ostaje doslovan, mijenjaju se samo
   relativne putanje poveznica (jedna razina dublje). Ledgeri ostaju datoteke (najveći 239).
7. **Zapisnici** (`PROGRESS`, `CHANGELOG`, `BACKLOG`) imaju granicu 300; kad je prijeđu, stariji unosi idu u mapu istog
   imena po mjesecima (`PROGRESS/2026-10.md`), indeks ostaje na vrhu. Danas nijedan nije blizu.
8. **Prijelazne iznimke** (dok traje Sloj 1): `docs/plan/2026-10-10-v2-sloj1.md` i `docs/architecture/MODEL_V2.md` — izuzeti od
   granica i od zabrane starih ADR poveznica; popis `PRIJELAZNE_IZNIMKE` u `docs-granice.mjs`, prazni se u Tasku 13.

## Protokol paralelnih sesija

| faza A (obje rade) | smije pisati |
|---|---|
| **sesija dokumentacije** (`../hotelixsim-dokumentacija`, `dokumentacija/raspored`) | `docs/**` osim iznimaka desno; `CLAUDE.md` izvan „## Stanje" (u njemu samo skriptom prepisane poveznice); korijenski `README.md` samo „Lokalni razvoj"; `.claude/skills/{kraj,newsession}/`; `scripts/check-docs*`, `docs-granice*`, `adr*`, `razdvoji-plan*`; `package.json` samo `scripts`; svoj ledger |
| **sesija koda** (`hotelixsim`, `model-v2/sloj1`) | kod; `docs/SLJEDECA_SESIJA.md`; `CLAUDE.md` „## Stanje"; `docs/architecture/MODEL_V2.md`; korijenski `README.md` izvan „Lokalni razvoj"; svoj ledger. Svoj plan samo čita |

- **Sesija koda u fazi A ne piše** `docs/records/*`, `docs/README.md`, `ROADMAP`, `ARHITEKTURA`, `DEMO`, skillove ni
  `check-docs`. Novi ADR, BACKLOG, CHANGELOG i PROGRESS unosi idu u **pretinac**
  `C:/Users/leonk/Documents/hotelixsim/.superpowers/sdd/2026-10-10-v2-sloj1/za-dokumentaciju.md` (odjeljci `## ADR`,
  `## BACKLOG`, `## CHANGELOG`, `## PROGRESS`; gitignoriran). Sesija dokumentacije pretinac samo čita.
- **Brojevi ADR-a:** ADR-029 je rezerviran za Sloj 1 (Task 22 plana), ADR-030 za ovaj plan; svaki daljnji novi ADR iz
  faze A ide u pretinac **bez broja** i dobiva ga pri prijenosu.
- **Redoslijed spajanja:** (1) grana dokumentacije → `main` (Task 10, uz Leonov OK); (2) sesija koda spaja `main` u
  `model-v2/sloj1` **između valova** (nijedno radno stablo taska otvoreno) i prenosi pretinac (postupak na kraju plana);
  (3) od tada sesija koda piše po novim pravilima; (4) nakon spajanja Sloja 1 u `main` slijede Taskovi 11–13.
- **Sukob pri spajanju:** tekst (činjenice) uzmi s grane koda, mjesto (raspored) s grane dokumentacije; zatim
  `node scripts/adr.mjs poveznice` i `corepack pnpm check:docs`. **Iznimka — sukob u `docs/records/DECISIONS.md`:**
  uzmi stranu koda, pa `node scripts/adr.mjs razdvoji` (iznova piše 001–028 identično), `corepack pnpm docs:adr`,
  `node scripts/adr.mjs poveznice`; pravilo „raspored s grane dokumentacije” ondje bi tiho izgubilo novi ADR sesije
  koda. Ako je kod ipak prvi u `main`: grana dokumentacije
  spaja `main`, novi `## ADR-…` blok iz staroga `DECISIONS.md` postaje datoteka, pa `corepack pnpm docs:adr`.

## Global Constraints

- Hrvatski, bez ćirilice; imena datoteka ASCII (č ć → c, đ → d, š → s, ž → z), mala slova i crtice u slugovima.
- **Premješta se, ne prepisuje:** nijedna činjenica se ne mijenja; dopušteno je samo razina naslova, putanja poveznice,
  redak `Dio [X.md](../X.md)` i tablica indeksa.
- Nikad ne diraj: `crates/`, `apps/`, `packages/`, `content/`, `migrations/`, `.github/`, `docs/plan/2026-10-10-v2-sloj1.md`,
  `docs/architecture/MODEL_V2.md` (do Taska 11), `docs/SLJEDECA_SESIJA.md`, ništa u `C:/Users/leonk/Documents/hotelixsim/`
  osim Taska 0. Bez `git stash`, bez `git add -A`, bez poziva Anthropic API-ja.
- Poruka commita u datoteku iz scratchpada, `git commit -F`, hrvatski, s `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Svaka provjera kao gola naredba pa `echo EXIT=$?` (cijev maskira kod). `pnpm` samo kao `corepack pnpm`.
- Skripte: ESM, bez ovisnosti, uvoz bez nuspojava (izvršni dio pod `if (import.meta.url === …argv[1])`), `\r\n` i `\n` jednako.

## Review Focus

1. **Izgubljen odlomak pri razdvajanju** → svaka razdvojena datoteka ima provjeru „spoji natrag = izvornik" (skriptom ili `comm`).
2. **Poveznica jednu razinu dublje** (`../x.md` u dijelu mora postati `../../x.md`), i slike/skripte (`.png`, `.mjs`) koje
   današnja brana ne gleda → pravilo 1 proširuje se na sve relativne ciljeve (Task 7).
3. **`### Task` unutar ograde koda** (predlošci u planovima) → razdvajanje svjesno ograda, isto kao `task-brief`.
4. **Sesija koda pokvarena usred rada** → njezin plan, MODEL_V2 i SLJEDECA_SESIJA nedirnuti; iznimke dok traje Sloj 1.
5. **Indeks ADR-a ručno uređen ili zastario** → `check:docs` uspoređuje s generiranim i pada.

## Graf i valovi

```text
Val 0: T0 postavljanje → T1 mjere
Val 1 (paralelno): T2 razdvoji-plan · T3 adr alat · T4 ARHITEKTURA · T5 DEMO
Val 2 (paralelno): T6 arhiva (treba T2) · T7 brana (treba T1, T3)
Val 3: T8 seoba ADR-a (treba T3, T4, T5, T6)          Val 4: T9 pravila i tok rada (treba T7, T8)
Val 5: T10 završna provjera + spajanje u main          Val 6 (nakon Sloja 1 u main): T11 MODEL_V2 · T12 arhiva Sloja 1
Val 7: T13 zatvaranje
```

### Task 0: Postavljanje radnog stabla

**Files:** stvori radno stablo; premjesti ovaj plan; Modify: `docs/README.md` (redak u „plan/").
- [ ] `git -C C:/Users/leonk/Documents/hotelixsim worktree add ../hotelixsim-dokumentacija -b dokumentacija/raspored main`
- [ ] `mv` ovaj plan iz glavnog radnog stabla u isto mjesto u novom (u glavnom je nepraćen; tamo ne smije ostati).
- [ ] Zaglavlje ostaje ⏸️ PAUZIRAN, zagrada postaje „(u izvođenju; 🟩 AKTIVAN u Tasku 7, kad brana zna područja)" — stara
  brana bi inače pala na dva aktivna plana. Redak u `docs/README.md` → „plan/".
- [ ] `corepack pnpm install --frozen-lockfile`, `corepack pnpm check:docs; echo EXIT=$?`. → `EXIT=0`. Commit.

### Task 1: Mjere — modul granica i popis veličina

**Files:** Create `scripts/docs-granice.mjs`, `scripts/docs-granice.test.mjs`; Modify `scripts/check-docs.mjs`, `package.json` (jedini
pisac u valovima 0–2: `"test:docs": "node --test \"scripts/*.test.mjs\""`, `"docs:plan": "node scripts/razdvoji-plan.mjs"`,
`"docs:adr": "node scripts/adr.mjs indeks"` — skripte nastaju u T2 i T3).
**Produces:** `brojRedaka(text) → number` (kao `wc -l`, `\r` se ne broji); `granicaZa(relPath, imaMapuIstogImena) →
{ vrsta: "predaja"|"adr"|"indeks"|"zivi"|"arhiva", redaka: number } | null` (null izvan `docs/`); `PRIJELAZNE_IZNIMKE` (objekt
putanja → razlog, točka 8 gore); `jeIznimka(relPath) → boolean`.
- [ ] Testovi za svaku vrstu iz tablice granica i za iznimku (prvo padaju, pa implementacija, pa prolaze).
- [ ] `node scripts/check-docs.mjs --popis` ispiše `redaka  granica  vrsta  datoteka`, silazno, ❌ uz prekoračenje, exit 0.
- [ ] Ispis spremi u ledger kao **polazno stanje**. Commit „Dokumentacija 1: mjere i granice".

### Task 2: Alat `razdvoji-plan`

**Files:** Create `scripts/razdvoji-plan.mjs`, `scripts/razdvoji-plan.test.mjs` (naredba `docs:plan` već postoji iz T1).
**Produces:** `razdvoji(text, ime) → { pregled: string, taskovi: [{ datoteka: "task-NN-<slug>.md", tekst }] }`;
`spoji(pregled, taskovi) → string` (izvornik); CLI `node scripts/razdvoji-plan.mjs <plan.md> [--provjeri]`.
- [ ] Pravila: task počinje naslovom `^#+\s+Task\s+\d+` izvan ograde; zadnji task završava na prvom `## ` naslovu iza
  njega (taj ostatak ide natrag u pregled); NN s vodećom nulom; slug iz naslova (ASCII, ≤6 riječi).
- [ ] U pregled na mjesto taskova ide `## Taskovi` — tablica `| N | [naslov](<ime>/task-NN-slug.md) |`.
- [ ] Relativne poveznice u taskovima dobivaju jedan `../` više; vanjske (`http`), sidra (`#…`) i kod netaknuti.
- [ ] `--provjeri`: `spoji(...)` uz vraćene putanje = izvornik bajt po bajt, inače exit 1.
- [ ] Testovi: ograda s `### Task 9` unutra; zadnji task + `## Ukupno`; CRLF; poveznica `../x.md` → `../../x.md`; `[a](#b)` ostaje.

### Task 3: Alat `adr`

**Files:** Create `scripts/adr.mjs`, `scripts/adr.test.mjs` (naredba `docs:adr` već postoji iz T1).
**Produces:** `slug(naslov) → string`; `procitajAdr(rootDir) → [{ broj, naslov, status, datum, datoteka }]`;
`indeksAdr(rootDir) → string` (puni sadržaj `DECISIONS.md`); `prepisiPoveznice(text, relPath, mapaAdr) → string`.
CLI: `razdvoji` (jednokratno: `## ADR-0NN` blokovi → datoteke, naslov `#`), `indeks [--provjeri]`, `poveznice [datoteke…]`
(bez argumenata: sve praćene `.md` osim `PRIJELAZNE_IZNIMKE`).
- [ ] `DECISIONS.md` (generiran): naslov, 3 retka „što je ADR, kako se dodaje (datoteka + `docs:adr`), ne uređuj ručno",
  tablica najnoviji gore; status = tekst iza `**Status:**` do prvog `·` ili `(`, bez `**`.
- [ ] `poveznice`: `(<put>DECISIONS.md#adr-0NN)` i u ADR datotekama `(#adr-0NN)` → ispravna relativna putanja do
  `DECISIONS/ADR-0NN-<slug>.md`; idempotentno (drugi prolaz ne mijenja ništa).
- [ ] Testovi: dijakritika u slugu, idempotentnost, putanja iz `CLAUDE.md`, iz `docs/product/` i iz `docs/archive/x/`.

### Task 4: ARHITEKTURA → indeks + `ARHITEKTURA/`

**Files:** Modify `docs/architecture/ARHITEKTURA.md`; Create `docs/architecture/ARHITEKTURA/` s
`01-domena-zahtjevi-rjecnik.md` (§1–3), `04-sustav-jezici-kod.md` (§4–6), `07-jezgra-i-runda.md` (§7–8),
`09-podaci-ai-api-prava.md` (§9–12), `13-sucelje-isporuka-sigurnost.md` (§13–17), `18-odbaceno-otvoreno-redoslijed.md` (§18–20).
- [ ] Indeks: naslov, postojeći blok statusa, §0 sažetak, tablica `§ → datoteka` (zamjenjuje „Sadržaj"), redak „kod
  citira ‚ARHITEKTURA §N' — § brojevi su sačuvani". Dio: prvi redak `# ARHITEKTURA §7–§8 — …`, drugi `Dio [ARHITEKTURA.md](../ARHITEKTURA.md)`, pa izvorni naslovi doslovno.
- [ ] Provjera gubitka: `comm -23 <(git show HEAD:docs/architecture/ARHITEKTURA.md | sed '/^\s*$/d' | sort) <(cat docs/architecture/ARHITEKTURA.md docs/architecture/ARHITEKTURA/*.md | sed '/^\s*$/d' | sort)` → smiju ostati samo retci „Sadržaja" i retci s poveznicom koja je dobila `../`.
- [ ] `node scripts/check-docs.mjs --popis` → nijedan dio iznad 300, indeks ≤200. Commit.

### Task 5: DEMO → indeks + `DEMO/`

**Files:** Modify `docs/product/DEMO.md`; Create `docs/product/DEMO/` s `01-opseg.md` (§1), `02-hotel-i-trziste.md` (§2–3),
`04-indeksi-profili-mehanike.md` (§4–6), `07-situacije.md` (§7), `08-ocjena-okolnosti-roboti-provjere.md` (§8–11),
`12-cjelobrojni-racun.md` (§12 i „Za Leonovu provjeru").
- [ ] Isti oblik i ista provjera gubitka kao Task 4 (putanje `docs/product/…`). Svaki dio ≤3 ISO datuma (pravilo 4).
- [ ] Commit.

### Task 6: Arhiva razdvojena (val 2, treba T2)

**Files:** `docs/archive/2026-10-04-kostur.md`, `2026-10-04-jezgra.md`, `2026-10-05-dnevnik-runde.md`, `2026-10-09-ai-sloj.md`,
`2026-10-10-ekrani.md` + po mapa istog imena. Ledgeri i `HOTEL_SIM_2026-08-22.md` se ne diraju.
- [ ] Za svaki: `node scripts/razdvoji-plan.mjs <plan> ; echo EXIT=$?`, zatim `… --provjeri ; echo EXIT=$?` → 0.
- [ ] `node scripts/check-docs.mjs --popis` → ništa u `archive/` iznad 1100; `check:docs` bez MRTVA POVEZNICA. Commit.

### Task 7: Brana — nova pravila u `check:docs` (val 2, treba T1, T3)

**Files:** Modify `scripts/check-docs.mjs`, `scripts/check-docs.test.mjs` (uz postojeće testove, isti uzorak `assertFails`).
- [ ] **1 prošireno:** svaki relativni cilj (ne samo `.md` i mapa) mora postojati — slike, skripte.
- [ ] **2 izmijenjeno + 8 PODINDEKS:** `.md` u `docs/<uloga>/X/` mora biti spomenut u `docs/<uloga>/X.md` (a ne u
  `docs/README.md`); `X.md` mora postojati i biti u `docs/README.md`; dublja razina pada.
- [ ] **3 izmijenjeno PODRUČJE:** najviše jedan aktivni plan po `**Područje:**` (`kod` | `dokumentacija`; bez retka = kod;
  druga vrijednost pada); računaju se samo `docs/plan/*.md`. Pravilo 7 gleda i `docs/plan/*/*.md`.
- [ ] **9 VELIČINA:** `brojRedaka > granicaZa(...).redaka` pada, osim `jeIznimka`. **10 ADR:** `DECISIONS.md ===
  indeksAdr()`, imena `ADR-\d{3}-[a-z0-9-]+\.md`, brojevi jedinstveni. **11 STARE ADR POVEZNICE:** `DECISIONS.md#adr-`
  izvan iznimaka pada. Za svako pravilo test koji pada i test koji prolazi.
- [ ] Ovaj plan: `**Status:** 🟩 AKTIVAN · **Područje:** dokumentacija`.
- [ ] `corepack pnpm test:docs; echo EXIT=$?` → 0 (stvarni `check:docs` smije biti crven do T8). Commit.

### Task 8: Seoba ADR-a (val 3, sam)

**Files:** `docs/records/DECISIONS.md` → generiran; Create `docs/records/DECISIONS/ADR-001-…` do `ADR-028-…`; poveznice
skriptom u svim praćenim `.md` (uklj. `CLAUDE.md`, `README.md`, `.claude/skills/kraj/SKILL.md`, arhivu), osim iznimaka.
- [ ] `node scripts/adr.mjs razdvoji`, `node scripts/adr.mjs indeks`, `node scripts/adr.mjs poveznice` (svako s `echo EXIT=$?`).
- [ ] `git grep -n 'DECISIONS.md#adr-' -- '*.md'` → pogoci samo u `docs/plan/2026-10-10-v2-sloj1.md`.
- [ ] Provjera gubitka: 28 datoteka; `cat` svih tijela bez naslova = blokovi iz `git show HEAD:docs/records/DECISIONS.md`
  (osim `##`→`#` i putanja). `node scripts/adr.mjs indeks --provjeri; echo EXIT=$?` → 0. Commit.

### Task 9: Pravila i tok rada zapisani (val 4)

**Files:** Modify `docs/README.md` (pravila 1–11 jednostavnim riječima, tablica granica, „indeks + mapa", područja,
uloga `records/DECISIONS/`), `docs/workflow/AGENTI.md` (§1–2: plan kao pregled + mapa, `docs:plan`, brief = task datoteka i
`task-brief` s trećim argumentom, dvije sesije po područjima, pretinac), `.claude/skills/kraj/SKILL.md` (§3 ADR = nova
datoteka + `corepack pnpm docs:adr`; §4 arhiviranje plana s `docs:plan`; §2 prelijevanje zapisnika; granice),
`.claude/skills/newsession/SKILL.md` (aktivni plan po području; čitaj pregled i samo tekući task), `CLAUDE.md` (redak
„Dokumentacija": granice i indeks + mapa; **ne** „## Stanje"), korijenski `README.md` „Lokalni razvoj" (`docs:adr`, `docs:plan`, `--popis`).
- [ ] Nijedna činjenica na dva mjesta: granice samo u tablici `docs/README.md` (izvor u kodu: `docs-granice.mjs`), drugdje poveznica.
- [ ] `corepack pnpm check:docs; echo EXIT=$?` → 0 (prvi put zeleno). Commit.

### Task 10: Završna provjera i spajanje u `main` (val 5)

- [ ] `corepack pnpm check:docs`, `corepack pnpm test:docs`, `node scripts/adr.mjs indeks --provjeri`, svaki `echo EXIT=$?` → 0.
- [ ] `node scripts/check-docs.mjs --popis` → **nijedan ❌** osim dvije iznimke; usporedi s polaznim stanjem (T1) i
  oba ispisa stavi u PROGRESS unos (najveća datoteka prije/poslije, broj datoteka).
- [ ] ADR-030 „Raspored dokumentacije" 🟨 ČEKA LEONOV PREGLED (točke 1–8 „Ciljnog rasporeda", cijena ako krivo) + `docs:adr`.
- [ ] `docs/records/PROGRESS.md` unos na vrh. Ovaj plan: `**Status:** ⏸️ PAUZIRAN (čeka spajanje Sloja 1; Taskovi 11–13)`.
- [ ] Završni pregled grane (opus) uz Review Focus; nalazi popravljeni. Commit.
- [ ] **Uz Leonov OK:** push grane, CI (`"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3`)
  zelen; `git -C ../hotelixsim-dokumentacija switch main && git merge --no-ff dokumentacija/raspored`; push `main`
  uz novi OK; CI zelen. Javi sesiji koda (preko Leona) da slijedi „Spajanje u sesiju koda".

### Task 11: MODEL_V2 → indeks + `MODEL_V2/` (val 6, tek kad je Sloj 1 u `main`)

**Files:** `docs/architecture/MODEL_V2.md` + `MODEL_V2/`: `01-danas-i-nacela.md` (§1–2), `03-entiteti.md` (§3),
`04-razdoblje.md` (§4), `05-ljudi.md` (§5), `06-uklapanje-i-konkurencija.md` (§6–7), `08-slojevi.md` (§8),
`09-odluke-redoslijed-pretpostavke.md` (§9–11); §0 ostaje u indeksu.
- [ ] Oblik i provjera gubitka kao Task 4. Ukloni MODEL_V2 iz `PRIJELAZNE_IZNIMKE`. Commit.

### Task 12: Arhiva Sloja 1 (val 6, paralelno s T11)

**Files:** `docs/archive/2026-10-10-v2-sloj1.md` (ako ga `/kraj` sesije koda još nije premjestio: `git mv` iz `docs/plan/`
uz pečat `✅ ISPUNJEN datum`) + mapa; ledger kao `docs/archive/2026-10-10-v2-sloj1-ledger.md` ako ga nema.
- [ ] Plan je već razdvojen 13.10. (pregled + mapa `2026-10-10-v2-sloj1/`, tvrda granica 1000): `git mv` pregleda **i mape**,
  ponovo izračunaj relativne poveznice (jedna razina se ne mijenja: obje ostaju u istoj mapi uloge); `node scripts/adr.mjs poveznice`. Commit.

### Task 13: Zatvaranje (val 7)

- [ ] `PRIJELAZNE_IZNIMKE` prazan (komentar: dodaje se samo uz datum i razlog); `check:docs`, `test:docs` → 0; `--popis` bez ❌.
- [ ] Ovaj plan `git mv` u `docs/archive/` s `✅ ISPUNJEN datum`; redak u indeksu; PROGRESS unos; commit; push uz Leonov OK.

## Spajanje u sesiju koda (radi sesija koda, nakon Taska 10)

1. Između valova (nijedno radno stablo taska otvoreno): `git -C C:/Users/leonk/Documents/hotelixsim merge main`.
2. Sukobi: tekst s grane koda, raspored s grane dokumentacije (pravilo iz Protokola); `node scripts/adr.mjs poveznice`.
   Sukob u `docs/records/DECISIONS.md`: uzmi stranu koda, pa `node scripts/adr.mjs razdvoji` (iznova piše 001–028
   identično), `corepack pnpm docs:adr`, `node scripts/adr.mjs poveznice`.
3. Pretinac `za-dokumentaciju.md`: ADR-ovi → `docs/records/DECISIONS/ADR-0NN-<slug>.md` (029 je Slojev, ostali dobivaju
   sljedeće slobodne brojeve) + `corepack pnpm docs:adr`; BACKLOG, CHANGELOG i PROGRESS unosi u svoje datoteke; pretinac se briše.
4. `corepack pnpm check:docs` i `corepack pnpm test:docs`, svaki `echo EXIT=$?` → 0; commit „Spoji raspored dokumentacije".
5. Task 22 Sloja 1 piše ADR-029 kao datoteku (ne u `DECISIONS.md`) i ne dira indeks ručno.

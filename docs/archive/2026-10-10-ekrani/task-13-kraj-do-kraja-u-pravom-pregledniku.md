### Task 13: Kraj do kraja u pravom pregledniku, dokumenti

Ovaj task radi **orkestrator** (glavna sesija), ne subagent: treba pravi poslužitelj, bazu i Playwright MCP.

**Files:**
- Modify: `README.md` (odjeljak „Lokalni razvoj“: kako pokrenuti igru u pregledniku), `docs/records/BACKLOG.md`
  (zatvorene stavke `[ekrani]`, nove odgođene), sitni ispravci izgleda koje provjera otkrije (`apps/web/src/**`).

- [ ] **Step 1: Pokreni sve** — baza `bash scripts/baza.sh start`; poslužitelj u pozadini
  (`set -a; source .env; set +a; cargo run -p server -- migrate && cargo run -p server -- serve`); web u pozadini
  (`corepack pnpm --filter @hotelixsim/web dev`). Čekaj `until curl -s localhost:8080/v1/health …`.
- [ ] **Step 2: Playwright MCP, 390×844** — `browser_resize`, pa cijeli tijek DEMO §1.5: `/demo` → „Započni igru“ → Moj hotel
  (snimka) → Situacije → S1: dvije poruke liku, čekaj odgovor → slobodan odgovor → presuda (snimka) → S3: izaberi →
  Plan: promijeni cijenu i održavanje, spremi → Zatvori mjesec (upozorenje) → Mjesec završen (snimka, mentor, ljestvica) →
  Nastavi u srpanj → S2 → Zatvori srpanj → rezultat → Tijek (snimka). Snimke u scratchpad.
- [ ] **Step 3: Provjeri na 360 px** da nema vodoravnog pomicanja (`document.documentElement.scrollWidth <= 360` kroz
  `browser_evaluate`) na svakom ekranu; konzola bez grešaka (`browser_console_messages`).
- [ ] **Step 4: Usporedi snimke sa slikom koncepta** i popravi ono što odstupa bez promjene ponašanja (razmaci, veličine,
  prelamanje). Svaki popravak = provjere (sve četiri).
- [ ] **Step 5: README** — u „Lokalni razvoj“ dodaj „Igra u pregledniku“: tri naredbe (baza, poslužitelj, web) i
  „otvori http://localhost:5173/demo“; za mobitel u istoj mreži `corepack pnpm --filter @hotelixsim/web dev --host` i
  upozorenje da tada svatko u mreži vidi igru.
- [ ] **Step 6: BACKLOG** — zatvori `[ekrani]`: problem+json, retry 3, health aria-live (ekran zdravlja više ne postoji),
  `vite build` u CI-ju, upozorenje na neodgovorene, `failed` obračun (djelomično: ekran kaže „traje dulje“). Dodaj
  odgođeno: e2e u CI-ju (Postgres + poslužitelj u poslu weba), množina „promjena“, ilustracija obale, `t()` parametri tipizirani.
- [ ] **Step 7: Provjere** (sve četiri + `corepack pnpm check:docs`) i commit — „Ekrani 13/13: provjera u pregledniku i
  dokumenti".

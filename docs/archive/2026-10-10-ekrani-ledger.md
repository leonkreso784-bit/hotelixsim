# SDD ledger — plan: docs/plan/2026-10-10-ekrani.md

Grana: `gradnja/ekrani` (od `main` b607395). Spec: docs/product/DIZAJN.md (v2) + DEMO §1.4–1.5 + ADR-021/022.
Modeli: izvođači sonnet (haiku samo za čistu prepisku), pregled sonnet, ponovni pregled haiku, završni opus.

## Preflight scan

| par / task | proizvodi → troši | nalaz |
|---|---|---|
| T1→T2 | `formatIndex`, `formatSigned`, `sortIndices`, `api/types` → `IndexBars` | slaže se |
| T1→T4..T10 | `format.ts`, `game.ts`, `api/types.ts` | imena i potpisi se slažu |
| T1/T4 `App.test.tsx` | T1 briše describe formatIndex, T4 zamjenjuje cijelu datoteku | slaže se |
| T3→T4 | `AvatarGallery` → ruta `/likovi` u App | slaže se |
| T3→T6, T9 | `Avatar`, `lookFor`, `PLAYER_LOOK`, `MENTOR_LOOK` | slaže se (zamijenjeno iz inicijala) |
| T4→T5..T10 | `queries.ts` (keys, useMe, useGame, useStartDemo), `FakeApi`, `fixtures`, `renderApp`, `poll` | slaže se |
| T4 stub `HotelScreen` → T5 zamjena | T4 test „Moj hotel“ regex prolazi i sa stubom i s pravim ekranom | slaže se |
| T5→T6..T7 | `useSituations` (ispituje dok pending) | slaže se |
| T6→T7 | `SituationScreen` mjesto „Task 7“ | slaže se |
| T9→T10 | `useFeed`, `FeedChange`, `Changes` | slaže se |
| `hr.ts` | T1..T10 dodaju ključeve redom | jedan pisac po tasku, redom — slaže se |
| T1 sam | testovi ↔ kod | slaže se |
| T2 sam | testovi ↔ kod | slaže se (inicijali maknuti) |
| T3 sam | testovi ↔ kod | slaže se |
| T4 sam | test 2/3 bili neuredni (dvostruka registracija, suvišan stubGlobal) | popravljeno u planu |
| T4 sam | `useStartDemo` removeQueries+invalidate | vidi Ruling 2 |
| T5..T8 | dvostruki klik: `disabled` + `isPending` ne štiti jer TanStack javlja asinkrono | vidi Ruling 3 |
| T9 sam | test `getByText("+8")` — „+8“ i u ljestvici (višestruki pogodak) | popravljeno: upit unutar kartice napretka |
| T9 sam | `useGameUntilResolved` ispituje zauvijek za mjesec koji nije trenutni | vidi Ruling 4 |
| cijeli plan | `„…"` s ASCII zatvaranjem prekida TS string | vidi Ruling 1 |

## Rulings

- Ruling 1: svi `„…"` u planu → `„…“` (77 mjesta) — ASCII `"` unutar TS stringa ga prekida; hrvatski zatvarajući navodnik je “ — cijena ako krivo: nijedna, samo tipografija.
- Ruling 2: `useStartDemo` onSuccess = `qc.resetQueries()` — `removeQueries` na aktivnom `me` upitu ne garantira ponovno dohvaćanje — cijena ako krivo: nakon „Nova igra“ ekran ostane na staroj igri do osvježavanja.
- Ruling 3: gumbi koji šalju imaju i `useRef` zastavicu u rukovatelju (Global Constraints) — TanStack javlja stanje asinkrono, `disabled` kasni jedan render — cijena ako krivo: malo više koda po gumbu.
- Ruling 4: `useGameUntilResolved` ispituje samo dok je `current.month === month` i mjesec nije obračunat — inače bi `/mjesec/jul` u otvorenom lipnju ispitivao zauvijek — cijena ako krivo: ako server nakon 202 ne postavi odmah `current` na taj mjesec, ekran ne bi čekao (server ga postavlja: zatvoreni mjesec ostaje `current` sa statusom closing).

## Tijek
Task 1: complete (commits 3123acc..89385d9, review clean)
Task 1: minor (deferred): formatEuros — hr-HR ne grupira 4 znamenke („1234 €“ uz „12.340 €“); prijedlog useGrouping "always"
Task 1: minor (deferred): formatSignedEuros(30) daje „+0 €“ — predznak iz sirovih centi, ne iz zaokruženog
Task 1: minor (deferred): indicesBefore pada na start ako prethodni mjesec nije obračunat (bezopasno uz redoslijed)
Task 1: minor (deferred): netestirano — negativni formatEuros, indicesAt
- Ruling 5: Task 2 je stilove vezao s <link href=/src/styles.css> u index.html (main.tsx nije bio u njegovim datotekama) — Task 4 uvodi import "./styles.css" u main.tsx i MORA maknuti taj <link> — cijena ako krivo: CSS dvaput učitan.
Task 2: complete (commits 89385d9..b70587a, review clean; ⚠️ provjereno: lucide 1.48.0 od 24.09., tailwind 4.3.3 od 16.07., znak loga ispravan)
Task 2: minor (deferred): Delta s 0 nije provjerena u testu
Task 2: minor (deferred): stupac naziva 7.5rem može rezati „Zadovoljstvo gostiju“ na 360 px — provjera u Tasku 11
- Ruling 6 (Task 3, nalaz Important, plan ga je propisao): lookFromSeed ovisi o živim duljinama paleta/opcija → izgled gosta iz imena se mijenja kad paleta naraste. Popravak SAD: zamrznuti skupovi za izvlačenje (eksplicitni popisi dopuštenih indeksa/opcija) — temelj uređivača; cijena ako krivo: nekoliko redaka.
- Ruling 7 (Task 3, Minor ali stvarna klasa greške): lookFor koristi Object.hasOwn — lookFor("constructor") vraća funkciju; ide u isti krug.
Task 3: fix round 1/5 (2 addressed, 0 open — stabilni izgledi iz imena, Object.hasOwn; commits 4b25a53..da55984; kontrolor vidio src/avatar 13/13)
Task 3: complete (commits b70587a..da55984, review clean)
Task 3: minor (deferred): oznake frizura u galeriji su sirovi id-ovi, ne t() (plan ih propisao; interna stranica)
Task 3: minor (deferred): test „samo slojevi koje izgled traži“ provjerava samo kapu; oči/usta/brada/naočale nisu u petlji
Task 3: minor (deferred): kapa na 64 px izgleda kao tanka traka (Task 11 pogledati)
- Ruling 8 (Task 4, ⚠️): navigate("/") nakon „Započni igru“ seli u onSuccess kuke useStartDemo — per-call callback se preskače kad resetQueries odmontira StartScreen — cijena ako krivo: nijedna (isto ponašanje, pouzdanije).
- Napomena za Task 11: port 5173 drži vite (PID 20540) pokrenut 04.10. iz OVOG repoa (stara sesija); nema tailwind plugina u memoriji → prije provjere u pregledniku ga ugasiti i pokrenuti svježi dev server.
Task 4: fix round 1/5 (3 addressed, 0 open — decodeURIComponent, navigate u kuki, min-h-11; commits 01eda28..4e66e9e)
Task 4: complete (commits da55984..4e66e9e, review clean)
Task 4: minor (deferred): GameShell tiho ignorira grešku useGame (chip ostane null)
Task 4: minor (deferred): mrtvi ključevi u hr.ts (status.db.*, status.events, status.indices.*, profile.heading, status.api.error) + njihova upotreba u packages/i18n/src/index.test.ts
Task 4: minor (deferred): queries.ts uvozi navigate iz routera (podatkovni sloj ovisi o UI-ju) — prihvaćeno Rulingom 8
Task 4: minor (deferred): text-white u čipu nije token
- Ruling 9 (Leon 10.10.: proširiti likove — više izgleda, više likova, uređivač; „dio odmah“): prije demoa Task 11 (više izgleda, paralelno u worktreeju gradnja/likovi-izgled) i Task 12 (više likova u ekranima, nakon ekrana); uređivač = zaseban plan poslije, jer izgled mora na poslužitelj (temelj) — cijena ako krivo: Leon je možda htio uređivač prije demoa → to je onda jedan plan s API-jem (~dan).
Task 5: dispatched (BASE b044c00); Task 11: dispatched paralelno u worktreeju C:/Users/leonk/Documents/hotelixsim-likovi, grana gradnja/likovi-izgled (BASE b044c00)
- Ruling 10 (Task 5, Important): navigacija nakon mutacije ide u onSuccess KUKE (useCloseMonth), ne u per-call mutate callback — refetch odmontira komponentu i TanStack v5 preskače per-call callback. Pravilo vrijedi za sve sljedeće taskove (nosi se u dispatch) — cijena ako krivo: nijedna.
- Ruling 11 (Leon 10.10.: „likovi su ok ali treba popravit detalje, neki izgledaju čudno, svi izgledaju nezadovoljno“): proslijeđeno izvođaču Taska 11 — prijateljski izraz po zadanom (obrve opuštene, „neutral“ usta blagi osmijeh), JEDNOKRATNA promjena SEED skupova (bez worried; ništa još nije spremljeno), čišćenje geometrije, dvije snimke za Leona — cijena ako krivo: izgled gostiju iz imena se jednom mijenja (nigdje nije spremljen).
Task 5: fix round 1/5 (3 addressed u kodu, 1 open — test možda ne razlikuje staru grešku; commits f5808e5..8d6ec8a)
Task 5: fix round 2/5 (1 addressed u kodu testa — mutacijska provjera: stari test prolazio na staroj grešci; novi RED „expected '/' to be '/mjesec/jun'“ → GREEN; commit 3d8c74e); ponovni pregled u tijeku
Task 11: complete (commits b044c00..de4c3bc na gradnja/likovi-izgled, review clean) — SPOJITI u gradnja/ekrani nakon Taska 6 (ne dok izvođač radi)
Task 11: minor (deferred): beanie dvostruki rect bez komentara; TRAIT_KEYS/TRAIT_ROWS dupliraju uniju u galeriji; predug komentar u look.ts; `void [...]` u testu; obrve u boji kose — niski kontrast za svijetloplavu kosu
Task 11: minor (deferred): „buzz“ frizura još sliči kapi; gosti iz imena nemaju spol (npr. ćelava Maja) — rješava uređivač
Task 5: complete (commits b044c00..3d8c74e, review clean nakon 2 kruga)
Task 5: minor (deferred): test zatvaranja ostavlja focusManager.setFocused(true) i release() nije u finally
Task 5: minor (deferred): nema testa dvostrukog klika na „Nova igra“ ni greške zatvaranja
Task 11: SPOJENO u gradnja/ekrani (e3752ec); kontrolor nakon spajanja: lint 68 datoteka, typecheck, web 73/73 + i18n 5/5, build — sve EXIT=0
Task 6: complete (commits 3d8c74e..59bcc47, review clean)
Task 6: minor (deferred, TRIJAŽA PRIJE DEMOA): Conversation — conv.isError nakon neuspjelog pozadinskog ispitivanja zamijeni cijeli razgovor porukom greške (treba isError && !data)
Task 6: minor (deferred, TRIJAŽA PRIJE DEMOA): useAnswer — already_answered ne osvježi situacije (gumbi ostanu, ista greška)
Task 6: minor (deferred): testovi dvostrukog klika vjerojatno ne razlikuju ref zaštitu od disabled; key={label} u ponuđenim odgovorima; nekorišteni ključevi situations.month, chat.you; novi odgovori lika bez aria-live; badge „AI ocjenjuje…“ bez role=status
Task 7: complete (commits e3752ec..2cd28c8, review clean)
Task 7: minor (deferred): neutralna presuda crta prazan <ul> okvir (criteria prazni); nema testa 503/already_answered nakon predaje
Task 8: complete (commits 2cd28c8..f518490, review clean; ⚠️ role=status/alert potvrdio kontrolor u primitives.tsx)
Task 8: minor (deferred): draft se ne sinkronizira ako se mjesec promijeni dok je /plan otvoren (ključ po state.month); greška ispod zatvorenog <details>; radio bez vidljivog fokusa; test decimalnog unosa i drugog 422 koda nedostaje
## Rana provjera u pregledniku (kontrolor, 390 px, pravi API + AI, prije Taska 9)
- Radi: početak → Započni igru → Moj hotel (pravi podaci) → S1 razgovor s pravim AI likom (Petra odgovorila ~20 s), lica se vide.
- ZA TASK 13 (dorada): (1) BUG: u razgovoru polje poruke stisnuto na ~20 px — Button ima w-full, className w-auto ne pobjeđuje (treba varijanta/širina bez sukoba klasa); (2) trake indeksa preuske, naziv „Zadovoljstvo go…“ rezan; (3) indeksi s 2 decimale („35,71“) — razmotriti cijele brojeve na Moj hotel; (4) „678.000 €“ lomi se u dva retka u kartici; (5) bilješke „nakon prvog mj…“ rezane; (6) .playwright-mcp/ dodati u .gitignore.
Task 9: review → Needs fixes (2 Important, vidi task-9-findings-round1.md); fix round 1 NIJE pokrenut — Leon zatvara sesiju. NASTAVAK: fix round 1 svježim izvođačem, pa ponovni pregled, pa Task 10, 12, 13, završni pregled (opus).
Task 9: deviacije izvođača potvrđene kao ispravne: (a) „traje dulje“ preko setTimeout (structural sharing ne re-renderira), (b) dataUpdateCount ne postoji na rezultatu upita
Stanje okoline na kraju sesije: API (cargo serve, 8080) i Vite (5173) pokrenuti iz ove sesije u pozadini; stari Vite PID 20540 (04.10.) ugašen.
## Sesija 2 (10.10., nastavak)
- Ruling 12 (Leon 10.10. novi zahtjev, spec): „Mjesec završen“ pokazuje IndexBars (vrijednost s kraja mjeseca + promjena u mjesecu) umjesto kartica s promjenom; Moj hotel nakon obračuna osvježava `me` — temelj: ispitivanje igre na prijelazu closing→obračunato invalidira me/situacije/feed (jedno mjesto u queries.ts). Ide u Task 9 fix round 1 zajedno s 2 Important nalaza (task-9-fix1.md) — cijena ako krivo: malo više dohvaćanja nakon obračuna.
Task 9: fix round 1 dispatched (FIX_BASE 6d7255b, svježi izvođač sonnet, task-9-fix1.md)
Task 9: fix round 1 done by implementer (commit bdadb85; 1 MonthResult dijete s feedom i satom, 2 key=month, 3 IndexBars na kraju mjeseca, 4 fetchGame invalidira me/situations/feed na prijelazu); re-review dispatched (sonnet, review-6d7255b..bdadb85.diff)
- Ruling 13 (Leon 10.10. usred sesije: mentor „4876 na 4269, −600, 200 bp, 24 395 000 centi“ — „ovo nije dobro“): novi Task 14 „Mentor govori ljudski“ na POSLUŽITELJU (iznimka od Global Constraint „poslužitelj se ne mijenja“ — API oblik ostaje isti): pripremljene činjenice u ljudskim jedinicama, mentor.v2 s četiri dijela, isti oblik bez AI-ja, ADR-023 🟨. Paralelno u worktreeju ../hotelixsim-mentor (grana gradnja/mentor-ljudski, BASE bdadb85), implementer opus (prompt i oblik činjenica su prosudba) — cijena ako krivo: jedan dan rada na Rust strani prije demoa.
- Ruling 14: indeksi se svugdje (ekran i mentor) prikazuju cijelim brojevima 0–100, promjena = razlika ZAOKRUŽENIH vrijednosti (da ekran i mentor kažu isto); web dio ide u Task 13 doradu (formatIndex/Delta) — cijena ako krivo: male promjene (<0,5) se vide kao 0.
Task 14: dispatched (worktree ../hotelixsim-mentor, BASE bdadb85)
- ZA TASK 13 (dorada, dodatak 10.10.): (7) cijeli brojevi za indekse i promjene po Rulingu 14 (formatIndex, Delta, formatChange za points); (8) mentorov tekst s `whitespace-pre-line` (mentor.v2 ima četiri dijela u retcima); (9) Leon provjerava razumljivost „Mjesec završen“ — svaka brojka s imenom i jedinicom.
Task 9: fix round 1/5 (4 addressed, 0 open — sat od obračuna, key=month, IndexBars na kraju mjeseca, me se osvježi nakon obračuna; commits 6d7255b..bdadb85)
Task 9: complete (commits f518490..bdadb85, review clean nakon 1 kruga)
Task 9: minor (deferred): ako prvi dohvat igre nakon reloada već vrati obračunat mjesec (cache nije bio closing), invalidacija me ne okine (queries.ts:83) — bezopasno jer je me tada svjež
Task 9: minor (deferred): fiksni id="ljestvica"; mentorov „nije se javio“ je sat (maxFeedPolls*feedMs), ne broj pokušaja; prefiksni ključevi ["situations"]/["feed"] (jedan hotel po gostu)
Task 10: dispatched (BASE bdadb85, sonnet)
- Ruling 15 (Task 10, nalaz suprotan planu): plan je prihvatio „2 promjena“; Leonovo pravilo 10.10. (svaka brojka razumljiva, ispravan hrvatski) ima prednost → jedna funkcija množine na Intl.PluralRules("hr") za promjene i bodove — cijena ako krivo: nekoliko redaka.
Task 10: fix round 1 dispatched (resume izvođača; množina promjena, „+8 bodova“)
Task 10: minor (deferred): „bez odgovora (ocjena 1)“ → „ocjena 1/5“ radi dosljednosti; isti mjesec može se pojaviti u dvije nesusjedne skupine
Task 10: fix round 1/5 (2 addressed, 0 open — formatCount na Intl.PluralRules("hr"), „+8 bodova“; commits 1a7fd73..bc8fc23)
Task 10: complete (commits bdadb85..bc8fc23, review clean nakon 1 kruga)
Task 10: minor (deferred): nema testa za negativne brojeve kroz formatCount (−1 bod, −3 boda)
Task 12: dispatched (BASE bc8fc23, sonnet)
Task 12: complete (commits bc8fc23..da1ba0e, review clean; kontrolor potvrdio HAIR[5] = #A7ADB4 sijeda)
Task 12: minor (deferred): FeedScreen mentor li duplicira prikaz rečenice; HotelScreen test traži glavu kroz parentElement.parentElement
- Ruling 16: Task 13 se dijeli — 13a dorada izgleda (poznati nalazi + Ruling 14 + pre-line) ide izvođaču (kontrolor ne popravlja sam), 13b preglednik/README/BACKLOG radi kontrolor nakon spajanja Taska 14 — cijena ako krivo: nijedna.
Task 13a: dispatched (BASE da1ba0e, sonnet)
Task 13a: complete (commits da1ba0e..481d696, review clean); ⚠️ otvoreno za 13b: stvarni prikaz na 360/390 px; isto zaokruživanje na poslužitelju (Task 14)
Task 13a: minor (deferred): Delta size="lg" mrtva grana; unit.points ključ više nekorišten; game.ts uvozi wholePoints iz format.ts
## Task 13b (kontrolor, preglednik 390 px, poslužitelj BEZ ključa, 10.10. ~04:03)
- Radi: Moj hotel (cijeli brojevi, puni nazivi, 678.000 € u retku), situacije s licem, razgovor (polje poruke široko — bug popravljen), rezerva lika, neutralna presuda 3/5, S3 izbor, plan (cijena 145, pojačano) spremljen.
- NALAZ (server, Important): /situations.unanswered broji samo `chosen` (schema.rs:694), igra broji i slobodan odgovor (game.rs:429) → lažno upozorenje „Bez odgovora: 1“ → Task 15.
- NALAZ (za Leona, sadržaj/server): rezerva lika bez AI-ja „Gošća je previše ljuta da bi nastavila razgovor“ zvuči kao ocjena studenta; S3 naslov nosi „(nacrt)“; S3 nema opisa situacije, samo naslov.
- Ruling 17: Task 15 (jedno pravilo za „bez odgovora“) — izvođač sonnet u glavnom stablu, samo crates/api|app|game, bez gradnje server.exe (zaključan dok radi preglednik) — cijena ako krivo: nijedna.
Task 15: dispatched (BASE 481d696, sonnet)
Task 14: implementer DONE_WITH_CONCERNS (commits 3fb7be2, df49a7a na gradnja/mentor-ljudski; 279 passed; izvještaj spremio kontrolor — harness blokirao pisanje izvan worktreeja); brige 2–3 već riješene u 13a
- NALAZ preglednik (web, Important): sve `change` stavke jednog obračuna dijele `seq` (21) → React dvostruki ključ (12 grešaka u konzoli) u Changes/Tijeku
- NALAZ preglednik (Leon-pravilo): kartica „Zašto“ — troškovi zeleni s „+“, „situacija S1“ (kod), „+2 ljudi“ zeleno (odlasci), „osnova“ bez značenja, sirovih 30 redaka; mentor avatar centriran uz dug tekst; negativan napredak −2 u boji naglaska
- Ruling 18 (temelj, Leonovo „indeksi moraju biti dobro objašnjeni“): kartica „Zašto“ prelazi na ISTE ljudske činjenice koje dobiva mentor (Task 14 debrief_facts: po indeksu što ga je pokrenulo, s uzrokom i iznosom riječima) — jedan izvor istine na poslužitelju, nova ruta GET za objašnjenje mjeseca (aditivna promjena API-ja, iznimka od Global Constraint) → Task 16 nakon spajanja 14 i 15 — cijena ako krivo: ~pola dana; alternativa (prepisati oznake u web) = dva izvora istine.
Task 14: review dispatched (opus, review-bdadb85..df49a7a.diff)
Task 15: complete (commits 481d696..125fec6, review clean)
Task 15: minor (deferred): „0 kad runda nije otvorena“ napisano dvaput u views.rs (305-310, 108-112); nema HTTP testa za /situations.unanswered
- preglednik: 360 px bez vodoravnog pomicanja na /, /situacije, /situacije/S1, /plan, /mjesec/jun, /tijek ✅; Tijek: prazan razmak ispod „Obračun: N promjene“ (details) → Task 16
Task 14: review → Needs fixes (2 Important: primjer u promptu opisuje stvarnu S2 a činjenice ne nose izabrani odgovor ni plan; Operativa bez opterećenja osoblja)
- Ruling 19: uz 2 Important u krug ulaze i Minor 3 (čuvar slijep za ≤3 znamenke / „indeks G“), 4 (hrvatska Rust imena polja → engleska + serde rename, Global Constraint), 5 (+6 osoba uz 3 osobe), 6 (dvostruki retci situacije, „marketing“), 7 (sav JSON = podatak, ne uputa), 8 (zastarjela bilješka u ADR-023) — Leonovo pravilo razumljivosti ih čini dijelom specifikacije, a jeftini su — cijena ako krivo: jedan dulji krug.
Task 14: fix round 1 dispatched (resume izvođača opus)
Task 14: fix round 1/5 (8 addressed, 0 open — izmišljen primjer + odabrani_odgovor + odluke_u_planu, Driver::StaffLoad, čuvar ≤3 znamenke, engleska imena, ljudi riječima, spojeni retci, JSON=podatak, ADR-023; commits df49a7a..13988af)
Task 14: complete (commits bdadb85..13988af na gradnja/mentor-ljudski, review clean nakon 1 kruga)
Task 14: minor (deferred): resolve.rs:178 i :364 još tvrdo kodiraju 180 i formulu kazne (moral) — nije na load_penalty; load_driver šteta 0 kad je preopterećenje bilo i ostalo
Task 14: minor (deferred): poruka commita 13988af ne spominje promjenu jezgre (Driver, load_penalty)
- Leon 10.10. (anketa „više AI-ja“): likovi i mentor OSTAJU AI kao prije (već jesu — kontrolor ih je samo u testu gasio); trošak tokena smanjiti „na drugačiji način, izračunat ćemo kasnije“ → BACKLOG; NOVI LIKOVI: Ivan Gudeljević (zaštitar, često se opija), Nikola Magdić, Dr. Tin Šljivović, Leona Krešo, Dominik Smurjević („prati sve što je prljavo“) — uloge 2–4 i situacije pitane tekstom.
Task 16: dispatched (BASE df2b7f4 = spoj Taska 14, opus — server ruta + openapi + web, prosudba)
- Leon 10.10. ODLUKA: presuda Sonnet 5.5 (ne Opus), lik Haiku 5.5 (anketa), mentor Sonnet 5.5 → Task 18 (ADR-024 ✅). Haiku nema poslužiteljski fallback → pravilo po modelu.
Task 18: dispatched (BASE df2b7f4+, sonnet, samo crates/ai + .env.example + DECISIONS.md; paralelno s Taskom 16)
- Leon 10.10. pitanje: „hoće li demo biti opsežniji da se može ući u sve ikone i vidjeti detaljnu analizu“ → Ruling 20: Task 17 „Detalj svakog pokazatelja“ — svaki indeks i kartica na Mom hotelu klikabilni → ekran detalja (vrijednost po mjesecima, što ga čini riječima, promjene zadnjeg mjeseca) iz objašnjenja Taska 16; nakon Taska 16 — cijena ako krivo: ~pola dana do dan prije demoa.
Task 18: implementer DONE (commit 86c376a; ai 18 passed; staru odluku zapisao kao zamijenjen ADR-018 t.4); review dispatched (sonnet)
Task 18: complete (commit 86c376a, review clean); ⚠️ živi API (Haiku effort low bez fallbacka) provjerava se u završnom prolazu s pravim AI-jem
Task 18: minor (deferred): popis modela s fallbackom tvrdo kodiran (novi model tiho bez fallbacka); nema testa s lažnim poslužiteljem za zaglavlje
- Leon 10.10.: više situacija i više od dva mjeseca — POSLIJE demoa (za demo je dovoljno što imamo) → BACKLOG u Tasku 13b. Pretpostavka (Ruling 21): novi likovi (Ivan, Dominik, Nikola, Tin, Leona) idu s novim situacijama, dakle također poslije demoa — cijena ako krivo: Leon ih je htio u demou → 1–2 h po situaciji.
- Leon 10.10. VIZIJA (poslije demoa, arhitektura kroz dokument): profesor pri kreiranju igre bira razdoblje (dani/tjedni/mjeseci), broj AI-situacija, dodaje vlastite situacije, bira vrste situacija i „branšu“ u kojoj se mogu dogoditi → BACKLOG „Otvoreno za Leona“ + zaseban dizajnerski dokument nakon demoa; pitanja postavljena tekstom.
- Leon 10.10. VIZIJA (razvoj cijele aplikacije): studenti ulaze u igru kodom kao u Kahootu (profesor kreira igru → kod → student upiše kod i ime) → BACKLOG uz profesorovo kreiranje igre.
Task 16: implementer DONE (commits 6ad6701, ce04fd9; baseline zelen; Rust 298, web 118); review dispatched (opus)
Task 16: review → Approved uz 1 Important (odlasci i ukupni moral sivi pod Zaposlenici iako Z = 60 % moral + 40 % zadržavanje; u lipnju Z +6 bez ijednog obojanog uzroka; izvođačevo obrazloženje u izvještaju je krivo)
- Ruling 22: Important 1 + Minor 1 → novi Task 19 (poslužitelj: odlasci ovaj vs prošli mjesec s predznakom Δzadržavanja, kao opterećenje; ukupna promjena morala obojana kad je jedini redak morala; harm⇔effect ostaje jedno pravilo) — paralelno s Taskom 17 (samo web); Task 16 zatvoren jer popravak traži nove ulazne podatke izvan opsega briefa — cijena ako krivo: nijedna, isti popravak, samo zasebno pregledan.
Task 16: complete (commits df2b7f4..ce04fd9, review approved; Important 1 → Task 19 po Rulingu 22)
Task 16: minor (deferred): 409 „Runda nije otvorena“ za neobračunat mjesec (treba month_not_resolved u app); težina ≥1 za retke potražnje (zapisati u ADR); ključ retka cause|amount ovisi o jedinstvenim naslovima situacija
Task 17: dispatched (BASE 86c376a, sonnet, samo web; + Minor 4 Taska 16: „Ovaj ga mjesec ništa nije pomaknulo.“)
Task 17: implementer DONE (commit 6569162; web 130); review dispatched (sonnet)
Task 17: complete (commits 86c376a..6569162, review approved); ⚠️ istaknuta varijabla sold/revenue/cash provjeriti u pregledniku sa stvarnim podacima (13b)
Task 17: minor (deferred): /pokazatelj/F/staff valjana ruta bez istaknutog; aria-current na isticanju upitan, tiho bez poklapanja; naslov „pomaknulo“ pokriva i „od čega se sastoji“; prateća kosa crta dopuštena
Task 19: implementer DONE (commit 29b58fc; Rust 307; uspoređuje stopu odlazaka, ne broj ljudi — stopa ovisi o moralu); review dispatched (opus) + balans kontrolor
Task 19: balans (kontrolor) cargo run -p balance --release -- --seeds 1000 --check na 29b58fc → EXIT=0, „Sve provjere prolaze.“, ponovljivost da
Task 19: review → Approved (formula točna, invarijanta drži, matematika jezgre ista); Minor 1 (siv redak „više“ kad je stopa ≥8 % oba mjeseca) i Minor 2 (lipanj kaže „prošli mjesec“ za početno stanje)
- Ruling 23: Minor 1 i 2 ulaze u kratki krug (demo počinje lipnjem — Minor 2 vidi svaki student; Leonovo pravilo razumljivosti) + nit 3 („udio osoblja koje je dalo otkaz“) — cijena ako krivo: jedan kratki krug.
Task 19: fix round 1 dispatched (resume izvođača)
Task 19: minor (deferred): dva retka stope uvijek prisutna i kad je „isto“ (šum na ekranu i u promptu); pritisci morala ×60 uvijek nadjačaju stopu u fallback rangiranju
Task 19: fix round 1 done (commit e565e40; lipanj „na početku igre“, 8 % istinit tekst, „udio osoblja koje je dalo otkaz“; 309 passed) — re-review spojen s krugom 2
Task 19: fix round 2 dispatched (svi „prošli mjesec“ u prvom mjesecu: opterećenje, sezonci; prvi mjesec iz popisa mjeseci igre, ne tvrdo lipanj)
Task 19: fix round 2 done (commit 4db99d9; prev_month iz popisa mjeseci; 310 passed); re-review dispatched (haiku→sonnet, krugovi 1+2)
Task 19: fix rounds 1–2/5 (4 addressed, 0 open; commits 29b58fc..4db99d9)
Task 19: complete (commits 6569162..4db99d9, review clean nakon 2 kruga; balans prolazi)
Task 19: minor (deferred): u slučaju poda kad udio pada (10 → 9 %) `because` kaže „većeg udjela“ (vjerojatno se ne prikazuje jer je neutralan)
## Task 13b-2 (kontrolor, preglednik, HEAD 4db99d9, bez ključa)
- Radi (390 px, bez ključa, nova igra): nema lažnog „Bez odgovora“ (Task 15 ✅); Lipanj završen: IndexBars s kraja mjeseca, mentor bez AI-ja u četiri dijela bez sirovih brojki, „Zašto se indeksi pomaknuli“ s bojama i opterećenjem „na početku igre“; /pokazatelj/Z/staff (kroz mjesece, Zaposleni istaknut, moral +5 zelen); /pokazatelj/F/revenue istaknuo „Prihod“ sa stvarnim podacima (⚠️ Taska 17 riješen).
- 360 px: /pokazatelj/Z/staff, /pokazatelj/F/revenue, /pokazatelj/R, /mjesec/jun, /tijek, / — scrollWidth ≤ 360 ✅; konzola 0 grešaka, 0 upozorenja ✅
- NALAZ (Minor, sadržaj): `znacenje` za „Zaposleni“ predugo i tehničko („od 8 % naviše taj je dio indeksa već na najnižem…“) → BACKLOG
Task 13c: dispatched (README „Igra u pregledniku“ + BACKLOG, sonnet)
Task 13c: complete (commit b8d7c7f, check:docs ✅; „failed obračun“ samo sužen; „množina“ nije bila u BACKLOG-u)
## Završni prolaz s PRAVIM AI-jem (HEAD 4db99d9 u ../hotelixsim-igra, ključ, 10.10. ~05:51 lokalno)
- Lik Haiku 5.5: 2 odgovora, prirodno i u ulozi, bez greške (fallback izostavljen ispravno). Presuda Sonnet 5.5: 5/5, 5 kriterija, <30 s.
- NALAZ (Important, Leonovo pravilo): presuda obrazloženje „Odgovor odgovara ponuđenom odgovoru 0“ — interni redni broj u tekstu za studenta → prompt presude (prompts/presuda*.md) zabraniti brojeve/indekse ponuđenih odgovora; ide u završni val popravaka.
## Kraj sesije 2 (10.10.): završni pregled grane NIJE pokrenut (Leon zatvorio sesiju) — sljedeća sesija: final review opus + val popravaka (presuda „ponuđenom odgovoru 0“) + spajanje. AI trošak ove sesije ≈ 0,06 $ (dnevnik).
## Sesija 3 (10.10. popodne): završni pregled i val popravaka
- Poslužitelj s ključem iz ../hotelixsim-igra (8080) ugašen na početku sesije.
- Presuda: commit 674f4fe — presuda.v3 + brana `VerdictError::InternalNumber` u validate_verdict (ADR-025 🟨); 77e0d96 FakeAi bez broja kriterija (e2e pao na brani — dobro uhvaćeno punim workspaceom).
- Završni pregled (opus, b607395..796618f): „With fixes“; 0 Critical; Important 1 utrka `me` nakon Zatvori mjesec (queries.ts:47, i T9 deferred minor — recenzent ne slaže s „bezopasno“); Important 2 S3 „(nacrt)“ i bez opisa. Rulings 1–23: nijedan nije označen kao kriv. Deferred: „fix before demo“ samo T6 Conversation isError i T9 me-refresh; ostalo „leave“.
- 72258b5: „(nacrt)“ maknut iz S3 i S4; test zamrznutog otiska koraka 3 prešao na zamrznutu kopiju (crates/content/tests/fixtures/step3-snapshot.json) — igre nose vlastiti snimak, pa se content/ smije mijenjati. Rust workspace 312 passed, clippy čist.
- Ruling 24: opis S3 (Important 2b) = Leonova sadržajna odluka + polje u API-ju → BACKLOG „PRIJE DEMOA“, ne u ovom valu — cijena ako krivo: profesor vidi S3 bez osnovne ponude (odgovori nose 15/20 %).
- Ruling 25: „bodovi“ i „osobe“ uz brojke (Minor recenzenta, DIZAJN kaže „+8“) — Leonovo pravilo razumljivosti ima prednost (kao Ruling 15) — cijena ako krivo: dvije riječi na ekranu.
- Ruling 26: Minor „pokazatelj iz Lipanj završen otvara zadnji obračunati mjesec“ ostaje odgođen (u demou se lipanj gleda prije zatvaranja srpnja) — cijena ako krivo: nakon srpnja klik iz lipnja pokaže srpanj.
- Task 20 (web val: utrka me, isError && !data, ljestvica boja, tab pokazatelja, tekstovi, jedinice, prazna lista neutralne presude): dispatched (BASE 72258b5, sonnet)
Task 20: complete (commit bfd1b7c, sonnet; web 134; RED→GREEN za utrku `me` potvrđen; izvođač koristio git stash za provjeru linta — necommitani BACKLOG i ledger preživjeli)
- 306b830: zamrznuta kopija s novim retkom (biome lint je pao), BACKLOG opis S3 PRIJE DEMOA.
- Ponovni pregled (sonnet, 796618f..306b830): Ready to merge — Yes; Minor: brana hvata „odgovornost 5“, propušta „2. kriterij“ → popravljeno; „prvi odgovor“, „odgovor broj dva“, „#2“ ostaju samo na uputi.
## Provjera u pregledniku (kontrolor, HEAD 306b830, poslužitelj BEZ ključa, 10.10. ~15:25)
- Nova igra → S3 bez „(nacrt)“ ✅; S1 slobodan odgovor → neutralna presuda 3/5 bez praznog okvira ✅; S3 protuponuda; Zatvori lipanj → „Lipanj završen“ „+3 boda“, ljestvica „+7 bodova / −3 boda“ ✅; Moj hotel odmah osvježen (Zadovoljstvo 57, „Rezultat: lipanj“, „56 osoba“) ✅.
- 360 px: /, /situacije, /situacije/S2, /plan, /mjesec/jun, /tijek, /pokazatelj/Z/staff, /pokazatelj/F/revenue, /pokazatelj/G — scrollWidth ≤ 360 ✅; aktivan tab „Hotel“ na /pokazatelj ✅; konzola 0 grešaka, 0 upozorenja ✅.
- NALAZ: S1 POKAZUJE ponuđene odgovore studentu → presuda.v3 „Student ne vidi ponuđene odgovore“ bilo netočno → ispravljeno (commit „presuda.v3 istinita o ekranu“), pravilo isto.
- Web test „obračunat lipanj“ jednom istekao na 5 s pod punim opterećenjem (samostalno ~1 s) → testTimeout 15 s u vite.config.ts.

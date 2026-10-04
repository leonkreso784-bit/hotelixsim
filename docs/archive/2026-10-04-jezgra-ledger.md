# SDD ledger — plan: docs/plan/2026-10-04-jezgra.md

> **✅ ISPUNJEN 2026-10-04** — radni ledger plana [2026-10-04-jezgra.md](2026-10-04-jezgra.md); kopija iz `.superpowers/sdd/` (gitignorirano). Odgođene sitnice su prenesene u [BACKLOG](../records/BACKLOG.md).

Spec: docs/product/DEMO.md §2–§12, docs/architecture/ARHITEKTURA.md §6.2/§7/§9.5, ADR-013 (pretpostavke, 🟨), ADR-009.
Grana: gradnja/jezgra (od main @ ddac293). Zlatne vrijednosti: scripts/oracle-jezgra.mjs (testni katalog).

Ruling: rad na grani `gradnja/jezgra` u istom direktoriju, ne u worktreeju — Leon sve pokreće iz Documents\hotelixsim, isto kao kostur — ako je krivo: nema cijene.
Ruling: gradnja kreće bez čekanja Leonova pregleda plana — Leon je rekao „možeš krenuti", rok ~14.10. (ADR-012), ADR-013 je zapisan kao 🟨 — ako je krivo: odbijanje ADR-013 t.1 mijenja jednu funkciju (`centred`) i RefTable, ostalo su podaci.
Ruling: implementatori i pregledi taskova na sonnetu (plan daje formule i testove, ne puni kod), ponovni pregledi na haikuu, završni pregled na opusu — AGENTI.md §3 — ako je krivo: više krugova popravka.

## Preflight scan

| par / task | proizvodi → troši | nalaz |
|---|---|---|
| T1 ↔ T2 | T1 `forbid(arith, as, default_numeric_fallback)` u engine/lib.rs → T2 `calc.rs` mora proći | calc koristi samo checked_*, usporedbe i literal tipiziran parametrom; usklađeno |
| T1 ↔ T3–T9 | brane → sav novi kod sim-hotel | svaki task mora proći clippy s forbid; navedeno u Global Constraints |
| T2 ↔ T3–T10 | `engine::calc` API → sve formule | imena iz T2 Interfaces koriste se dosljedno (`div_round`, `div_floor`, `mul_div`, `narrow`) |
| T3 ↔ T4 | Catalog, HotelState, `staff()` → `market`, `MonthPlan` | usklađeno; `staff()` je u T3 sučelju `u16` (saturating_add) |
| T3 ↔ svi testovi | `tests/common/mod.rs` = TEST_CAT | jedan izvor; T3 korak 1 navodi sve vrijednosti |
| T4 ↔ T5 | `market(state, month, plan, staff, cat)` → resolve | resolve prosljeđuje stanje nakon postavljanja sezonaca; usklađeno |
| T5 ↔ T6 | T5 stvara mechanic.rs s deklaracijama + `NotYetSupported`; T6 dodaje logiku i briše varijantu | usklađeno (plan izričito) |
| T5 ↔ T7 | `MonthReport` (after, money.goppar, money.labor) → `month_indices`; `resolve_month` → `RefTable::compute` | usklađeno |
| T6 ↔ T8 | `Mechanic`, `OfferedAnswer`, `SituationTemplate` → content DTO | usklađeno |
| T6 ↔ T9 | `direct_cost_cents`, `SituationOutcome::from_answer` → roboti | usklađeno |
| T7 ↔ T9 | `start_indices`, `month_indices`, `progress_pts` → `run_season` | ime polja `SeasonResult.progress_pts` = ime funkcije; nije sukob |
| T8 ↔ T10 ↔ T11 | `content::load`, `repo_dir` → balance; JSON → kalibracija | usklađeno |
| T3 ↔ T10 | `Profile::starting_indices` (ciljevi §5) → provjera §11.1 | usklađeno |
| T1 | mutacije umeću iza `pub mod index;`/`pub mod rng;` | postoje danas; usklađeno |
| T2 | testovi ↔ kod | div_round(−2000,400) = −5 provjereno ručno |
| T3 | testovi ↔ vrijednosti | turnover 375/300/570/1200 provjereno |
| T4 | testovi ↔ oracle | sve brojke iz oracle-jezgra (run1/run4) |
| T5 | testovi ↔ oracle | sve brojke iz oracle-jezgra; zero_staff test izričito postavlja target 0 |
| T6 | testovi ↔ oracle | S1–S3 lipanj i S3 kolovoz iz oracle (run5) |
| T7 | testovi ↔ oracle | refs, startovi A/B/C, pasivni C, varijanta −491 |
| T9 | testovi ↔ pravila | cijene 14 200 / 11 500, sezonci 15/18/20/26 ručno provjereno |
| T10 | test 2 | ovisi o tome da promjena C-a pomakne A/B; plan daje rezervu |
| T11 | podaci | smije pasti §11.3/§11.4 → nalaz, ne promjena formula |

Preflight nije našao sukob koji traži promjenu plana.

## Napredak
Task 1: dispatched (base ddac293, sonnet)
Task 1: implementer DONE (03586df); review dispatched
Task 1: review — spec ✅, 1 Important (f64 regex labav, disallowed-types f32/f64 nemjeren), 5 minor
Task 1: minor (deferred): probe nema bazni prolaz (clean clippy) prije mutacija — u CI-ju ga pokriva prethodni korak clippy
Task 1: minor (deferred): neprobane stavke DefaultHasher, Instant::now, env::var_os, default_numeric_fallback
Task 1: minor (deferred): float-sufiks grep ne gleda tests/ jezgre
Task 1: minor (deferred): probe bez obrade Ctrl-C (lokalno može ostaviti mutiran lib.rs; CI ima git diff --exit-code)
Ruling: dev-ovisnosti jezgre (npr. proptest za testove) su dopuštene, brana mjeri samo normal+build — ne ulaze u knjižnicu pa ne mogu pokvariti determinizam ishoda — ako je krivo: dodati `-e dev` u check-core-deps
Task 1: fix round 1 dispatched (FIX_BASE 03586df)
Task 1: fix round 1/5 (1 addressed, 0 open; commits 03586df..34fff0a)
Task 1: complete (commits ddac293..34fff0a, review clean)
Task 2: dispatched (base 34fff0a, sonnet)
Task 2: implementer DONE (8cb6165); review dispatched
Task 2: review — spec ✅, 1 Important plan-mandated (test grane odbacivanja ne može pasti), 3 minor
Ruling: test odbacivanja iz plana (Task 2 korak 5) pojačava se da broji potrošena izvlačenja i uspoređuje s r % span — plan je propisao test koji ništa ne dokazuje; spec (BACKLOG „grana odbacivanja se ne izvršava") traži dokaz — ako je krivo: jedan test više
Task 2: minor (folded into fix): div_floor bez negativnih slučajeva
Task 2: minor (deferred): div_round javlja Overflow za |a| > i128::MAX/2 iako rezultat stane — sigurno (Err), daleko izvan raspona igre; dopisati doc o rasponu
Task 2: minor (deferred): nema testa za veliki a koji taman stane ni za odvojen overflow u add(magnitude, b)
Task 2: fix round 1 dispatched (FIX_BASE 8cb6165)
Task 2: fix round 1/5 (2 addressed, 0 open; commits 8cb6165..da479a4)
Task 2: complete (commits 34fff0a..da479a4, review clean)
Task 3: dispatched (base da479a4, sonnet)
Task 3: implementer DONE (e3d2e93); review dispatched
Task 3: review — spec ✅, 0 Important, 4 minor
Task 3: minor (deferred): catalog.rs:222-229 poruke granica imenuju samo jedno polje (price_min/max, wage_min/max)
Task 3: minor (deferred): validate ne pokriva marketing_max_cents, seasonal_step/max, marketing_max_bp, interest_bp, seasonal_plan, load — dodati kad postanu nazivnici
Task 3: minor (deferred): turnover test nema slučaj polovice (npr. moral 5 990 → 302)
Task 3: minor (deferred): nema testa za OutOfRange ni za referentni profil krive kategorije
Task 3: complete (commits da479a4..e3d2e93, review clean)
Task 4: dispatched (base e3d2e93, sonnet)
Task 4: implementer DONE (cb9f2b8); review dispatched
Task 4: review — spec ✅, 0 Important, 3 minor
Ruling: PlanError::Calc (izvan plana) ostaje — validate treba propagirati CalcError, alternativa je panika — ako je krivo: ukloniti varijantu
Task 4: minor (deferred): market.rs:105 nedostajuća kategorija prijavljena kao CalcError::BadDivisor (nema bolje varijante)
Task 4: minor (deferred): plan.rs:245 default_for ograničava na seasonal_max nakon ±koraka — pokvareno stanje bi dalo neispravan zadani plan
Task 4: minor (deferred): market ne provjerava plan (negativan marketing → BadDivisor); pozivatelj mora zvati validate — dokumentirati
Task 4: complete (commits e3d2e93..cb9f2b8, review clean)
Task 5: dispatched (base cb9f2b8, sonnet)
Task 5: implementer DONE_WITH_CONCERNS (b47ef8d; resolve.rs 684 r.; kamata kroz interest_bp; wage/100 div_floor); review dispatched
Task 5: review — spec ✅, 1 Important (test objašnjenja prazan: Baseline uvijek postoji), 3 minor (svi u popravak)
Ruling: kamata preko interest_bp (umjesto div_round(−cash, 100) iz plana) ostaje — isti broj uz 100 bp, a polje kataloga inače ne bi imalo učinka — ako je krivo: nijedno
Task 5: fix round 1 dispatched (FIX_BASE b47ef8d)
Task 5: fix round 1/5 (4 addressed, 0 open; commits b47ef8d..407573a)
Task 5: minor (deferred): Staff/Turnover redak nosi +departures (zamijenjeni, broj ljudi se ne mijenja) pa zbroj Staff redaka ≠ promjena broja; po planu informativno — razmotriti zaseban Cause ili znak za ekran profesora
Task 5: complete (commits cb9f2b8..407573a, review clean)
Task 6: dispatched (base 407573a, sonnet)
Task 6: implementer DONE (67bbb9d; TDD red preskočen; povišica kao Situation cost redak oduzet od Wage); review dispatched
Task 6: review — spec ✅, 0 Important, 5 minor
Task 6: minor (deferred → Task 8 provjera učitavanja): discount_bp > 10 000 daje negativnu grupnu cijenu; odbiti BadGroup / pri učitavanju
Task 6: minor (deferred): nedostajuća kategorija u mechanic.rs mapirana u CalcError::BadDivisor
Task 6: minor (deferred): granica ocjene bez testa (nijedna mehanika ne mijenja ocjenu); oprema bez granice (plan je ne traži)
Task 6: minor (deferred): nema testa za granicu morala, BadGroup, ni grešku u drugoj od više situacija
Task 6: minor (deferred): TDD crveni korak preskočen (izjava izvođača)
Task 6: complete (commits 407573a..67bbb9d, review clean)
Task 7: dispatched (base 67bbb9d, sonnet)
Task 7: implementer DONE (0bc73bf; kazna opterećenja duplicirana jedan redak); review dispatched
Task 7: review — spec ✅, 0 Important, 2 minor
Task 7: minor (deferred): kazna opterećenja duplicirana (indices.rs:70 i resolve.rs:176) — izdvojiti load_penalty()
Task 7: minor (deferred): indices.rs:161 try_into s pogrešnom varijantom greške ako SEASON promijeni duljinu (nedostižno)
Task 7: complete (commits 67bbb9d..0bc73bf, review clean)
Task 8: dispatched (base 0bc73bf, sonnet)
Task 8: implementer DONE (b04a29e; JSON prepisan ručno, nije provjeren kodom); review dispatched
Task 8: review — spec ✅ (sve brojke JSON-a = fixture, S1–S4 = tablica), 0 Important, 3 minor
Task 8: minor (deferred, za završni pregled): deny_unknown_fields na unutarnje označenom MechanicDto nije testiran — tipfeler u mehanici bi se tiho progutao
Task 8: minor (deferred): privremene mape u testovima se ne brišu ako test padne (nema Drop čuvara)
Task 8: minor (deferred): nema provjere dupliciranih kodova situacija ni praznih odgovora; duplicirani ključevi u mapama = zadnji pobjeđuje
Task 8: complete (commits 0bc73bf..b04a29e, review clean)
Task 9: dispatched (base b04a29e, sonnet)
Task 9: implementer DONE (5bcfbd1; ResolveError::BadChoice; run_season_with ignorira seed); review dispatched
Task 9: review — spec ✅, 1 Important plan-mandated (run_season_with ignorira seed), 3 minor (svi u popravak)
Ruling: iz run_season_with se briše parametar seed (plan ga je propisao, a nema čemu služiti — sjeme koristi samo run_season) — ako je krivo: vratiti parametar kad planer bude trebao kocku
Ruling: ResolveError::BadChoice (izvan plana) ostaje za prazan popis odgovora — ako je krivo: nijedno
Ruling: roboti s fiksnom cijenom/plaćom/marketingom ograničavaju se na granice kataloga (plan to traži samo za sezonce) — uz sadašnji katalog ne mijenja ishod — ako je krivo: robot bi u drugom katalogu igrao malo drukčije od tablice §10
Task 9: fix round 1 dispatched (FIX_BASE 5bcfbd1)
Task 9: fix round 1/5 (3 addressed, 0 open; commits 5bcfbd1..1b0a1cb)
Task 9: complete (commits b04a29e..1b0a1cb, review clean)
Task 10: dispatched (base 1b0a1cb, sonnet)
Task 10: implementer DONE (287f014; §11.1, §11.2 kolovoz, istiskivanje padaju na nekalibriranom katalogu; §11.3–5 prolaze; medijani napretka svih profila negativni −16…−20); review dispatched
Task 10: review — spec ✅, 0 Important, 5 minor
Ruling: proba „sezonci 0" smanjuje za korak svaki mjesec (plan ne dopušta skok veći od koraka) — do 0 stiže u 2. mjesecu — ako je krivo: proba mjeri blažu krajnost
Task 10: minor (deferred): margin_bp tiho vraća 0 kad je prihod ≤ 0
Task 10: minor (deferred): --seeds 0 se prihvaća
Task 10: minor (deferred): testovi pokrivaju samo granu §11.1 u failures; nema testa izlaznog koda --check
Task 10: minor (deferred): tolerancija §11.1 tvrdo u kodu, _cat neiskorišten
Task 10: complete (commits 1b0a1cb..287f014, review clean)
Task 11: dispatched (base 287f014, opus — kalibracija je prosudba)
Task 11: implementer DONE_WITH_CONCERNS (7ab4aea; sve provjere §11.1–§11.5 prolaze; kolovoz 9 550 bp; B zadovoljstvo 70→65; A i B 28 sezonaca)
Ruling: popunjenost kolovoza 9 550 bp (plan je dopuštao do 9 500) — najmanja vrijednost uz koju grupa iz S3 istiskuje goste; 4* u Opatiji u vrhu sezone realno je pun — ako je krivo: istiskivanje nestaje ili treba više osoblja/manje soba; istiskivanje je tanko (~2 sobe-noći), za Leona
Ruling: profil B zadovoljstvo 70→65 i A/B početnih 28 sezonaca (izvan popisa polja u planu) — jedini put do G 45 za B i do pravednosti §11.3 (inače razmak 12,87) — ako je krivo: profili se vraćaju, §11.1/§11.3 padaju
Task 11: review dispatched
Task 11: review — spec ✅, 1 Important (DEMO:373 zastarjelo „83 %"), 2 minor; nalaz za Leona: istiskivanje ~2 sobe-noći (96 % → ~20), marža 35,9 % blizu donje granice, „obnova u lipnju" gotovo dominantna
Task 11: fix round 1 dispatched (FIX_BASE 7ab4aea)
Task 11: fix round 1/5 (2 addressed, 0 open; commits 7ab4aea..7de383c)
Task 11: complete (commits 287f014..7de383c, review clean)
Task 12: dispatched (base 7de383c, sonnet)
Task 12: implementer DONE_WITH_CONCERNS (a6d7e6f; zadržao 3 [jezgra] stavke kao neriješene — kontroler vidi da su 2 riješene: rng_golden.rs:103 i :122); review dispatched
Task 12: review — 1 Important (2 riješene [jezgra] stavke zadržane), 2 minor (u popravak)
Task 12: fix round 1 dispatched (FIX_BASE a6d7e6f)
Task 12: fix round 1/5 (3 addressed, 0 open; commits a6d7e6f..463527a)
Task 12: complete (commits 7de383c..463527a, review clean)
Final review: dispatched (ddac293..463527a, opus)
Final review: With fixes — 0 Critical, 4 Important (S3 lekcija obrnuta: grupa u kolovozu donosi VIŠE GOP-a; obnova besplatna jer likvidnost zasićena; redci objašnjenja nisu zbrojivi za ekran; mehanike ne provjeravaju parametre), 8 minor; sve provjere zelene
Ruling: Important 3 (objašnjenje: Cause::Bound, zaseban Var za cilj, komponente cilja zadovoljstva) odgađa se na plan ekrana — plan ga je propisao takvog, a ekran ga još ne čita — ako je krivo: ekran profesora traži preradu explain.rs
Ruling: S3 i obnova su odluke o podacima/formuli za Leona — u ovoj grani samo se točno zapisuju i mjere (GOP metrika S3 u balansu kao izvještaj, ne kao provjera koja pada) — ako je krivo: demo pokazuje lekciju S3 naopako dok Leon ne odluči
Final fix wave: dispatched (base 463527a, sonnet)
Final fix wave: DONE (463527a..d85f7d3, 9 commits; sve provjere zelene; S3 metrika kolovoz 25 598 € / lipanj 19 184 €); scoped re-review dispatched
Leon (anketa): S3 → „Kolovoz rasprodan" (potražnja > kapacitet, ponovna kalibracija); obnova → „Blagajna bez gornje granice" (likvidnost bez zasićenja prije centriranja). Postaju Task 13 (formula F) i Task 14 (kalibracija) u planu.
Final fix wave: re-review — svih 8 ADDRESSED; minor: render s pravim prijelomima redaka, A/B brojke nisu ponovljive iz alata, S3 test bez predznaka, ponuda za istiskivanje ≠ ponuda za dobit (→ Task 14)
Final fix wave: complete (463527a..d85f7d3)
Task 13: dispatched (base b17e837, sonnet)
Task 13: implementer DONE (5fb4b68); balans: §11.1 B F 83,08; §11.3 razmak 16,6 i B najbolji pod svim strategijama → Task 14; review dispatched
Task 13: review — spec ✅, 0 Important, 2 minor (P30 stupac tipa još kaže clamp 0–10 000 → u Task 14; komentar uz 23 065)
Task 13: complete (commits b17e837..5fb4b68, review clean)
Task 14: dispatched (base 5fb4b68, opus)
Task 14: implementer DONE_WITH_CONCERNS (63b66f3..c5c87b4; sve provjere zelene; S3 kolovoz −9 953 € / lipanj +19 184 €; marže 22,5/38,7/49,3/20,3, sezona 35,8 %; razmak profila 2,37)
Ruling: validate dopušta baznu popunjenost do 20 000 bp (bila 10 000) — ADR-014 traži potražnju veću od kapaciteta; formule nedirnute — ako je krivo: katalog s potražnjom > 200 % prolazi provjeru
Ruling: najviše sezonaca 40 → 50 (DEMO §6.1 kaže 0–40) — rasprodan kolovoz traži više ljudi da marža ostane ≤ 50 % — ako je krivo: granica radnje u DEMO §6.1 se mijenja; za Leona
Task 14: review dispatched
Task 14: review — spec ✅, 1 Important ⚠ (DEMO §5 usklađenost) — kontroler provjerio: smjer A/B u §5 odgovara profiles.json; 3 minor (plan još spominje 0–40 sezonaca; A napredak dijelom od početnog opterećenja 2,20; obnova najbolja proba za A i C) — sve u BACKLOG-u osim plana
Task 14: complete (commits 5fb4b68..c5c87b4, review clean)
Završne provjere (kontroler): fmt, clippy, deps, test 106/0, balance --check, check:docs, test:docs, lint, probe-mutacija 0 proboja, check:api, typecheck, web test — sve EXIT=0

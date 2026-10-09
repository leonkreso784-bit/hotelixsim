# SDD ledger — plan: docs/plan/2026-10-09-ai-sloj.md

Spec: ARHITEKTURA §8.2–8.4, §9.4, §10; DEMO §1.2, §1.5, §6.2, §7, §11 t.7; ADR-018 (Leon), ADR-019 (pretpostavke).
Grana: gradnja/ai-sloj · početak grane (merge-base main): 5e3f452 · plan commit: 27f85aa
Modeli (AGENTI.md §3): izvođači sonnet · pregled sonnet · kratki ponovni pregled haiku · završni pregled opus.

## Preflight scan

| par / task | proizvodi → troši | nalaz |
|---|---|---|
| T1 sam | test `resolve_refuses_other_core_version` piše `resolve_job(Month::Jun, false)`, a T1 ima stari potpis | Ruling ispod (R1) |
| T1 → T2 | `Snapshot::to_value/from_value/legacy_fingerprint` → T2 test starog snimka | slaže se |
| T2 → T3/T4/T6 | `Content.ai` (`SituationAi`, `Character.fallback_reply`) → talk, ai_input, run_chat | slaže se; svaki literal `Content {..}` izvan content mora dobiti `ai` |
| T3 → T5 | T3 feed preskače 6 novih događaja → T5 dodaje stavke | slaže se |
| T3 → T6 | `ReplyBody`, `AiSource`, `AiMeta`, `ChatLine`, `MAX_CHAT_MESSAGES` → app | moraju biti re-exportani iz `game` (R2) |
| T4 → T6 | `resolve_job(m, bool) -> ResolveStep`, `NEUTRAL_FAILED`, `fallback_debrief`, `DebriefInput: Clone`, `response_pending` (dodaje T6) | re-export (R2); `response_pending` dodaje T6 u game — dopušteno |
| T4 → T6 (privremeno) | T4 mijenja `app/jobs.rs` samo da se prevodi; T6 zamjenjuje | slaže se |
| T5 → T6 | `FeedItem::Verdict { late }`, `LastMonth.mentor` → T6 test | slaže se |
| T6 → T7 | `JobQueue::defer`, `Job.age_secs`, `Decision.enqueue: Vec` → store | T6 privremeni `defer` = Err; T7 pravi — slaže se |
| T6 → T9 | `DisabledAi` u api/tests/common → T9 rute | slaže se |
| T7 → T11 | povećanje razmaka 2/4/8/16 s → e2e s neuspjelim AI-jem | e2e bi čekao ~30 s: R3 |
| T8 → T9/T10 | `ai::Anthropic`, `ai::Models`, `render_assess` → server, mjerilo | `render_assess` mora biti `pub` (R4) |
| T6 sam | test `close_waits…` traži da memorijski `claim` preskače `running` | provjeriti u T6 (već navedeno u briefu) |
| T6 sam | `run_job_for_test` je `#[doc(hidden)] pub` test-API u produkcijskom kodu | prihvaćeno (R5) |
| T10 sam | `is_none_or` (Rust 1.82+), toolchain 1.98 | slaže se |
| ostali taskovi | testovi prema kodu u istom tasku | bez proturječja |

## Rulings

- R1 Ruling: u T1 test koristi `g.resolve_job(Month::Jun)`; T4 ga prebacuje na `(Month::Jun, false)` — plan to već kaže u napomeni — cijena ako krivo: nikakva.
- R2 Ruling: sve nove javne stavke `game`-a koje troši `app`/`ai` (event tipovi, `ReplyBody`, `ChatLine`, `Speaker`, `VerdictDraft`, `ValidVerdict`, `ResolveStep`, `NEUTRAL_*`, `AssessInput`, `CharacterInput`, `DebriefInput`, `fallback_debrief`, `MAX_*`) re-exportaju se iz `game/src/lib.rs` — cijena ako krivo: dulji `use` putovi.
- R3 Ruling: T11 e2e s neuspjelim AI-jem prije svakog `run_next_job` radi `UPDATE jobs SET run_after = now()` (umjesto čekanja odgode) — cijena ako krivo: test ne mjeri stvarno vrijeme odgode (to mjeri T7 test).
- R4 Ruling: `ai::request::render_assess` je `pub` jer ga mjerilo (T10) koristi — cijena: veće javno sučelje crate-a `ai`.
- R5 Ruling: `app::run_job_for_test` ostaje `#[doc(hidden)] pub` (jedini način da test simulira spori AI bez sata) — cijena: test-API vidljiv u crateu.

## Tasks
Task 1: minor (deferred): legacy_fingerprint_differs test je slab; from_value klonira Value pri svakom foldu (BACKLOG puni fold)
Task 1: Ruling: minor „legacy test ne zaključava stvarni hash koraka 3" prenesen u Task 2 kao obavezan test sa zamrznutim literalom — cijena ako krivo: jedan test više
Task 1: complete (commits 27f85aa..40a6da8, review clean)
Task 2: Ruling: odbijanje (f) „broj criteria nizova ≠ broj odgovora" je nemoguće jer je criteria polje po odgovoru — test (f) izostavljen — cijena ako krivo: nikakva
Task 2: minor (deferred): testovi odbijanja provjeravaju samo „S1" u poruci, ne razlikuju pravilo (load.rs)
Task 2: complete (commits 40a6da8..5e686b3, review clean)
Task 3: minor (deferred): nema testova da fold odbija pokvaren dnevnik (7. poruka, dvije studentove zaredom, krivi reply_to, AnswerChosen nakon ResponseSubmitted); nema testa trimanja ni record_debrief; odgovor lika prihvaća se i nakon ResponseSubmitted; check_talk klonira chat
Task 3: complete (commits 5e686b3..4cca64e, review clean)
Task 4: ⚠️ nazivi indeksa provjereni ručno prema DEMO §4 — slažu se
Task 4: minor (deferred): DebriefSituation ne razlikuje neodgovoreno od ponuđenog q1 (fallback bez „bez odgovora"); validate_verdict vraća NotFreeText i za nepoznatu šifru; doc „−" vs „-" u ai_input; my_hotel(_g) ne koristi argument
Task 4: complete (commits 4cca64e..4198354, review clean)
Task 5: minor (deferred): nema negativnog testa privatnosti feeda (tuđi hotel ne vidi Response/Verdict/Mentor); nema testa late/neutral stavke u feedu; nema API testa novih varijanti
Task 5: complete (commits 4198354..dfe90f1, review clean)
Task 6: Ruling: run_assess ne izlazi rano na !response_pending nego na „AI presuda (ili kasna) već postoji" — plan bi tako izgubio kasnu presudu kad posao počne nakon neutralne; cijena ako krivo: jedan AI poziv više nakon neutralne presude
Task 6: Ruling: AiError::Disabled ide odmah na rezervu bez ponavljanja — bez ključa ponavljanje nema smisla; cijena ako krivo: nikakva (ključ se ne pojavi usred posla)
Task 6: note: TDD bez RED snimke (testovi napisani nakon koda)
Task 6: minor (deferred): response_view unwrap_or_default na rubrici; view_of unwrap_or(MAX); WaitForVerdicts unutar decide troši pokušaj (rijetka utrka); pred-AI provjere nad starom igrom bez komentara; memory defer ignorira secs
Task 6: complete (commits dfe90f1..284bd04, review clean)
Task 7: minor (deferred): defer SQL ima niz razmaka (izgubljeni nastavci retka); nema testa točnih odgoda 2/4/8/16/60
Task 7: complete (commits 284bd04..71013d1, review clean)
Task 8: minor (deferred, PREPORUKA POPRAVITI): Client builder unwrap_or_default gubi timeout 150 s (lib.rs); retci lika u prijepisu za presuditelja bez quote_student
Task 8: minor (deferred): novi red u studentovoj poruci može krivotvoriti „Lik:" redak unutar <razgovor>; unwrap_or_default pri serijalizaciji DebriefInput; nema testa UTF-8 rezanja tijela; aws-lc-sys treba C alate (CI ubuntu — provjeriti prvi CI)
Task 8: complete (commits 71013d1..a872f30, review clean)
Task 9: minor (deferred): nema 404 testa za nove rute; neutral/DisabledAi nije provjeren na razini API-ja
Task 9: complete (commits a872f30..9dba812, review clean)
Task 10: minor (deferred, PREPORUKA POPRAVITI): mjerilo kad SVI AI pozivi padnu javlja „ne zadovoljava" (exit 1) umjesto „mjerenje nemoguće" (exit 2); leon_quality izvan 1..=5 prihvaćen tiho
Task 10: minor (deferred): AssessInput gradnja duplicirana u benchmark.rs i game/ai_input.rs; sadržaj se učitava dvaput; nema testa granice 800/799
Task 10: complete (commits 9dba812..3c8fd0a, review clean)
Task 11: minor (deferred, PREPORUKA POPRAVITI): na pravom Postgresu nije dokazan put „rok prošao dok procjena još radi" (posao procjene running, obračun odgađa, neutralna na roku) — Review Focus 3
Task 11: minor (deferred): odgoda se ne provjerava prije UPDATE roka; tick resetira i running; dupli replay blok; S1 upit presude bez filtra situacije; Biome preformatirao mjerilo JSON (samo razmaci)
Task 11: complete (commits 3c8fd0a..f2b2711, review clean)
Final review (opus): With fixes — 0 critical, 4 important (injekcija preko replika lika, timeout klijenta, mjerilo exit kod, e2e komentar roka)
Final fix wave: commit 3bdacd0, re-review: all 9 findings addressed, no new breakage
Final: minor (deferred): navodnik u atributu ime nije escapan (ime iz sadržaja); exit.max u benchmark::run bez testa; FakeAi fail/refuse kao dva bool-a
Ruling: ledger se NE briše na kraju — AGENTI.md §2 traži da ga /kraj kopira u docs/archive — cijena: gitignorirana mapa ostaje do /kraj
Kontrolor: sve provjere na 3bdacd0 EXIT=0 (fmt, clippy, core-deps, 262 testa/0 palih, balance, check:docs, check:api, lint, typecheck, pnpm test)

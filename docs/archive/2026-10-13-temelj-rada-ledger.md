# SDD ledger — plan: docs/plan/2026-10-13-temelj-rada.md

Spec: ADR-038, ADR-037, docs/plan/ROADMAP/kvaliteta.md. Grana model-v2/sloj1 (glavno stablo), BASE plana 82ae466.
Usporedno (drugi plan / ledger 2026-10-10-v2-sloj1): provjeravatelj S-4 u ../hotelixsim-s1-sit, graditelj spajanja KS-3 u ../hotelixsim-s1-obnova (2 Rust agenta).

## Preflight scan

| par / task | dijeljeno | nalaz |
|---|---|---|
| T1 ↔ T2 | ništa (T1: check-docs.*, AGENTI, README, ADR-038, plan; T2: check-kod.*, package.json, ci.yml, BACKLOG) | čisto |
| T1 ↔ T3 | `docs/README.md` (T1 pravilo 3 i odlomak područja; T3 redak u `architecture/`) | različiti odjeljci; sekvencijalno → čisto |
| T2 ↔ T3 | ništa | čisto |
| T1 sam | test očekuje „(podrucje: igra)" za kod+igra → kod mapira alias prije brojanja; postojeći test r.218 mijenja očekivanje kod→igra (u koraku 1) | dosljedno |
| T2 sam | testovi: dug iznad 1000 ne pada dok ≤ max (izuzima se od tvrde 1000); poruke u testovima = poruke u sučelju | dosljedno; implementacija mora DUG izuzeti od tvrde granice |
| T3 sam | „test" je grep naredba s EXIT=1 na kraju | dosljedno |

Ruling: cigle se izvode jedna za drugom (T1 → T2 → T3) iako su neovisne — skill zabranjuje paralelne graditelje u istom stablu; dijele README — cijena: ~1 h dulje.
Ruling: ADR-038 traži čuvara testova prije graditelja → za T1 i T2 zaseban agent piše testove i dokazuje RED (bez commita), graditelj ih ne smije mijenjati; za T3 je „test" grep iz plana pa nema zasebnog čuvara — cijena: ako je grep preslab, prepis promakne (provjeravatelj čita i sadržaj).
Ruling: dokumentacijski koraci su unutar taska (T1 korak 5), pa čuvar dokumentacije = provjera check:docs u graditelju + provjeravatelj; zasebni agent tek kad cigla mijenja kod igre — cijena: mala.
Task 1: RED potvrđen (čuvar testova, 94/92/2, task-1-tests.md); graditelj poslan, BASE 82ae466
Task 1: minor (deferred): novi test u prvom assertFails nema PLAN_C.md → usputna MRTVA POVEZNICA u izlazu; test i dalje ispravan (podniz)
Task 1: ⚠️ riješen (voditelj): test identičan briefu (pregled r.316-330), promjena starog testa navedena u task-1-tests.md prije graditelja → graditelj nije mijenjao testove.
Task 1: minor (deferred): ADR-038 t.8 i dalje kaže "Danas su područja kod i dokumentacija"; Posljedice bez kvalitete.
Task 1: minor (deferred): AGENTI.md:153,155,157 "(t. N)" bez imena ADR-a → "ADR-038 t. N".
Task 1: minor (deferred): check-docs.mjs:13 i 260 predugi komentari, poravnanje.
Task 1: complete (commits 82ae466..a910ea2, review clean)
Task 2: RED potvrđen (čuvar testova, 95/94/1 ERR_MODULE_NOT_FOUND, task-2-tests.md); graditelj poslan, BASE a910ea2
Task 2: Ruling: test 2 iz plana ("601 daje upozorenje") proturječi testu 4 (ZASTARIO UNOS DUGA u mapi bez DUG datoteka) — plan je kriv (voditelj), ispravak: test 2 provjerava `includes("IZNAD 600  b.mjs  601")` i da nema grešaka; ispravlja ČUVAR TESTOVA (ne graditelj), uz `biome format --write` testne datoteke — cijena ako krivo: test 2 slabiji (ne hvata višak upozorenja), ali test 4 pokriva zastarjele unose.
Task 2: GREEN 9d830ba (98/98, lint 0, check:kod 0 uz 10 upozorenja 600-1000); provjeravatelj poslan (paket review-a910ea2..9d830ba.diff)
Ruling: Task 3 (dokumentacija, disjunktne datoteke) kreće dok traje pregled Taska 2 — graditelji se ne preklapaju u datotekama; cijena: mogući index.lock pri istodobnom commitu (agent čeka i ponavlja).
Task 3: graditelj poslan, BASE 9d830ba (bez zasebnog čuvara testova — Ruling iz preflighta)
Task 2: minor (deferred, ZA ZAVRŠNI PREGLED — kandidat za popravak): check-kod.mjs:168 usporedba import.meta.url/argv[1] može tiho preskočiti glavni dio (EXIT 0 bez ispisa) uz symlink ili drugo slovo pogona — brana koja šuti; danas poziv relativan pa radi.
Task 2: minor (deferred): readdirSync baca bez poruke na nečitljivoj mapi; nema testa za argument/CHECK_KOD_ROOT/razmake (ručno probano: radi); BACKLOG prazan redak.
Task 2: complete (commits a910ea2..9d830ba, review clean)
Task 3: GREEN 2ab1c4b (grep EXIT=1, check:docs 0, PLATFORMA.md 46 + 8 dijelova 52–137); provjeravatelj poslan (čita sadržaj, ne samo grep)
Task 3: pregled Spec ✅, Needs fixes (I1 hosting prikazan kao odluka; I2 IDEJE zastarjele ćelije + ROADMAP/platforma.md:8 ledger) → popravak 1 poslan graditelju (+ minori poveznice ADR, D2 otvoren, preotimanje otvoreno, 2 retka BACKLOG Otvoreno za Leona)
Task 3: minor (deferred): Railway stupac i ukupni redak troška pri širenju su novi izračun [PROC] bez izvora; ARHITEKTURA §12 JWT čeka Leonovu potvrdu L10 (BACKLOG).
Task 3: fix round 1/5 graditelj gotov e3e9ebe (grep EXIT=1, check:docs 0); ponovni pregled poslan (haiku)
Task 3: fix round 1/5 (5 addressed, 0 open Critical/Important; M1 Railway brojke [PROC] bez izvora ostaje minor; kozmetički ostatak poveznica 02:130, 05:12, 05:41; commits 2ab1c4b..e3e9ebe)
Task 3: complete (commits 9d830ba..e3e9ebe, review clean)
Završni pregled: poslan (opus), raspon 161ff26..HEAD (cijela sesija: roadmap, ADR-037/038, granica 1200, K0-1, plan, K0-2, K0-3, K0-7)
Završni pregled (opus): POPRAVCI POTREBNI — I1 check:kod šuti (junction/symlink: EXIT 0 bez ispisa); I2 lažna crvena (broji .claude/worktrees/); I3 check:kod nije u README „Provjere prije commita"; I4 granica koda 600/1000 + dug nije Leonova odluka ni zapisana (BACKLOG stavka obrisana); I5 tablica vlasništva datoteka (K0-2) nije izrađena; + ADR-038 t.8 „Danas su područja kod i dokumentacija"; preporuka: kvaliteta.md status K0-1/2/3/7, pokazivači BACKLOG, tvrdnja o 600–1000 na popisu duga.
Ruling: I4 → granica koda 600/1000 i popis duga postaju ADR-039 🟨 ČEKA LEONOV PREGLED (tok rada, ADR-005), BACKLOG „Otvoreno za Leona" dobiva redak — cijena ako krivo: Leon promijeni brojke, jedna konstanta.
Ruling: završni popravak radi JEDAN graditelj i smije DODATI nove testove (ne mijenjati postojeće) jer završna faza ima jedan val popravaka — cijena: test piše isti agent koji popravlja (manja neovisnost), ublaženo ponovnim pregledom.
Završni val popravaka: f5871aa (I1–I5 + ADR-038 t.8 + kvaliteta.md); test:docs 102/0, check:kod 0, check:docs 0; ponovni pregled poslan
Završni val: ponovni pregled — sve ADDRESSED (I1–I5, ADR-038 t.8, kvaliteta.md); test:docs 102/0, check:kod 0, check:docs 0, lint 0; nova greška ne. Minor (deferred): rezerva po imenu datoteke u jePokrenuta; K0-1 čeka push + zeleni CI (Leonov OK).
FAZA TEMELJ RADA: kod gotov (161ff26..f5871aa). Otvoreno: K0-1 dokaz u CI-ju (push), K0-8 Leonova sjednica ADR-ova; K0-4/5/6 nakon spajanja grana. Plan u archive/ tek nakon K0-1.
K0-1: DOKAZAN 13.10. (6) — CI run 38446984164 za b495c8d: rust success, web success, Runner Image ubuntu-24.04 (Provisioner 20261002.596). Faza 0 otvorena još samo zbog K0-8 (Leonova sjednica ADR-ova) i K0-4/5/6 (nakon spajanja grana). Plan u archive/ ide na /kraj.

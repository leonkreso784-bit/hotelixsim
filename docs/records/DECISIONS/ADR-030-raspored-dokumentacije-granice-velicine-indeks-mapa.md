# ADR-030 — Raspored dokumentacije: granice veličine, indeks + mapa, ADR po datoteci
**Datum:** 2026-10-10 · **Status:** 🟨 **ČEKA LEONOV PREGLED** · **Vezano:** [plan](../../plan/2026-10-10-dokumentacija.md), [docs/README.md](../../README.md)

**Kontekst.** Leon 10.10.: *„Neke md datoteke imaju previše informacija … Na nekim mjestima ima preko 3000 linija — to je
JAKO JAKO loše. Sve treba biti savršeno raspoređeno u dokumentacijama jer to je srce naših operacija."* Plan sloja 1 imao
je preko 4000 redaka, arhivirani planovi do 4700, ADR-ovi su bili jedna datoteka od 623 retka.

**Odluka (pretpostavka).** Osam pravila; `check:docs` provodi točke 1, 2, 4, 5 i 8, a ostale su način rada:
1. **Indeks + mapa istog imena.** Dokument `X.md` koji preraste granicu postaje kratak indeks (svrha, tablica
   „§ → datoteka"), a dijelovi idu u mapu `X/` pokraj njega. `docs/README.md` ostaje jedini ulaz. Samo jedna razina
   (`docs/<uloga>/X/Y/*.md` je zabranjen). Stare poveznice i navodi u kodu („ARHITEKTURA §7.2") ostaju točni jer se
   § brojevi čuvaju u naslovima dijelova.
2. **Granice veličine** po vrsti dokumenta (predaja, ADR, indeks, živi dokument/plan/task, arhiva, `CLAUDE.md`) —
   tablica je u [docs/README.md](../../README.md#granice-veličine), a izvor u `scripts/docs-granice.mjs`.
3. **Planovi.** Plan do granice smije biti jedna datoteka; veći je pregled plus mapa taskova
   (`task-NN-<slug>.md`), a brief je sama task datoteka. Plan se piše kao jedna datoteka pa se prije aktivacije
   razdvaja (`corepack pnpm docs:plan`). Ledger ostaje u `.superpowers/sdd/`.
4. **Jedan aktivni plan po području** (`kod` ili `dokumentacija`); dvije sesije smiju raditi paralelno samo na
   različitim područjima.
5. **ADR po datoteci** u `docs/records/DECISIONS/`; `DECISIONS.md` je generirani indeks (`corepack pnpm docs:adr`) sa
   sidrima `adr-0NN`, pa stare poveznice i commit poruke i dalje vode na pravi redak. Poveznice u dokumentima idu
   ravno na datoteku ADR-a.
6. **Arhiva** se razdvaja istim alatom (pregled + mapa taskova); sadržaj ostaje doslovan, mijenjaju se samo putanje
   poveznica (jedna razina dublje). Ledgeri ostaju datoteke.
7. **Zapisnici** (`PROGRESS`, `CHANGELOG`, `BACKLOG`): kad prijeđu granicu, stariji unosi idu po mjesecima u mapu
   istog imena, indeks ostaje na vrhu. Danas nijedan nije blizu.
8. **Prijelazne iznimke** dok traje sloj 1: `docs/plan/2026-10-10-v2-sloj1.md` i `docs/architecture/MODEL_V2.md`
   (popis `PRIJELAZNE_IZNIMKE` u `docs-granice.mjs`); prazni se u Tasku 13 plana, a MODEL_V2 se razdvaja u Tasku 11.

**Dopuna 2026-10-13 (Leonova odluka, ne pretpostavka).** *„Nijedan dokument (.md) u repou nema više od 1000 redaka;
dugi se razgranaju u manje dijelove s kazalom, i brana pazi da se to ne vrati."* Zato: pravilo 12 u
[docs/README.md](../../README.md) (tvrda granica 1000 za svaki `.md`, bez prijelaznih iznimaka; izuzeci samo za
generirano/vanjsko, izrijekom), arhiva s 1100 na 1000 (ekrani Task 4 podijeljen po smislu na 4 i 4b), a plan Sloja 1
razdvojen je odmah, dok sloj još traje (prijelazna iznimka sada pokriva i njegovu mapu, ali samo za granicu 300).

**Pretpostavke izvedbe.** Zajednički alati (slug, prepoznavanje ograda koda, prepisivanje relativnih poveznica) žive u
jednoj datoteci `scripts/docs-alati.mjs` da se ne razilaze. Slug ADR-a i taskova ima najviše 6 riječi. Razdvajanje
ima `--provjeri` koji dokazuje „spoji natrag = izvornik" prije nego što se izvornik zamijeni. Broj ADR-029 ostaje
rezerviran za sloj 1, pa u indeksu namjerno postoji praznina.

**Cijena ako krivo.** Drukčije granice: jedna konstanta u `docs-granice.mjs` i ponovni `--popis`. Drukčija podjela
dokumenta: ponovno `razdvoji` s drugim brojem dijelova (stari § brojevi se ne gube). ADR natrag u jednu datoteku:
jedna skripta koja spoji datoteke u `DECISIONS.md`. Premještanje je išlo bez promjene sadržaja, pa se ništa ne
prepisuje ispočetka; najskuplji dio, navikavanje na novi raspored, ne vraća se nikakvim alatom.

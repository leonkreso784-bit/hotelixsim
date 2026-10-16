# Sljedeća sesija — dvije ankete, PRAVILO, Leonovi pregledi i banka ljudi K2

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Spojeni su B4c (f7ac7cc), K1a (adf5a4d) i K1b u `model-v2/sloj1` (16.10.; **ništa nije pushano**, 38 commita ispred `origin/model-v2/sloj1` — provjeri `git log origin/main..` i `git status -sb`).
Plan: [2026-10-14-popravak-temelja.md](./plan/2026-10-14-popravak-temelja.md) → „Faza B". Ledgeri (vjeruj im i git logu):
`.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` (popravak) i `.superpowers/sdd/banka-ljudi-k1/progress.md` (banka ljudi);
kako se šalju agenti: `upute-izvodacima.md` u mapi popravka (popis CI provjera = `.github/workflows/ci.yml`). Izvještaj prošlog kraja: `kraj-16-10-6.md`.

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo. Ledgeri: .superpowers/sdd/2026-10-14-popravak-temelja/progress.md i .superpowers/sdd/banka-ljudi-k1/progress.md. Prvo pročitaj upute-izvodacima.md.
Prvo `df -h /c` (mora biti > 100 GB). Laptop je na punjaču. Push samo uz moj OK.
1) Anketa o redoslijedu faza (plan: PRAVILO-a → L1-13 → L1-13b → L1-14 → L1-15 → C; moja želja: faza C i likovi s izgledom u sučelju) uz kratki nacrt "izgled lika".
2) Anketa o pravilu za zalijepljene odgovore (kvar B13) prije PRAVILO-a.
3) Moji pregledi: 20 likova banke ljudi (pečat serija) i slijepi pregled tekstova.
4) K2 banke ljudi (50 + 50) usporedo; svako stablo svoj CARGO_TARGET_DIR.
```

## Cilj sesije

1. **Anketa o redoslijedu** ([ADR-054](./records/DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 3): prvo kratki nacrt "izgled lika"
   (deterministički iz roda, dobi i uloge, ili slike; cijena i licenca), zatim anketa s cijenom preskakanja (faza C prije L1-13 ostavlja bonus popravka +1 i dug
   `SilenceNotBetter` 16). Odluka je Leonova; voditelj ne odlučuje umjesto njega.
2. **Anketa o pravilu za zalijepljene odgovore** ([BUGS](./records/BUGS.md) B13): klik odgovarajućeg odgovora / neutralna 3 / najlošija. Prepoznavanje je u kodu (70 % riječi),
   ne u AI-ju, da ne troši tokene.
3. **PRAVILO-a** ([kartica](./plan/2026-10-14-popravak-temelja/task-12-pravilo-vlastiti-odgovor.md), `CORE_VERSION` 2.5.0, čuvar prije graditelja), zatim PRAVILO-b i -c usporedo;
   mora sletjeti prije L1-13. Prije spajanja `--check` 1000 tjedno i mjesečno.
4. **Leonovi pregledi:** (a) 20 likova (`banka-ljudi-k1/pregled-20.html`) pa pečat serija-000 i odluke o rodu po ulozi; (b) slijepi pregled tekstova B4a, B4b, B4c.
5. **K2 banke ljudi** ([BANKA_LJUDI](./architecture/BANKA_LJUDI.md) §5.1): 50 radnika + 50 gostiju, dva agenta; čuvar piše odgođene minore K1 (pravilo "Azija hrvatski 0–1", S6 unutar uloge).
6. **Push `model-v2/sloj1` uz Leonov OK** (B1b, B4a–c, K1a/K1b, kartica PRAVILO i docs; danas ništa nije pushano).

## Upozorenja

- **Svako radno stablo svoj `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>`**, briše se sa stablom ([BUGS](./records/BUGS.md) B12; [AGENTI](./workflow/AGENTI.md) t. 6).
- **Tjedna težina B je 48,9 % uz granicu 50 %**; svaka promjena sadržaja ili učinka može srušiti „B3 težina". Brane se ne labave.
- **Najviše 2 agenta prevode Rust**; gradnja samo na punjaču; disk > 100 GB. `check:api` EXIT 127 = okolina (0xC0000142), ponoviti; `SeasonScreen` test „ne javi se" zna pasti pod opterećenjem — izolirano.
- JSON sadržaja formatirati prije commita (biome lint ga provjerava).
- Stare probne v2 igre daju 409 (jezgra 2.4.0, a nakon PRAVILO-a 2.5.0). Novo stablo traži `corepack pnpm install --frozen-lockfile`.
- U glavnom stablu je Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- Dvije ankete na početku (redoslijed faza; pravilo za zalijepljene odgovore) i pregled 20 likova (rod po ulozi, imena, sumnjivi hoteli Kvarnera).
- Slijepi pregled tekstova B4a, B4b, B4c (u mapi popravka; blokira L1-15).
- Imena profesora FMTU-a za `zabranjeno.json` banke ljudi.
- **Nacrt „strani radnici iz Azije: plaća i obuka"** — sukob s MODEL_V2 §5.4; odluka prije L2b-3 ([BACKLOG](./records/BACKLOG.md)).
- L1-13: samo najbolji odgovor troši novac (BACKLOG); „oko +10" za popravak; 🟨 [ADR-051](./records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md),
  [ADR-053](./records/DECISIONS/ADR-053-pretpostavke-16-10-oznake-testovi-b4.md) i [ADR-055](./records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md);
  5 pitanja [VODITELJ_I_TRZISTE](./architecture/VODITELJ_I_TRZISTE.md) §3.

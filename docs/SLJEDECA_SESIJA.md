# Sljedeća sesija — POPRAVAK TEMELJA, faza B (prva cigla: B1b)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Spojeni su B2a, B2b, B3a, B3b i B1a (`model-v2/sloj1` 39e5f25; push samo uz Leonov OK — provjeri `git log origin/main..` i
`git status -sb`). Plan: [2026-10-14-popravak-temelja.md](./plan/2026-10-14-popravak-temelja.md) → „Faza B". Ledger (vjeruj mu
i git logu): `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`; kako se šalju agenti: `upute-izvodacima.md` u istom
direktoriju (popis CI provjera = `.github/workflows/ci.yml`).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo POPRAVAK TEMELJA, faza B. Ledger .superpowers/sdd/2026-10-14-popravak-temelja/progress.md.
Prvo pročitaj upute-izvodacima.md u istom direktoriju. Prva cigla B1b (kartica task-08b), pa B4a.
Prvo `df -h /c` (mora biti > 100 GB). Laptop je na punjaču. Push samo uz moj OK.
```

## Cilj sesije

1. **B1b** ([kartica](./plan/2026-10-14-popravak-temelja/task-08b-b1b-uvjet-condition-of.md)): `condition_of` (uvjet stanja
   „čijeg odjela"), balans bajt-isti, bez dizanja `CORE_VERSION`. Stablo `../hotelixsim-b1b`, čuvar → graditelj → pregled →
   spajanje → puni CI popis → brisanje stabla isti dan.
2. Zatim **B4a** ([kartica](./plan/2026-10-14-popravak-temelja/task-11-b4-prepis-oznaka-i-uvjeta.md)); **prije spajanja
   svakog B4 koraka** vrti balans v2 `--check` 1000 sjemena tjedno i mjesečno.
3. Ako ostane vremena: napiši karticu **PRAVILO** ([ADR-047](./records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md);
   B3b je spojen, pa se kartica piše sada).

## Upozorenja

- **Tjedna težina B je 48,9 % uz granicu 50 %** (bonus popravka privremeno +1, [ADR-050](./records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md));
  svaka promjena sadržaja ili učinka može srušiti „B3 težina". Brane se ne labave.
- **Stare probne v2 igre daju 409** (jezgra 2.4.0, pečat s 10 slučajeva); za igranje treba nova igra.
- **`check:api` nakon spajanja s vlastitim `CARGO_TARGET_DIR=target/check-api`** dok drugo stablo prevodi ([AGENTI](./workflow/AGENTI.md)).
- Gradnja (prevođenje Rusta, balans, testovi) samo na punjaču; najviše 2 agenta prevode Rust; disk > 100 GB.
- `SeasonScreen` vitest test „ne javi se" zna pasti pod opterećenjem — ponovi izolirano prije nego ga proglasiš kvarom.
- U glavnom stablu može biti Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj; pitaj Leona.

## Otvoreno za Leona

- 🟨 [ADR-051](./records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md) (pretpostavke faze B),
  [ADR-048](./records/DECISIONS/ADR-048-lik-v3-redni-broj-poruke-i-zakljucak.md) (lik.v3) i stariji 🟨 ADR-ovi; P1–P8 u planu prije B4.
- Slijepi pregled tekstova B4 (dokument, ne anketa); „oko +10" za popravak odlučuje Leon u L1-13.
- Vidljivost morala u igri — BACKLOG; 5 pitanja [VODITELJ_I_TRZISTE](./architecture/VODITELJ_I_TRZISTE.md) §3; banka ljudi K1 nakon popravka.

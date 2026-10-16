# Sljedeća sesija — POPRAVAK TEMELJA (B4c) i banka ljudi (K1a) usporedo

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Spojeni su B1b, B4a i B4b (`model-v2/sloj1` 8d134a4, 16.10.; **ništa nije pushano** — provjeri `git log origin/main..` i `git status -sb`).
Plan: [2026-10-14-popravak-temelja.md](./plan/2026-10-14-popravak-temelja.md) → „Faza B". Ledgeri (vjeruj im i git logu):
`.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` (popravak) i `.superpowers/sdd/banka-ljudi-k1/progress.md` (banka ljudi);
kako se šalju agenti: `upute-izvodacima.md` u mapi popravka (popis CI provjera = `.github/workflows/ci.yml`).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo POPRAVAK TEMELJA (B4c) i banku ljudi (K1a) usporedo. Ledgeri: .superpowers/sdd/2026-10-14-popravak-temelja/progress.md i .superpowers/sdd/banka-ljudi-k1/progress.md.
Prvo pročitaj upute-izvodacima.md. Prvo `df -h /c` (mora biti > 100 GB). Laptop je na punjaču. Push samo uz moj OK.
```

## Cilj sesije

1. **B4c dovršiti od c69f022** (grana `popravak/b4c-oznake`, stablo `../hotelixsim-b4c`, čuvar 479f6e2): provjeri commit 1 (balansi bajt-isti
   prema `nakon-b4b/`, omjeri, T4b); commit 2 = uvjeti kvarova P-BRAVE, P-CURENJE-VODE, P-DIZALO (`condition_of: rooms`, Smještaj ispod 65) i
   `external_cause` P-INSPEKCIJA; dokument za Leona; cijeli CI; pregled opus; **anketa Leonu o pragu 65 s brojkama prije spajanja**
   (uz prag pada brana J-3 i tri testa kapaciteta, [BACKLOG](./records/BACKLOG.md)); spajanje; CI na spojenom.
2. **K1a** ([BANKA_LJUDI](./architecture/BANKA_LJUDI.md) §5.1): novi graditelj (opus) nastavlja nad **necommitanim kodom** u `../hotelixsim-ljudi`
   (grana `sadrzaj/banka-ljudi`, čuvar 34dd18c, 42 testa) ili kreće ispočetka uz obrazloženje; pregled; spajanje u sloj1. Zatim **K1b**
   (20 primjera; Leon pregleda svih 20).
3. Ako ostane vremena: kartica **PRAVILO** ([ADR-047](./records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md)); mora sletjeti prije L1-13.
4. **Push `model-v2/sloj1` uz Leonov OK** (B1b, B4a, B4b i docs; danas ništa nije pushano).

## Upozorenja

- **Tjedna težina B je 48,9 % uz granicu 50 %**; svaka promjena sadržaja ili učinka može srušiti „B3 težina". Brane se ne labave.
- **Najviše 2 agenta prevode Rust** (popravak + banka ljudi dijele tu granicu); `check:api` s vlastitim `CARGO_TARGET_DIR=target/check-api`
  ([AGENTI](./workflow/AGENTI.md)); jedan zajednički `CARGO_TARGET_DIR`, stablo se briše isti dan nakon spajanja.
- Gradnja samo na punjaču; disk > 100 GB. `SeasonScreen` vitest test „ne javi se" zna pasti pod opterećenjem — ponovi izolirano.
- Stare probne v2 igre daju 409 (jezgra 2.4.0). Novo stablo traži `corepack pnpm install --frozen-lockfile`.
- U glavnom stablu je Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- **Prag 65 za P-BRAVE, P-CURENJE-VODE, P-DIZALO** (anketa s brojkama, blokira spajanje B4c commita 2).
- Slijepi pregled tekstova B4a i B4b (`b4a-pregled-tekstova.md`, `b4b-pregled-tekstova.md` u mapi popravka; blokira L1-15).
- Imena profesora FMTU-a za `zabranjeno.json` banke ljudi.
- **Nacrt „strani radnici iz Azije: plaća i obuka"** — sukob s MODEL_V2 §5.4; odluka prije L2b-3 ([BACKLOG](./records/BACKLOG.md)).
- L1-13: samo najbolji odgovor troši novac (BACKLOG); „oko +10" za popravak; 🟨 [ADR-051](./records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md)
  i [ADR-053](./records/DECISIONS/ADR-053-pretpostavke-16-10-oznake-testovi-b4.md); 5 pitanja [VODITELJ_I_TRZISTE](./architecture/VODITELJ_I_TRZISTE.md) §3.

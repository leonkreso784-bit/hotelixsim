# Sljedeća sesija — POPRAVAK TEMELJA, faza A (nastavak: A4 pregled, A5, A6, kraj faze)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Leon 14.10. odlučio: **popravak, ne gradnja ispočetka** ([ADR-043](./records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md),
[revizija](./architecture/REVIZIJA_TEMELJA.md)), i samo popravak dok nije gotov. Plan:
[2026-10-14-popravak-temelja.md](./plan/2026-10-14-popravak-temelja.md). Ledger (vjeruj mu i git logu):
`.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` → tablica „STANJE".

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo POPRAVAK TEMELJA, faza A. Ledger .superpowers/sdd/2026-10-14-popravak-temelja/progress.md (tablica STANJE).
Redom: pregled A4, graditelj A5, pa A6, pa kraj faze A. Prvo `df -h /c` (mora biti > 100 GB). Push samo uz moj OK.
```

## Stanje (15.10.)

- **Spojeno u `model-v2/sloj1` i pushano na GitHub** (0269e07): A1 brane modela / mjere robota + situacije (L1-11),
  A2 načela igre kao provjere sadržaja, A3 pečat v2 (`CORE_VERSION` 2.2.0 — Leonove stare probne v2 igre daju 409),
  A7 alat `corepack pnpm grane` i pravila kratkih grana. Balans, pečat i v1 nakon svakog spajanja zeleni.
- **`main`** (19f7b27): README s cijelom idejom igre, `LICENSE` sva prava pridržana ([ADR-044](./records/DECISIONS/ADR-044-licenca-sva-prava-pridrzana.md)),
  popravak `rustls`. Kolega 16.10. pregledava repo.
- **Otvoreno:** A4 `../hotelixsim-a4` @ 677e94c (graditelj gotov, igra = balans) → **pregled**; A5 `../hotelixsim-a5`
  @ e1148eb (testovi čuvara, RED) → **graditelj**; A6 čeka A5.
- **Disk** ([BUGS](./records/BUGS.md) B11): sva stabla dijele `CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target`;
  stablo se briše isti dan kad se spoji. Svaka cigla vrti cijeli popis provjera iz CI-ja (B12).

## Cilj sesije

1. Pregled A4 → spajanje → balans na `sloj1` → brisanje stabla `-a4`.
2. A5 graditelj → pregled → spajanje; živi AI test samo uz Leonov OK. Zatim A6 (čuvar → graditelj → pregled).
3. **Kraj faze A:** kopija (`hotelixsim-kopije/<datum>/`), završni pregled faze (najjači model), Leon igra mjesečnu v2
   igru na jednoj adresi; kartice faze B (B1–B4) pišu se tek tada.

## Otvoreno za Leona

- 🟨 pretpostavke P1–P8 u planu popravka (najvažnije: P3 duljina odgovora 1,5×, P4 šutnja gleda i trošak).
- Kopija i na privatni Google Drive (slanje van — treba izričit „da").
- Brisanje 31 spojene oznake grane (`git branch -d`; oznake ne zauzimaju disk, samo smetaju preglednosti).
- 5 pitanja [VODITELJ_I_TRZISTE](./architecture/VODITELJ_I_TRZISTE.md) §3; banka ljudi K1 tek nakon popravka.

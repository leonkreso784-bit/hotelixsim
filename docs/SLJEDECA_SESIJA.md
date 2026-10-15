# Sljedeća sesija — POPRAVAK TEMELJA, faza B (prva cigla: B2a)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Faza A je gotova i na GitHubu (`model-v2/sloj1` 024160a, CI zelen); Leon je odigrao mjesečnu v2 igru („puno bolja je
igrica"). Plan: [2026-10-14-popravak-temelja.md](./plan/2026-10-14-popravak-temelja.md) → „Faza B". Ledger (vjeruj mu i
git logu): `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` → tablica „STANJE" i „Preflight faze B". Kako se
šalju agenti: `upute-izvodacima.md` u istom direktoriju (popis CI provjera = `.github/workflows/ci.yml`).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo POPRAVAK TEMELJA, faza B. Ledger .superpowers/sdd/2026-10-14-popravak-temelja/progress.md (STANJE i
Preflight faze B). Prvo pročitaj upute-izvodacima.md u istom direktoriju. Prva cigla B2a (kartica task-09).
Prvo `df -h /c` (mora biti > 100 GB). Laptop je na punjaču. Push samo uz moj OK.
```

## Cilj sesije

1. **B2a** ([kartica](./plan/2026-10-14-popravak-temelja/task-09-b2-broj-situacija-u-postavkama.md)): broj situacija u
   postavkama igre, „jednom u krugu", `CORE_VERSION` 2.2.0 → 2.3.0. Stablo `../hotelixsim-b2a` od `model-v2/sloj1`,
   čuvar → graditelj → pregled (opus: jezgra i pečat) → spajanje → puni CI popis → brisanje stabla isti dan.
   Nakon spajanja reci Leonu: stare probne v2 igre opet daju 409.
2. Zatim **B2b** ([kartica](./plan/2026-10-14-popravak-temelja/task-09b-b2b-balans-prima-broj-situacija.md)), pa **B3a**
   ([kartica](./plan/2026-10-14-popravak-temelja/task-10-b3-mjera-udjela-situacija.md)) — B3a ispisuje brojke za Leonovu
   anketu o težini situacija (B3b ne kreće bez odgovora).

## Pravila koja vrijede (sažetak; izvor su ledger i memorija)

- **Gradnja (prevođenje Rusta, balans, testovi) samo kad je laptop na punjaču.** Na bateriji: samo čitanje i pisanje.
- Najviše 2 agenta koji prevode Rust; jedan `CARGO_TARGET_DIR` = glavno stablo `target/`; disk > 100 GB.
- Redoslijed faze B je Leonov ([ADR-046](./records/DECISIONS/ADR-046-leonove-odluke-15-10-za-fazu-b.md)):
  B2a → B2b → B3a → B3b → B1a → B1b → B4a–c → PRAVILO → L1-13. Kartica PRAVILO
  ([ADR-047](./records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md)) piše se kad je B3b spojen.

## Otvoreno za Leona

- Anketa B3 (koliko situacije teže) kad B3a izmjeri brojke; slijepi pregled tekstova B4 (dokument, ne anketa).
- 🟨 [ADR-048](./records/DECISIONS/ADR-048-lik-v3-redni-broj-poruke-i-zakljucak.md) (lik.v3) i stariji 🟨 ADR-ovi;
  pretpostavke P1–P8 u planu najkasnije prije B4.
- Vidljivost morala u igri (Leon nije našao vezu plaća → moral) — BACKLOG, pitati treba li u fazu C.
- Ostaci mapa `../hotelixsim-s1-ot` (1,2 GB) i `-s1-zv` bez `.git` — brisanje je dozvola odbila; Leon može sam.
- 5 pitanja [VODITELJ_I_TRZISTE](./architecture/VODITELJ_I_TRZISTE.md) §3; banka ljudi K1 (radnici pojedinačno) nakon popravka.

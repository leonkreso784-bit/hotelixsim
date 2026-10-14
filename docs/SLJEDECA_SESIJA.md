# Sljedeća sesija — REVIZIJA TEMELJA (samo čitanje), pa završetak Sloja 1

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Leon (14.10.): *„sve više mi se čini da ćemo morati krenuti ispočetka jer ne gradimo od baze nego od sagrađenog bungalova
radimo ciglenu nadogradnju koja se može samo srušit u jednom trenutku"*. Dogovor: sljedeća sesija najprije **provjerava
temelj** i ništa ne gradi; Leon odlučuje na temelju dokumenta.

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Radimo REVIZIJU TEMELJA (samo čitanje, ništa se ne gradi). Predaja: docs/SLJEDECA_SESIJA.md i ledger
.superpowers/sdd/2026-10-10-v2-sloj1/progress.md („STANJE NA ZAUSTAVLJANJU 14.10. (večer)" + dopuna). Pokreni paralelne
read-only agente po slojevima, pa mi daj dokument s preporukom. Push samo uz moj OK.
```

## Cilj

1. **Revizija temelja** — paralelni agenti koji samo čitaju, svaki svoj sloj, izlaz u datoteke:
   jezgra i model (`engine`, `sim-hotel`, `sim-hotel-v2`, `game`: dnevnik događaja, determinizam, granice v1/v2);
   sadržaj i njegove provjere (`content`, predlošci, što načela igre NISU zapisana kao provjera — vidi [BUGS.md](./records/BUGS.md) B1–B3);
   balans i roboti kao mjerilo (što brane stvarno mjere — B6); AI sloj (granice u ulazu — B4); web i API; način rada s granama.
2. Iz toga **jedan dokument** u `docs/architecture/` (≤300 redaka): što je pravi temelj, što je nadogradnja, gdje puca (s
   `datoteka:redak`), cijena popravka dijela prema početku ispočetka (sati, rizik), **jasna preporuka**. Prije prijedloga
   vlastita provjera nalaza (memorija „analiziraj prije prijedloga").
3. Tek nakon Leonove odluke: L1-13 kalibracija na grani `model-v2/sloj1-l111` (situacije ~20 % napretka,
   [ADR-042](./records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 15), `--ff-only` u sloj1, `check:api`;
   zatim L1-13b (ADR-042 t. 1–14).

## Stanje (lokalno, ništa nije pushano)

- `model-v2/sloj1` @ docs-commit ove sesije (iznad 24b4e2a): obnova + KS-3 spojeni (L1-10), sve provjere zelene.
- `model-v2/sloj1-l111` @ b1bf446: situacije spojene, 879/0, pada samo brana „udio situacija" (B Brižni 10,2 % tjedno).
- Pregled grana za Leona radi: 5182 (situacije, API 8095) i 5183 (obnova, API 8096), zasebne baze, s AI ključem.
- Probne v2 igre stvorene prije L1-10 na demu iz sloj1 daju 409 `game_outdated` (podaci ostaju; v1 radi).

## Otvoreno za Leona

- Odluka nakon revizije: popravak dijela ili ispočetka.
- 🟨 tekstovi iz grane situacija (popis u ledgeru `za-leona-pretpostavke.md`); 5 pitanja [VODITELJ_I_TRZISTE](./architecture/VODITELJ_I_TRZISTE.md) §3.
- Banka ljudi: znak za korak K1 ([BANKA_LJUDI](./architecture/BANKA_LJUDI.md)).
- Push grana na GitHub (repo je javan).

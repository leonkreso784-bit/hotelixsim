# Sljedeća sesija — UI popravci za demo, put B sloj 1; paralelno preuređenje dokumentacije

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Dvije sesije mogu raditi istodobno: **A — kod** (ova predaja) i **B — dokumentacija** (prompt je u planu
`docs/plan/2026-10-10-dokumentacija.md`, odjeljak „Prompt za početak"). Sesija A ne dira `docs/` dok B radi; stavke za
dokumente piše u `.superpowers/sdd/2026-10-10-v2-sloj1/za-dokumentaciju.md`.

## Prompt za početak sesije A (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo kod: (1) UI val 1 za demo na grani gradnja/ui-popravci (../hotelixsim-ui) — pregled UI čuvara u
pregledniku, popravci, spajanje u main prije demoa 14.10.; (2) put B, sloj 1 po planu docs/plan/2026-10-10-v2-sloj1.md
na grani model-v2/sloj1. Ledger je .superpowers/sdd/2026-10-10-v2-sloj1/progress.md — vjeruj ledgeru i git logu.
Ne diraj docs/ (paralelna sesija dokumentacije). Push samo uz moj OK. Testovi i preglednik bez ključa.
```

## Cilj

1. **Demo (~14.10.) čist:** UI val 1 (2 blokera + 12 važnih iz `.superpowers/sdd/2026-10-10-v2-sloj1/ui-audit.md`)
   pregledan od UI čuvara u pregledniku (360–1920 px) i spojen u `main`.
2. **Sloj 1 puta B:** Taskovi 1–6 (temelj: zlatni test v1, `sim-hotel-v2`, snimak s modelom, ključ kruga, jedno mjesto
   grananja) pa faza B po grafu iz plana; web taskovi 18–22 prolaze kroz UI čuvara.

**Prvi korak:** `/newsession`, `git worktree list`, ledger — koje su grane i commitovi Taskova 1–2 i UI vala već gotovi;
`corepack pnpm check:docs`; CI (`gh run list`); baza `bash scripts/baza.sh start`.

## Za znati

- **Radna stabla:** `../hotelixsim-s1-t1`, `../hotelixsim-s1-t2` (Taskovi 1–2), `../hotelixsim-ui` (UI val). Glavno stablo je
  na `model-v2/sloj1`. Poslužitelj za preglednik pokretati BEZ ključa (`unset ANTHROPIC_API_KEY`).
- **Presude preflighta (24 nalaza)** su u ledgeru i unesene u plan (Taskovi 3–22).
- **Kalibracija procjena:** agenti rade ~10× brže od procjena u planu; usko grlo su pregledi i preglednik.

## Otvoreno za Leona

- **S3 nema opisa ponude** — UI val stavlja uvjete u naslov; pravi opis traži tvoj tekst (BACKLOG).
- ADR-ovi 🟨: 025 (brana presude), 026 i 028 (pretpostavke puta B, pravilo o nacionalnosti), 027 (raspored za računalo).
- Kalibracija profila C (udio rada ~27 %, GOP marža ~21 % prema ~35 % i ~44 % u istraživanju) — rješava sloj 1.

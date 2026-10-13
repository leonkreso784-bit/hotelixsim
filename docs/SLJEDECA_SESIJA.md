# Sljedeća sesija — kod: Sloj 1 puta B (S-4, O-5, KS-3) i demo oko 14. 10

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Dokumentacija je preuređena i spojena u `main` i `model-v2/sloj1` (13.10.). Od sada **jedna sesija** piše i kod i
`docs/`, po pravilima iz [docs/README.md](./README.md); nema više paralelne sesije dokumentacije ni zabrane diranja `docs/`.

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo kod, put B sloj 1 na grani model-v2/sloj1. Predaja koda je u ledgeru
.superpowers/sdd/2026-10-10-v2-sloj1/progress.md, odjeljak „KRAJ SESIJE 13.10. (4)" — vjeruj njemu i git logu.
Dokumentaciju pišeš sam po docs/README.md (nijedan .md preko 1200 redaka). Moj demo radi na 8080 i 5173 iz glavnog
stabla — ne gasi ga. Push samo uz moj OK.
```

## Cilj

1. **S-4** (`../hotelixsim-s1-sit`, e9e349b „dio"): probe-mutacija, pregled, kontrolorove provjere; zatim G-1 → G-2 → A-1 → W-1.
2. **O-5** (`../hotelixsim-s1-obnova`, e43d99f „dio"): `mentor-sezona.v2.md`, puni popis provjera, pregled; UI čuvar O-4+O-5.
3. **KS-3** (`../hotelixsim-s1-ks3`, b8c0d83, spremno) → u `sloj1` tek na Leonovo „demo gotov".

**Prvi korak:** `/newsession`, `git worktree list`, ledger (odjeljak gore), `corepack pnpm check:docs`; baza
`bash scripts/baza.sh start` ako je stroj restartan.

## Za znati

- **Pretinac** `.superpowers/sdd/2026-10-10-v2-sloj1/za-dokumentaciju.md` drži samo bilješke nespojenih grana (Obnova,
  Situacije, KS-3). Kad se grana spoji u `sloj1`, njezine bilješke prenesi u `docs/` (ADR datoteke + `corepack pnpm docs:adr`,
  BACKLOG, CHANGELOG, MODEL_V2) i obriši iz pretinca. Nove bilješke od sada idu ravno u `docs/`.
- **Plan Sloja 1** je pregled + mapa taskova (`docs/plan/2026-10-10-v2-sloj1/`); taskove uređuj u njihovim datotekama.
- **MODEL_V2** ima 843 retka; kad bi prešao 1000, razdvaja se po §-ima (raspored u Tasku 11
  [plana dokumentacije](./plan/2026-10-10-dokumentacija.md)).
- Lokalno nepushano: `main` (f38755b, raspored dokumentacije) i `sloj1` — push samo uz Leonov OK.

## Otvoreno za Leona

- **Push** `main` i `sloj1` (repo je javan) — nakon OK-a gledaj CI.
- ADR-ovi 🟨 029–033, 035, 036 (Sloj 1) i 030 (raspored dokumentacije) — popis u [DECISIONS.md](./records/DECISIONS.md).
- Iz predaje koda: m7 (stroga granica situacija po odjelu), prag popusta najamnine lokala (profil B), tekst zaključane obnove.
- [BACKLOG](./records/BACKLOG.md) [alati]: granica veličine koda 600/1000 i popis duga ([ADR-039](records/DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md)) i Sokratis koji mjeri staro pravilo indeksa.

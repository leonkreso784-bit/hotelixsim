# Sljedeća sesija — završetak Sloja 1 (kolosijek IGRA) i početak SPREMNO ZA INTERNET

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Radimo po [ADR-038](./records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md): faze i cigle, čuvar testova prije
graditelja, provjeravatelj, staje se na kraju faze. Raspored: [ROADMAP](./plan/ROADMAP.md) i
[ROADMAP/igra.md](./plan/ROADMAP/igra.md).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo po ROADMAP-u: faza ZAVRŠETAK SLOJA 1 (kolosijek IGRA). Predaja je u ledgeru
.superpowers/sdd/2026-10-10-v2-sloj1/progress.md, odjeljak „KRAJ SESIJE 13.10. (5)" — vjeruj njemu i git logu.
Najprije analiziraj stanje (stabla, CI), pa radi. Moj demo radi na 8080 i 5173 iz glavnog stabla — ne gasi ga.
Push samo uz moj OK.
```

## Cilj

1. **Provjeri CI** za `f5871aa` (`gh run list`): dokaz da je CI pribijen na Ubuntu 24.04 prije 19.10. (cigla K0-1).
2. **G-1** (`../hotelixsim-s1-sit`, kod necommitan): čuvar testova ispravi kriv test u `model_v2.rs`, graditelj commita,
   pregled (najjači model). Zatim osnovica bruto (ADR-040 t. 2), G-2 → A-1 → W-1.
3. **Obnova** (`../hotelixsim-s1-obnova` @ 07f1a2b): popravak retka „Obnova" na Kraju sezone (UI čuvar: SPREMNO NE), pa
   ponovni UI čuvar.
4. Prije spajanja: **dijagnoza tjedan ≈ mjesec** (ADR-040 t. 3) i Leonovo „demo gotov" (KS-3 ulazi s obnovom).
5. Kolosijek PLATFORMA može krenuti s fazom SPREMNO ZA INTERNET (lokalno, 0 €), ali najviše 2 agenta prevode Rust.

## Za znati

- Brane: `check:docs` (područja igra/platforma/kvaliteta, `.md` ≤ 1200) i nova `check:kod` (600/1000, popis duga).
- Faza 0 ostaje otvorena samo zbog K0-1 (CI) i K0-8 (Leonova sjednica ADR-ova 🟨, ~1 h); rez `schema.rs` i mentora
  (K0-4/5/6) ide tek nakon spajanja grana obnove i situacija.
- Pretinac `za-dokumentaciju.md` drži bilješke nespojenih grana (Obnova, Situacije) do njihova spajanja.

## Otvoreno za Leona

- „Demo gotov" (prije spajanja obnove s KS-3 u `sloj1`); push grana kad dođu.
- ADR-ovi 🟨: 029–033, 035, 036, 039 i 030 — popis u [DECISIONS.md](./records/DECISIONS.md); sjednica ~1 h (K0-8).
- Iz igre: prag popusta najamnine lokala (profil B), tekst zaključane obnove; hosting i sesije (BACKLOG).

# Sljedeća sesija — završetak Sloja 1 (kolosijek IGRA): G-2 popravak, A-1, W-1

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Radimo po [ADR-038](./records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md): faze i cigle, čuvar testova prije
graditelja, provjeravatelj, staje se na kraju faze. Raspored: [ROADMAP](./plan/ROADMAP.md) i
[ROADMAP/igra.md](./plan/ROADMAP/igra.md).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo po ROADMAP-u: faza ZAVRŠETAK SLOJA 1 (kolosijek IGRA). Predaja je u ledgeru
.superpowers/sdd/2026-10-10-v2-sloj1/progress.md, odjeljak „STANJE NA ZAUSTAVLJANJU 14.10." — vjeruj njemu i git logu.
Najprije analiziraj stanje (stabla, necommitani rad u s1-sit), pa radi. Prvo: G-2 popravak 1 (svježi graditelj po
kartici sit-g2-p1-brief.md), pa kratki ponovni pregled, pa A-1 → W-1. Moj demo radi na 8080 i 5173 iz glavnog stabla —
ne gasi ga. Push samo uz moj OK.
```

## Cilj

1. **G-2 popravak 1** (`../hotelixsim-s1-sit` @ 252fe8b): u radnom stablu su **necommitani** čuvarovi testovi (gotovi, crveni iz
   pravog razloga) i **nedovršen, neprovjeren** kod zaustavljenog graditelja (7 datoteka; kopija `g2p1-wip.diff` u ledgeru).
   Svježi graditelj po `sit-g2-p1-brief.md` (Korak 2): preuzme ili krene iznova, sve provjere, commit. Zatim kratki pregled.
2. **A-1** (API situacija: odjel, vrsta, važnost, uloge; `openapi.json`) → **W-1** (ekran situacija: 3–5 kartica, „prepusti
   voditelju") → UI čuvar (L1-4 … L1-6).
3. **Prije spajanja (L1-10, L1-11):** Leonovo „demo gotov"; u commitu spajanja situacija dug `check-kod.mjs` za
   `game/tests/mentor.rs` 1670 → 1671. Grana obnove (`../hotelixsim-s1-obnova` @ f76905c) je spremna (UI čuvar DA).
4. Kolosijek PLATFORMA smije krenuti (SPREMNO ZA INTERNET, lokalno, 0 €), ali najviše 2 agenta prevode Rust.

## Za znati

- Roboti su **samo za demo** (Leon 13.10.): nalazi o njima idu u BACKLOG, osim ako kao mjerač obore branu modela.
- Pretinac `za-dokumentaciju.md` drži bilješke nespojenih grana (G-1, SB-1, G-2, RB-1, OKS-1); prenosi se pri spajanju.
- Dva agenta u istom stablu: pregled koji pokreće testove i graditelj koji mijenja kod idu **jedan za drugim**.

## Otvoreno za Leona

- **5 pitanja o voditelju odjela i grafu tržišta** — [VODITELJ_I_TRZISTE.md](./architecture/VODITELJ_I_TRZISTE.md) §3.
- 🟨 iz grane situacija (u pretincu, Leonu na pregled): rezervacija grupa za kolektivnu situaciju; naziv uloge partnera
  (P-INSPEKCIJA, P-VJENCANJE).
- „Demo gotov" (prije spajanja obnove); push grana kad dođu; ADR-ovi 🟨 — sjednica ~1 h (K0-8,
  [DECISIONS.md](./records/DECISIONS.md)).

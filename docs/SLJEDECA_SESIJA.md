# Sljedeća sesija — završetak Sloja 1 (kolosijek IGRA): spajanje grana, živi testovi, zajednički balans

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Radimo po [ADR-038](./records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md): faze i cigle, čuvar testova prije
graditelja, provjeravatelj, staje se na kraju faze. Raspored: [ROADMAP](./plan/ROADMAP.md) i
[ROADMAP/igra.md](./plan/ROADMAP/igra.md).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo po ROADMAP-u: faza ZAVRŠETAK SLOJA 1 (kolosijek IGRA). Predaja je u ledgeru
.superpowers/sdd/2026-10-10-v2-sloj1/progress.md, zadnji odjeljak „STANJE NA ZAUSTAVLJANJU 14.10. (noć)" — vjeruj njemu
i git logu. Grana situacija je gotova (L1-3 … L1-6). Najprije analiziraj stanje, pa radi: <moji odgovori na pitanja iz
predaje>. Moj demo radi na 8080 i 5173 iz glavnog stabla — ne gasi ga. Push samo uz moj OK.
```

## Cilj

1. **Grana situacija je spremna** (`../hotelixsim-s1-sit` @ ea19d54, lokalno): G-2 popravak 1, A-1 (API + naredba „prepusti
   voditelju" po P4), W-1 (ekran situacija) i popravak; svi pregledi Approved, UI čuvar 2 SPREMNO DA.
2. **Spajanje** (L1-10 obnova, pa L1-11 situacije) čeka Leonovo „demo gotov". U commitu spajanja situacija dug
   `scripts/check-kod.mjs`: `game/tests/mentor.rs` 1672 i `game/src/mentor.rs` 1375 (podići), `api/src/schema.rs` 1568 (spustiti).
3. **Živi testovi s ključem** (Leonov OK): G-2 jedan predložak, L1-9 KS-3 sezonski mentor.
4. Zatim L1-13 zajednički balans i L1-14 igriva verzija za Leona.

## Za znati

- Roboti su **samo za demo** (Leon 13.10.): nalazi o njima idu u BACKLOG, osim ako kao mjerač obore branu modela.
- Pretinac `za-dokumentaciju.md` (ledger) drži bilješke nespojenih grana (G-1 … W-1 i otvorene Minore); prenosi se pri spajanju.
- Grane obnove i situacija dijele bazu `hotelixsim_agenti`: UI čuvari dviju grana ne rade istodobno.
- U glavnom stablu je 5 necommitanih web datoteka iz 14.10. 01:17 koje su **duplikati** (dvostruki import i funkcija; web se ne
  prevodi). Kopija je u scratchpadu sesije a229bc3f. Ne dirati bez Leonova odgovora.

## Otvoreno za Leona

- Smije li se vratiti 5 dupliciranih datoteka u glavnom stablu (bez toga demo na 5173 ne radi).
- „Demo gotov" (L1-10); OK za ključ (živi testovi); push grana kad dođu.
- 🟨 tekstovi u pretincu: nazivi uloga (P-INSPEKCIJA, P-VJENCANJE), rečenica mentora „prepuštena je voditelju (2/5)",
  tekstovi W-1 (potvrda, obvezne, brojač); rezervacija grupa za kolektivnu situaciju.
- **5 pitanja o voditelju odjela i grafu tržišta** — [VODITELJ_I_TRZISTE.md](./architecture/VODITELJ_I_TRZISTE.md) §3.
- ADR-ovi 🟨 — sjednica ~1 h (K0-8, [DECISIONS.md](./records/DECISIONS.md)).

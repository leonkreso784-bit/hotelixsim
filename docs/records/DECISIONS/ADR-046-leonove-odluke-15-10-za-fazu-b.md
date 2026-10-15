# ADR-046 — Leonove odluke 15.10. za fazu B (popravak +10, duljina odgovora, sidra, P-INSPEKCIJA, redoslijed)
**Datum:** 2026-10-15 · **Status:** ✅ **Leonova odluka 15.10.** (ankete za kartice B1 i B4; redoslijed: *„moze super svida mi se ideja"*) ·
**Vezano:** [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md), [ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md),
[plan popravka](../../plan/2026-10-14-popravak-temelja.md) (kartice B1–B4)

**Odluka:**
1. **Hitni popravak popravlja prostor odjela (+10) i uz ocjenu 1** (B1a): „popravak je popravak, ocjena mjeri postupak s gostom".
2. **Duljina ponuđenih odgovora najviše 1,5×** među odgovorima iste situacije (B4; pretpostavka P3 potvrđena, više nije 🟨).
3. **Sidra S1–S4 prepisuju se u zajedničkoj datoteci** (v1 i v2 vide nove tekstove; brojke iste) (B4a).
4. **P-INSPEKCIJA je vanjski događaj** (`external_cause`), kao oluja; skida se iz `BreakdownCondition` (B4).
5. **Redoslijed faze B** (Leon odobrio okvir): B2a → B2b → B3a → (anketa s brojkama B3a) → B3b → B1a → B1b → B4a–c → PRAVILO →
   L1-13 → L1-13b → L1-14 → L1-15. B3b prije B1 jer inače B1 gradi bez brane koja mjeri situacije (završni pregled faze A, I1);
   B4 nakon B1b zbog `condition_of`.

**Cijena ako krivo:** popravak uz ocjenu 1 nagrađuje loš postupak s gostom (ocjena i popravak se mogu razdvojiti); drugačiji
redoslijed košta oko dan rada (B1 bez brane težine situacija).

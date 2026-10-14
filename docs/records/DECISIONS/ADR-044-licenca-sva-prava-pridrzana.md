# ADR-044 — Licenca: sva prava pridržana (kod javno vidljiv radi pregleda)
**Datum:** 2026-10-15 · **Status:** ✅ **Leonova odluka 15.10.** (anketa) · **Vezano:** [ADR-001](ADR-001-hotelixsim-je-zaseban-projekt-odvojen-od.md),
[ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md)

**Kontekst.** Repozitorij je javan od 12.10., a nije imao licencu, a README je i dalje pisao „Privatno". Kolega 16.10.
pregledava cijelu ideju igre, pa repo mora jasno reći što je, u kojem je stanju i što drugi smiju s kodom. Ponuđene su
četiri mogućnosti: sva prava pridržana, PolyForm Noncommercial, AGPL-3.0 i MIT.

**Odluka.**
1. **Sva prava pridržana.** Kod je javno vidljiv radi pregleda i suradnje, ali se ne smije koristiti, kopirati,
   mijenjati, distribuirati ni nuditi kao usluga bez Leonove pisane dozvole. Tekst je u `LICENSE` (hrvatski i engleski).
2. Repo ostaje **javan**. README na `main` i u `model-v2/sloj1` opisuje cijelu ideju, stanje i licencu, a upućuje na
   granu `model-v2/sloj1` za najnoviji rad.

**Posljedice.**
- Licenca se kasnije može otvoriti (npr. nekomercijalna ili otvoreni kod); obrnuto ne vrijedi za verzije već objavljene
  pod otvorenom licencom. Zato je zatvorena licenca sigurniji početak.
- Vanjski doprinosi (pull requestovi) traže izričit dogovor o pravima prije prihvaćanja.
- Tekst licence nije pravni savjet; prije prodaje ili ugovora s fakultetom treba ga pregledati pravnik.

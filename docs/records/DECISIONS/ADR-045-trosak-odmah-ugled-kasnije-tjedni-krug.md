# ADR-045 — Trošak odmah, ugled kasnije: brojke tjednog kruga se ne mijenjaju
**Datum:** 2026-10-15 · **Status:** ✅ **Leonova odluka 15.10.** (anketa: „Igra ostaje, objasni") · **Vezano:** [ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md),
[plan popravka](../../plan/2026-10-14-popravak-temelja.md) (A6, B3, C), [BACKLOG](../BACKLOG.md)

**Kontekst.** Čuvar A6 („što student vidi") pao je na tjednoj v2 igri: u prvom tjednu (t01, sjeme 42) odgovor 0 na sve situacije
daje napredak 341, a šutnja 360. Dijagnoza (klasa B, vrijeme, ne kvar): trošak odgovora pada odmah u F (−91; 890 € prema 160 € šutnje,
uz tjedni GOP od ~29 tis. €), a korist u gostima i ugledu (G +66, R +36) stiže tek idućih tjedana. Od t02 odgovoreni hotel vodi
(397 prema 349) do t06. Student vidi da F padne nakon dobrih odgovora, bez objašnjenja.

**Odluka:**
1. **Brojke igre se ne mijenjaju.** Tjedni krug ostaje kakav je: trošak odmah, ugled kasnije.
2. **Scenarij A6 za tjedni krug** uspoređuje napredak na kraju **idućeg** kruga (t02); mjesečni ostaje na istom krugu. `pa > pb`
   ostaje strogo. Slijepa točka (pogoršanje unutar istog tjedna) je svjesna.
3. **Objašnjenje studentu → faza C.** Rečenica „trošak se vidi odmah, učinak na ugled stiže idućih tjedana" na ekranu kruga i u
   mentoru ([BACKLOG](../BACKLOG.md)).
4. **Mjera kroz više krugova → faza B uz B3:** „najbolji odgovor naprema šutnji u napretku kroz N krugova, po vrsti kruga".
   Načelo A2 `silence_dominates` uspoređuje šutnju samo s najlošijim odgovorom polje po polje i ovo ne može uhvatiti.

**Cijena ako krivo:** tjedni student u prvom tjednu osjeti da se „isplati ne raditi ništa" dok mu ne objasnimo; brana A6 ne vidi
pogoršanje unutar istog tjedna. Povratak: promjena obračuna uz dizanje `CORE_VERSION` i kalibraciju.

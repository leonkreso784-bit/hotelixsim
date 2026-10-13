# ADR-005 — Radi bez pitanja: pretpostavke u dokument; push i deploy samo uz Leonov izričit OK
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-006](ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md), [ADR-008](ADR-008-odluke-iz-gradnje-kostura-koje-je.md)

**Kontekst.** Čekanje potvrde svakog koraka usporava Leona; pravilo „nijedan korak ne počinje dok Leon ne potvrdi
prethodni" nadjačano je za tok rada.
**Odluka.** Leon usred ugradnje arhitekture v2: *„automatski radi, nemoj me pitati"*. Claude ide korak po korak bez anketa
i bez „smijem li nastaviti", a odluke koje bi bile upit donosi sam i zapisuje kao **pretpostavku s razlogom**. Ne vrijedi
za push i deploy: za njih treba izričit OK, a odobrenje jedne radnje ne proteže se na sljedeću. Lokalni commit je u redu.
**Posljedice.** Stvari koje zna samo Leon (npr. koje besplatne AI ključeve ima) ostaju otvorene, a gradi se tako da ne
blokiraju (AI iza porta s lažnim pružateljem). Nagomilane pretpostavke vidi [ADR-008](ADR-008-odluke-iz-gradnje-kostura-koje-je.md).

# ADR-065 — Leonove odluke 24.10.: profesor vodi tempo igre, lokal kao mehanika, mentor bez obećanja, generatori
**Datum:** 2026-10-24 · **Status:** ✅ **Leonove odluke 24.10.** (t. 1–6), 🟨 t. 7–10 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-037](ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) (razred, profesor otvara krug), [ADR-063](ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) (pilot),
[ADR-064](ADR-064-plan-je-jedna-datoteka-do-1000-redaka.md), [generator ljudi](../../architecture/BANKA_LJUDI/08-generator-ljudi.md), [BACKLOG](../BACKLOG.md) (L14-2, L14-4, pitanja iz voditeljeve igre)

**Povod.** Sesija 13 je za plan sesije 14 složila podloge (L1-15, pilot 8–9, faza C) i postavila Leonu šest pitanja. Odgovori su niže.

## A — Leonove odluke (tekstom, 24.10.)
1. **Profesor vodi tempo igre.** Profesor napravi igru i bira kako se ide u sljedeći krug (tjedan ili mjesec):
   - **po vremenu:** zadani razmak (npr. 7 dana do sljedećeg predavanja ili 5 minuta na satu), nakon kojeg se krug sam otvara;
   - **ručno:** profesor otvara krug gumbom, kad njemu odgovara.
   Uz to bira **kako student ulazi u novi krug:** (a) student je **automatski** u novom krugu ili (b) profesor otvori krug, a
   **student sam stisne** „dalje". Leon: „veoma bitna stvar je opcija za profesora".
2. **Najam lokala postaje mehanika** (proširuje L14-2): kome se lokal iznajmljuje i za koliko, preuređenje lokala; lokal u vrlo
   dobrom stanju iznajmljuje se skuplje, u lošem jeftinije.
3. **Mentor ne obećava povrat ulaganja.** Ulaganje se ne mora vratiti ako student ne kontrolira druge faktore (npr. loš
   dobavljač, nizak moral radnika). Mentor objašnjava stvarne uzroke, ne „sljedeći mjesec se vraća".
4. **Stručni pojmovi (celijakija i sl.):** jedna rečenica objašnjenja u opisu situacije (mogućnost a).
5. **Tekst privatnosti** (pilot dio 8: voditelj obrade, kontakt, rok čuvanja, je li igra dio ocjene) rješava se s profesorima
   kasnije; dio 8 čeka.
6. **Generator ljudi:** preporuka C + D prihvaćena („super, odlično"; [§8.5](../../architecture/BANKA_LJUDI/08-generator-ljudi.md)).
   **Novo — generator događaja:** nova igra ne smije uvijek početi istom situacijom (Leon: „jednom mi se otvori klima, drugi put
   kuhar") — redoslijed i izbor situacija mijenjaju se od igre do igre.

## B — Pretpostavke voditelja 🟨 (čekaju Leonov pregled)
7. 🟨 Iz t. 1: igra se i **na satu** (krug od nekoliko minuta), pa više AI radnika (faza C, C2 dio „N radnika", ~2–3 h) ide
   **prije pilota**; jedan radnik pri 40 studenata daje neutralne presude 3/5 nakon roka od 120 s.
8. 🟨 Iz t. 1: pilot dosad nije imao profesora (svaka igra za sebe). Profesorov tempo (t. 1) znači da pilot treba bar dio
   kolosijeka PLATFORMA/RAZRED (igra koju profesor napravi i vodi). **Opseg pilota odlučuje se na početku sesije 14**:
   (a) pilot s profesorovim tempom (veći posao prije pilota) ili (b) prvo kratki pilot bez profesora, tempo odmah iza.
9. 🟨 Lokal (t. 2) i generator događaja (t. 6) mijenjaju jezgru ili sadržaj: rade se prije zamrzavanja jezgre za pilot ili
   poslije pilota — ovisi o t. 8.
10. 🟨 Generator ljudi, pitanja §8.6 t. 2–5: po preporuci (recenzent smije biti lik iz generatora; ime iz dijelova uz popis
    zabranjenih javnih osoba; generator u igri nakon pilota, alat D odmah; K8 čeka) dok Leon ne kaže drukčije.

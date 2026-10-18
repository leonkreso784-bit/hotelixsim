# ADR-061 — Leonove odluke 18.–19.10. i pretpostavke sesije 10: kvar slabi kao situacija, pojas 15–25 %, šutnja novinaru gubi ugled
**Datum:** 2026-10-19 · **Status:** ✅ **Leonove odluke 18.10.** (t. 1–5), 🟨 t. 6–12 ČEKA LEONOV PREGLED (četvrta anketa L1-13, sesija 10) ·
**Vezano:** [ADR-045](ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md), [ADR-046](ADR-046-leonove-odluke-15-10-za-fazu-b.md),
[ADR-054](ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md), [ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md),
[ADR-060](ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 7, 9, 10, 18, [kartica L1-13](../../plan/2026-10-14-popravak-temelja/task-14-l1-13-kalibracija.md)

**Izvori.** Graditelj dijela 4 (kalibracija) bio je 18.10. BLOCKED: ni faktor kruga 0 nije spuštao tjednu težinu situacija u hotelu B na 25 %,
jer kvar prostora (bez faktora) nosi hotelu B ~4,1 boda, a B ima ~15 kvarova po sezoni bez obzira na održavanje; uz to je lanac „srednji >
prepušteno" bio negativan. Dijagnoza opusa (`.superpowers/sdd/2026-10-14-popravak-temelja/l113-4-dijagnoza.md`) potvrdila je uzrok i dala
opcije, a Leon je u četvrtoj anketi izabrao sve četiri preporučene (zapis na dnu [kartice L1-13](../../plan/2026-10-14-popravak-temelja/task-14-l1-13-kalibracija.md)).
Brojke su iz `l113-4-report.md`, `l113-4-a2-tablica.md`, `l113-4-prije-poslije.md` i pregleda `l113-4-review.md` (1000 sjemena).
Točke 1–5 su Leonove odluke; 6–12 su pretpostavke voditelja i izvođača ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)), svaka s cijenom.

## A — Leonove odluke 18.10. (četvrta anketa)
1. **Kvar i popravak slabe kao ostali bodovi situacije (K1).** Množe se faktorom kruga `situation_effect_bp`: tjedno ≈ −0,34 / +0,68 boda
   stanja prostora, mjesečno ostaje −1 / +2. **Mijenja A1** (kvar i popravak bez faktora, [ADR-060](ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 10).
2. **„Zadovoljan gost" = ocjena 3, 4 ili 5 (L1).** Uz hitni popravak takav odgovor vraća +2 (`FAULT_FIXED_MIN_QUALITY = 3`), a stari +1 za svaki
   hitni popravak (`repair_urgent_condition_pts`) je obrisan. **Mijenja** odluku 18.10. „ocjena 4 ili 5" (ADR-060 t. 9).
3. **Pojas „situacije ~20 %" = prosjek tri hotela 15–25 %, a svaki hotel 10–35 % (K4)**, tjedno i mjesečno. Stari dug brane („B3 dug", oprost) je obrisan.
4. **P-POHVALA ostaje „samo najbolji nosi nagradu"** — iznimka od A4 (slabiji odgovor s troškom). A4 tako vrijedi za 10 predložaka, ne 11.
5. **Šutnja na P-NOVINAR gubi ugled, ne novac.** Test S-4 sada traži „šutnja nije bolja od lošeg odgovora" (≤ u svakom bodu, strogo niže u ugledu),
   umjesto „iste bodove šutnje i lošeg odgovora".

## B — pretpostavke (🟨 ČEKA LEONOV PREGLED)
6. **Kvar i popravak množe se i faktorom razine, ne samo kruga** (odluka čuvara runde 2: „kao ostali bodovi situacije", koji imaju oboje).
   Posljedica: od 9 kvarova prostora 6 važnih ima mjesečno −1 / +2, a **tri svakodnevna (P-KLIMA, P-DOTRAJALO, P-LOBBY-KAVA) nose 0,6 toga**
   (−0,6 / +1,2 mjesečno). Leon je rekao samo „faktor kruga". Cijena ako je krivo: argument razine → 10 000 u dva poziva `scaled` i dva testa, pa ponovno mjerenje.
7. **RULING A2 (voditelj):** Leonovi pragovi od 16.10. (6 500 / 8 000, T8/T9) i provjera `chance.base ≥ 1` se ne mijenjaju, pa su mjerene inačice
   I0, I1, I2' (osnova 1, ×6) i I3' (osnova 1, ×8); izvorne I2 i I3 su u tablici samo kao redak „traži tvoju odluku". Prolazi samo **I0**
   (današnja čestoća kvarova). Cijena ako je krivo: Leon traži I2/I3 → nova anketa o pragovima i mjerenje.
8. **Mjesečni faktor 350 → 325, a klik ostaje 8 500** (umjesto spuštanja klika na 8 000). Uz 350 klik u B mjesečno je 75,3 % (pada branu); 8 000 bi
   prošao, ali bi klik tjedno pao na 59–66 % (dalje od cilja 70–75 % i od Leonova „8 500–8 800"). Zato su **ključevi kvara 3 077 / 6 154**
   (= 100 × 10 000 / 325), da mjesečno važna ostane točno −1 / +2. Pri svakoj promjeni mjesečnog faktora treba ih preračunati (test
   `repo_fault_keys_are_on`). Cijena ako je krivo: drugi par brojki (350 / 8 000) i ponovno mjerenje.
9. **Konačne brojke dijela 4 (prijedlog, čeka Leonov OK na tablice):** faktor tjedno 110, mjesečno 325; klik 85 %; razine 1,5 / 1,0 / 0,6; `guest_extra_slots`
   uključen; čestoća I0; `CORE_VERSION` 2.6.0. Težina tjedno 20,8 / 22,2 / 19,2 % (prosjek 20,7), mjesečno 20,1 / 29,4 / 15,3 % (21,6); klik na
   situacijama s likom 63–73 %; najmanji korak lanca 0,04. Ovo zatvara ADR-060 t. 18. Cijena ako je krivo: nova runda kalibracije prije spajanja.
10. **Kontrolirani test 3 dijela 3:** razlika „ostalih" situacija mjeri se u istom krugu, hotelu i sjemenu uz različit broj mjesta, a ne na stvarnim
    krugovima niske i visoke popunjenosti (oni miješaju sezonu, razlika 0,25). Pregled je to prihvatio, a stvarni redak J-3 ide u Leonovu tablicu.
    Cijena ako je krivo: test na stvarnim sezonama s pragom koji uključuje sezonu.
11. **Dio 3 rađen je usporedo s pregledom dijela 2** (druge datoteke), a spajanje je išlo redom (dio 2 pa 3, merge). Cijena ako je krivo: nema je,
    oba su spojena i CI je zelen.
12. **Nalazi pregleda dijela 4 (Approved, 6 Minor) idu u [BACKLOG](../BACKLOG.md)**, a ne u ovu ciglu. Iznimka: tablicu prije/poslije voditelj ispravlja prije
    nego je Leon pogleda (redak klika je približan, do 4 boda razlike od ispisa balansa). Cijena ako je krivo: sitni popravci prije spajanja.

## Što se mijenja
- ADR-060 t. 9 („ocjena 4 ili 5") i t. 10 (kvar bez faktora) zamijenjeni su t. 1–2 ove odluke. ADR-046 t. 1 vrijedi kako ga je ADR-060 t. 9 već izmijenio, uz prag 3.
- [ADR-045](ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md) dobiva dopunu 19.10.: od 2.6.0 u prvom tjednu šutnja je u medijanu bolja od najboljeg odgovora.
- Dio 4 je izgrađen (575ef92 na `popravak/l113-4-kalibracija`), ali **nije spojen**: spaja se tek uz Leonov OK na dvije tablice.

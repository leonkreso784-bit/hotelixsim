# BANKA_LJUDI §6 · Pitanja za Leona (preporuka je prva)

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md). **Pitanja 1–11 Leon je odgovorio 14.10.** ([ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 8);
pitanje 10 dopunio je 16.10. (treća vrsta VANJSKI odmah, §1.5). **Pitanja 12–16 Leon je odgovorio 17.10.** (C9, C10: „ok" na
preporuku; [ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)). Odluke C1–C11 iz
[Leonova pregleda](../../plan/2026-10-17-leonov-pregled/03-banka-ljudi.md) su na dnu.

1. **Gosti s imenom i prezimenom ili samo s imenom?** Preporuka: **ime + prezime** — recepcija zna prezime gosta, AI lik
   se predstavlja punim imenom, a recenzija svejedno pokazuje samo ime i zemlju.
2. **Koliko stranih radnika?** Preporuka: **25 % banke** (~750 u M3), zemlje po službenoj statistici dozvola za rad (MUP).
3. **Raspon dobi?** Preporuka: radnici **18–65**, gosti **18–85**; djeca samo u opisu roditelja.
4. **Smije li lik imati „priču" od 2 rečenice?** Preporuka: **da**, do 220 znakova, samo činjenice puta (gdje je radio,
   zašto je tu), nikad osobine — da priča ne izda skrivenu osobinu.
5. **Rod po ulozi:** stvarni omjer (domaćinstvo uglavnom žene, održavanje uglavnom muškarci) ili **najmanje 25 % svakog
   roda u svakoj ulozi**? Preporuka: **25 % najmanje** — igra je nastavni materijal i ne smije učiti „sobarica = žena".
6. **Ciljni brojevi:** **3 000 radnika + 1 500 gostiju** u tri prekretnice (§2.1), ili stati na 2 000 ukupno? Preporuka:
   tri prekretnice; M2 (3 000 ukupno) je dovoljna za pilot.
7. **Razred: isti brojevi, različita lica?** Hoteli istog profila u razredu dobivaju **iste vještine i razine** (pravednost,
   ADR-028 t. 1), ali **različite ljude iz banke** (ADR-037: tuđi radnici su vidljivi i preotimaju se). Preporuka: da.
8. **Brojke izvan banke:** vještine, osobine i motivaciju izvlači **jezgra** iz razine (preporuka, §1.1) ili ih agenti
   upisuju ručno uz priču?
9. **Pregled:** Leon pregleda **svih 100** prve serije, zatim **uzorak od 10** po seriji (preporuka), ili svaku seriju cijelu?
10. **Partneri** (dobavljač, inspektor, agencija) kao treća vrsta zapisa? Preporuka: **da, nakon M1**, ~200 zapisa;
    do tada ostaju u `names.json`.
11. **Hrvatski kod radnika iz susjednih zemalja** (BiH, Srbija, Crna Gora): upisuje se materinji jezik kakav jest, a
    razumijevanje s hrvatskim igra broji kao **razinu 3** (preporuka) ili kao 4?

**Vanjski (16.10.; odgovoreno 17.10., ✅ = preporuka prihvaćena):**

12. ✅ **da, ali rijetko.** **Smije li isti vanjski lik doći u dva hotela iste razredne igre?** Preporuka: **da, ali rijetko** — vanjski se ne
    preotimaju, a jedan sanitarni inspektor u stvarnosti pokriva cijelo mjesto; izbor ga izbjegava dok ima slobodnih, uz
    M3 = 600 (§2.4). Druga mogućnost: nitko dvaput u razredu (M3 ~800, više posla agentu VANJSKI).
13. ✅ **svih 14.** **Koje uloge?** Preporuka: **svih 14** iz §1.5 (policija, vatrogasci, sanitarna inspekcija i sedam uloga koje igra već ima
    + četiri vrste inspekcije, hitna i grad/turistička zajednica). Druga mogućnost: samo 9 koje igra ima uz policiju i
    vatrogasce, a ostale kad zatrebaju situacijama.
14. ✅ **nova uloga u postojećim predlošcima, nakon popravka temelja.** **Policija, vatrogasci i hitna kao likovi u situacijama:** danas ih spominju samo odgovori (P-DIJETE-IZGUBLJENO, P-DIZALO,
    P-GOST-POZLILO). Preporuka: **nova uloga u postojećim predlošcima** u fazi situacija nakon popravka temelja (student s njima
    razgovara), uz uobičajeni `--check`; druga mogućnost: posebni novi predlošci (npr. vatrogasna inspekcija, prijava krađe).
15. ✅ **150 / 400 / 600.** **Ciljni brojevi vanjskih:** **150 / 400 / 600** (M1 / M2 / M3, §2.4)? Preporuka: da; M2 dovoljan za pilot.
16. ✅ **da, iz uloge.** **Uniforma na avataru** policije, vatrogasaca i hitne? Preporuka: **da, iz uloge** (ne iz banke), kao oznaka koju student
    odmah prepozna; ide uz izgled lika ([IZGLED_LIKA](../IZGLED_LIKA.md)), ne mijenja banku.

## Odluke 17.10. iz Leonova pregleda (C1–C11)

| | Leon | što vrijedi |
|---|---|---|
| C1 | *„Pogledao sam ljude, super su"* + podvrste | **pečat serija 000 i 001** (120 ljudi) bez OK po zapisu; izvodi paralelna sesija banke (§5.6). Podvrste radnika (voditelji, studenti, radnici; svi trenirljivi do razine voditelja): §1.2 |
| C2 | ok | rod po ulozi: brojke ostaju privremeno; agent izmjeri Eurostat (LFS) prije sljedeće serije, nove serije prate izmjereno; tehničarka se ne dodaje na silu |
| C3 | *„dodao bih ljude koji su poznati i javni i političare, ali s izmijenjenim imenom… kao BitLife… pazio bih na granicu"* | 🟨 **čeka Leona:** nacrt rizika [§7](./07-poznate-osobe-rizik.md). Do odluke: nijedan takav lik; deset nesigurnih imena iz pregleda agent provjeri pretraživanjem i zamijeni |
| C4 | *„Student sam imenuje svoj hotel"* | ime hotela slobodno; `zabranjeno.json` samo za likove (§3.5) |
| C5 | *„Neće se pogoditi… riješit ćemo"* | popis profesora ne dolazi (§3.5) |
| C6 | ok | hrvatski oblici gradova i hrvatski zapis ćiriličnih imena |
| C7 | ok | „lokalno" = gdje osoba živi; opis kaže odakle je |
| C8 | ok | a–e: Uzbekistan u „Aziji", „ostale" zemlje po jedan, talijanski 4 kod Istrana, ruski 2 kod gosta iz Dresdena, svi gosti sa zanimanjem; zemlje se zamijene kad stignu podaci MUP-a |
| C9 | ok | pitanja 12–13 gore |
| C10 | ok | pitanja 14–16 gore i 13 pretpostavki vanjskih (službe iz Hrvatske, dob 20–80, službe do 65, inspektori i hitna od 25, ton iz predloška) |
| C11 | ok | izgled lika: tri pretpostavke ostaju; Leon pogleda 10–20 lica kad IZGLED bude spojen |

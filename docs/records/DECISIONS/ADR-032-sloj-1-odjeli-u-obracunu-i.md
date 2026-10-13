# ADR-032 — Sloj 1: odjeli u obračunu i stanju
**Datum:** 2026-10-11 · **Status:** 🟨 **ČEKA LEONOV PREGLED** (t. 8: smjer je Leonov iz ankete 11.10. „Po odjelu,
isti zbroj"; t. 9: brojke 4\* čekaju Leonovu potvrdu) · **Vezano:** [ADR-029](ADR-029-sloj-1-pretpostavke-plana.md)
P7–P12, [ADR-031](ADR-031-sloj-1-tjedni-krug-u-jezgri.md), [MODEL_V2.md](../../architecture/MODEL_V2.md) §3.2, §3.8

**Kontekst.** Taskovi 11–14 i 23 te završni popravci (ZP) uveli su odjele po USALI-ju u stanje, plan i obračun v2.
Odjeli su morali ostati ekvivalentni v1 u degeneriranom katalogu, pa su se morala izabrati pravila za ono što spec
(MODEL_V2 §3.2) ne kaže: odakle Smještaj uzima brojke, kamo idu troškovi odluka, kako odjeli daju hotelske brojke.

**Odluka (pretpostavke).**
1. **Plan sezonaca kategorije je plan cijelog hotela** (kao u v1 i u robotima): plan Smještaja = plan kategorije −
   Σ planova ostalih odjela (najmanje 0); `plan::dept_seasonal_plans`. U degeneriranom katalogu to je točno v1 plan.
   Plaću Smještaja od Taska 17 čita `departments.json` (`rooms.wage_cents`, [ADR-033](ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md));
   trošak po prodanoj sobi, trošenje i održavanje Smještaja i dalje su iz kategorije hotela (v1 ekvivalencija).
2. **Korak sezonaca vrijedi za zbroj pomaka po odjelima** (Σ |cilj − trenutno| ≤ `seasonal_step`); najam i puštanje
   broje se po odjelu (premještaj 2 sezonca iz restorana u Smještaj = 2 najma + 2 puštanja, uz moral puštanja).
   Kapacitet tržišta računa samo osoblje Smještaja. (Task 11)
3. **Troškovi odluka u neraspoređenim odjelima:** Prodaja i marketing ← marketing + odgovaranje na recenzije;
   Održavanje ← održavanje svih odjela (i Smještaja); Uprava ← obuka + zapošljavanje (najam i odlasci) + jednokratni
   troškovi situacija; Energija ← ništa. Trajne povišice iz situacija su plaće Smještaja (P12). Fiksne naknade su
   ispod GOP-a. (Task 9, Task 12)
4. **Hotelske brojke iz odjela:** kazna i opterećenje hotela (moral, sažetak) = prosjek odjela s kapacitetom i
   osobljem, ponderiran brojem zaposlenih (bez takvog odjela: opterećenje Smještaja, v1 300). Kvaliteta i cilj
   zadovoljstva hotela = prosjek odjela ponderiran gostima (Smještaj: noćenja `div_round(prodano·gosti_po_sobi, 100)`;
   ostali: usluženi; bez gostiju: Smještaj), pa v1 pravilo pomaka + obnova + situacije. Opterećenje odjela ne ovisi o
   iskustvu i moralu, pa ga faza ljudi uzima iz prvog poziva `dept_step`; račun odjela koristi iskustvo i moral nakon
   faze ljudi (kao Smještaj u v1). (Task 12)
5. **Račun Smještaja:** opterećenje i kazna su njegovi (ne hotelski) — trošenje prostora (> 180) i kvaliteta
   Smještaja; korištenje = popunjenost raspoloživih soba; kapacitet = manji od kapaciteta osoblja i raspoloživih
   soba-noći; potražnja = grupe + individualna; ostali izravni trošak = trošak po prodanoj sobi; stanje prostora nakon
   kruga uključuje obnovu i situacije. Zadovoljstvo gostiju Smještajem se pomiče prema cilju Smještaja istim pravilom
   kao hotel, bez obnove i situacija. (Task 12)
6. **Budžet (P8)** = račun odjela kruga kad hotel od početnog stanja profila u svakom krugu igra
   `PeriodPlan::default_for`, bez situacija (`Model::budget_accounts`); za sezonu se zbrajaju krugovi od prvog do
   traženog (`sum_statements`), i za ostvareno i za budžet. Ne sprema se, računa se pri čitanju. (Task 14)
7. **Zadovoljstvo odjela nije zadovoljstvo hotela:** `DepartmentView.guest_satisfaction_pts` je odjelna brojka za sve
   odjele s gostima (i Smještaj); `None` za najam i neraspoređene. Hotelsko zadovoljstvo (indeks) uključuje obnovu i
   situacije, odjelno ne — ekran ih zove „zadovoljstvo hotelom" i „zadovoljstvo gostiju odjelom". **Izravni trošak po
   noći** ima jedan izvor: pokazatelji v2 čitaju `cpor_cents` kategorije, isto polje kao obračun Smještaja, a
   `Catalog::validate` traži da je `rooms.direct_per_use_cents` isti broj (ZP-4; zamijenilo je pretpostavku Taska 14
   da pokazatelji čitaju odjel). (Task 14, ZP-4)
8. **Odlasci po odjelu, isti zbroj** (Leon, anketa 11.10.): hotelski broj odlazaka računa se kao prije
   ([ADR-031](ADR-031-sloj-1-tjedni-krug-u-jezgri.md) t. 1); trošak zamjene ostaje u Upravi, iskustvo se miješa na
   razini hotela. Broj se raspodijeli po odjelima razmjerno broju ljudi odjela (stalni + sezonci nakon faze osoblja)
   **kroz cijelu sezonu**, ne svaki krug ispočetka (ZP-1; stanje odjela pamti `season_departures`), metodom
   najvećeg ostatka; jednaki ostaci redom koda odjela; odjel bez ljudi dobiva 0. Bez kocke, pa novac, balans i
   ekvivalencija ostaju bajt-isti. Polje `departures` u računu odjela i službe (API `DeptRoundBody.departures`,
   `ServiceRoundBody.departures`); objašnjenje, mentor i sažetak tjedna kažu „otišle 3 osobe (Smještaj 2,
   Restoran 1)" (OT, `DebriefInput.department_departures`). (Task 23, ZP-1)
9. **Norma osoblja po klasi hotela i broju soba** (Leon: *„hotel 5\* s 54 sobe ima više zaposlenih nego 4\* ili 3\* s 54
   sobe"*): u `content/hotel/v2/departments.json` Smještaj, Uprava, Prodaja i marketing i Održavanje imaju
   `staffing_tenths_per_100_rooms` po kategoriji (raspon ljudi na 100 soba u desetinkama). `Catalog::validate` traži
   da početni ljudi profila budu u rasponu (`ljudi · 1000 ∈ [min · sobe, max · sobe]`), inače greška
   `v2.profiles.<X>.departments.<code>.staffing`; norma bez retka za kategoriju profila, s min 0 ili min > max →
   `v2.departments.<code>.staffing`. Hrana i piće, ostali operativni, najam i Energija nemaju normu. Norma se provjerava
   samo na početnom stanju; obračun je ne koristi. **Brojke 4\* [PROC], čekaju Leona:** Smještaj 17–25, Uprava 5–7,
   Prodaja i marketing 1,5–3, Održavanje 3–6 ljudi na 100 soba. A/B/C (120 soba, 4\*) prolaze; 3\* i 5\* još nemaju
   redak. (Task 23)

**Cijena ako krivo:** t. 1–5 i 7 su pravila u `sim-hotel-v2/src/resolve/` i `game/src/model.rs` uz ponovni balans;
t. 6 je jedna funkcija; t. 8 je raspodjela bez utjecaja na novac; t. 9 su brojke u sadržaju.

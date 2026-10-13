# ADR-035 — Sučelje Odjela: odluke u detalju odjela i ocjena odjela
**Datum:** 2026-10-12 · **Status:** 🟨 **ČEKA LEONOV PREGLED** (pretpostavke kontrolora i izvođača grane `model-v2/sloj1-odjeli`),
osim Leonovih odluka ✅ 12. i 13.10. (anketa: t. 1, formula u t. 2, pragovi u t. 3) ·
**Vezano:** [ADR-029](ADR-029-sloj-1-pretpostavke-plana.md) P7, P18, [ADR-032](ADR-032-sloj-1-odjeli-u-obracunu-i.md),
[UI_PRAVILA](../../product/UI_PRAVILA.md) §1, §7, §8

**Kontekst.** Taskovi 19–21 dali su ekrane Odjela i plan po odjelima unutar tjednog plana. Leon je 12.10. tražio da
odjel pokaže koliko je dobar (traka), da odluke budu uz odjel i da svaka odluka kaže koliko košta (obnova od
150.000 € bila je „bez riječi"). Kod: `apps/web/src/screens/departments/` (`score.ts`, `levels.ts`, `insights.ts`).

**Odluka.**
1. ✅ **Odjeli izlaze iz tjednog plana** (Leon, anketa 12.10.): plan kruga drži odluke za cijeli hotel i sažetak
   promjena po odjelima s poveznicom; kontrole odjela (sezonski radnici, cijene, otvoren, održavanje) su u detalju
   odjela. „Spremi odluke odjela" i „Spremi plan" šalju cijeli plan kruga: svježe dohvaćen + samo dirana polja.
2. ✅ **Ocjena odjela** (Leon, anketa) = prosjek onoga što odjel ima od: stanje prostora, zadovoljstvo gostiju odjela,
   usklađenost ljudi s poslom; kvaliteta usluge hotela ne ulazi; zaokruženo na bod. Najam (razred `rentals`: parking,
   lokal) = samo stanje prostora. Zatvoren odjel i službe nemaju ocjenu. Sastavnice su neutralne trake.
3. ✅ **Pragovi boje** (Leon 13.10., UI M-1), po prikazanom cijelom broju: „dobro" ≥ 80 (`bg-up`), „srednje" 60–79
   (token `warn`, jantar), „slabo" < 60 (`bg-down`); uz traku uvijek „72/100 · srednje". Iste granice daju rečenice o
   stanju prostora: „dotrajao" < 60, „u prosječnom stanju" 60–79 (slaba strana, s prijedlogom pojačanog održavanja
   osim za najam), „u dobrom stanju" ≥ 80 (`levels.ts`, jedan izvor; test veže rečenicu i razinu za 0–100).
4. 🟨 **Usklađenost ljudi s poslom** (`score.ts` → `staffFit`) je ista mjera kao rečenice o ljudima: opterećenje /
   180 (prag kazne u jezgri) = „% granice za dobru uslugu". 60–100 % granice → 100 (zona bez kazne); iznad 180 pada
   ravnomjerno do 0 pri 240 (puni kapacitet osoblja, najveća kazna u jezgri 6.000), razmjerno kazni opterećenja;
   ispod 60 % (opterećenje 108) pada ravnomjerno do 0 kod odjela bez gostiju. Primjeri: 181 → 98, 210 → 50,
   240 → 0; 54 → 50. Odjel bez posla (najam, zatvoren, potražnja 0) nema usklađenost.
5. 🟨 **„Što mu fali / U čemu je dobar"** (`insights.ts`, `INSIGHT_LIMITS`): samo jedna mjera iskorištenosti, „N %
   granice za dobru uslugu" — > 100 % „Osoblje je preopterećeno…" uz prijedlog više sezonskih radnika, < 60 %
   „Previše ljudi za ovoliko gostiju…", inače „Broj ljudi odgovara poslu…"; posluženo < potražnja → „Odbijeno N
   gostiju jer nije bilo kapaciteta" (API `DeptRoundBody.demand`); promjena zadovoljstva gostiju je strana tek od 2
   prikazana boda (1 bod je šum tjednog koraka); gubitak/dobit kruga s iznosom. Pravila o ljudima se preskaču kad
   odjel nema korištenja. Zatvoren odjel kaže da ne prima goste, a stalni radnici se i dalje plaćaju (s iznosom).
   „Udio gostiju" se zove „Gosti hotela koji dnevno dođu u odjel" (Smještaj: „Popunjenost soba").
6. 🟨 **Najam — jedno pravilo za prikaz i ocjenu:** prihod je ugovoren i ne ovisi o gostima ni cijenama (jezgra
   `RevenueModel::Fixed`, zadovoljstvo zamrznuto), pa najam nema kartice Gosti, zadovoljstva, razine cijena ni
   usklađenosti; ljudi samo ako ih ima; brojač sezonskih radnika samo kad granice dopuštaju promjenu. Održavanje mu se
   ne predlaže (danas gotovo ne djeluje); lokal s održavanjem 0/0/0 € nema odluka. Leon (anketa): najam treba
   održavanje koje djeluje — to je pravilo igre i ide u obnovu v2, ne u sučelje.
7. 🟨 **Cijena uz svaku odluku** (OD-3, OD-4): `PlanLimitsBody.costs` (obnova s učinkom, odgovaranje na recenzije,
   obuka, održavanje, procjena plaća), `PlanCostsBody.payroll_round_cents`, `DeptLimitsBody.maintenance` i
   `seasonal_worker_cents` — iz kataloga, za krug kako knjiži obračun (v1 mjesečno, v2 `prorate`, obnova puni iznos).
   Web pokazuje „Blagajna sada X €, nakon obnove Y €" i upozorava kad je Y < 0 ili Y < plaće kruga.
8. 🟨 **Nespremljene izmjene** (I1): plan hotela i odluke odjela čuvaju samo dirana polja u `sessionStorage` (ključ
   hotel + krug + obrazac) i prežive odlazak plan ↔ odjel i osvježavanje kartice; „Izmjene još nisu spremljene."
   dok se razlikuju od plana. Odabrano umjesto upozorenja jer student namjerno skače iz plana u odjele. Nacrt prošlog
   kruga se javlja rečenicom i briše; promijenjen krug je rečenica „Otvoren je novi tjedan…" (web kod
   `round_changed`, bez PUT-a). Potvrda „Zatvori tjedan" upozorava na nespremljene izmjene i vodi u plan, ne sprema
   sama (odluke odjela imaju svoja pravila koraka).
9. **Jezik:** „sezonski radnici" / „stalni radnici" umjesto „sezonci" u v2 tekstovima, uz objašnjenje „Stalni rade
   cijelu godinu. Sezonski su zaposleni samo za ljeto (lipanj–rujan) i trošak su samo dok rade."
10. 🟨 **Ekrani Taskova 19–21** (što je od njih ostalo): `price_level_pct` 100 = uobičajena cijena; povijest odjela po
    krugovima bez prihoda; rute `/racun-dobiti` i `/racun-dobiti/:kod`, izbornik v2 s pet stavki (Hotel, Situacije,
    Odjeli, Račun dobiti, Tijek); prekidač „Ovaj krug / Sezona do sada" je stanje ekrana, krug je u adresi; v2 plan
    skriva v1 polja „Sezonski radnici" i zajedničko „Održavanje" (zbroj sezonaca i održavanje Smještaja šalju se
    izvedeni iz odjela); ponovno otvoren odjel kreće s 0 sezonskih; zatvaranje je onemogućeno kad bi potrošilo više
    koraka nego što ostaje; „trenutno" za korak = `seasonal` iz `GET /departments`.

**Cijena ako krivo:** sve je u `apps/web` (pragovi u `levels.ts`, mjere u `score.ts` i `insights.ts`); API polja su
samo dodana.

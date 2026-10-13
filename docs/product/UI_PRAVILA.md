# HotelixSim — pravila sučelja (obvezni popis za svaki web task)

> Svaki task koji dira `apps/web` mora proći **sve** točke ispod prije spajanja; čuvar UI-ja ih provjerava u
> pregledniku. Izvori: tokeni u `apps/web/src/styles.css`, komponente u `apps/web/src/ui/`, Leonov koncept u
> [DIZAJN.md](./DIZAJN.md), raspored za računalo ADR-027. Nalazi pregleda od 2026-10-10:
> `.superpowers/sdd/2026-10-10-v2-sloj1/ui-audit.md` (oznake I1…, N1… ispod upućuju na njega).

## 1 · Boje, slova, oblik

1. Boja, zaobljenje i slovo **samo kroz token** (`bg-accent`, `text-accent-ink`, `text-ink`, `text-muted`, `bg-sand`,
   `bg-mint`, `text-up`, `text-down`, `text-warn`/`bg-warn`, `border-line`, `rounded-card`). `warn` (#a16207, jantar)
   je razina „srednje" (ocjena odjela 60–79): jedan token za tekst i traku, jasno različit od crvene `down`. U `.tsx`
   nema heksa ni `style={{ color }}`; jedina dopuštena inline vrijednost je širina trake (`style={{ width }}`) i boje
   lika u `src/avatar/`. **Iznimka:** boje uvoda logotipa su u `apps/web/index.html` napisane izravno (prijepis
   tokena), jer se uvod crta prije `styles.css`.
2. **Tekst u boji na svijetloj podlozi** mora imati kontrast ≥ 4,5:1 (veliki tekst ≥ 24 px ili ≥ 18,66 px podebljan:
   ≥ 3:1). Za **tekst** u boji naglaska koristi `text-accent-ink` (bijela 5,57 · `bg-accent-soft` 4,91 · pozadina
   5,23); `text-accent` samo za ikone, obrube i veliki tekst (logo, naslov starta). `text-up` (bijela 5,43 ·
   `bg-mint` 4,55) i `text-down` (bijela 5,75 · `bg-sand` 4,75) smiju na te podloge; `warn` ima 4,9 na bijeloj i
   ≥ 3:1 na podlozi trake `bg-line`. Bijeli tekst na `bg-accent` je 4,57. Novi par boja = novi redak u
   `src/contrast.test.ts` (test računa omjer iz tokena u `styles.css`).
3. `Card` (`ui/primitives.tsx`) se **ne boji kroz `className`**: boja ide kroz `tone` (`plain` · `accent` · `mint` ·
   `sand`), obrub naglaska kroz `emphasis` (2 px `border-accent`, za potvrdu i izbor), unutarnji razmak kroz `pad`
   (`md` zadano · `sm` · `list` za popise s razdjelnicima · `lg` za istaknutu brojku). `className` nosi samo raspored
   (razmaci među djecom, flex, `order-*`, vidljivost).
4. Ljestvica slova: naslov ekrana `text-3xl font-extrabold tracking-tight`; naslov kartice `text-lg font-bold`;
   oznaka podatka `text-sm`; objašnjenje `text-sm text-muted` (`text-xs` samo za jednu kratku napomenu, ne odlomak);
   istaknuta brojka `text-xl`–`text-5xl font-extrabold tabular-nums`.
5. Ikone iz `lucide-react`, 18–22 px, uvijek uz tekst i `aria-hidden="true"`; ikona u krugu kao u `Stat`
   (`size-8 rounded-xl bg-accent-soft text-accent`).

## 2 · Brojke (Leonovo pravilo: svaki broj ima ime i jedinicu)

1. Svaki broj na ekranu ima **ime** (oznaka uz njega ili naslov retka) i **jedinicu** (€, %, noći, osobe, bodovi,
   „/100", „/10", „/5"). Gol broj nije dopušten, ni u rasponu („81–202 €", ne „81–202"; I9).
2. Oblikovanje **samo kroz `apps/web/src/format.ts`**: iznos `formatEuros` (cijeli euri, „1.234 €"), iznos po sobi ili
   noći `formatEurosCents`, postotak `formatPercentBp`, indeks i bodovi 0–10 000 `formatIndex` (prikaz 0–100),
   promjena `formatWholeSigned`/`formatSignedPoints`, broj s imenicom `formatCount`/`formatNights`, ocjena
   `formatRating`. Nikad `toFixed`, `toLocaleString` ni broj ugrađen u predložak bez funkcije. Nova jedinica = nova
   funkcija u `format.ts` s testom u `format.test.ts`. Novac i postotak imaju **nelomljivi razmak** prije jedinice
   („2.100 €", „105 %"), da se jedinica ne odvoji u novi redak. Sloj 1 je dodao `formatEurosSigned`, `formatLoad` i
   imenice `guest`, `permanent`, `seasonal` u `formatCount`.
3. Predznak je pravi minus „−" i plus „+"; nula bez predznaka. Isti podatak ima isti broj decimala na svakom ekranu.
4. Množina uvijek kroz `formatCount` („1 osoba, 3 osobe, 5 osoba"), i u rečenicama („Ljudi: 2 stalna, 1 sezonski").
5. Datumi kruga samo kroz `roundDates`/`roundTitle` („Tjedan 5 (29. 6. – 5. 7.)"); ekran ne računa kalendar.

## 3 · Tekst

1. Svaki tekst kroz `t()` (`packages/i18n/src/hr.ts`); tekst iz API-ja (naslovi, rečenice lika, retci objašnjenja) se
   prikazuje kakav jest, ali ekran ne dodaje vlastitu rečenicu koja ga ponavlja (I4).
2. Hrvatski, kratko, bez žargona; kratice (ADR, RevPAR, GOP, USALI, EBITDA) samo uz puno ime ili rečenicu što znače
   na istom ekranu (obrazac: `screens/kpi/rows.ts` — naziv + `.mean`).
3. **Bez roda prema studentu**: ne „Potrošio si", „upravitelj", „odlučio si" — nego „Poruke su potrošene", „Ti vodiš
   hotel", „Tvoja odluka" (B2).
4. **Nikad interni kod na ekranu**: `t05`, `F`, `staff_load`, `model_v1`, `restaurant` → uvijek ljudski naziv.
5. Ime mjeseca usred rečenice malim slovom, na početku kroz `capitalize`. Ime igre uvijek „HotelixSim".

## 4 · Raspored

1. **Mobitel prvi**: sve radi na 360–430 px bez vodoravnog pomicanja stranice (provjera u §8 na 360 px, gdje stolni
   preglednik ostavlja 345 px). Dijete flexa s tekstom ima `min-w-0`; dugo ime se lomi, ne reže (`truncate` samo uz
   `title`).
2. Okvir je `ui/Layout.tsx`: ispod 768 px donja traka, od 768 px bočna traka. Nova stavka izbornika ide u `TABS`
   (ikona + `t()` oznaka + `match`), a stavka koja vrijedi samo za v2 se **ne prikazuje** u v1 igri.
3. **Ekran za čitanje** (jedan stupac) se omata u `Reading` (`App.tsx`, `max-w-2xl mx-auto`) — **cijeli** ekran,
   uključujući natrag-poveznicu i `h1`, inače naslov i sadržaj nisu poravnati (I5).
4. **Dva stupca od 1024 px** (`lg:grid lg:grid-cols-2 lg:items-start lg:gap-6`) za nadzorne ekrane; **popis + detalj**
   kao `lg:grid lg:grid-cols-[22rem_1fr]` s `data-layout="list-detail"`. Ispod 1024 px detalj je zaseban ekran.
5. **Tablica samo od 1024 px** (`hidden lg:block`), ispod toga kartice (`lg:hidden`); obje iz istih redaka (obrazac
   `screens/KpiScreen.tsx` + `screens/kpi/rows.ts`). Brojčani stupci desno, `tabular-nums whitespace-nowrap`.
6. Redoslijed na mobitelu se ne slaže s `order-*` na elementu s `contents` — ne djeluje (I2). Mijenjaj redoslijed u JSX-u
   ili stavi `order-*` na samu karticu.
7. Razmaci: među karticama `space-y-3`/`gap-3`, u kartici `space-y-2`–`space-y-4`, rub ekrana `px-4` (mobitel).

## 5 · Postojeći dijelovi — koristi, ne kopiraj

| Potreba | Dio | Pravilo |
|---|---|---|
| Okvir sadržaja | `Card` — `ui/primitives.tsx` | `<section>` s `rounded-card border-line`; naslov u njoj je `h2` |
| Glavna / sporedna radnja | `Button` (`variant`, `width`) | jedna primarna radnja po ekranu; `min-h-12` |
| Brojač, oznaka stanja | `Badge` (`tone`) | tekst, ne samo boja („Zatvoren", „Ocjena 4/5") |
| Učitavanje, čekanje | `StatusText` | `role="status"`; tekst kaže što se čeka |
| Greška | `ErrorText` | `role="alert"`; hrvatska rečenica iz `problemMessage` (`api/errors.ts`) |
| Pet indeksa s trakama | `IndexBars`, `Delta`, `INDEX_ICON` — `ui/IndexBars.tsx` | traka ima `role="img"` i `aria-label` „ime: vrijednost od 100" |
| Brojka-poveznica | `Stat` — `screens/hotel/links.tsx` | ikona + oznaka + vrijednost + napomena, cijela kartica je `Link` |
| Redak popisa sa strelicom | `Row` — `screens/hotel/links.tsx` | ikona, naziv, vrijednost desno, `ChevronRight`, `min-h-12` |
| Naslov i povratak | obrazac iz `KpiScreen.tsx` | `Link` s `ChevronLeft` i `min-h-11`, pa jedan `h1`, pa kratka rečenica `text-muted` |
| Objašnjenje uzroka | `DriverList` — `screens/month/Explanation.tsx` | retke piše poslužitelj; boja po učinku + znak |
| Lik | `Avatar` — `avatar/Avatar.tsx` | izgled iz `lookFor(ime)`; dekorativan uz ime |

## 6 · Pristupačnost

1. Točno jedan `h1` po ekranu; svaka kartica s naslovom ima `h2`, podnaslov `h3`.
2. Radnja je `<button>`, prijelaz je `Link`; cijela kartica-poveznica je jedan `Link` (bez gumba unutra).
3. Dodirna meta ≥ 44 px visine (`min-h-11`/`min-h-12`), i za poveznice u tekstu (N12).
4. **Vidljiv fokus** na svemu što prima tipkovnicu: `focus-visible:outline-2 focus-visible:outline-offset-2
   focus-visible:outline-accent`; za skriveni radio/checkbox fokus se crta na oznaci (`has-[:focus-visible]:…`, I7).
5. Kad radnja ukloni gumb koji je kliknut (potvrda, slanje), fokus ide na novi sadržaj (naslov potvrde, rezultat) i
   on se pomakne u vidno polje (I6), i nakon uspješnog slanja (izbor odgovora, predaja). Gumb za spremanje dok traje
   zahtjev ima `aria-disabled`, ne `disabled`, da fokus ostane na njemu („Spremi plan", „Spremi odluke odjela").
6. Ponovljeni gumbi imaju razlikovno ime (`aria-describedby` na tekst stavke); „Izaberi" ×4 nije dopušteno (I8).
7. Stanje koje se mijenja samo (AI ocjenjuje, obračun, spremljeno) je u `role="status"`; greška u `role="alert"`.
8. **Boja nikad sama**: rast/pad uvijek sa znakom i strelicom (▲ +3 / ▼ −2) ili riječju („bolje od budžeta").
9. Nepovratna radnja (izbor odgovora, zatvaranje kruga, nova igra) ima potvrdu u dva koraka.

## 7 · Stanja

Svaki upit ima četiri stanja i svako se vidi: **učitavanje** (`StatusText` „Učitavam…"), **greška** (`ErrorText` +
gumb „Pokušaj ponovno", N13), **prazno** (rečenica što će se pojaviti i kada — nikad nule ni prazna kartica),
**podaci**. Izravno otvorena putanja (osvježavanje usred tijeka) sama dohvaća podatke. Krug koji se još obračunava
čeka i ispituje, ne pokazuje nule. v2 ekran u v1 igri pokazuje rečenicu (409 `model_v1`), ne grešku.
**Nespremljene izmjene** obrasca (plan kruga, odluke odjela) čuvaju se kao dirana polja u `sessionStorage` (ključ
hotel + krug + obrazac) i prežive navigaciju; dok se razlikuju od plana, ekran kaže „Izmjene još nisu spremljene.";
spremanje slaže dirana polja preko svježe dohvaćenog plana. Novi krug je rečenica („Otvoren je novi tjedan…") s
fokusom, ne greška ([ADR-035](../records/DECISIONS/ADR-035-sucelje-odjela-odluke-u-detalju-odjela.md) t. 8).

## 8 · Obrasci za nove ekrane (sloj 1)

**Kartica odjela** (`screens/departments/DepartmentCard.tsx`): vanjski element je `<article>`, a jedina poveznica je
ime odjela u `h2` (`Link` na `/odjeli/:code`) razvučeno preko cijele kartice (`after:absolute after:inset-0`) — jedna
meta, jedan `Link`; ikona odjela (lucide); `Badge` „Zatvoren" kad je zatvoren; jedna traka **ocjene odjela**
(„72/100 · srednje", boja po razini uz riječ: ≥ 80 `up`, 60–79 `warn`, < 60 `down`); redak „Rezultat zadnjeg tjedna
+7.896 €" (iznos kroz `format.ts`, znak + boja); „Ljudi: 8 stalnih, 10 sezonskih" kroz `formatCount`. Popis je u
jednom stupcu do 1024 px (uz bočnu traku su dvije kartice u retku bile preuske), od 1024 px popis lijevo i detalj
desno. „Službe bez prihoda" su `Row` retci s troškom, bez traka.

**Račun dobiti** (`screens/pl/`): sekcije redom USALI-ja. **≥ 1024 px:** `<table>` s `<caption>` (može `sr-only`),
stupci „Stavka | Ostvareno | Budžet | Razlika", `th scope="col"`, naziv sekcije kao redak `th scope="rowgroup"
colSpan={4}` (ne `colgroup`, koji se čita kao zaglavlje stupca), zbirni redak podebljan s gornjim obrubom; brojevi desno. **< 1024 px:** `Card` po sekciji, redak = ime
lijevo i ostvareno desno, ispod sitno „budžet 12.300 €" i razlika riječima. Naslov sekcije je `h2` u točno jednoj
inačici (druga koristi `th`/`aria-labelledby`), da se ne čita dvaput. Na 360 px nijedan iznos ne prelama u dva retka.

**Budžet i ostvareno**: razlika je uvijek **znak + iznos + riječ**: „▲ +1.200 € bolje od budžeta" (`text-up`),
„▼ −800 € lošije od budžeta" (`text-down`), „jednako budžetu" (`text-muted`). „Bolje" ovisi o vrsti retka (veći prihod
je bolje, veći trošak je lošije) — vrstu daje poslužitelj ili redak, nikad se ne zaključuje samo iz predznaka.
Odmah ispod naslova jedna rečenica što je budžet.

**Birač kruga (17 tjedana)**: na mobitelu traka „‹ Tjedan 5 (29. 6. – 5. 7.) ›" s dva gumba ≥ 44 px (`aria-label`
„Prethodni tjedan"/„Sljedeći tjedan", onemogućen na rubu) i izbornik svih krugova (`<select>` s `roundTitle`); od 1024
px mreža čipova po mjesecima (lipanj t01–t05…), čip ≥ 44 × 44 px, oznaka „T5". Stanja čipa nisu samo boja: obračunat
(✓), trenutni (`aria-current="true"` + puni `bg-accent`), budući (onemogućen, `text-muted`). Izabrani krug
`aria-pressed`. Čip u zaglavlju ostaje „TJEDAN 5 · 5/17". Bez vodoravnog pomicanja stranice.

## 9 · Provjera prije spajanja (radi task, ponavlja čuvar UI-ja)

1. Testovi: ekran izravno na putanji, prazno stanje, greška kao rečenica, jedan zahtjev po kliku.
2. Preglednik na **360, 390, 768, 1024, 1280** px (+ 1920 za nadzorne ekrane), s podacima iz prave igre bez AI ključa.
3. Na svakoj širini: `document.documentElement.scrollWidth <= document.documentElement.clientWidth`.
4. Tab kroz ekran: redoslijed prati čitanje, fokus se vidi na svakom koraku, ništa se ne gubi na `<body>`.
5. Konzola bez grešaka i upozorenja aplikacije. Snimke: `.playwright-mcp/ui-<task>-<ekran>-<širina>.png`.
6. Prođi §2 i §3 nad svakim brojem i rečenicom na ekranu (ime, jedinica, rod, kodovi).

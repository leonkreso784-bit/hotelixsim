# Tim HotelixSim

> Za Rina i Simona (i svakoga tko nam se pridruži): tko smo, što se sada događa, što iz Simpolisa prenosimo i kako radimo.
> Odluka o timu: [ADR-067](docs/records/DECISIONS/ADR-067-tim-rino-tomic-i-simon-seifert-simpolis-kao-izvor.md). Stanje od 29. 10. 2026.

## Tko smo

Tim **Pojedinci**: na [RI-HACK 4.0](https://github.com/leonkreso784-bit/riihack) (Rijeka, 23.–25. 10. 2026.) napravili smo
**Simpolis** i osvojili **3. mjesto od 8**. Od 29. 10. zajedno gradimo HotelixSim.

| tko | GitHub | u Simpolisu | u HotelixSimu (🟨 prijedlog, dogovaramo) |
|---|---|---|---|
| **Leon Kreso** | [@leonkreso784-bit](https://github.com/leonkreso784-bit) | integracija, baza, procesi | vlasnik projekta i voditelj; jezgra igre (Rust), backend, baza, spajanje u `main`, deploy |
| **Rino Tomić** | [@rtomic20](https://github.com/rtomic20) | frontend i dizajn sučelja | sučelje (`apps/web`, React + TypeScript): ekrani profesora i grupe, studentski ekrani, mobitel |
| **Simon Seifert** | [@simonseifert](https://github.com/simonseifert) | igra, AI, pitch | AI sloj: upute za AI (`prompts/`), mjerilo kvalitete presude, mentor; istraživanje, pitch, inkubator |

## Što je HotelixSim (u dvije minute)

Ista ideja kao Simpolis, ali stariji i dublji projekt (od rujna 2026.): student dobije hotel od 120 soba na Opatijskoj
rivijeri i vodi ga od lipnja do rujna, po tjednima (17 krugova, ≈ semestar) ili mjesecima. **Ocjenjuje se napredak, ne zarada.**
AI glumi likove, sudi slobodan odgovor po rubrici i kao mentor objašnjava ishod; **sve brojke računaju pravila igre**,
deterministički (ista igra s istim početkom uvijek daje isti ishod).

- **Danas radi:** cijela sezona u pregledniku (mobitel i računalo), 55 situacija (40 s likom), odjeli po USALI-ju,
  hotelski pokazatelji (ADR, RevPAR, GOPPAR), ~2 000 automatskih provjera, 1 000 sezona za balans, banka od 930 likova.
- **Stack:** Rust (jezgra `crates/sim-hotel*`, igra `crates/game`, API Axum `crates/api`, baza `crates/store`, AI `crates/ai`) ·
  PostgreSQL 16 · React + TypeScript + Vite (`apps/web`) · Claude (lik na Haikuu, presuda i mentor na Sonnetu).
  Razlike prema Simpolisu (Next.js, Drizzle, Neon) su namjerne: jezgra mora biti deterministička i provjerljiva.
- **Ime:** uvijek **HotelixSim**, nikad „Hotelix" (postoji takav hotelski softver; [ADR-002](docs/records/DECISIONS/ADR-002-ime-hotelixsim-nikad-skraceno-na-hotelix.md)).

Više: [README.md](README.md) (pokretanje), [VIZIJA](docs/product/VIZIJA.md), [ARHITEKTURA](docs/architecture/ARHITEKTURA.md).

## Što se sada događa

| kada | što | stanje |
|---|---|---|
| **29. 10.** | Leon predstavlja HotelixSim **odboru FMTU-a** (15 min + igra uživo): trošak, podaci studenata, uloga profesora, šest odluka koje tražimo | danas |
| sljedeće | **Plan „Pilot s profesorom"** ([plan](docs/plan/2026-10-24-pilot-s-profesorom.md), [ADR-066](docs/records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md)): profesor na stranici napravi **grupu**, dobije **profesorski kod**, studenti uđu **kodom za ulaz**, profesor vodi tempo | 🟨 čeka Leonov OK na pitanja 1–9 |
| val 0 → 4 | testovi prije svega → AI temelj (rok, trake, predmemorija) → grupa igara + mentor → sudac u dva koraka, generator događaja → mjerenja i deploy | ~145 h rada |
| **16. 11.** | prijava u **Startup inkubator Rijeka**; i Simpolisov plan je ciljao isti rok, pa biramo jednu prijavu (pitanje 3) | u pripremi |
| studeni–prosinac | probni pilot s jednom grupom | plan |
| siječanj 2027. | verzija za profesore | plan |
| ožujak 2027. | pilot sa studentima, ljetni semestar | plan |

**Važno dok traje pilot:** jezgra igre (`crates/sim-hotel*`, `CORE_VERSION` 2.6.0) je **zamrznuta**. Sve novo za pilot
živi u `app`/`api`/`store`/`web`. Puni raspored: [ROADMAP](docs/plan/ROADMAP.md).

## Što iz Simpolisa prenosimo

Ne prenosimo kod jedan na jedan (drugi jezik i drugi model igre), nego **ideje, tokove i dio uputa za AI**. Kod Simpolisa
pripada svoj trojici zajedno ([licenca Simpolisa](https://github.com/leonkreso784-bit/riihack/blob/main/LICENSE)), pa
dio koda ili teksta prenosimo samo uz pisanu suglasnost sve trojice, npr. komentar na PR-u (ADR-067 t. 5).

| u Simpolisu | u HotelixSimu danas | prijedlog 🟨 |
|---|---|---|
| profesor otvori igru, razred uđe kodom | grupa igara s profesorskim kodom i kodom za ulaz je u planu (val 2) | **+ QR kod** za ulaz (mali posao, Rino) |
| zajednički grad, hoteli dijele turiste | razred u jednoj igri ([ADR-037](docs/records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)) je nacrt, **poslije pilota** (dira jezgru) | Simpolisov model tržišta kao ulaz u ADR-037 |
| botovi popune tržište | roboti postoje, ali samo za demo i balans | kao suparnici u razredu, poslije pilota |
| profesor šalje situaciju razredu ili jednom studentu; AI predloži situaciju za najslabiji pokazatelj, profesor pregleda | generator događaja (val 3); profesorove situacije F2b u [planu platforme](docs/plan/2026-11-platforma.md) | Simpolisov tok „AI predloži → profesor odobri" kao predložak |
| profesor mijenja ocjenu odgovora | ispravak presude F2-7 (plan platforme) | prednost za pilot? |
| **mentor prvo pita, tek onda objašnjava** | mentor objašnjava na kraju kruga (val 2: bez obećanja povrata) | **novo** — Simon; pravilo koje student osjeti, ide Leonu na anketu |
| podij po napretku | ljestvica napretka postoji | podij na kraju kruga kao ekran |
| izvoz ocjena u CSV, brisanje igre | F2-10 CSV, F2-11 uklanjanje studenta (plan platforme) | u pilot (profesori to traže) |
| „kako je izračunat mjesec" (lanac od tržišta do dobiti) | uz svaku promjenu igra napiše zašto; račun dobiti po USALI-ju | Simpolisov lanac kao ekran |
| izometrijski grad, gradske tvrtke, ugovori | — | ideje za poslije ([IDEJE](docs/product/IDEJE.md)) |

Popis je prijedlog. Što ulazi u pilot, a što ide poslije, dogovaramo zajedno; konačno odlučuje Leon (pravila igre i brojke
koje student osjeti idu kroz anketu prije gradnje).

## Kako radimo

**Repo je javan: push je objava.** Nikad tajne, lozinke, ključevi ni osobni podaci studenata u gitu.

1. **Grane:** svatko na svojoj: `rino/<tema>`, `simon/<tema>`, `leon/<tema>`. Bez rada izravno na `main`, bez `git push --force` na zajedničke grane.
2. **Pull request u `main`**, kratak opis što i zašto + kako je provjereno. Spaja Leon (🟨 dok ne dogovorimo drukčije). Deploy samo uz Leonov izričit OK.
3. **Provjere prije PR-a:** [README → Lokalni razvoj](README.md#lokalni-razvoj), korak 5. Za sučelje najmanje
   `corepack pnpm lint`, `corepack pnpm typecheck`, `corepack pnpm test`; za Rust `cargo fmt`, `clippy`, `cargo test --workspace`;
   za dokumentaciju `corepack pnpm check:docs`. CI na GitHubu vrti sve to.
4. **Mala veličina:** datoteka koda najviše 1000 redaka (`check:kod`), dokument najviše 1200, plan jedna datoteka.
5. **Promjena sadržaja igre** (situacije, brojke) uvijek uz balans: `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check` (i `--round month`).
6. **AI ključ** samo u `.env` (u `.gitignore`), nikad u pregledniku ni u `.env.example`. Bez ključa igra radi na rezervnim tekstovima.
7. **Odluke** se pišu kao ADR (`docs/records/DECISIONS/`, jedna odluka = jedna datoteka), zatim `corepack pnpm docs:adr`.
   Gdje što ide u dokumentaciji: [docs/README.md](docs/README.md).
8. **pnpm samo kao `corepack pnpm`.** Windows: Git Bash za skripte.

**Claude Code:** `CLAUDE.md` se učitava sam. Njegov odjeljak „Tko je Leon" upućuje na Leonove lokalne bilješke kojih kod
vas nema; to preskočite, a pravila rada vrijede za sve.

## Prvi koraci (za Rina i Simona)

- [ ] Prihvati poziv u repo (stiže na GitHub i e-mail) i `git clone https://github.com/leonkreso784-bit/hotelixsim.git`
- [ ] Pročitaj: ovaj dokument, [README.md](README.md), [VIZIJA](docs/product/VIZIJA.md), [plan pilota](docs/plan/2026-10-24-pilot-s-profesorom.md) (tablica na vrhu)
- [ ] Pokreni lokalno po [README → Lokalni razvoj](README.md#lokalni-razvoj) (Rust 1.98, Node 24, PostgreSQL 16) i odigraj jedan tjedan
- [ ] Javi Leonu što bi uzeo prvo; prvi zadatak neka bude mali PR (npr. QR kod za ulaz, jedan ekran, jedna uputa za AI)

## Otvorena pitanja za tim

1. **Uloge** iz tablice gore — odgovaraju li vam? Simon i Rust: AI sloj je u Rustu (`crates/ai`); upute (`prompts/`) i mjerilo su tekst.
2. **Vlasništvo i licenca:** [LICENSE](LICENSE) danas glasi samo na Leona. Kako ćemo za HotelixSim (suautorstvo, udjeli, j.d.o.o. za inkubator)? Odlučuje Leon, prije prijave u inkubator.
3. **Simpolis dalje:** gasimo ga ili ostaje zaseban portfelj? Što od Simpolisova istraživanja (konkurencija, GDPR i AI Act) seli ovamo u `docs/research/`?
4. **Zaštita grane `main`** (PR obavezan, CI mora proći) — prijedlog uključiti sada kad nas je troje.
5. **Komunikacija:** gdje dogovaramo dnevno (Discord kao na hackathonu?) i kad je zajednički sastanak.

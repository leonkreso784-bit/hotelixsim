# ARHITEKTURA §13–§17 — sučelje, isporuka, testiranje, sigurnost, proširivost

Dio [ARHITEKTURA.md](../ARHITEKTURA.md)

## 13 · Sučelje (TypeScript)

- **Tanko:** prikazuje stanje i šalje naredbe. Ništa ne računa i ne odlučuje.
- **Dva dijela jedne aplikacije:** `/igra` (student, mobitel prvi) i `/profesor` (laptop: tablice, postavke, uvid).
- **Podaci:** samo kroz generirani klijent i TanStack Query; nema ručno pisanih tipova API-ja.
- **Tekstovi:** samo kroz prijevode (`packages/i18n`), nikad zapisani u komponentu.
- **Jezgra u pregledniku (kasnije, po želji):** `crates/wasm` za pregled „što bi se otprilike dogodilo" s istim
  pravilima. Mjerodavan je uvijek poslužitelj.
- **Mobilna aplikacija (kasnije):** `apps/mobile` (Expo) uvozi `api-client` i `i18n`; prepisuju se samo ekrani.

---

## 14 · Infrastruktura i isporuka

**Redoslijed (Leon, 04.10.): sve se gradi i pokazuje lokalno, a Railway dolazi na kraju.** Ništa u kodu ne smije
ovisiti o Railwayu. Postavke dolaze iz varijabli okoline, pa je isporuka zaseban, završni korak.

- **Lokalno:** jedan proces (`api` + `worker` zajedno) + **nativni PostgreSQL na Windowsu** (Dockera nema). Ista baza
  služi SQLx-u za provjeru upita pri prevođenju (`cargo sqlx prepare`). Sučelje se vrti preko Vite razvojnog
  poslužitelja.
- **Na kraju — Railway:** pilot ide kao **jedan servis** (API i pozadinski poslovi u istom procesu) + službeni
  PostgreSQL (bez dodatnih ekstenzija), samo produkcija. Staging ostaje lokalno. Kad opterećenje to zatraži, isti
  program se pokreće kao dva servisa (`api`, `worker`) bez promjene koda. Regija u EU. Stvarnu potrošnju mjerimo u
  prvom mjesecu.
- **Slika (za Railway):** vlastiti Dockerfile u više koraka (cargo-chef ili cache mountovi, pinirana verzija Rusta,
  `SQLX_OFFLINE=true`), jer Railwayev zadani builder ne dokumentira rad s Cargo workspaceom.
- **Sučelje:** statične datoteke — Railway ili CDN (Cloudflare Pages / Vercel); odlučuje se pri prvoj isporuci.
- **Tajne:** lokalno `.env` (nikad u gitu), na Railwayu varijable okoline.
- **Promatranje:** strukturirani zapisi (tracing), Sentry za greške, `/v1/health`.
- **CI (GitHub Actions) na svakom pushu:** rustfmt · clippy · `cargo test` · cargo-deny · testovi determinizma ·
  provjera usklađenosti OpenAPI ↔ klijent · TypeScript provjera tipova, Biome, Vitest · Playwright na mobilnim profilima.
- **Push i deploy samo uz Leonov izričit OK.**

---

## 15 · Testiranje

| razina | što | alat |
|---|---|---|
| **Mehanike** | svaka mehanika: očekivane promjene, granice | `cargo test` |
| **Invarijante** | isti ulaz → isti izlaz · promjene nikad izvan granica · svi profili kreću s istim zbrojem · ponovni obračun = izvorni | **proptest** |
| **Spremljene sezone** | dnevnici stvarnih/sintetičkih sezona moraju nakon svake promjene dati iste brojke | **insta** (snimke) |
| **Balans** | tisuće sezona s robotskim strategijama (nasumična, „samo profit", uravnotežena) → izvještaj pravednosti A/B/C; strategija „digni cijenu, otpusti pola ljudi" ne smije pobijediti | `balance` alat |
| **Aplikacija i API** | slučajevi upotrebe i prava po ulogama, nad pravom bazom | SQLx testovi s Postgresom |
| **AI** | lažni pružatelj u testovima; skup primjera za kvalitetu (§10.3) | |
| **Sučelje** | komponente; cijeli tok na mobilnim ekranima | Vitest · Playwright |

**Pravilo:** „gotovo" znači da je izlaz testa viđen, ne pretpostavljen.

---

## 16 · Sigurnost i privatnost

- AI ključ samo na poslužitelju; nikad u pregledniku ni u gitu.
- Prava se provjeravaju u jednom sloju (`app`), s testom za svaku ulogu.
- Dnevnik je zaštićen pravima baze (bez izmjene i brisanja).
- Ograničenje broja zahtjeva po korisniku; ograničena duljina slobodnog teksta.
- Tvrdi limit AI troška.
- Revizija ovisnosti u CI-ju (cargo-deny, npm audit).
- **Privatnost (GDPR):** minimum osobnih podataka (ime, e-pošta); hosting u EU; koliko se dugo čuvaju podaci nakon
  kolegija — pitanje za profesora/fakultet.

---

## 17 · Proširivost — kako ovo postaje „bilo što"

| budući scenarij | što se radi | što ostaje netaknuto |
|---|---|---|
| **Mobilna aplikacija** | `apps/mobile` (Expo) nad istim API-jem | sve u Rustu |
| **Platforma s više simulatora** | novi `sim-*` paket + `content/*`; platforma koristi naš API ili ugrađuje sučelje | `engine`, dnevnik, runde, prijava, AI sloj |
| **Simulator raste** | nove mehanike (kod + test), novi podaci; zajedničko tržište u fazi 4 obračuna | završene sezone: rezultati i objašnjenja spremljeni trajno; ponovni obračun moguć **svojom** verzijom jezgre (vidi ispod) |

**Iskreno obećanje o starim sezonama:** sezona je ponovljiva s verzijom s kojom je igrana. Ne obećavam iste brojke
stare sezone na novoj jezgri. Zato se svako izdanje backenda označava u gitu, slika kontejnera se čuva, a puni snimak
pravila je u dnevniku (§7.2).
| **Timovi** | upravitelj dobiva više korisnika + pravila tko odlučuje | jezgra (ona ne zna tko je kliknuo) |
| **Vizualni 3D hotel** | Unity / Unreal / Bevy kao novo sučelje nad API-jem (ili jezgra kao knjižnica) | jezgra |
| **Drugi AI pružatelj** | postavka | sve |
| **Druga baza ili hosting** | konfiguracija i `store` | jezgra, `app`, API |

---

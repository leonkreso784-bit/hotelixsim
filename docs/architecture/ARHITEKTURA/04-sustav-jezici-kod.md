# ARHITEKTURA §4–§6 — pregled sustava, jezici, struktura koda

Dio [ARHITEKTURA.md](../ARHITEKTURA.md)

## 4 · Pregled sustava

### 4.1 Kontekst — tko i što okružuje sustav

```text
   Student (mobitel)            Profesor (laptop)
          │                            │
          └──────────┬─────────────────┘
                     ▼
              ┌─────────────┐        ┌──────────────────────┐
              │  HotelixSim │───────►│ AI pružatelj          │
              │             │        │ (Anthropic/OpenAI/    │
              │             │        │  Google — zamjenjivo) │
              └─────────────┘        └──────────────────────┘
                     │
                     ▼
              Pružatelj prijave (OIDC; kasnije AAI@EduHr)
```

### 4.2 Dijelovi koji se pokreću

| dio | jezik | posao |
|---|---|---|
| **web** | TypeScript | Statična aplikacija u pregledniku. Student i profesor. Ništa ne računa, ništa ne odlučuje. |
| **api** | Rust | Prima zahtjeve, provjerava prijavu i prava, poziva slučajeve upotrebe, šalje uživo obavijesti. |
| **worker** | Rust | Isti program u drugom načinu rada: runde po kalendaru, obračun, AI poslovi s ponovnim pokušajima. |
| **PostgreSQL** | — | Dnevnik, projekcije, platformski podaci, red poslova. |

`api` i `worker` su **jedna izvršna datoteka** (`hotelixsim serve` / `hotelixsim work` / `hotelixsim migrate`).
U pilotu mogu raditi i kao jedan proces; razdvajaju se kad zatreba, bez promjene koda.

**Namjerno nema:** Redis, Kafka, mikroservisa, posebnog servisa za poslove. Postgres nosi i red poslova.

---

## 5 · Jezici i tehnologije

### 5.1 Podjela: Rust za istinu, TypeScript za ekran

| | Rust | TypeScript |
|---|---|---|
| **Što** | jezgra simulacije, backend, rad s bazom, AI sloj, alat za balans | web sučelje; kasnije mobilna aplikacija |
| **Zašto** | prevoditelj kao najstroža ograda za kod koji piše AI; brzina za masovne simulacije (alat za balans vrti tisuće sezona); ista jezgra radi na poslužitelju, u alatu za balans, u pregledniku (WASM) i na mobitelu | ekosustav za sučelja nema premca; isti jezik za web i za React Native |
| **Veza** | opisuje API (OpenAPI) | iz tog opisa automatski dobiva tipove i klijenta |

**Što Rust ne daje:** determinizam. On dolazi iz **dizajna** (§7.2) i jednako bi se morao graditi u bilo kojem
jeziku. Rust tu pomaže samo alatima koji pravila provjeravaju umjesto čovjeka: provjera prekoračenja broja i zabrana
nesigurnih tipova kroz clippy. Sam jezik ne jamči ništa. Brojevi i verzije alata provjereni su 04.10.:
[research/RUST_ALATI_2026-10.md](../../research/RUST_ALATI_2026-10.md).

### 5.2 Rust strana

| područje | izbor | zašto / napomena |
|---|---|---|
| jezik | Rust, izdanje 2024, stabilni kanal | `#![forbid(unsafe_code)]` u svim našim paketima |
| asinkronost | **Tokio** | standard za Rust poslužitelje |
| web okvir | **Axum 0.8** (+ `tower-http`) | malen, sastavljiv, iz Tokio obitelji |
| OpenAPI | **utoipa 6** + `utoipa-axum` | opis API-ja iz Rust tipova i rukovatelja; izlaz mora biti OpenAPI **3.1** (TS generator ne čita 3.2) |
| JSON Schema | **schemars** | sheme za sadržaj i za strukturirani AI izlaz |
| serijalizacija | **serde** / `serde_json` | |
| baza | **SQLx 0.9** + PostgreSQL | SQL koji se provjerava pri prevođenju (`.sqlx/` u gitu, build bez baze s `SQLX_OFFLINE`); migracije u repozitoriju. Sve što dijeli transakciju s nama mora biti na **istoj** verziji SQLx-a (§9.4) |
| HTTP klijent | **reqwest** | pozivi AI pružatelja |
| greške | **thiserror** (knjižnice) · **anyhow** (rubovi programa) | |
| zapisi | **tracing** | strukturirani zapisi, povezani po zahtjevu i po poslu |
| konfiguracija | varijable okoline → tipizirana struktura | jedno mjesto, provjereno pri pokretanju |
| WASM | **wasm-bindgen** | po potrebi: jezgra u pregledniku |
| testovi | `cargo test` · **proptest** · **insta** | svojstveni testovi i snimke |
| kvaliteta | `rustfmt` · **clippy** (`-D warnings`) · **cargo-deny** | licence, ranjivosti, zabranjene ovisnosti |

### 5.3 TypeScript strana

| područje | izbor | zašto / napomena |
|---|---|---|
| jezik | TypeScript **strict** | |
| okvir | **React** + **Vite** | brz razvoj; isti model kao React Native |
| usmjeravanje | **TanStack Router** | tipizirane rute |
| podaci s poslužitelja | **TanStack Query** | predmemorija, ponovni pokušaji, osvježavanje |
| API klijent | **openapi-typescript** + **openapi-fetch** | generirano iz `openapi.json`; ručno se ne piše |
| stil | **Tailwind CSS** | |
| prijevodi | **i18next** (ili Lingui) | HR sad, EN kasnije (K9) |
| instalacija na mobitel | PWA (`vite-plugin-pwa`) | |
| testovi | **Vitest** · **Playwright** (mobilni profili) | |
| alati | **pnpm** · **Biome** | |

### 5.4 Ostalo

| područje | izbor |
|---|---|
| baza | PostgreSQL (aktualna glavna verzija) |
| hosting | razvoj i demo **lokalno**; na kraju **Railway** (jedan servis + Postgres), §14 |
| CI | GitHub Actions |
| greške u produkciji | Sentry (Rust i TypeScript) |
| prijava | **otvoreno** (§12) |

---

## 6 · Struktura koda

### 6.1 Repozitorij

```text
hotelixsim/
├─ Cargo.toml                  (Rust workspace)
├─ crates/
│  ├─ engine/                  generičko: kocka, dnevnik, životni ciklus runde, sučelje simulacije
│  ├─ sim-hotel/               hotelska domena: stanje, indeksi, mehanike, okidači, raspored, ocjena
│  ├─ content/                 učitavanje i provjera kataloga; otisak verzije pravila
│  ├─ app/                     slučajevi upotrebe + „portovi" (sučelja prema bazi i AI-ju)
│  ├─ store/                   PostgreSQL: dnevnik, projekcije, red poslova, platformske tablice
│  ├─ ai/                      AI pružatelji, upute (promptovi), provjera izlaza
│  ├─ api/                     Axum rukovatelji, prijava, OpenAPI
│  ├─ server/                  izvršna datoteka: sastavlja sve; načini serve / work / migrate
│  ├─ balance/                 alat: masovne simulacije i ponovni obračun s novim pravilima
│  └─ wasm/                    (kasnije) jezgra za preglednik
├─ content/
│  └─ hotel/                   katalog, predlošci, profili A/B/C, koeficijenti (JSON; JSON Schema je u drugom valu)
├─ prompts/                    upute za AI, verzionirane
├─ apps/
│  └─ web/                     React sučelje (student + profesor)
├─ packages/
│  ├─ api-client/              GENERIRANO iz openapi.json
│  └─ i18n/                    tekstovi HR/EN
├─ openapi.json                GENERIRANO, u repozitoriju (da se vidi svaka promjena API-ja)
└─ docs/
```

Postoje: `engine`, `sim-hotel`, `content`, `game`, `app`, `store`, `api`, `server`, `balance` i `content/hotel/`; ostalo je plan.

### 6.2 Smjer ovisnosti — pravilo koje se ne krši

```text
engine ◄── sim-hotel ◄── content
   ▲           ▲            ▲
   └───────── game ◄────────┘      (app ──► game ──► engine, sim-hotel, content)
                ▲
               app ◄── store
               ▲  ◄── ai
               │
              api ◄── server ──► (store, ai, api)
balance ──► sim-hotel, content (bez baze, bez mreže)
```

- **`engine` i `sim-hotel` ne smiju ovisiti** o Tokiju, SQLx-u, reqwestu, `std::time` ni o bilo kakvom izvoru
  slučajnosti. Provjerava CI (cargo-deny + provjera ovisnosti), ne dogovor.
- **`app`** definira portove (`EventStore`, `AiPort`, `Clock`, `JobQueue`) kao Rust `trait`ove; `store` i `ai` ih
  implementiraju. U testovima se umjesto njih stavljaju lažne implementacije.
- **`server`** je jedino mjesto gdje se sve spaja (kompozicijski korijen).
- Ovo je **heksagonalna arhitektura**: domena u sredini, sve vanjsko na rubu iza sučelja.

---

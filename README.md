# HotelixSim

**Simulacija vođenja hotela za studente hotelijerstva.** Student dobije hotel i vodi ga kroz sezonu: rješava situacije
s gostima i zaposlenicima, razgovara s likovima koje glumi AI i donosi poslovne odluke. Ocjenjuje se **koliko je hotel
unaprijedio u odnosu na početak, a ne koliko je zaradio.**

Projekt nastaje za nastavu na studiju Hospitality Managementa (FMTU Opatija). Autor: **Leon Kreso**.

## Ideja

- **Hotel se dobije, ne bira.** Svi hoteli kreću s istom ukupnom početnom ocjenom, ali s drukčije raspoređenim snagama:
  jedan ima odlične recenzije i slabe financije, drugi dobre financije i lošiju operativu, treći je prosječan u svemu.
  Tako je natjecanje fer.
- **Sezona u krugovima.** Igra prati sezonu od lipnja do rujna, po tjednima ili po mjesecima. U svakom krugu stižu
  situacije: gost se žali na sobu, radnik traži povišicu, pokvari se pumpa na bazenu, javi se agencija s grupom.
- **Odgovor je pravi posao.** Student bira ponuđeni odgovor, piše svoj ili prvo razgovara s gostom ili zaposlenikom kojeg
  glumi AI. Uz situacije vodi hotel po odjelima (smještaj, hrana i piće, wellness…): cijene, sezonski radnici, održavanje,
  obnova, a vidi i račun dobiti po hotelskom standardu USALI.
- **AI ne odlučuje o brojkama.** AI glumi likove, procjenjuje slobodan odgovor po unaprijed zadanim kriterijima i na kraju
  kruga, kao mentor, objasni što je bilo dobro, a što ne. Sve brojke računaju pravila igre, uvijek jednako: ista igra s
  istim početkom daje isti ishod.
- **Ocjena je napredak.** Pet pokazatelja (zadovoljstvo gostiju, recenzije, zaposlenici, operativa, financije) uspoređuje
  se s početnim stanjem vlastitog hotela.
- **Razred u jednoj igri** (u izradi). Studenti jednog kolegija vode svoje hotele u istoj igri i natječu se jedni s
  drugima, a profesor otvara krugove i prati razred.

## Stanje (listopad 2026.)

- **Igriv demo** radi u pregledniku, na mobitelu i na računalu: hotel, situacije, razgovor s likom, AI ocjena, mentor i
  usporedba s tri računalna igrača.
- **Novi model igre** (odjeli, račun dobiti, tjedni ili mjesečni krug, situacije iz predložaka, obnova, kraj sezone)
  je u grani [`model-v2/sloj1`](https://github.com/leonkreso784-bit/hotelixsim/tree/model-v2/sloj1). Ondje je i
  [revizija temelja](https://github.com/leonkreso784-bit/hotelixsim/blob/model-v2/sloj1/docs/architecture/REVIZIJA_TEMELJA.md)
  s [planom popravka](https://github.com/leonkreso784-bit/hotelixsim/blob/model-v2/sloj1/docs/plan/2026-10-14-popravak-temelja.md),
  koji je trenutno u tijeku.
- **Sljedeće:** razred u jednoj igri, likovi s vlastitim osobinama, profesorov pregled; studentski pilot planiran je za
  ljetni semestar 2027.

## Kako je građeno

Jezgra igre je u **Rustu**: čista knjižnica bez baze, ekrana i AI-ja, s cjelobrojnim računom i dnevnikom događaja iz
kojeg se stanje hotela uvijek može ponovno izračunati. Oko nje su API (Axum), baza (PostgreSQL) i sučelje (React,
TypeScript). AI (Claude) je iza jednog sučelja; ključ je samo na poslužitelju.

Više: [vizija](docs/product/VIZIJA.md) · [plan i faze](docs/plan/ROADMAP.md) ·
[arhitektura](docs/architecture/ARHITEKTURA.md) · [model v2](docs/architecture/MODEL_V2.md) ·
[sva dokumentacija](docs/README.md).

## Licenca

© 2026. Leon Kreso. **Sva prava pridržana.** Kod je javno vidljiv radi pregleda i suradnje, ali se ne smije koristiti,
kopirati, mijenjati ni distribuirati bez autorove pisane dozvole. Vidi [LICENSE](LICENSE).

## Lokalni razvoj

Treba: Rust 1.98 (rustup sam povuče verziju iz `rust-toolchain.toml`), Node 24, PostgreSQL 16 (samo programi; baza
radi kao zaseban klaster na priključku 5433 i ne dira sistemski servis na 5432).

1. Jednom: izradi lokalnu bazu. U Git Bashu, iz korijena projekta (ništa ne treba zamjenjivati, `$USERNAME` je tvoje Windows ime):
   ```bash
   PGBIN="/c/Program Files/PostgreSQL/16/bin"
   D="C:/Users/$USERNAME/AppData/Local/hotelixsim"
   mkdir -p "$D"
   echo hotelixsim_lokalno > "$D/pwfile.txt"
   "$PGBIN/initdb.exe" -D "$D/pgdata" -U hotelixsim --pwfile="$D/pwfile.txt" -A scram-sha-256 -E UTF8 --locale=C
   bash scripts/baza.sh start
   PGPASSWORD=hotelixsim_lokalno "$PGBIN/createdb.exe" -h localhost -p 5433 -U hotelixsim hotelixsim
   rm "$D/pwfile.txt"
   ```
   Svaki dan (nakon ponovnog pokretanja računala): samo `bash scripts/baza.sh start`
   (`bash scripts/baza.sh status` pokazuje radi li; `bash scripts/baza.sh stop` je gasi).
2. Jednom: `cp .env.example .env` i `corepack pnpm install`. Claude ključ upiši u `.env` (`ANTHROPIC_API_KEY=…`); bez njega igra radi na rezervama.
3. API (Git Bash, prvi terminal): `set -a; source .env; set +a; cargo run -p server -- migrate && cargo run -p server -- serve`
   `serve` radi u prvom planu i drži terminal zauzetim dok ga ne zaustaviš (Ctrl+C). Pri startu učitava i provjerava
   `content/hotel` (ili mapu iz `HOTELIXSIM_CONTENT`) i pokreće radnika koji obračunava zatvorene mjesece; migracije se
   ne pokreću same — uvijek `migrate` prije `serve`.
4. Sučelje (drugi terminal): `corepack pnpm --filter @hotelixsim/web dev` → <http://localhost:5173>
5. Provjere prije commita: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `node scripts/check-core-deps.mjs`, `cargo test --workspace`, `corepack pnpm check:api`, `corepack pnpm lint`,
   `corepack pnpm typecheck`, `corepack pnpm test`; za dokumentaciju `corepack pnpm check:docs` i `corepack pnpm test:docs`.
6. Dokumentacija ([pravila i granice](docs/README.md)): popis ADR-ova
   nakon novog ADR-a `corepack pnpm docs:adr`; razdvajanje plana `corepack pnpm docs:plan <plan.md>`; veličine svih dokumenata
   uz granice `node scripts/check-docs.mjs --popis`.

### Igra u pregledniku

Baza (`bash scripts/baza.sh start`), API i sučelje kao u koracima 3 i 4 gore; zatim otvori <http://localhost:5173/demo>.

- **Bez AI-ja** (rezerve, bez troška): prije `cargo run -p server -- serve` upiši `unset ANTHROPIC_API_KEY`.
- **Na mobitelu u istoj mreži:** `corepack pnpm --filter @hotelixsim/web dev --host`. Pazi: tada svatko u istoj mreži vidi igru.
- **Modeli:** presuda Sonnet 5.5, lik Haiku 5.5, mentor Sonnet 5.5 ([ADR-024](docs/records/DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md));
  mijenjaju se varijablama `HOTELIXSIM_AI_MODEL_PRESUDA`, `_LIK` i `_MENTOR` (vidi `.env.example`).

Gradnja koristi spremljene podatke iz `.sqlx/` (`SQLX_OFFLINE=true` u `.env`), pa ne treba živu bazu. Nakon promjene SQL upita (baza mora raditi): `cargo sqlx prepare --workspace` i commitaj `.sqlx/`; alat se instalira jednom: `cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres --locked`.
Nakon promjene API-ja: `corepack pnpm gen:api` i commitaj `openapi.json` i `schema.d.ts`.
Mjerilo AI-ja (DEMO §11 t. 7): u `content/hotel/mjerilo/*.json` upiši `leon_quality`, pa `set -a; source .env; set +a; cargo run -p ai --bin mjerilo -- S1 S2`.

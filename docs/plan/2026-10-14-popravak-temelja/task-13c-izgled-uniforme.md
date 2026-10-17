Cigla **IZGLED-c** dodaje **uniformu** na avatar vanjskog lika: policija, vatrogasci, hitna i liječnik. Nastavlja IZGLED a+b
(spojeno 872ff46, kartica [task-13](./task-13-izgled-lika.md)): isti crtić iz podataka, isti put (app → API → web), isti
oprez (stari izgledi bajt-isti). Jezgra, dnevnik, balans, pečat i `CORE_VERSION` se ne diraju; banka ljudi ne mijenja format.

### Task IZGLED-c: Uniforma iz uloge vanjskog lika

**Veličina:** a ~2 h · b ~2,5 h · pregled ~0,5 h (ukupno ~5 h) · **Ovisi o:** IZGLED a+b (spojeno) i **PRAVILO-c spojen prije**
(obje mijenjaju `schema_situations.rs`, `openapi.json`, `schema.d.ts`; tko spaja drugi, ponovno pokreće `corepack pnpm gen:api`,
bez ručnog spajanja generiranih datoteka) · **Paralelno s:** banka ljudi K2 (druge datoteke) · **Model:** čuvar i graditelj
sonnet, pregled opus · **Prevodi Rust:** IZGLED-c-a da (`crates/app`, `crates/api`); IZGLED-c-b ne (samo pnpm).

## 1. Što je odlučeno (izvor)

- [BANKA_LJUDI §6](../../architecture/BANKA_LJUDI/06-pitanja.md) pitanje 16 ✅ (Leon 17.10.): **uniforma na avataru policije,
  vatrogasaca i hitne — iz uloge, ne iz banke**; „oznaka koju student odmah prepozna"; ide uz IZGLED, ne mijenja banku.
- [ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md) **C10**: policija, vatrogasci i
  hitna ulaze kao **nova uloga u postojećim predlošcima** nakon popravka temelja; „uniforma iz uloge". **C11**: tri vidljive
  pretpostavke IZGLED-a ostaju; Leon pogleda 10–20 lica.
- Leon 17.10. (`.superpowers/sdd/banka-ljudi-k1/za-dokumentaciju.md`, „ZA GLAVNU SESIJU"): uniforme **policije, vatrogasaca,
  hitne i liječnika**. Uloge banke: `policajac`, `vatrogasac`, `djelatnik_hitne` (u 60 vanjskih serija 000–001: 4 + 3 + 3).
  Prijedlog sesije banke: neobvezno polje uloge u `hint` avatara (`uniform: "police" | "fire" | "ems"`).
- Leonovo pravilo rada: web + API, čuvar testova prije graditelja (ADR-038), jezgru ne dirati ako nije nužno.

**Što ova cigla NE radi:** ne dodaje uloge u predloške (to je zasebna cigla sadržaja po C10, uz `--check` 1000 tjedno i
mjesečno). Zato **danas u igri nijedan lik nema uniformu**; Leon je vidi na stranici **/likovi** (novi red „Službe") i na
stranici pregleda banke, a u situacijama čim cigla sadržaja doda ulogu.

## 2. Zašto ovim putem (nalaz, retci provjereni na 4a767ee)

- Lik u pogledu je samo `content::Character { name, role }` (`crates/content/src/lib.rs:64-77`); `role` je rečenica za studenta
  („sanitarni inspektor"), ne vrsta uloge. Riječi iz rečenice su krhke (pregled IZGLED-a M4).
- **Ključ uloge** (npr. `inspektor`, `gost`) stoji u predlošku: `Template.character.role` → `roles[key]`
  (`crates/content/src/templates.rs:236-248`, `:297-`). Igra ga već čita na točno ovaj način u `situation_card`
  (`crates/game/src/drawn.rs:472-478`: predložak po `i.drawn.template`, `character_key`). Sve je javno (`Game.content`
  `game.rs:95`, `Game::hotel` `game.rs:411`, `HotelRecord.drawn` `game.rs:71`, `DrawnInstance.drawn.template`), pa app čita
  ključ **bez promjene jezgre**.
- **Ključ uloge u predlošku = naziv uloge u banci** (`policajac`, `vatrogasac`, `djelatnik_hitne`): isti rječnik vrijedi i danas
  (predložak) i u K5 (zapis banke `uloga`), pa se uniforma odlučuje na jednom mjestu (`uniform_for_role`). Bez novog polja u
  formatu predloška (`Role` je `deny_unknown_fields`) i bez novog polja u banci.
- Web već ima prazna mjesta za to: `OUTFITS`, `HATS`, `NECKWEAR` i palete rastu samo dodavanjem na kraj (`look.ts:22-28`,
  `palette.ts:1-2`); test „svaka opcija svake osobine ima svoj sloj" (`avatar.test.tsx:167-191`) automatski traži sloj za svaku
  novu opciju. Postojeća opcija `uniform` (osoblje: ovratnik, gumbi, pločica s imenom, `Avatar.tsx:293-306`) ostaje kakva jest.

## 3. Files

- Create (c-a): `crates/app/tests/character_uniform.rs`.
- Modify (c-a):
  - `crates/app/src/look.rs:25-30` (`CharacterLook.uniform: Option<Uniform>`), novi `enum Uniform` iza `:23`, nove funkcije iza
    `:88` (`uniform_for_role`, `character_uniform`, `character_role_key`, `look_of_character`); ~88 → ~150 r. `character_look`
    (`:59-88`) dobiva samo `uniform: None` u literalu (`:79`), potpis isti.
  - `crates/app/src/views.rs:10` (uvoz), `:341-345` → `look: look_of_character(&g, hotel, &t.code, c)`.
  - `crates/app/src/talk.rs:7` (uvoz), `:46` → `character_look: look_of_character(g, hotel, code, c)`.
  - `crates/app/tests/character_look.rs:73`, `:81`, `:89`, `:97` (literali `CharacterLook {…}` dobivaju `uniform: None`).
  - `crates/api/src/schema_situations.rs:57-66` (`CharacterLookBody.uniform`, `#[schema(required = true)]` kao `age_group`),
    novi `UniformBody` uz enume `:68-88`, `From` `:90-110`.
  - `crates/api/tests/character_look.rs:41-` (ključ `uniform` postoji, danas `null`).
  - `openapi.json` i `packages/api-client/src/schema.d.ts` — **samo** kroz `corepack pnpm gen:api`.
- Create (c-b): `apps/web/src/avatar/uniforms.tsx` (slojevi četiri odore, kape i stetoskopa, ~100 r.; `Avatar.tsx` je 528 r. i
  bez seljenja prešao bi granicu upozorenja 600), `apps/web/src/avatar/uniforms.test.tsx`.
- Modify (c-b):
  - `apps/web/src/avatar/look.ts:22` `HATS` + `"police"`; `:27` `NECKWEAR` + `"stethoscope"`; `:28` `OUTFITS` + `"police"`,
    `"firefighter"`, `"paramedic"`, `"coat"` (sve na kraj); `:221-222` `LookHint.uniform?`; novi tip `Uniform`, tablica
    `UNIFORM_LOOK` i `applyUniform` iza `:253`; `lookFor` `:389-392`. 392 → ~430 r.
  - `apps/web/src/avatar/palette.ts:24-36` `CLOTHES` + dvije boje na kraj (indeks 10, 11), komentar `:24`.
  - `apps/web/src/avatar/Avatar.tsx:271-330` (`Outfit`: nove četiri grane zovu `ServiceOutfit` iz `uniforms.tsx`), `:332-359`
    (`Neckwear`: `stethoscope`), `:448-475` (`HatLayer`: `police`). Raste ~10 r.
  - `apps/web/src/screens/AvatarGallery.tsx` (novi red „Službe": četiri lika s uniformom, muški i ženski), `packages/i18n/src/hr.ts:27-31`
    (`avatars.services`).
  - `apps/web/src/screens/characterLook.test.tsx` (ekran s `uniform: "police"`).
- Ne dirati: `crates/game`, `crates/sim-hotel-v2`, `crates/content` (format predloška i provjere), `crates/balance`, `content/`
  (nijedan JSON; uloge u predlošcima dodaje cigla sadržaja), `prompts/`, banka ljudi (`content/hotel/v2/ljudi`, i u stablu
  `../hotelixsim-ljudi`), `scripts/ljudi-pregled.mjs` i `ljudi-avatari-build.mjs` (mijenja ih grana banke; vidi korak 6),
  `SEED_*`, `HINT_*`, `lookFromSeed`, `lookFromHint` (tijelo), `KNOWN_LOOKS`, `PLAYER_LOOK`, `MENTOR_LOOK`, postojeće boje
  palete (indeksi 0–9), opcija `uniform`, `apps/web/src/test/fixtures.ts` (sve fiksture imaju `look: null`, pa ostaju valjane),
  `docs/` (osim ove kartice i tablice plana), `docs/architecture/PLATFORMA/03-prijava.md`.

## 4. Interfaces

- Consumes: `Game.content.situation_templates` (`.templates`, `Template.code`, `.character.role`), `Game::hotel`,
  `HotelRecord.drawn` (`DrawnInstance.situation.code`, `.drawn.template`), `content::Character`, `app::character_look`.
- Produces (konvencija API-ja kao IZGLED: ključ engleski `snake_case`, vrijednosti enuma `snake_case`):
```rust
// app::look
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Uniform { Police, Fire, Ems, Doctor }
pub struct CharacterLook { pub gender: LookGender, pub age_group: Option<AgeGroup>, pub origin: Origin,
    pub uniform: Option<Uniform> }
/// Rječnik uloga banke (BANKA_LJUDI §1.5); isti ključ u predlošku. Samo točan ključ, bez podniza i velikih slova.
pub const UNIFORM_ROLES: [(&str, Uniform); 4] = [("policajac", Uniform::Police), ("vatrogasac", Uniform::Fire),
    ("djelatnik_hitne", Uniform::Ems), ("lijecnik", Uniform::Doctor)];          // "lijecnik" 🟨 (pitanje 1)
pub fn uniform_for_role(key: &str) -> Option<Uniform>;
/// Uniforma lika predloška `template` iz snimka sadržaja: ključ `character.role` → `uniform_for_role`. None: v1, nepoznat
/// predložak, predložak bez lika, uloga bez uniforme.
pub fn character_uniform(content: &content::Content, template: &str) -> Option<Uniform>;
/// Ključ uloge lika primjerka hotela (kao `situation_card`, drawn.rs:472-478); None za sidra S1–S4 i v1.
pub fn character_role_key<'a>(g: &'a game::Game, hotel: HotelId, code: &str) -> Option<&'a str>;
/// = character_look(..) s uniformom iz character_role_key; ime izvan popisa imena → None (i bez uniforme).
pub fn look_of_character(g: &game::Game, hotel: HotelId, code: &str, c: &content::Character) -> Option<CharacterLook>;
// api
pub struct CharacterLookBody { /* gender, age_group, origin */ #[schema(required = true)] pub uniform: Option<UniformBody> }
#[serde(rename_all = "snake_case")] pub enum UniformBody { Police, Fire, Ems, Doctor } // "police"|"fire"|"ems"|"doctor"
```
```ts
// apps/web/src/avatar/look.ts
export const UNIFORMS = ["police", "fire", "ems", "doctor"] as const;
export type Uniform = (typeof UNIFORMS)[number];
export type LookHint = { gender: "f" | "m"; age_group: Age | null; origin: "regional" | "foreign"; uniform?: Uniform | null };
/** Odjeća službe: mijenja SAMO outfit, clothes, hat, neckwear; ne troši rng (lice ostaje isto). Nepoznata vrijednost → look. */
export function applyUniform(look: AvatarLook, uniform: Uniform | null | undefined): AvatarLook;
export function lookFor(name: string, hint?: LookHint | null): AvatarLook;
// KNOWN_LOOKS[name] pobjeđuje; bez hinta lookFromSeed (bajt-isto); s hintom applyUniform(lookFromHint(name, hint), hint.uniform)
export const UNIFORM_LOOK: Record<Uniform, Pick<AvatarLook, "outfit" | "clothes" | "hat" | "neckwear">> = {
  police: { outfit: "police", clothes: 10, hat: "police", neckwear: "none" },
  fire: { outfit: "firefighter", clothes: 10, hat: "fire", neckwear: "none" },
  ems: { outfit: "paramedic", clothes: 11, hat: "none", neckwear: "none" },
  doctor: { outfit: "coat", clothes: 6, hat: "none", neckwear: "stethoscope" },
};
```

## 5. Kako se uniforma crta (isti stil: ravne boje, tanki obrubi, bez teksta)

Lik je krug 64 × 64; tijelo je `clothes` (boja) + `Outfit` (oblik preko boje). Uniforma ne dira lice, kosu ni nakit.

| služba | boja odjeće | oblik na prsima | kapa / ovratnik |
|---|---|---|---|
| **policija** | nova tamnoplava (indeks 10, prijedlog `#1F2F4D`) | svjetloplavi ovratnik košulje, dvije tamnije naramenice na ramenima, mala zlatna zvjezdica (značka) lijevo na prsima (`GOLD`) | **šapka**: tamnoplava kruna, crni šilt, zlatna točka-grb (`hat-police`) |
| **vatrogasci** | ista tamnoplava (10) | **dvije vodoravne reflektirajuće trake** preko prsa: žuta, tanka srebrna, žuta | **kaciga** (Leon 18.10.): crvena vatrogasna kaciga s tamnim vizirom podignutim i srebrnom trakom; pokriva kosu iznad čela (`hat-fire`) — jedina iznimka od „uniforma ne dira kosu" |
| **hitna** | nova (indeks 11, crvena `#C8102E`) | jedna široka srebrna reflektirajuća traka i **plava zvijezda života** (šest krakova na bijelom krugu) lijevo na prsima | bez kape |
| **liječnik** | bijela (postojeći indeks 6) | **bijela kuta**: sivi obrub dvaju revera u obliku V, kroz V svjetloplava košulja, džep s plavom olovkom | **stetoskop** oko vrata: siva cijev u obliku U i srebrni kružić na prsima (`neckwear-stethoscope`) |

- Nikad **crveni križ** na bijelom: zaštićen znak (Ženevske konvencije, Zakon o Hrvatskom Crvenom križu). Hitna nosi zvijezdu
  života, liječnik stetoskop.
- Slojevi nose `data-part` po obrascu: `outfit-police`, `outfit-firefighter`, `outfit-paramedic`, `outfit-coat`, `hat-police`, `hat-fire`,
  `neckwear-stethoscope`. Naziv u pregledniku (`aria-label`) ostaje ime lika; uniforma nije jedini znak službe — uloga piše uz lik
  (`character.role`), pa i čitač zaslona i daltonist znaju tko je lik.
- Male veličine: avatar je 36–56 px; oznake moraju biti vidljive na 36 px (traka ≥ 2 jedinice, značka ≥ 2,4). Graditelj
  provjeri na /likovi u pregledniku (snimka zaslona u izvještaj).

## 6. Nepoznata uloga — bez uniforme, nikad pad

- Ključ uloge koji nije u `UNIFORM_ROLES` (`gost`, `inspektor`, …), predložak bez lika, sidra S1–S4, v1, stari klijent, ime izvan
  popisa imena → `uniform: None` / `null` → lik izgleda **točno kao danas** (`lookFromHint` ili `lookFromSeed`).
- Web: `applyUniform` s vrijednošću koje nema u `UNIFORMS` (npr. novija verzija API-ja pošalje `"pilot"`) vraća izgled
  nepromijenjen; nema iznimke. `normalizeLook` prihvaća nove opcije jer su u `OUTFITS`/`HATS`/`NECKWEAR`.
- Sudar ključa: ključ se uspoređuje **točno** (`"policajac"`, ne „sadrži polic"), pa „inspektor_sanitarni" ili „Policajac"
  nemaju uniformu.

## 7. Testovi čuvara

IZGLED-c-a — `crates/app/tests/character_uniform.rs` (nova; snimak `Snapshot::read(&repo_dir())`, v2 igra kao `start_v2` u
`crates/app/tests/character_look.rs:141`):
1. `uniform_for_role_knows_four_bank_roles`: `policajac` → `Police`, `vatrogasac` → `Fire`, `djelatnik_hitne` → `Ems`,
   `lijecnik` → `Doctor`; `inspektor`, `inspektor_sanitarni`, `gost`, `""`, `"Policajac"`, `"policajac "` → `None`.
2. `character_uniform_reads_template_role_key`: na **klonu** sadržaja preimenuj ključ lika jednog predloška s partnerom
   (npr. `inspektor` → `policajac` u `roles` i `character.role`) → `Some(Police)`; isti predložak u izvornom sadržaju → `None`;
   nepoznat predložak i v1 sadržaj → `None`.
3. `no_template_has_a_uniform_today`: za svaki predložak izvornog sadržaja `character_uniform == None` (stanje prije cigle
   sadržaja C10; kad ona doda ulogu, mijenja ovaj test svjesno, u svom RED commitu).
4. `character_role_key_matches_situation_card`: sjeme 42, svi krugovi: za svaki primjerak s likom ključ je jednak
   `template.character.role`; sidra S1/S2 i nepoznata šifra → `None`.
5. `views_carry_uniform_field`: `SituationItem.character.look.uniform` i `ConversationView.character_look.uniform` su `None`
   za sve primjerke (danas), a ostala tri polja jednaka kao prije (rod, dob, podrijetlo iz `character_look`).
6. `service_roles_use_bank_keys` (čuvar budućeg sadržaja; zelen već na RED-u): svaka uloga predloška čiji `title` ili `types`
   sadrži „polic", „vatrogas", „hitn* pomoć"/„hitne medicin" ili „liječni" ima ključ iz `UNIFORM_ROLES` i `from: partner`.
   Danas 0 takvih uloga (provjereno: jedini pogodak gole riječi „hitn" je „hitno odgovoriti" u `P-RED-NA-RECEPCIJI` — zato
   fraze, ne korijen). Cijena krive konvencije bez ovog testa: lik policajca bez uniforme, tiho.
7. API (`crates/api/tests/character_look.rs`): `character.look` i `character_look` imaju ključ `uniform`, danas `null`;
   iz `From` (preko HTTP-a nije moguće bez uloge u sadržaju) — `serde_json::to_value(UniformBody::Ems)` = `"ems"` u jediničnom
   testu u `schema_situations.rs` (`#[cfg(test)]`, četiri vrijednosti).
RED: `Uniform`, `uniform`, `look_of_character` ne postoje (E0432/E0560/E0425); literali `CharacterLook {…}` u
`character_look.rs` dobivaju `uniform: None` (čuvar ih nabroji u RED commitu).

IZGLED-c-b — `apps/web/src/avatar/uniforms.test.tsx` (nova), 300 imena `lik-${i}`, oba roda, sve dobi i podrijetla:
1. `bez uniforme isto kao prije`: hint bez polja `uniform`, s `uniform: null` i s `uniform: undefined` → `lookFor` ===
   `lookFromHint`; zamrznuti testovi „Ana Kovač" (`avatar.test.tsx:43`) i „Anna Müller" (`:450`) netaknuti i zeleni.
2. `uniforma iz hinta`: za svaku od četiri vrijednosti `outfit`, `clothes`, `hat`, `neckwear` jednaki `UNIFORM_LOOK`.
3. `uniforma ne mijenja lice`: sva ostala polja (koža, kosa, boja kose, oči, usta, brada, naočale, obrve, nos, dob, naušnice,
   pozadina) jednaka `lookFromHint` istog imena i hinta bez uniforme.
4. `nepoznata uniforma ne ruši`: `applyUniform(look, "pilot" as never)` === `look`; `lookFor` ne baca.
5. `poznati likovi pobjeđuju uniformu`: `lookFor("Petra Horvat", {…, uniform: "police"})` === `KNOWN_LOOKS["Petra Horvat"]`.
6. `palete i bazeni samo rastu`: `CLOTHES[0..9]` jednaki snimku prije cigle, nove boje na 10 i 11; `SEED_*`, `HINT_*`, prvih
   pet `OUTFITS`, četiri `HATS`, četiri `NECKWEAR` nepromijenjeni (redom).
7. `uniforma je uvijek valjana`: `isValidLook` za 300 × 4; `normalizeLook` zadrži `outfit: "police"`, `hat: "police"`,
   `neckwear: "stethoscope"`.
8. `slojevi službe`: policija crta `outfit-police` i `hat-police`; vatrogasac `outfit-firefighter` i `hat-fire`; hitna
   `outfit-paramedic`; liječnik `outfit-coat` i `neckwear-stethoscope`. (Opći test `avatar.test.tsx:167` sam traži sloj za
   svaku novu opciju — pada na RED-u.)
9. Ekran (`characterLook.test.tsx`): fikstura `look: {…, uniform: "police"}` crta `[data-part="hat-police"]`;
   `uniform: null` ne crta; `look: null` kao prije.
10. Galerija (`avatar.test.tsx:457` ili novi `it`): /likovi ima red „Službe" s četiri `outfit-*` službe.
RED: `Uniform`, `UNIFORM_LOOK`, `applyUniform` ne postoje (greška tipova); testovi 2–10 i opći test slojeva padaju.
Čuvar dijela b piše tek kad je c-a spojen u granu (tipovi API-ja postoje) — pouka pregleda IZGLED-a M1.

## 8. Koraci

- **c-a, čuvar:** testovi 1–7, `uniform: None` u postojećim literalima; commit RED s popisom grešaka prevođenja.
- **c-a, graditelj:** (1) `Uniform`, `UNIFORM_ROLES`, `uniform_for_role`, `character_uniform` u `look.rs`; (2)
  `character_role_key` po uzoru na `situation_card` (bez novog javnog API-ja u `game`); (3) `look_of_character`, pozivi u
  `views.rs` i `talk.rs`; (4) `UniformBody`, polje, `From`; (5) `corepack pnpm gen:api`; (6) `typecheck` weba (fiksture su
  `null`, pa bez izmjene); (7) sve provjere.
- **c-b, čuvar:** testovi 1–10 (vitest), commit RED. **Graditelj:** (1) opcije i boje na kraj nizova; (2) `uniforms.tsx`
  (`ServiceOutfit`, šapka, stetoskop) i tri grane u `Avatar.tsx`; (3) `Uniform`, `UNIFORM_LOOK`, `applyUniform`, `lookFor`;
  (4) red „Službe" u `AvatarGallery` (policajac i policajka, vatrogasac, djelatnica hitne, liječnik i liječnica); (5) snimka
  zaslona /likovi na 36 i 56 px u izvještaj; (6) provjere.
- **Pregled (opus):** Review Focus niže; voditelj pokaže Leonu /likovi (privatni prozor, put do novog reda „Službe").
- **Korak 6 — alat pregleda banke (nije u ovoj cigli):** kad su IZGLED-c i grana banke spojeni, `hint(z)` u
  `scripts/ljudi-pregled.mjs` dobiva `uniform` iz `z.uloga` (ista tablica kao `UNIFORM_ROLES`: četiri retka). Voditelj to upiše
  u ledger sesije banke (`.superpowers/sdd/banka-ljudi-k1/progress.md`) kao zadatak nakon spajanja.

## 9. Provjere

Svaki dio, cijeli popis iz `.github/workflows/ci.yml`; izlazi u `.superpowers/sdd/2026-10-14-popravak-temelja/izgled-c-provjere/`
(zasebna imena po naredbi, `k7-cargo-test.txt` i `k7-pnpm-test.txt` — pouka M2):
`cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` ·
`node scripts/probe-mutacija.mjs` · `cargo sqlx migrate run --source crates/store/migrations` + `cargo sqlx prepare --workspace
--check` · `cargo test --workspace` · `corepack pnpm check:docs`, `test:docs`, `check:kod`, `check:api`, `lint`, `typecheck`,
`test` · `corepack pnpm --filter @hotelixsim/web build` · `git diff --exit-code` nakon commita · cargo deny samo u CI-ju.
- **Bajt-isto:** balans v1 (`--seeds 100 --check`, `--seeds 1000`) i v2 (`--model v2 --round week --seeds 1000 --check`,
  `--round month`) `cmp` s osnovicom na vrhu `model-v2/sloj1` prije cigle (balans ne ovisi o `app`/`api`/webu). Pečat:
  `cargo test -p game --test v2_sealed` i `v1_frozen` zeleni bez izmjene `v2-golden.json`; `git diff --stat <osnovica> --
  crates/sim-hotel-v2 crates/game crates/content crates/balance content prompts` prazan.
- `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>` (B12); IZGLED-c-b ne prevodi Rust. Disk ispod 50 GB → ne gradi (B16).

**CORE_VERSION:** ostaje **2.5.0**. Jezgra se ne dira: uniforma je pogled (app čita ključ uloge iz snimka sadržaja igre koji
već postoji), nema novog događaja ni polja dnevnika, obračun i presuda ne vide uniformu. Jezgra bi bila nužna samo kad bi
uniforma ulazila u obračun ili u dnevnik (npr. „lik je doista policajac" kao činjenica igre) — tada nova verzija i pečat; ovdje ne.

## 10. Pretpostavke 🟨 (za ADR na `/kraj`, Leonov pregled)

1. **Ključ uloge u predlošku = naziv uloge u banci** (`policajac`, `vatrogasac`, `djelatnik_hitne`), konvencija bez promjene
   formata; test 6 hvata predložak koji opisuje službu drugim ključem. Druga mogućnost (novo polje `uniform` u `Role`) mijenja
   format sadržaja i `crates/content` — skuplje, a ništa ne dobiva dok K5 ionako nosi `uloga`.
2. **Liječnik** (Leon ga je naveo, banka nema tu ulogu; `djelatnik_hitne` su i liječnica V-00008, i sestra V-00068, i tehničar V-00069):
   - **Preporuka:** (a) **djelatnik hitne — i liječnik hitne — nosi odoru hitne** (na terenu tim hitne nosi istu odoru), i (b) **bijela
     kuta + stetoskop crta se već sada** kao `doctor`, za **novu ulogu vanjskog `lijecnik`** (npr. liječnik turističke
     ambulante ili dežurni liječnik kojeg hotel zove gostu). Cijena: crtež kute ~30 min u ovoj cigli (ništa se ne vidi u igri
     dok uloga ne postoji); uloga u banci = nova vrijednost u popisu uloga (format isti), ~10–20 zapisa u sljedećoj seriji
     sesije banke (~1 h + Leonov uzorak) i redak u BANKA_LJUDI §1.5; uloga u predlošku ide s ciglom sadržaja C10.
   - Druga mogućnost: (c) kuta kao **varijanta hitne** po riječi u opisu („liječnica hitne") — ne preporučujem: opet ključne
     riječi (krhko, M4) i netočno (liječnik hitne na terenu ne nosi kutu). (d) bez liječnika — ako Leon kaže ne, `lijecnik` i
     `doctor` se brišu (jedan redak u Rustu, jedan u webu, kuta ostaje kao opcija odjeće za uređivač).
3. **Boje i oznake** (tablica §5): policija i vatrogasci tamnoplavo (vatrogasci po trakama), hitna crveno sa zvijezdom života,
   liječnik bijelo. Boju odore hitne u Primorsko-goranskoj Leon potvrdi ili ispravi (mijenja se jedna boja, indeks 11).
4. ~~Vatrogasac bez kacige~~ → **odlučeno 18.10.: s kacigom** (§11). Izvorna pretpostavka: bez kacige; s kacigom je prepoznatljiviji, ali skriva kosu — jedan redak u
   `UNIFORM_LOOK` ako Leon želi kacigu (+ ~20 min crteža).
5. **Uniforma iz uloge, ne iz opisa**: kriminalistički policajac (V-00065) u stvarnosti radi u civilu, a ovdje dobiva odoru.
   Ako zatreba lik u civilu, to je druga uloga (drugi ključ), ne iznimka u crtežu.
6. **Uniforma je dio `look`**: lik čije ime nije u popisu imena (`look: null`) nema ni uniformu. Danas službe dolaze iz popisa
   `partner` (jedinstvena imena, test IZGLED-a 1), u K5 iz banke — pa se to ne bi smjelo dogoditi; pregled to gleda.
7. **Uniforma mijenja samo odjeću** (oblik, boja, kapa, ovratnik), ne lice, kosu ni nakit; isti lik u civilu i u odori je
   prepoznatljivo ista osoba (ADR-021/022: lik je podatak).
8. Alat pregleda banke prima uniformu tek nakon spajanja grane banke (korak 6); do tada lica vanjskih na stranici pregleda su
   bez odore.

## 11. Leonove odluke (anketa 18.10.) — zamjenjuju pretpostavke 2–4 iz §10

1. **Liječnik:** prihvaćena preporuka — liječnik hitne u odori hitne, bijela kuta + stetoskop za **novu ulogu vanjskog `lijecnik`**
   (sesija banke dodaje ~10–20 ljudi; zadatak za ledger banke).
2. **Hitna:** crvena odora sa **znakom hitne — plava zvijezda života**, bez crvenog križa (Leon prihvatio objašnjenje o zaštićenom znaku).
3. **Vatrogasac: s kacigom** (`hat-fire`, ~20 min crteža više; kaciga, kao i šapka, crtom pokriva kosu iznad čela; polja izgleda (test 3) ostaju ista).

**Leonov dio na kraju:** /likovi → red „Službe" (10–20 sekundi gledanja, C11 obrazac); kad cigla sadržaja doda uloge, lica u
situacijama jednog kruga.

## 12. Review Focus kandidati

1. Ključ uloge se uspoređuje točno; `inspektor_sanitarni`, `gost`, sidra i v1 nemaju uniformu i izgledaju bajt-isto.
2. `applyUniform` ne troši rng i ne dira polja lica: test 3 prolazi za sve kombinacije, zamrznuti snimci netaknuti.
3. Nove opcije i boje su **na kraju** nizova; `normalizeLook` starog izgleda daje isto.
4. `character_role_key` traži primjerak **hotela** (kao `ai_entry_for`), ne prvi u igri (ista šifra kod više hotela).
5. Oznake vidljive na 36 px; nigdje crveni križ; uloga lika i dalje piše tekstom uz avatar.
6. `Avatar.tsx` ispod 600 redaka, `check:kod` bez novog upozorenja; `openapi.json` samo iz `gen:api`.
7. Test 6 (čuvar sadržaja) ne daje lažni pogodak na „hitno" i hvata „policajka" pod krivim ključem.

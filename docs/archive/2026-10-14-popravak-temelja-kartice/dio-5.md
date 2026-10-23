# 2026-10-14-popravak-temelja-kartice — dio 5

Arhiva (zamrznuto). Indeks: [2026-10-14-popravak-temelja-kartice.md](../2026-10-14-popravak-temelja-kartice.md).

Cigla **IZGLED-c** dodaje **uniformu** na avatar vanjskog lika: policija, vatrogasci, hitna i liječnik. Nastavlja IZGLED a+b
(spojeno 872ff46, kartica [task-13](./dio-4.md#task-izgled-izgled-lika-iz-roda-dobi-i-podrijetla)): isti crtić iz podataka, isti put (app → API → web), isti
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

Dio 1 cigle [L1-13](#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 1 · kvar i popravak (A1) — `l113-1-kvar`

**Cilj:** kvar prostora spušta stanje prostora za `fault_condition_pts` u krugu u kojem se pojavio (i uz šutnju); hitni popravak
uz ocjenu ≥ 4 diže `fault_fixed_condition_pts`. U repou oba 0 i `repair_urgent_condition_pts` ostaje 100 do dijela 4.

**„Kvar prostora" 🟨:** predložak vrste `kvar` **ili** s `EquipmentRepair` u nekom odgovoru, a bez `external_cause`. Danas točno 9:
P-KLIMA, P-DOTRAJALO (pritužbe s hitnim popravkom), P-BOJLER, P-BAZEN-PUMPA, P-LOBBY-KAVA, P-DIZALO, P-PRAONICA, P-BRAVE,
P-CURENJE-VODE. Nisu: P-DOBAVLJAC, P-TERMINAL, P-INSPEKCIJA, P-TOPLINSKI-VAL, P-OLUJA, P-NESTANAK-STRUJE (vanjski uzrok).
Prostor je `condition_of`, inače odjel primjerka, inače Smještaj (postojeće pravilo `situation_effect` `:52-55`). Sidra S1–S4 nisu kvar.

**Files** (f81a622):
- Create: `crates/sim-hotel-v2/tests/situation_fault.rs`, `crates/content/tests/v2_situation_fault.rs`, `crates/game/tests/situation_fault.rs`.
- Modify:
  - `crates/sim-hotel-v2/src/catalog.rs:33-50` (dva polja uz `repair_urgent_condition_pts` :46-48), `:392-397` (ista provjera
    raspona za oba), `:686` (testni literal: `0`).
  - `crates/content/src/v2_dept.rs:238-240` (DTO `Option<i32>`, `skip_serializing_if`), `:499-500` (natrag u DTO samo kad ≠ 0),
    `:550` (`unwrap_or(0)`).
  - `crates/sim-hotel-v2/src/situations/template.rs:59-80` (`SituationSpec.space_fault: bool`), `:31-45` (`TemplateSet::space_fault`
    uz `no_penalty` :34 i `condition_of` :41).
  - `crates/content/src/templates_core.rs:47-` (`template()`: `space_fault` iz vrste, odgovora i `external_cause`);
    `crates/content/src/templates_instance.rs:215-234` (`SituationTemplates::space_fault(code)`, isti obrazac šifre kao `no_penalty`).
  - `crates/sim-hotel-v2/src/situations/mod.rs:80-96` (`fault`), `crates/sim-hotel-v2/src/resolve/situations.rs:15-57` (koraci 3 i 5).
  - `crates/sim-hotel-v2/src/season.rs:356-375` (uz `no_penalty` i `condition_of` u n-torci primjerka i `space_fault`; graditelj smije
    n-torku pretvoriti u mali `struct`), `:407-428` (`fault` u `RoundSituation`).
  - `crates/game/src/drawn.rs:20-32` (`DrawnInstance.space_fault`), `:231` (punjenje), uz `:399-412` novi `Game::fault(h, r, code)`;
    `crates/game/src/resolve.rs:37-46` (`fault` u `push`).
  - Ostali literali `RoundSituation {`: `crates/content/src/templates_principles.rs:324`, `crates/game/tests/situations_delegate.rs:315`,
    `crates/game/tests/situations_replay.rs:134`, `crates/sim-hotel-v2/tests/situation_space.rs:26` (`fault: false`).
- Ne dirati: `content/hotel/v2/economy.json` (ključevi tek u dijelu 4), `sim-hotel` (v1), `fixtures_v2/*`, `prompts/`, `docs/`.

**Testovi čuvara** (RED: polja i funkcije ne postoje, E0560/E0599; katalog s ključevima 100/200 i `repair_urgent_condition_pts` 0
gradi se u testu iz repoa, obrazac `crates/content/tests/v2_principles.rs:287`):
1. `fault_lowers_space_condition_by_fault_pts` — `fault: true`, šutnja bez mehanika opreme → oprema prostora točno −100.
2. `urgent_repair_with_good_answer_nets_plus_one` — hitni popravak uz q 5 i uz q 4 → +100 (−100 + 200).
3. `urgent_repair_with_weak_answer_keeps_minus_one` — uz q 3 i q 1 → −100.
4. `fault_rule_is_off_without_keys` — ključevi 0: `SituationEffect` jednak današnjem za svih 9 predložaka i oba kruga (karakterizacija).
5. `repair_without_fault_gets_no_fix_bonus` — `fault: false`, hitni popravak q 5 → samo stari ključ.
6. `fault_effect_has_no_round_factor` — isti −100 / +200 tjedno i mjesečno.
7. `fault_keys_out_of_range_are_rejected` — −1 i 10 001 → `CatalogError::OutOfRange("v2.economy.fault_…")`.
8. `fault_outcomes_table` — ispisuje (`--nocapture`) Leonovu tablicu za P-BRAVE i P-KLIMA: šutnja / popravak uz 3 / uz 5 → −1 / −1 / +1.
9. (content) `space_fault_templates_are_the_nine` — popis gore, i šest vanjskih `false`; `no_template_is_fault_and_reward_only`;
   `repo_fault_keys_are_off_until_part_four` (repo: ključeva nema, `repair_urgent_condition_pts` 100; dio 4 ga mijenja);
   `missing_fault_keys_read_as_zero`.
10. (game) `game_applies_fault_from_drawn_template` — v2 igra nad snimkom s ključevima (`Snapshot::read_v2(&repo_dir())` + izmjena
    JSON-a, obrazac `v2_sealed.rs:428-438`, pa `common::v2_game_with`): prvi krug s kvarom prostora, šutnja → stanje prostora 100 bp
    niže nego u istoj igri bez ključeva (isto sjeme; uspoređuje se samo taj krug).
Ostaju zeleni bez izmjene: `v2_sealed` (dokaz neutralnosti), `answer_paths_in_game_match_balance` (`answer_paths_v2.rs:260`, isto
pravilo u igri i balansu), `v1_frozen`, `templates_principles` (šutnja i odgovor dobivaju isti −100).

**Koraci graditelja:** (1) `EconomyV2` + DTO + provjera; (2) `space_fault` u specifikaciji, pretvorbi i oba skupa; (3) `RoundSituation.fault`
i svi literali; (4) `situation_effect` koraci 3 i 5; (5) igra (`DrawnInstance`, `Game::fault`, `resolve.rs`) i sezona; (6) `cargo test
--workspace`; balans tjedno i mjesečno `cmp` s osnovicom (**bajt-isto**); (7) probni balans: kopija `content/hotel` u scratchpad s
`repair_urgent_condition_pts` 0, `fault_condition_pts` 100, `fault_fixed_condition_pts` 200, `--content <kopija>` tjedno i mjesečno, B3 blok u izvještaj.

**Rizik:** probni balans pokaže smjer: zapušten hotel (više kvarova) gubi više nego danas (🟨 17.10. „−1 može spustiti lošeg studenta").
Ako probni balans ruši pojas, to je ulaz za dio 4, ne razlog za izmjenu pravila u dijelu 1.

Dio 1b cigle [L1-13](#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 1b · klik nosi ~75 % (Leon 18.10. t. 1) — `l113-1b-klik`

**Cilj:** klik na ponuđeni odgovor gdje se smije pisati (`click_capped`) i prepisan odgovor (`copied`) dobivaju `click_gain_bp` od svake
dobiti (korak 4); vrijednost u `economy.json` izmjerena tako da je težina klika ≤ 75 % težine vlastitog najboljeg za A, B i C,
tjedno i mjesečno (cilj 70–75 %), a lanac `klik − srednji > 0` i dalje vrijedi. Nova brana „B3 klik".

**Cilj mjerenja uz današnju težinu vlastitog** (izračunato iz `nakon-pravilo-c`; Δ = najbolji/klik − šutnja, stotinke):

| | tjedno A | B | C | mjesečno A | B | C |
|---|---|---|---|---|---|---|
| Δ klika danas | 540 | 557 | 618 | 259 | 321 | 258 |
| Δ klika za omjer 75 % | ≤ 383 | ≤ 393 | ≤ 462 | ≤ 194 | ≤ 231 | ≤ 196 |
| pad prema „klik − srednji" danas | 157 < 274 | 164 < 224 | 156 < 283 | 65 < 193 | 90 < 217 | 62 < 161 |

Pad od ~25–30 % Δ klika stane u razmak do srednjeg na svim profilima, pa lanac ostaje. `click_gain_bp` 🟨 [PROC] vjerojatno 3 000–6 000.

**Files:**
- Modify (jezgra): `crates/sim-hotel-v2/src/situations/mod.rs:80-96` (`click`), `crates/sim-hotel-v2/src/resolve/situations.rs` (korak 4,
  i za `fault_fixed_condition_pts` i grupe), `crates/sim-hotel-v2/src/catalog.rs:33-50`, `:392-397` (`click_gain_bp`), `:686`;
  `crates/content/src/v2_dept.rs:238-240`, `:499-500`, `:550` (`unwrap_or(10_000)`; u DTO samo kad ≠ 10 000).
- Modify (sezona): `crates/sim-hotel-v2/src/season.rs:310-312` — `Responder` vraća `Vec<Answered>`, novi
  `pub struct Answered { pub outcome: SituationOutcome, pub click: bool }` s `From<SituationOutcome>` (`click: false`);
  `:324-334` (`run_season_from`: `.into()`), `:403-428` (`click` u `RoundSituation`); izvoz u `crates/sim-hotel-v2/src/lib.rs:41-43`.
- Modify (igra): `crates/game/src/resolve.rs:47-70` (`click: self.click_capped(..)`), `:71-109` (`click: v.copied.is_some() &&
  self.click_capped(..)`), ostale grane `false`; `crates/game/src/own_words.rs:22` (`CAPPED_FACT`: „…ograničena na 4/5 i dobiva manji
  učinak…"); `packages/i18n/src/hr.ts:256` (`answers.clickCap`: „Klikom dobivaš najviše 4 od 5 i manji učinak odluke. Za punih 5 i
  puni učinak odgovori gostu svojim riječima.").
- Modify (balans): `crates/balance/src/v2_answers.rs:86-137` (`respond` → `Vec<Answered>`, `ClickBest ∧ writable` → `click: true`),
  `:149-150`, uz `:268-269` nova `pub const CLICK_WEIGHT_MAX_BP: i32 = 7_500;`, `:306-357` (brana), `:393-405` (naslov bez „samo
  ispis, nije brana" + redak omjera); `crates/balance/src/v2.rs:28-29` (izvoz).
- Modify (sadržaj): `content/hotel/v2/economy.json` (`"click_gain_bp": <izmjereno>`).
- Modify (testovi): `crates/balance/tests/answer_gates_v2.rs:42-51` (`clean`: `click_vs_silent = ruler / 5` → 16,7 % uz 25 %, omjer 66,7 %),
  `:309-332` (`weight_band_stays_on_own_words_best` prepisati: pojas i dalje na vlastitom, klik ima svoju branu).
- Ne dirati: put `Best`, roboti (`run_season_from` uvijek `click: false`), `Classic`, AI upute (`prompts/`), `docs/`.

**Brana (format poruke čuva čuvar):** za svaki profil s `best_vs_silent[2] > 0`: `share_bp(click) × 10 000 > CLICK_WEIGHT_MAX_BP ×
share_bp(best)` → `"B3 klik: {p} klik najbolji nosi {x} težine vlastitog najboljeg (cilj najviše 75,0 %)"`. Redak ispisa:
`"  klik prema vlastitom (težina klika / težina najboljeg; brana najviše 75,0 %):"` pa `"    A 71,2 %"`.

**Testovi čuvara** (RED: `Answered`, `click`, `click_gain_bp`, `CLICK_WEIGHT_MAX_BP` ne postoje):
1. (jezgra, `crates/sim-hotel-v2/tests/situation_click.rs` nova) `click_scales_positive_effects_only` (zadovoljstvo i oprema × faktor,
   trošak pun); `click_full_gain_is_identity` (10 000 → isti `SituationEffect`); `click_scales_group_room_nights` (najmanje 1);
   `click_scales_fault_fix_bonus` (s ključevima dijela 1: −100 + 200 × faktor); `click_never_scales_losses` (šutnja, negativni bodovi).
2. (igra, `crates/game/tests/answer_rule.rs`) `click_loses_part_of_gain` (klik najboljeg s likom: napredak niži nego nad snimkom s
   `click_gain_bp` 10 000, isto sjeme); `copied_answer_loses_gain_like_click`; `own_words_best_keeps_full_gain`;
   `classic_click_keeps_full_gain`; `robot_click_keeps_full_gain`.
3. (balans) `click_weight_gate_fails_above_three_quarters` (klik 80 % težine → točna poruka); `click_weight_gate_passes_at_three_quarters`
   (rub prolazi); `click_gate_needs_positive_best` (uz Δ najboljeg ≤ 0 pada samo „B3 odgovori"); `mirrors_match_the_game` ostaje;
   `answer_paths_in_game_match_balance` (`answer_paths_v2.rs:260`) mora ostati zelen (igra i balans istom zastavicom).
4. `crates/app/tests/answer_rule.rs` / vitest `OfferedAnswers.test.tsx`: regex `/Klikom dobivaš najviše 4 od 5/` ostaje; novi dio `/manji učinak/`.
Ostaje zelen bez izmjene: `v2_sealed` (snimak bez ključa = 10 000).

**Koraci graditelja:** (1) jezgra i sadržaj s ključem 10 000 (balans bajt-isti — kontrolna točka); (2) igra i balans; (3) mjerenje
`click_gain_bp` u koracima od 1 000 (tjedno i mjesečno 1000 sjemena; tablica u izvještaj), izbor najvećeg koji drži omjer ≤ 75 % na
svih 6 stupaca; (4) branu i ispis; (5) sve provjere. **Diff balansa** smije dirati samo stupac „klik najbolji", korake 0 i 1, težinu
klika i novi redak omjera; sve ostalo (uključivo težinu vlastitog) bajt-isto.

**Rizik:** nakon dijela 4 težina vlastitog pada na ~20 %, a omjer težina tada se približava omjeru Δ (niži od današnjeg), pa isti
faktor daje omjer ispod 70 %; dio 4 ga diže natrag u 70–75 %. Ako ni `click_gain_bp` 0 ne spusti omjer ispod 75 % (dobit nije glavni
dio Δ), čuvar to javlja voditelju prije koda — tada se umanjuje i trošak klika, što je nova odluka za Leona.

Dio 2 cigle [L1-13](#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 2 · tri razine značaja (B2, Leon 18.10. t. 2) — `l113-2-znacaj`

**Cilj:** razina iz `weight`: 4 **ključna**, 3 **važna**, 1–2 **svakodnevna** (danas 9 / 17 / 29 predložaka). Bodovi situacije × faktor
razine (`economy.significance_bp`, u repou 10 000 sva tri; dio 4 kreće od 15 000 / 10 000 / 6 000 [PROC]). Troškovi, grupe i kvar
se ne množe. Student vidi razinu kao **tekst** uz naslov (oznaka, ne samo boja; [UI_PRAVILA](../../product/UI_PRAVILA.md)).
Sidra S1–S4 nemaju `weight`: računaju se kao važna, oznake nema 🟨. `major` (`templates.rs:83-85`) ostaje uvjet izvlačenja.

**Files:**
- Modify (jezgra): `crates/sim-hotel-v2/src/situations/mod.rs` (`Significance`, polje), `crates/sim-hotel-v2/src/catalog.rs:33-50`,
  `:392-397`, `:686`; `crates/sim-hotel-v2/src/resolve/situations.rs:37-51`, `:122-128` (korak 2); `crates/sim-hotel-v2/src/season.rs:407-428`
  (`item.drawn.map(|d| Significance::from_weight(d.weight))`, sidra `Important`).
- Modify (sadržaj): `crates/content/src/v2_dept.rs:230-241`, `:490-501`, `:540-552` (`SignificanceBpDto`, `deny_unknown_fields`).
- Modify (igra): `crates/game/src/drawn.rs:399-428` (uz `space` novi `Game::significance(h, r, code)` iz `i.drawn.weight`),
  `crates/game/src/resolve.rs:37-46`.
- Modify (pogled): `crates/app/src/views.rs:48-75` (uz `weight` :63 `significance: Option<Significance>`; `None` za sidra i v1), `:363`;
  `crates/api/src/schema_situations.rs:13-40` (uz `weight` :36 `significance: Option<SignificanceBody>`, `rename_all = "snake_case"`,
  vrijednosti `kljucna` / `vazna` / `svakodnevna`), `:279`; `openapi.json` i `packages/api-client/src` (`corepack pnpm gen:api`).
- Modify (web): `apps/web/src/screens/SituationsScreen.tsx:74-77` (`<Badge>` razine prije statusa), `apps/web/src/screens/SituationScreen.tsx:86-96`
  (`<Badge>` uz naslov); `packages/i18n/src/hr.ts:227` (`situation.significance.kljucna` „Ključna", `.vazna` „Važna", `.svakodnevna`
  „Svakodnevna"); `apps/web/src/test/fixtures.ts` (novo polje).
- Ne dirati: `content/hotel/v2/economy.json`, `situation_templates.json` (razina se izvodi, nema novog polja), mentor (🟨 kasnije).

**Testovi čuvara** (RED: `Significance`, `significance_bp`, polja pogleda ne postoje):
1. `significance_from_weight` (4 → Key, 3 → Important, 2 i 1 → Everyday) i `code()`.
2. `significance_scales_points_not_costs` (Key 15 000: bodovi × 1,5 uz jedno zaokruživanje, trošak isti); `neutral_significance_is_identity`.
3. (content) `levels_cover_all_templates_9_17_29`; `missing_significance_reads_neutral`; `significance_out_of_range_is_rejected`.
4. (app) `situations_view_shows_significance` (primjerak `Some`, sidro `None`, v1 `None`); (api) polje u `openapi.json`.
5. (vitest, `SituationsScreen.test.tsx` i `SituationScreen` test) oznaka „Ključna" vidi se kao tekst; sidro bez oznake.
Ostaju zeleni: `v2_sealed`, balans bajt-isti (faktori 10 000).

**Koraci graditelja:** jezgra → sadržaj → igra i sezona → pogled, API (`gen:api`), web → probni balans s 15 000 / 10 000 / 6 000
(`--content` kopija; težina i B3 u izvještaj) → sve provjere.

**Rizik:** `weight` sada nosi tri značenja (poredak, prednost u izvlačenju, faktor bodova). Ključnih ima malo, ali se češće izvlače, pa
prosječni faktor ne treba biti 1; mjeri dio 4. Ako Leon želi razdvojiti, dolazi polje `significance` u predlošku (~2 h, 55 predložaka).

Dio 3 cigle [L1-13](#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 3 · situacije s gostima rastu s popunjenošću (A7) — `l113-3-gosti`

**Nalaz 18.10.:** broj već raste s **očekivanom popunjenošću kruga** (`crates/sim-hotel-v2/src/situations/count.rs:74-81`, niska /
srednja / visoka; zadano tjedno 3/4/5, mjesečno 4/5/6), ali dodatna mjesta puni bilo koji predložak. Novo pravilo (uz ključ):
mjesta **iznad `low`** (`SituationCounts::levels().0`) smiju puniti samo situacije s gostima, pa je očekivani broj ostalih isti na
svakoj popunjenosti. Ukupni broj ostaje u postavci igre; prve 3 ostaju obvezne (`game::MANDATORY_SITUATIONS`). Izvor popunjenosti
ostaje očekivana popunjenost kruga (ne prošli krug kao u obrisu; poznata je na početku kruga).

**„S gostima" 🟨:** uloga `from: guest` ili odjel `*guest_facing` (`SituationSpec::guest_related`: `RoleSource::Guest`, `DeptTarget::GuestFacing`,
`template.rs:59-80`, `:192-200`); danas 27 od 55. Strogo „isti broj ostalih u svakom krugu" ne gradi se (sukob sa sidrima i raznolikošću
vrsta); vrijedi u prosjeku. Cijena: Leon može vidjeti krug s dvije negostinjske uz nisku popunjenost.

**Files:**
- Modify: `crates/sim-hotel-v2/src/situations/template.rs:14-29` (`guest_extra_slots`), `:59-80` (`guest_related`);
  `crates/sim-hotel-v2/src/situations/draw.rs:182-187` (uz `round_count` `round_base`), `:465-558` (kad je ključ uključen i
  `pre + drawn ≥ round_base`, `allowed` zadržava samo `guest_related`); `crates/content/src/templates.rs:12-25` i `:35-54`
  (`guest_extra_slots: Option<bool>`, `serde(default, skip_serializing_if)`), `crates/content/src/templates_core.rs:15-` (`to_core`);
  `crates/balance/src/v2_situations.rs:170-190` (`DrawStats`: gostinjske i ostale po razini popunjenosti), ispis bloka J-3 u
  `crates/balance/src/v2.rs` (jedan redak; v2.rs ima 936 od 1000 dopuštenih redaka, kod ispisa ide u `v2_situations.rs`).
- Ne dirati: `count.rs` (broj se ne mijenja), `situation_templates.json` (ključ tek u dijelu 4), igra (izvlačenje je zajedničko).

**Testovi čuvara** (`crates/sim-hotel-v2/tests/situations_guest_slots.rs` nova; RED: polje i metoda ne postoje):
1. `extra_slots_above_low_are_guest_related` — 1000 sjemena, visoka popunjenost: negostinjskih primjeraka ≤ `low` − sidra u svakom krugu.
2. `more_occupancy_never_fewer_guest_situations` — prosjek gostinjskih po krugu: visoka ≥ srednja ≥ niska.
3. `other_situations_same_on_average` — prosjek ostalih po krugu, niska prema visokoj, razlika ≤ 0,15 🟨.
4. `flag_off_is_todays_draw` — ključ `false`: izlaz `draw_situations_with` jednak današnjem za 200 sjemena (karakterizacija).
5. (content) `guest_related_templates_are_27` (popis); `missing_guest_flag_reads_false`.
6. (balans) `report_prints_guest_and_other_counts` (novi redak u bloku J-3).
Ostaju zeleni: `v2_sealed`, paritet igre i balansa.

**Koraci graditelja:** ključ i metoda → izvlačenje → ispis → `cmp` balansa: razlika **samo novi redak** → probni balans s ključem `true`
(prosjek po krugu i po sezoni, težina) → sve provjere. **Rizik:** gostinjskih kandidata ponekad nema (uvjeti, razmak), pa krug ima manje
situacija; prosjek po sezoni (danas tjedno 64,67) mora se vidjeti u izvještaju.

Dio 4 cigle [L1-13](#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 4 · kalibracija (A2, A3, A4, A8) i 2.6.0 — `l113-4-kalibracija`

**Cilj:** uključiti dijelove 1–3 u repou, naći brojke s kojima vlastiti najbolji nosi **15–25 %** za A, B i C tjedno i mjesečno, klik
**70–75 %** težine vlastitog (brana ≤ 75 %), lanac pozitivan, `SilenceNotBetter` 0; zapečatiti 2.6.0.

**Files:**
- `content/hotel/v2/economy.json`: `repair_urgent_condition_pts` 100 → ključ se briše (0), `fault_condition_pts` 100, `fault_fixed_condition_pts`
  200, `significance_bp` (početak 15 000 / 10 000 / 6 000), `click_gain_bp` (ponovno izmjereno).
- `content/hotel/v2/situation_templates.json`: `guest_extra_slots: true`; `chance` i `when.dept_condition_pts` 9 kvarova prostora
  (inačica iz tablice A2); A4 trošak na slabijem odgovoru (popis daje čuvar testom; grubo: P-VJENCANJE, P-LEZALJKE, P-KONGRES,
  P-GOST-POZLILO, P-LJUBIMAC, P-BUCNA-SOBA, P-POHVALA, P-DIJETE-IZGUBLJENO, P-RACUN-GRESKA, P-SLOBODNI-DANI, P-PROSIDBA); A8 `unanswered`
  za 16 predložaka s popisa `crates/content/src/templates_principles_debt.rs:15-35`.
- `content/hotel/v2/periods.json:7` i `:15` (`situation_effect_bp` 350 mjesečno, 325 tjedno → izmjereno).
- `crates/content/src/templates_principles_debt.rs:1-35` (popis `SilenceNotBetter` prazan, opis).
- `crates/balance/src/v2_answers.rs:268-269` (`WEIGHT_BAND_BP` → `(1_500, 2_500)`), brisanje `:271-280` (`ORDER_DEBT_FLOOR`, `ORDER_DEBT`),
  `:336` (oprost) i `:345-356` (petlja „B3 dug"); `crates/balance/src/v2.rs:28-29` (izvoz).
- `crates/balance/src/v2_faults.rs` (nova): blok A2 (niže); ožičenje u `v2.rs` najviše ~20 redaka (granica 1000).
- Testovi: `crates/balance/tests/answer_gates_v2.rs:12`, `:18-21` (brisanje), `:39-51` (`clean`: najbolji `ruler / 4` = 20 %, klik
  `ruler / 6` = 14,3 %, omjer 71 %), `:82` (`(1_500, 2_500)`), `:198-215` (`ORDER_DEBT_FLOOR` → `-25` doslovno), brisanje `:217-234`,
  `:255-264` (rub: Δ = ravnalo / 3 = 25 % prolazi, + 1 pada; donji rub 15 %); `crates/content/tests/v2_situation_space.rs:26`, `:42`
  (100 → 0), `crates/content/tests/v2_principles.rs:287`; test dijela 1 `repo_fault_keys_are_off_until_part_four` → `repo_fault_keys_are_on`.
  „Grana `clean(Month)`" iz obrisa više ne postoji (`clean` je od B1a jedan oblik); ništa za brisanje.
- Pečat: `crates/game/tests/v2_sealed.rs:31-40` (`Case.l113: bool`), `:41-128` (tri nova slučaja: `t7-odgovara-l113`, `m7-klik-l113`
  uz `OwnWords`, `t42-sutnja-l113`), `:189-` (`play`: uz `l113` zamrznuti snimak dobiva **u testu zapisane** ključeve dijela 4 i
  `guest_extra_slots`), novi `digest_reacts_to_l113_keys` uz `:428-438`; `crates/sim-hotel-v2/Cargo.toml:3` `2.5.0` → `2.6.0` i
  `Cargo.lock`; `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest` (pečat 2.5.0 ostaje).
- Ne dirati: `plan_ruler` (`v2_answers.rs:287-296`, mijenja se samo Leonovom odlukom), `fixtures_v2/v2-snapshot.json`, `prompts/`.

**A2 — tablica mjerenja koju Leon vidi prije spajanja.** Novi blok balansa (zadani plan, put „najbolji", održavanje **svih** odjela
postavljeno kao u probi `ProbeV2::MaintenanceAll`, `crates/balance/src/v2.rs:252-276`):
```
== Kvarovi prostora (A2): zadani plan, najbolji odgovor; održavanje svih odjela pojačano / zadano / odgođeno ==
  kvarova po sezoni, medijan:            A 2 / 4 / 9   B … C …
  stanje Smještaja na kraju, medijan:    A 71,0 / 64,5 / 48,2 …
  napredak na kraju, medijan:            A … / … / …
```
Inačice (čestoća 9 kvarova prostora, ostalo isto) 🟨 [PROC]: **I0** današnja (osnova 2, ×3 ispod praga stanja); **I1** osnova 1, ×4;
**I2** osnova 1, ×6, prag +10 bodova; **I3** osnova 0 izvan `when` (kvar samo kad je prostor loš), ×8. Za svaku: kopija
`cp -r content/hotel <scratchpad>/l113-I<n>`, izmjena `chance`/`when` skriptom (`node`, zapisana u izvještaj), pa
`cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check --content <scratchpad>/l113-I<n>` i isto `--round month`.
Leonova tablica (`.superpowers/sdd/2026-10-14-popravak-temelja/l113-4-a2-tablica.md`, sažetak u izvještaju):

| inačica | čestoća (osnova, ×, prag) | kvarova/sezonu tjedno A·B·C: pojačano / odgođeno | omjer odgođeno : pojačano | stanje Smještaja na kraju (pojačano / odgođeno) | težina vlastitog tjedno A·B·C | mjesečno A·B·C | klik : vlastiti | `--check` |
|---|---|---|---|---|---|---|---|---|

Voditelj bira inačicu „pravedno": dobro održan hotel ima malo kvarova (cilj ≤ 1 na mjesec 🟨), zapušten barem trostruko, a sve brane
prolaze. Leon vidi tablicu prije spajanja; spaja se tek uz njegov OK.

**Testovi čuvara** (RED: brane i popisi stari):
1. `weight_band_is_fifteen_to_twenty_five` (`WEIGHT_BAND_BP == (1_500, 2_500)`, rubovi 15 % i 25 % prolaze, izvan padaju).
2. `debt_mechanism_is_gone` (nema „B3 dug" ni oprosta; korak ≤ 0 uvijek pada, i mjesečno).
3. `silence_never_better_has_no_debt` (`KNOWN_VIOLATIONS` za `SilenceNotBetter` prazan; učitavanje repoa prolazi).
4. `weaker_answer_costs_where_only_best_did` (A4: nijedan predložak nema trošak samo u najboljem odgovoru; točan popis kao karakterizacija).
5. `fault_report_prints_three_maintenance_levels` (blok A2) i `well_kept_hotel_has_fewer_faults` (pojačano ≤ zadano ≤ odgođeno, medijan, 3 sjemena).
6. `repo_fault_keys_are_on`, `repo_guest_extra_slots_on`, `repo_significance_factors_set`.
7. Pečat: `digest_reacts_to_l113_keys`; `digest_cases_cover_rounds_profiles_and_answer_paths` traži barem jedan slučaj s kvarom i klikom.

**Koraci graditelja:** (1) uključiti ključeve dijelova 1–3 s početnim brojkama, izmjeriti (tablica „prije kalibracije"); (2) A8 i A4
(sadržaj), izmjeriti; (3) A2 tablica i izbor inačice; (4) `situation_effect_bp` dok vlastiti nije 15–25 % na svih 6 stupaca (A3: broj
situacija se ne dira); (5) faktori razina (ključna mora nositi više od svakodnevne i nakon kalibracije); (6) `click_gain_bp` na 70–75 %;
(7) pojas, brisanje duga, testovi; (8) `CORE_VERSION` 2.6.0 i pečat; (9) sve provjere i tablica prije/poslije za Leona.

**Rizik:** `situation_effect_bp` množi samo bodove, ne novac i grupe; ako ni 0 ne spušta težinu na 25 %, slabi se novac odgovora (A3 kaže
„novac i bodovi odgovora") — javiti voditelju s brojkama prije izmjene iznosa naknada.

Cigla L1-13 je **jedna** zajednička kalibracija brojki v2 igre na kraju faze B, po Leonovim odlukama od 17.10.
([ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)) i 18.10. (anketa sesije 9, dno ove kartice).
**Puna kartica za izvođače** (čuvar → graditelj → provjeravatelj), po uzoru na [PRAVILO](./dio-4.md#task-pravilo-vlastiti-odgovor-vrijedi-više-od-klika-klik-najviše-4-vlastite-riječi-do-5): ovo je
pregled (odluke, redoslijed, verzija, zajednička sučelja, provjere, rizici), a svaki dio ima svoju karticu (tablica „Kartice dijelova“);
retci su provjereni 18.10. na f81a622 (`model-v2/sloj1`). Pitanja: [Leonov pregled, dio A](../../plan/2026-10-17-leonov-pregled.md#leonov-pregled--a--pravila-igre-i-brojke)
(A1–A13) i [dio B](../../plan/2026-10-17-leonov-pregled.md#leonov-pregled--b--tekstovi-odgovora-b4-i-mjerilo-ai-ja) (B2).

### Task L1-13: Jedna kalibracija brojki (kvar i popravak, klik 75 %, značaj, gosti i popunjenost, težina ~20 %)

**Veličina:** 🟨 [PROC] 1: 4–5 h · 1b: 4 h · 2: 4 h · 3: 3–4 h · 4: 6–8 h + Leonova tablica · **Ovisi o:** B4c ✅, PRAVILO-a ✅
(9a275d8), PRAVILO-b ✅ (ee6de80), PRAVILO-c i IZGLED-c spojeni · **Slijedi ga:** L1-13b, L1-14, L1-15 · **Model:** čuvar i
graditelj **sonnet**, provjeravatelj **opus** (jezgra, balans) · **Prevodi Rust:** da, svih pet dijelova; **nikad dva dijela
usporedo** (svi diraju napredak i `situation_effect`), jedan Rust slot po sesiji (ADR-059 t. 6).

**Uvjet za svaku izmjenu:** `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check` i isto s
`--round month`, prije i poslije, uz `cmp`/`diff` s osnovicom `.superpowers/sdd/2026-10-14-popravak-temelja/nakon-pravilo-c/`
(ili izlazom prethodnog dijela). Tjedna težina B je **48,9 % uz granicu 50 %**: zato dijelovi 1, 2 i 3 grade mehaniku
**iza ključa sadržaja** (u repou isključen) i balans im mora ostati bajt-isti; brojke se mijenjaju samo u 1b (samo put klika) i 4.

## Leonove odluke (17.10. i 18.10.)

| # | Leon (skraćeno) | što radi L1-13 | dio |
|---|---|---|---|
| A1 | *„Hitni popravak ne rješava stanje tog odjela, već samo privremeno popravlja klimu."* Kvar −1, popravak uz zadovoljnog gosta +2 (neto +1), na **stanju odjela 0–100** | kvar prostora spušta stanje prostora za 100 bp; hitni popravak uz ocjenu ≥ 4 diže 200 bp; privremeni +1 iz [ADR-050](../../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md) (`repair_urgent_condition_pts` 100) → 0 | 1 (mehanika), 4 (uključenje) |
| 18.10. t. 3 | „zadovoljan gost" = ocjena **4 ili 5** uz hitni popravak → +2; 1–3 ostavlja −1 | `FAULT_FIXED_MIN_QUALITY = 4` | 1 |
| 18.10. t. 1 | klik na ponuđeni odgovor nosi **~75 % učinka vlastitog**; gubi i dio mehanike (stanje odjela, novac, gosti); **brana**: težina klika ≤ ~75 % težine vlastitog najboljeg, tjedno i mjesečno | klik pod `OwnWords` u situaciji s likom (i prepisan odgovor) dobiva samo `click_gain_bp` od svake **dobiti**; nova brana „B3 klik" | **1b** |
| B2 + 18.10. t. 2 | tri razine, imenovane, bodovi po razini, student vidi razinu: **ključna / važna / svakodnevna** iz `weight` 4 / 3 / 1–2 | `Significance` iz `weight`, faktor bodova po razini, oznaka na ekranu | 2 (mehanika), 4 (faktori) |
| A7 | *„Broj bi trebao ovisiti o popunjenosti… Broj situacija koje nisu vezane za goste ostaje isti."* | mjesta iznad niske razine broja (`SituationCounts.low`) pune samo situacije s gostima | 3 (mehanika), 4 (uključenje) |
| A2 | *„Prepuštam tebi da pravedno odlučiš brojke."* | čestoću kvarova bira voditelj uz **tablicu mjerenja** (niže) koju Leon vidi prije spajanja | 4 |
| A3 | *„20 %"*, isti broj situacija, slabiji učinak svake | pojas brane 15–25 % (`WEIGHT_BAND_BP`), `situation_effect_bp` niže | 4 |
| A4 | ok | trošak jednom slabijem odgovoru gdje „samo najbolji troši novac" (grubo brojanje 18.10.: ~11 predložaka [PROC]) | 4 |
| A8 | ok | 16 predložaka `SilenceNotBetter` popraviti (šutnja dobiva gore `unanswered`); dug 16 → 0 | 4 |

Ostaje iz tumačenja 17.10. (🟨, bez promjene): **„kvar prostora"** nije samo vrsta `kvar` nego svaki vlastiti kvar (t. 2 niže);
**−1 dolazi u krugu u kojem se kvar pojavio**, i uz šutnju; **sučelje soba** je faza C ili kasnije ([ADR-050](../../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md) Izmjena t. 4), ne L1-13.

## Dijelovi, grane i redoslijed

| dio | grana (iz `model-v2/sloj1`, nakon spajanja prethodnog) | što | balans nakon spajanja | pečat |
|---|---|---|---|---|
| 1 · kvar | `l113-1-kvar` | −100 / +200 na stanju prostora, ključevi u `economy`, u repou 0 | **bajt-isti** | bajt-isti |
| 1b · klik 75 % | `l113-1b-klik` | `click_gain_bp`, zastavica klika, brana „B3 klik"; vrijednost u repou izmjerena | mijenja se **samo put klika** | bajt-isti |
| 2 · značaj | `l113-2-znacaj` | `Significance`, faktori u `economy` (u repou 10 000), oznaka u API-ju i webu | bajt-isti | bajt-isti |
| 3 · gosti | `l113-3-gosti` | dodatna mjesta samo gostima, ključ u predlošcima (u repou `false`), redak ispisa | samo novi redak ispisa | bajt-isti |
| 4 · kalibracija | `l113-4-kalibracija` | uključenje 1–3, A2/A3/A4/A8, pojas 15–25 %, dug, **`CORE_VERSION` 2.6.0**, pečat | nove brojke, tablica za Leona | novi pečat 2.6.0 |

Radno stablo `../hotelixsim-l113-<dio>`, `CARGO_TARGET_DIR=…/hotelixsim/target/l113-<dio>` (B12), briše se odmah nakon spajanja
(B16). Izlazi provjera u `.superpowers/sdd/2026-10-14-popravak-temelja/l113-<dio>-provjere/`.

## Gdje ide klik 75 % i zašto u zaseban dio 1b

**Gdje se klik danas ograničava (samo ocjena):**
- igra: `crates/game/src/game.rs:477-483` (`click_capped` = `OwnWords` ∧ ljudski hotel ∧ `free_text`), `crates/game/src/resolve.rs:52-57`
  (klik `min(q, 4)`) i `:90-95` (prepisan odgovor, `copied`); konstanta `crates/game/src/own_words.rs:11`;
- balans: `crates/balance/src/v2_answers.rs:99-102` (`ClickBest` ∧ `writable` → `min(q, 4)`), zrcalo `:20-22`, `writable_v2`
  u `crates/balance/src/v2_situations.rs:33-46`; težina klika je samo ispis (`:393-405`).

**Zašto klik danas nosi 92–95 %** (nalaz I1, ADR-059 t. 4): bodovi svih mehanika su `(q − 3) × k` (`crates/sim-hotel/src/mechanic.rs:145-236`),
pa klik 4 prema šutnji 1 ima 3/4 bodovnog dijela vlastitog (5 prema 1), a novac, grupe i popravak ne ovise o ocjeni i klik ih
dobiva cijele. Izmjereno (`nakon-pravilo-c`): Δ klika = 86–90 % Δ vlastitog; težina klika tjedno 38,9 / 45,1 / 37,7 % uz
41,6 / 48,9 / 41,2 % (omjer 92–94 %), mjesečno 24,0 / 32,2 / 19,0 % uz 25,7 / 33,8 / 20,3 % (93–95 %).

**Odluka: zaseban dio 1b, odmah nakon dijela 1** (ne u dijelu 4):
1. Dira istu funkciju kao dio 1 (`situation_effect`) i mora odlučiti dobiva li klik +2 popravka (da, umanjeno, jer Leon izrijekom
   kaže „gubi stanje odjela"); dva dijela u istoj funkciji redom, ne u kalibraciji.
2. Mehanika klika **ne dira put „najbolji"** (vlastiti odgovor i roboti nisu klik), pa se može uključiti odmah: težina vlastitog
   (margina 1,1 bod) ostaje ista, mijenjaju se samo retci klika. Brana dobiva pravi broj od prvog dana.
3. Dio 4 je i bez toga najveći; mehanika u njemu bila bi treće štimanje u istom diffu.
Cijena: jedna grana više (~4 h) i dio 4 opet podešava `click_gain_bp` jer niža težina vlastitog mijenja omjer (vidi 1b, Rizik).

## CORE_VERSION i ključevi sadržaja (odluka)

**Jedno dizanje 2.5.0 → 2.6.0, u dijelu 4.** Igra sprema svoj snimak pravila (`GameCreated.snapshot`, `crates/game/src/event.rs:15-36`),
pa promjena **sadržaja** u repou ne mijenja staru igru; verziju traži samo promjena **koda** koja mijenja obračun istog snimka
(pečat `crates/game/tests/v2_sealed.rs:276-298`). Zato:
- Svaki novi ključ ima **neutralnu zadanu vrijednost** kad ga nema (0 bp gubitka, 10 000 bp faktora, `false`): zamrznuti snimak
  pečata i stare igre računaju **bajt-isto**, pa dijelovi 1–3 i 1b ne dižu verziju (pečat 2.5.0 ostaje zelen bez izmjene, to je dokaz).
- Dijelovi 1, 2, 3 drže ključ u repou **isključen** (balans bajt-isti); 1b ga uključuje (samo klik); dio 4 uključuje sve i kalibrira.
- Dio 4 diže verziju jer **dodaje slučajeve pečata** koji igraju zamrznuti snimak s ključevima L1-13 (test 1 traži iste slučajeve
  za istu verziju, `:286-290`; snimak se ne mijenja, `:301-307`). Bez toga nove mehanike ne bi bile zapečaćene.
- **Iza postavke igre (`GameSettings`) ne** — sadržajni ključ je jeftiniji i već je obrazac (`repair_urgent_condition_pts`, `groups`).
**Cijena:** svaka mehanika nosi ključ i granu „isključeno = danas" (test po ključu); dijelovi 1–3 na balansu nisu izmjereni dok
ih dio 4 ne uključi, pa provjeravatelj svakog dijela vrti **probni balans** s uključenim ključem (`--content` kopija, ne commita) i
zapisuje brojke u izvještaj. Leonove probne v2 igre daju **409 jednom**, nakon dijela 4 (Leonu se kaže).

## Zajednička sučelja (Produces; svaki dio dodaje svoj redak)

```rust
// sim_hotel_v2::situations (mod.rs:80-96) — svaki konstruktor dobiva nova polja; From<SituationOutcome>: false/false/Important
pub struct RoundSituation {
    pub outcome: SituationOutcome,
    pub space: Option<DeptCode>,
    pub fault: bool,                 // dio 1: kvar prostora (TemplateSet::space_fault)
    pub click: bool,                 // dio 1b: klik (ili prepisan odgovor) gdje se smije pisati, OwnWords, čovjek
    pub significance: Significance,  // dio 2: iz weight primjerka; sidra Important
}
pub const FAULT_FIXED_MIN_QUALITY: u8 = 4;                      // dio 1, Leon 18.10.
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Significance { Key, Important, Everyday }  // dio 2
impl Significance { pub fn from_weight(w: u8) -> Self; /* 4 Key, 3 Important, ostalo Everyday */ pub fn code(self) -> &'static str /* "kljucna" | "vazna" | "svakodnevna" */ }
// sim_hotel_v2::catalog::EconomyV2 (catalog.rs:33-50), sadržaj content/hotel/v2/economy.json, DTO crates/content/src/v2_dept.rs:230-241
pub fault_condition_pts: i32,          // dio 1; bez ključa 0; provjera 0..=10_000
pub fault_fixed_condition_pts: i32,    // dio 1; bez ključa 0; provjera 0..=10_000
pub click_gain_bp: u16,                // dio 1b; bez ključa 10_000; provjera 0..=10_000
pub significance_bp: SignificanceBp,   // dio 2; { key, important, everyday }, bez ključa 10_000 svaki; provjera 1_000..=30_000 🟨
// sim_hotel_v2::situations::TemplateSet (template.rs:14-29)
pub fn space_fault(&self, code: &str) -> bool;   // dio 1 (uz SituationSpec.space_fault)
pub guest_extra_slots: bool;                     // dio 3; content TemplateFile.guest_extra_slots, bez ključa false
impl SituationSpec { pub fn guest_related(&self) -> bool }  // dio 3
```

**Redoslijed računa u `situation_effect`** (`crates/sim-hotel-v2/src/resolve/situations.rs:27-57`), konačno stanje nakon dijela 2:
1. `e = effects(outcome)` (v1 mehanike, bez izmjene);
2. bodovi (zadovoljstvo, moral, ocjena, privremeni popravak) × **jedan** faktor `situation_effect_bp × significance_bp(razina) / 10 000`
   (jedno zaokruživanje; uz 10 000 točno današnji `scaled`, `:123-128`);
3. oprema += broj hitnih popravaka × `repair_urgent_condition_pts` (stari ključ ostaje radi starih snimaka);
   oprema += `fault_fixed_condition_pts` ako `fault` ∧ hitni popravak u odgovoru ∧ `q ≥ 4` (jednom po situaciji);
4. ako `click`: svaki **pozitivan** dio zadovoljstva, morala, ocjene i opreme × `click_gain_bp / 10 000` (zaokruženo), a
   `room_nights` svake grupe isto (najmanje 1); troškovi, povišice i negativni dijelovi ostaju puni;
5. ako `fault`: oprema −= `fault_condition_pts` (bez faktora kruga, razine i klika: fizički kvar).

## Kartice dijelova (redom, jedna grana po dijelu)

| dio | kartica |
|---|---|
| 1 · kvar i popravak (A1) | [task-14-1-kvar.md](#task-l1-13-dio-1--kvar-i-popravak-a1--l113-1-kvar) |
| 1b · klik nosi ~75 % (18.10. t. 1) | [task-14-1b-klik.md](#task-l1-13-dio-1b--klik-nosi-75--leon-1810-t-1--l113-1b-klik) |
| 2 · tri razine značaja (B2) | [task-14-2-znacaj.md](#task-l1-13-dio-2--tri-razine-značaja-b2-leon-1810-t-2--l113-2-znacaj) |
| 3 · situacije s gostima i popunjenost (A7) | [task-14-3-gosti.md](#task-l1-13-dio-3--situacije-s-gostima-rastu-s-popunjenošću-a7--l113-3-gosti) |
| 4 · kalibracija (A2, A3, A4, A8) i 2.6.0 | [task-14-4-kalibracija.md](#task-l1-13-dio-4--kalibracija-a2-a3-a4-a8-i-260--l113-4-kalibracija) |

## Provjere (svaki dio; cijeli popis iz `.github/workflows/ci.yml`)

`cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` ·
`node scripts/probe-mutacija.mjs` · `git diff --exit-code` (nakon commita) · `cargo sqlx migrate run --source crates/store/migrations` +
`cargo sqlx prepare --workspace --check` · `cargo test --workspace` (uz bazu `hotelixsim_agenti`, ADR-059 t. 8) · `cargo run -q -p balance
--release -- --seeds 100 --check` i `--seeds 1000` (v1 **bajt-isti**, `cmp`) · `--model v2 --round week --seeds 1000 --check` i
`--round month` (`cmp`/`diff` s osnovicom; očekivani diff po dijelu iz tablice dijelova) · `corepack pnpm check:docs`, `test:docs`,
`check:kod`, `check:api`, `lint`, `typecheck`, `test` · `corepack pnpm --filter @hotelixsim/web build`. cargo deny samo u CI-ju.
Lažno crveno pod opterećenjem (vitest timeout, 0xC0000142) samo se ponovi (B15). Ispod 50 GB slobodno ne gradi (B16).

**Provjeravatelj (opus) uz to:** (a) `v2_sealed` zelen bez izmjene u dijelovima 1–3 i 1b; (b) probni balans s uključenim ključem
ponovljen sam, ne prepisan iz izvještaja; (c) u 1b diff balansa samo u retcima klika; (d) ekran izvještaja kruga pokazuje −1 / +2 kao
učinak **situacije** na stanje prostora (redak `explain.rs:220`, uzrok situacija), ne kao održavanje (ADR-050 t. 4 i Izmjena t. 5).

## Rizici i odluke 🟨 (svaka s cijenom ako je kriva)

1. **Kvar prostora = vrsta `kvar` ili hitni/privremeni popravak u odgovoru, bez vanjskog uzroka** (9 predložaka). Cijena: jedno
   pravilo u `templates_core.rs`; ako Leon misli samo na sobe, uvjet `prostor == rooms` (P-BAZEN-PUMPA i P-LOBBY-KAVA ispadaju).
2. **+2 jednom po situaciji**, bez obzira na broj hitnih popravaka u odgovoru; bez faktora kruga i razine. Cijena: jedan uvjet.
3. **Klik gubi dobit, ne štedi trošak** (troškovi puni, dobit × `click_gain_bp`, uključivo +2 popravka i noćenja grupe). Cijena: ako
   Leon ne želi manju grupu za klik, grupe se izuzmu (jedan redak), a faktor ostalog pada.
4. **Brana klika na omjeru težina** (≤ 75,0 %), cilj 70–75 %; nema donje brane. Cijena: jedna konstanta (`CLICK_WEIGHT_MAX_BP`).
5. **Razina iz `weight`**, sidra važna bez oznake, faktori 1,5 / 1,0 / 0,6 kao početak. Cijena: polje `significance` (~2 h) ako se razdvaja.
6. **Dodatna mjesta samo gostima, jednako u prosjeku** (ne strogo po krugu); „s gostima" = uloga gost ili odjel uz goste (27/55).
   Cijena: drugi predikat u `guest_related`.
7. **Ključevi sadržaja umjesto postavke igre; jedno dizanje 2.6.0 u dijelu 4** uz nove slučajeve pečata. Cijena: Leonove probne
   igre 409 jednom; dijelovi 1–3 mjereni samo probnim balansom do dijela 4.
8. **A2 čestoća:** inačice I0–I3 i cilj „održavan ≤ 1 kvar na mjesec, zapušten ×3" su voditeljev prijedlog (Leon: „pravedno").
9. **Tekstovi klika** (`answers.clickCap`, `CAPPED_FACT`) kažu i „manji učinak"; mentor dobiva istu činjenicu, `mentor.v4` se ne mijenja.

## Leonov dio

- **Prije spajanja dijela 4:** tablica A2 (izbor inačice) i tablica prije/poslije (težina vlastitog i klika, A/B/C, tjedno i mjesečno).
- ✅ Odlučeno 18.10. (druga anketa): 🟨 1 — kvar prostora = 9 predložaka (i P-KLIMA, P-DOTRAJALO); 🟨 3 — klik gubi dio dobiti, trošak cijeli; 🟨 6 — „s gostima" = uloga gost ili odjel uz goste (27/55).
- Odlučeno 18.10.: klik ~75 %, imena razina, „zadovoljan gost" ≥ 4.

**Review Focus kandidati:**
1. Prepisan odgovor uz AI ispad: ide kao klik (manja dobit) prije pravila „neutralno 4" (mehanike šutnje, bez dobiti za umanjiti).
2. Klik na slabiji odgovor (q ≤ 4, `capped: false`) i dalje gubi dobit: `click` prati `click_capped`, ne `capped` — namjerno (Leon: „klik nosi 75 %").
3. Kvar uz `no_penalty`: ne postoji (test), inače bi −1 prekršio „samo nagrada".
4. Ponovljivost: `Responder` s `Answered` ne mijenja redoslijed RNG-a (izvlačenje ne čita ishode istog kruga).
5. Pečat 2.5.0 ostaje u `v2-golden.json`; slučajevi `*-l113` postoje samo pod 2.6.0.

## Leonove odluke 18.10. (anketa, sesija 9) — ugrađene gore
1. **Klik (nalaz I1, ADR-059 t. 4):** klik na ponuđeni odgovor nosi **~75 % učinka vlastitog** (ne 92–95 %): gubi i dio mehanike
   (stanje odjela, novac, gosti), ne samo ocjenu. Nova **brana** u balansu: težina klika ≤ ~75 % težine vlastitog najboljeg, tjedno i mjesečno. → dio 1b.
2. **Razine značaja (B2):** **ključna / važna / svakodnevna**, izvedeno iz `weight` (4 / 3 / 1–2). → dio 2.
3. **„Zadovoljan gost":** ocjena odgovora **4 ili 5** uz hitni popravak → +2 (neto +1); ocjena 1–3 ostavlja −1. → dio 1.

## Leonova odluka 18.10. (treća anketa) — klik 75 % samo gdje se piše
Dijagnoza (`.superpowers/…/l113-1b-dijagnoza.md`): situacije bez lika nose 39 % (tjedno B) / 70 % (mjesečno B) Δ i u njima su klik i vlastiti
odgovor isti, pa ukupni omjer ne može pasti na 75 %. **Odluka:** brana „B3 klik" mjeri **omjer Δ samo na situacijama s likom** (≤ 75 %);
`click_gain_bp` ≈ 8 500–8 800 (najveći koji drži sva 6 stupaca). Ukupno u igri klik i dalje nosi ~90 % težine vlastitog — zapisati u ADR na `/kraj`.

## Leonova odluka 18.10. (četvrta anketa, sesija 10) — odblokiranje dijela 4
Izvor: `.superpowers/sdd/2026-10-14-popravak-temelja/l113-4-dijagnoza.md` (graditelj BLOCKED: hotel B je od početka zapušten, ~15 kvarova po sezoni bez obzira na održavanje; lanac srednji < prepušteno).
1. **Kvar i popravak slabe kao ostali bodovi situacije** (množe se faktorom kruga `situation_effect_bp`): tjedno ≈ −0,34 / +0,68, mjesečno ostaje −1 / +2 (K1). Mijenja A1 (bez faktora).
2. **„Zadovoljan gost" = ocjena 3, 4 ili 5** uz hitni popravak → +2 (L1). Mijenja odluku 18.10. (bilo ≥ 4).
3. **Pojas „situacije ~20 %" = prosjek tri hotela 15–25 %, svaki hotel 10–35 %** (K4), tjedno i mjesečno.
4. **P-POHVALA ostaje „samo najbolji nosi nagradu"** (iznimka od A4); **šutnja na P-NOVINAR gubi ugled, ne novac** (test S-4 → „šutnja nije bolja od lošeg odgovora").
Izmjereno (bez faktora razine; ponoviti s njim): tjedno K1+L1 uz f 105–115 sve brane prolaze (f 110: 20,8 / 24,0 / 20,0 %, klik ≤ 72 %); mjesečno f 350 uz L1 sve osim B 31,3 % (K4 to prima).

## Stanje 19.10. (sesija 11): spojeno
Leon je u prvoj anketi 19.10. dao OK na obje tablice (sve preporučene; [ADR-062](../../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 1–4).
Dio 4 spojen kao e4ec659 (`CORE_VERSION` 2.6.0), pushan, GitHub CI zelen. Svi dijelovi L1-13 (1, 1b, 2, 3, 4) su gotovi.

Cigla L1-13b je **ostatak Leonovih odluka iz pregleda 14.10.** ([ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md))
koji nije ušao u fazu B ni u L1-13. Leon 19.10. (sesija 11, anketa): **cijela L1-13b prije pilota** i **rad u valovima** (neovisni dijelovi
istodobno, svaki u svom stablu). Ova kartica je pregled i raspored; analize s retcima (samo čitanje, 19.10., na `e4ec659`):
`.superpowers/sdd/2026-10-14-popravak-temelja/l113b-analiza-mijesanje.md`, `l113b-analiza-odrzavanje.md`, `l113b-analiza-web.md`.

### Task L1-13b: Izmiješani odgovori, pojačano održavanje +5, jedan gumb „Spremi plan", opis Smještaja

**Veličina:** 🟨 [PROC] ukupno 11–17 h rada agenata, oko 1–1,5 sesije uz valove · **Ovisi o:** L1-13 ✅ (e4ec659, `CORE_VERSION` 2.6.0) ·
**Slijedi ga:** L1-14 (Leon igra tjednu i mjesečnu sezonu), L1-15 (`sloj1` → `main`), zatim pilot · **Model:** čuvar i graditelj sonnet,
dio 1 provjeravatelj **opus** (pravilo igre u `game`), ostali sonnet · **Prevodi Rust:** dijelovi 1 i 4; nikad oba usporedo
(jedan Rust slot po sesiji, drugi drži sesija banke).

## Što je već napravljeno iz ADR-042 (ne radi se ponovo)

t. 1 i 7 (kvar i stanje) — L1-13 · t. 6 (lik zaključi u 6 poruka, `lik.v3`) · t. 11 (izgled lika) — IZGLED · t. 12–14 (jednom u krugu,
mjesečno 4/5/6 kao postavka) — B2a. t. 3 (raspon potrebnih radnika) čeka banku ljudi (L2a), t. 9 (ime hotela) je HN-1 u fazi C — **nisu u L1-13b**.

## Dijelovi

| dio | što | dira | Rust | veličina | prije gradnje |
|---|---|---|---|---|---|
| **1 · miješanje** (t. 10) | redoslijed ponuđenih odgovora iz sjemena igre + slota hotela + šifre situacije; app prikazuje izmiješano i kliknuti položaj prevodi natrag u izvorni broj; dnevnik (`game/src/feed.rs`) ne odaje izvorni broj | `game`, `app`, opisi u `openapi.json`; web i ugovor API-ja isti | da (`game`, `app`, `api`) | 3–5 h | 🟨 t. 1–2 niže |
| **2 · jedan gumb** (t. 4) | jedan „Spremi plan" sprema plan i odluke svih odjela odjednom (API to već radi jednim PUT-om i događajem `PlanSet`); upozorenje pri zatvaranju kruga ostaje | samo `apps/web` (+ `packages/i18n`) | ne | 5–7 h (~40 vitest testova se prilagođava) | Leon: gdje stoji gumb |
| **3 · opis Smještaja** | 🟨 vidi pitanje 2 | web + i18n (ili objašnjenje kruga) | ne / da | 1–2 h | **Leon: što je točno traženo** |
| **4 · održavanje +5** (t. 2) | pojačano održavanje neto **+5 bodova stanja mjesečno** (tjedno ~+1,17); danas Smještaj +4, Restoran +3, barovi +1,5–3,5, Wellness +2,5 | `content/hotel/v2/departments.json` (`intensive.equipment_pts`), `crates/content/tests/v2_obnova.rs` | da (test + balans 1000) | 2–3 h | **Leonova anketa s brojkama** (pitanje 3) |

**Brane i pečat:** dio 1 ne mijenja `CORE_VERSION`, pečat ni balans (`crates/balance` ne ovisi o `game`; roboti biraju po kvaliteti).
Dio 4 mijenja samo brojke sadržaja (svaka igra nosi svoj snimak pravila, pečat igra na zamrznutom) — 2.6.0 ostaje; zadani plan koristi
redovno održavanje pa brane `--check` (težina, klik, lanac 0,04) ne bi trebale vidjeti promjenu, ali se **mjere** (1000 tjedno i mjesečno, `cmp`).

## Valovi

```text
val 1 (odmah po Leonovu OK-u):   dio 1 (Rust slot glavne)   ∥   dio 2 (web, bez Rusta)   ∥   dio 3 (web, ako Leon odgovori)
val 2 (kad je dio 1 spojen):     dio 4 (Rust slot glavne; anketa iz pitanja 3 već odgovorena)
spajanje redom:                  1 → 2 → 3 → 4, svaki s cijelim CI-jem (scratchpad/nakon-l113b-N.sh), cmp balansa prema prethodnoj osnovici
```

Svaki dio: svoje stablo `../hotelixsim-l113b-N` (apsolutni put, B22), svoj `CARGO_TARGET_DIR=…/target/l113b-N`, tok čuvar → graditelj →
provjeravatelj (ADR-038). Dijelovi 1, 2 i 3 nemaju zajedničkih datoteka osim `packages/i18n/src/hr.ts` (različiti ključevi — trivijalan
sukob pri spajanju). Ubrzanje prema radu redom oko 1,5× (granica od dva Rust procesa).

## 🟨 Pretpostavke (ADR na `/kraj`)

1. **Miješanje živi u `game` + `app`, ne u webu** — odstupa od doslovnog teksta ADR-042 t. 10 („web šalje izvorni broj"), ali čuva smisao:
   API i web i dalje govore „broj = položaj na ekranu", izvorni broj se nigdje ne vidi, isti student nakon osvježavanja vidi isti redoslijed,
   različiti hoteli različit. Pravilo miješanja se ne mijenja bez dizanja verzije (inače bi klik usred kruga zapisao drugi odgovor).
2. **v1 (zamrznuti demo) ostaje nepromiješan.**
3. **Održavanje +5 bez prekidača sadržaja i bez nove verzije jezgre** (mijenjaju se samo brojke).

## Leonovi odgovori 19.10. (anketa, sesija 11)

- **Opis Smještaja = tekst na stranici Smještaja** (zgrada i sobe određuju kvarove; ADR-042 t. 7) — samo web, ~1 h.
- **Cijena pojačanog ista**, **+5 prije trošenja zbog gužve** (u gužvi +4).
- **Odjeli:** „to se mora izbalansirati isto s financijama, jer ponekad se moraju financije uzeti u obzir, ponekad održavanje".
  Tumačenje voditelja (🟨, potvrditi): **nijedna razina održavanja ne smije biti najbolja uvijek** — dio 4 prije spajanja daje Leonu
  tablicu (kao A2): napredak pojačano / zadano / odgođeno po hotelu, tjedno i mjesečno, za šest operativnih odjela i za svih osam,
  uz to koji hotel „mora štedjeti" (financije) a koji „mora ulagati" (stanje). Polazno šest operativnih; parking i lokal samo ako tablica pokaže potrebu.
- **Gumb „Spremi plan"** (🟨 voditelj): traka vidljiva i na Planu i na stranici odjela.

## Pitanja za Leona (prije vala 1; odgovori gore)

1. **Jedan gumb — gdje stoji?** Preporuka: traka „Spremi plan" vidljiva i na Planu i na stranici odjela (anketa).
2. **„Opis Smještaja" — što si točno htio?** Fraza stoji samo u planu (14.10.), bez zapisa odluke. Dvije mogućnosti:
   (a) tekst na stranici Smještaja da zgrada i sobe određuju kvarove (klima, curenje, brave, dizalo, praonica — ADR-042 t. 7);
   (b) objašnjenje kruga: redak „Održavanje" danas nosi i dio promjene Smještaja, što zbunjuje (nalaz Sloja 1, Task 12 minor).
3. **Održavanje +5 — anketa s brojkama:** (1) koji odjeli (šest operativnih — preporuka; parking i lokal ostaju); (2) cijena pojačanog
   ista (preporuka; pojačano postaje jeftinije po bodu) ili ista po bodu (Smještaj 33.000 €, Bar uz bazen +67 %); (3) +5 prije trošenja
   zbog gužve (preporuka; u gužvi +4) ili poslije (+6 u mirnom krugu); (4) hotel B i uz +5 ostaje ~14–15 kvarova (sobe ~62 < pragovi 65/80)
   — potvrditi „ostaviti" (odluka 19.10.).

**Nalaz uz dio 4:** uz istu cijenu pojačano dobiva oko +1 napretka — u hotelu B tjedno bi se „odgođeno bolje od pojačanog" vjerojatno
okrenulo, u A ne (razlika 3,2). Stavka u [BACKLOG](../../records/BACKLOG.md) ostaje dok se ne izmjeri.

## Stanje 19.10. (kraj sesije 11): svi dijelovi spojeni
Spojeno redom 2 → 3 → 4 → 1, svaki `--no-ff` i pregledan (Approved): **dio 2** jedan gumb 13db069, **dio 3** opis Smještaja 4be1db4, **dio 4**
održavanje +5 bdbdebe (tablica: A štedi, B i C ulažu; šest odjela), **dio 1** miješanje 7556d80. Cijeli CI `nakon-l113b/` zelen, balans bajt-isti
s ispisom dijela 4; pushano, GitHub CI zelen. Pretpostavke 🟨 1–3 i nove su u [ADR-062](../../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md)
(t. 10–17); odgođeni minori u [BACKLOG](../../records/BACKLOG.md). Sljedeće: L1-14.

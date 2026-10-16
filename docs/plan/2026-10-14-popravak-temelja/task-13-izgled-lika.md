Cigla IZGLED provodi [ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 11: izgled lika
situacije mora odgovarati njegovu **rodu, dobi i podrijetlu**. Leon 16.10. (sesija 7, anketa): ide **sada**, zasebno i usporedo
s PRAVILO-om; stil ostaje **crtić iz podataka** (mogućnost A nacrta [IZGLED_LIKA](../../architecture/IZGLED_LIKA.md)). Dva dijela:
**IZGLED-a** (app i API: lik dobiva tri polja) i **IZGLED-b** (web: `lookFor` ih poštuje). Jezgra, dnevnik, balans, pečat i
`CORE_VERSION` se ne diraju. Iz L1-13b time ispada „izgled lika po rodu (rod u `RoleFill`)".

### Task IZGLED: Izgled lika iz roda, dobi i podrijetla

**Veličina:** a ~2,5 h · b ~2 h · pregled ~0,5 h (ukupno ~5 h) · **Ovisi o:** ništa iz faze B (čita samo `content` snimak igre
i postojeće poglede) · **Paralelno s:** PRAVILO-a/b/c i banka ljudi K2, uz „najviše dva agenta prevode Rust" (IZGLED-a prevodi;
IZGLED-b ne). Sudar s PRAVILO-c: obje mijenjaju `schema_situations.rs`, `openapi.json`, `schema.d.ts` i `fixtures.ts` — tko
spaja drugi, ponovno pokreće `corepack pnpm gen:api` (bez ručnog spajanja generiranih datoteka) · **Model:** čuvar i graditelj
sonnet, pregled opus · **Prevodi Rust:** IZGLED-a da (`crates/app`, `crates/api`); IZGLED-b ne (samo pnpm).

**Cilj:**
- Uz svakog lika izvučenog primjerka API šalje `look`: **rod** (`f`/`m`), **dobnu skupinu** (`young`/`adult`/`senior` ili
  `null` kad nije poznata) i **podrijetlo** (`regional`/`foreign`). Isto i u razgovoru (`character_look`).
- Web: `lookFor(ime, look)` — rod određuje bradu, frizuru i naušnice; dob skupinu (i sijedu kosu starijih); podrijetlo raspon
  boje kože. Ime i dalje bira sve ostalo, pa isti lik uvijek izgleda isto.
- Gošća nikad nema bradu; gost nikad naušnice ni žensku frizuru; lik s hrvatskim imenom nikad tamnu put.
- Bez `look` (sidra S1–S4, v1, roboti-hoteli na ljestvici, stari klijent) sve ostaje **bajt-isto** kao danas.

**Zašto (nalaz):** `lookFor(name)` (`apps/web/src/avatar/look.ts:355-357`) zna samo ime: `lookFromSeed` bira kožu iz svih šest
tonova (`SEED_SKIN` :163), frizuru iz osam (`:164`, i „long"/„bun" za muškarca), bradu bez obzira na rod (`:189`). Rod se gubi na
putu: predložak ga ima (`content::Role.gender`, `crates/content/src/templates.rs:239-248`), jezgra po njemu bira ime
(`crates/sim-hotel-v2/src/situations/fill.rs:90`, `ctx.names.list(role.source, role.gender)`), ali `CharacterCardView`
(`crates/app/src/views.rs:74-77`) i `CharacterBody` (`crates/api/src/schema_situations.rs:50-54`) nose samo `name` i `role`.

**Najmanji šav (provjereno na a992973):** app već ima snimak sadržaja igre (`Game.content`, javno, `crates/game/src/game.rs:95`)
s popisom imena `content.situation_templates.names` (`content::Names`, `templates.rs:374-387`). Upit „u kojem je popisu ovo
ime" daje i **rod** i **izvor** (gost/osoblje/partner). Danas je **120 imena, nijedno u dva popisa** (provjereno čitanjem
`content/hotel/v2/names.json`); `check_names` (`crates/content/src/templates_check.rs:54-78`) jamči jedinstvenost samo unutar
jednog popisa, pa app za ime u dva popisa različitog roda vraća `None` (staro ponašanje), a test čuvara drži da takvog imena nema.
Rezerva bez promjene jezgre, ako imena prestanu biti jedinstvena (K5): predložak primjerka (`HotelRecord.drawn` → `drawn.template`
→ `Template.character.role` → `roles[key].gender/from`), sve javno; složenije (krug i primjerak), zato nije prvi izbor.

**Files** (retci provjereni na a992973):
- Create (a): `crates/app/src/look.rs` (tipovi i `character_look`, ~70 r.), `crates/app/tests/character_look.rs`.
- Modify (a):
  - `crates/app/src/lib.rs:3-10` `pub mod look;`, `:12-19` `pub use look::*;`.
  - `crates/app/src/views.rs:74-77` (`CharacterCardView.look: Option<CharacterLook>`), `:336-341` (punjenje:
    `look: character_look(&g.content, &c.name, &c.role)`).
  - `crates/app/src/talk.rs:13-24` (`ConversationView.character_look`), `:39-48` (punjenje istom funkcijom).
  - `crates/api/src/schema_situations.rs:50-54` (`CharacterBody.look`, `#[schema(required = true)]` kao `character` :22-23),
    novi `CharacterLookBody` i tri enuma uz `SituationKindBody` (:56-80, isti obrazac `From`), `:180-183` (punjenje).
  - **`crates/api/src/schema.rs` je dug na točno 1770 redaka** (`scripts/check-kod.mjs:21`) i ne smije narasti: `SpeakerBody`,
    `ChatLineBody`, `ConversationBody` i njegov `From` (`schema.rs:1154-1205`) sele u `schema_situations.rs` (ili novi
    `schema_talk.rs`), ponovno se izvoze u `schema.rs:16-19` (imena u OpenAPI-ju ista), a `DUG` u `check-kod.mjs:21` spušta se na
    novi broj redaka u istom commitu. `ConversationBody.character_look` dodaje se tek u novom domu.
  - `openapi.json:1614` (`CharacterBody`), `:1644` (`ConversationBody`) i `packages/api-client/src/schema.d.ts:420`, `:428` —
    **samo** kroz `corepack pnpm gen:api`.
- Modify (b):
  - `apps/web/src/avatar/look.ts`: novi tip `LookHint` i zamrznuti bazeni `HINT_*` iza `:179`; nova `lookFromHint`; `lookFor`
    (:355-357) prima drugi argument. Sada 357 r., nakon ~420 (granica upozorenja 600).
  - `apps/web/src/screens/SituationsScreen.tsx:64` → `lookFor(s.character.name, s.character.look)`.
  - `apps/web/src/screens/situation/Conversation.tsx:30` → `lookFor(c.character_name, c.character_look)`.
  - `apps/web/src/test/fixtures.ts:196`, `:216` (`look: null` — Petra i Marko su poznati likovi), `:265` (`character_look: null`).
  - `apps/web/src/avatar/avatar.test.tsx` (novi `describe`, postojeći testovi :32-109 netaknuti).
- Ne dirati: `crates/sim-hotel-v2`, `crates/game` (ni dnevnik ni poglede), `crates/content` (popis imena i provjere), `crates/balance`,
  `prompts/`, `content/` (nijedan JSON), `palette.ts` (nizovi samo dodavanjem; ovdje ništa), `Avatar.tsx`, `KNOWN_LOOKS`,
  `PLAYER_LOOK`, `MENTOR_LOOK`, `SEED_*` (:161-179), `lookFromSeed`, `Leaderboard.tsx:45` i `HotelCard.tsx:26` (ime hotela robota nije
  lik; ostaju na `lookFor(ime)`), `AvatarGallery.tsx`, `docs/` (osim ove kartice i tablice plana), `03-prijava.md` (Leonova izmjena).

**Interfaces:**
- Consumes: `Game.content.situation_templates: Option<content::SituationTemplates>` (`.names`), `content::Names` / `NameList`
  (`guest|staff|partner` × `f|m`), `content::Character { name, role }` (`crates/content/src/lib.rs:66-76`; `role` = opis iz
  `types`, `title` ili „gošća hotela", `templates_instance.rs:58-77`), `Game::ai_entry_for` (`drawn.rs:445`).
- Produces (konvencija API-ja: ključevi engleski `snake_case` kao `character_name`, `department`; vrijednosti enuma `snake_case`
  kao `SituationKindBody`; rod `f`/`m` kao ključevi `names.json` i `content::Gender`; dob = web `AGES` (`look.ts:25`)):
```rust
// app::look
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum LookGender { F, M }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum AgeGroup { Young, Adult, Senior }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Origin { Regional, Foreign }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterLook { pub gender: LookGender, pub age_group: Option<AgeGroup>, pub origin: Origin }
/// Ime u točno jednom rodu popisa imena snimka igre; inače None. Izvor: guest → Foreign, staff/partner → Regional.
/// Dob: ključna riječ u opisu uloge (age_hint), inače staff/partner → Adult, guest → None (web bira iz imena).
pub fn character_look(content: &content::Content, name: &str, role: &str) -> Option<CharacterLook>;
pub fn age_hint(role: &str) -> Option<AgeGroup>;
// riječi = mala slova, sve osim slova razmak; Senior: riječ počinje s „umirovljen" ili je „baka", „djed", „stariji",
// „starija", „starica", „starac"; Young: počinje s „mladić" ili „student", ili je „mlada", „mladi". Senior ima prednost.
// ostarjelom / starim (psom) NE pogađaju (cijele riječi, ne podnizovi).
// app views/talk
pub struct CharacterCardView { pub name: String, pub role: String, pub look: Option<CharacterLook> }
pub struct ConversationView { /* … */ pub character_look: Option<CharacterLook> }
// api
#[derive(Serialize, ToSchema)] pub struct CharacterBody { pub name: String, pub role: String,
    #[schema(required = true)] pub look: Option<CharacterLookBody> }
#[derive(Serialize, ToSchema)] pub struct CharacterLookBody { pub gender: GenderBody,
    #[schema(required = true)] pub age_group: Option<AgeGroupBody>, pub origin: OriginBody }
#[serde(rename_all = "snake_case")] pub enum GenderBody { F, M }            // "f" | "m"
#[serde(rename_all = "snake_case")] pub enum AgeGroupBody { Young, Adult, Senior }
#[serde(rename_all = "snake_case")] pub enum OriginBody { Regional, Foreign }
// ConversationBody: #[schema(required = true)] pub character_look: Option<CharacterLookBody>
```
```ts
// apps/web/src/avatar/look.ts
export type LookHint = { gender: "f" | "m"; age_group: Age | null; origin: "regional" | "foreign" };
export function lookFor(name: string, hint?: LookHint | null): AvatarLook;
// KNOWN_LOOKS[name] uvijek pobjeđuje; bez hinta = lookFromSeed(name) (bajt-isto); s hintom = lookFromHint(name, hint)
export function lookFromHint(name: string, hint: LookHint): AvatarLook;
// = lookFromSeed(name), pa iz NOVOG toka rng(`${name}|izgled`) fiksnim redom: skin, hair, facialHair, earrings, hairColor;
// skin: regional HINT_SKIN_REGIONAL = [0, 1, 6], foreign HINT_SKIN_FOREIGN = [0, 1, 2, 6] (palette.ts:3-12; 3, 4, 5, 7 tek iz banke)
// hair: f HINT_HAIR_F = ["long","ponytail","bun","bob","wavy","curly","side","short"]; m HINT_HAIR_M = ["short","buzz","side","slick","curly","wavy","none"]
// facialHair: f "none"; m r(3) === 0 ? pick(HINT_BEARD_M = ["stubble","mustache","beard","goatee","full"]) : "none"
// earrings: f r(2) === 0 ? pick(["studs","hoops"]) : "none"; m "none"
// age: hint.age_group ?? (iz lookFromSeed: young/adult); senior → hairColor 5 (sijeda), inače hairColor iz lookFromSeed
// Bazeni HINT_* su zamrznuti (samo dodavanje na kraj, kao SEED_*), svako izvlačenje troši rng i kad se ne koristi.
```

**Testovi čuvara:**
IZGLED-a — `crates/app/tests/character_look.rs` (nova; snimak `Snapshot::read(&repo_dir())`; v2 igra kao `start_v2` u
`crates/app/tests/instance_ai.rs:15-25`):
1. `names_are_unique_across_lists`: svako ime iz `names.json` je u točno jednom popisu (izvor × rod). Čuva pretpostavku šava.
2. `look_from_name_lists`: „Anna Müller" → `F`, `Foreign`; „Ivan Kovač" → `M`, `Foreign`; „Ana Jurić" (staff) → `F`,
   `Regional`, `Some(Adult)`; „Damir Tomić" (partner) → `M`, `Regional`, `Some(Adult)`.
3. `unknown_name_has_no_look`: „Petra Horvat" (sidro), prazno ime i `Content` bez predložaka (v1) → `None`.
4. `age_hint_reads_role_words`: „umirovljenica na godišnjem odmoru" → `Senior`; „baka koja je s unucima…" → `Senior`;
   „stariji gost koji teško hoda…" → `Senior`; „mladić koji je došao s prijateljima" → `Young`; „gost koji putuje s ostarjelom
   majkom" → `None`; „gošća koja putuje sama sa starim psom" → `None`; „gošća hotela" → `None`.
5. `situations_view_carries_look`: svaki `SituationItem` s likom iz primjerka (šifra s točkom) ima `character.look = Some`, rod
   jednak popisu imena; sidro S1/S2 `look = None`.
6. `conversation_view_carries_look`: `app::conversation` za isti primjerak daje isti `character_look` kao pogled situacija.
7. API (u postojećoj `crates/api/tests/student_sees.rs` ili novoj `character_look.rs`): JSON `character.look` ima ključeve
   `gender`, `age_group`, `origin` s vrijednostima `"f"|"m"`, `"young"|"adult"|"senior"|null`, `"regional"|"foreign"`.
RED: `app::look` i polja ne postoje (E0432/E0560). Postojeći testovi koji grade `CharacterCardView {…}` literalom dobivaju
`look: None` (čuvar ih nabroji u RED commitu).

IZGLED-b — `apps/web/src/avatar/avatar.test.tsx`, novi `describe("izgled iz podataka lika")`, 300 imena `lik-${i}`:
1. `bez hinta isto kao prije`: `lookFor(ime)` i `lookFor(ime, null)` === `lookFromSeed(ime)`; postojeći zamrznuti test „Ana Kovač"
   (:36-55) ostaje netaknut i zelen.
2. `poznati likovi pobjeđuju hint`: `lookFor("Petra Horvat", {gender: "m", …})` === `KNOWN_LOOKS["Petra Horvat"]`; `PLAYER_LOOK`,
   `MENTOR_LOOK` i `KNOWN_LOOKS` jednaki snimku (`toEqual` objekta prije izmjene).
3. `žensko ime nikad brada`: za sve `f` × dobi × podrijetla `facialHair === "none"`, `hair ∈ HINT_HAIR_F`.
4. `muško ime nikad naušnice ni ženska frizura`: `earrings === "none"`, `hair ∈ HINT_HAIR_M`; brada se pojavi barem jednom.
5. `dob iz hinta`: `age_group` zadan → `age` jednak; `senior` → `hairColor === 5`; `null` → `age ∈ ["young","adult"]`.
6. `podrijetlo bira raspon kože`: `regional` → `skin ∈ [0, 1, 6]`; `foreign` → `skin ∈ [0, 1, 2, 6]`; nikad 3, 4, 5, 7.
7. `isti ulaz uvijek isti izgled` i `izgled iz hinta je uvijek valjan` (`isValidLook`); raznolikost: 10 ženskih imena daju ≥ 4 frizure.
8. `zamrznut izgled iz hinta`: jedan `toEqual` snimak za `lookFor("Anna Müller", {gender: "f", age_group: null, origin: "foreign"})`
   (graditelj upiše vrijednost iz prvog zelenog prolaza; od tada je pravilo kao za `SEED_*`).
9. Ekran: `SituationsScreen`/`Conversation` test s fiksturom `look: {gender: "f", …}` crta avatar bez `data-part^="facial-"`.
RED: `lookFromHint` i `LookHint` ne postoje (greška tipova), testovi 3–9 padaju.

**Koraci:**
- IZGLED-a, čuvar: testovi 1–7 i `look: None` u postojećim literalima; commit RED s popisom grešaka prevođenja.
- IZGLED-a, graditelj: (1) `crates/app/src/look.rs` (`character_look`, `age_hint`); (2) `views.rs`, `talk.rs`; (3) seljenje
  razgovornih tijela iz `schema.rs` (najprije čista selidba, `check:api` mora dati **isti** `openapi.json`), pa `DUG` niže;
  (4) `CharacterLookBody`, enumi, polja; (5) `corepack pnpm gen:api`; (6) fiksture weba `look: null` / `character_look: null`
  (bez njih `typecheck` pada); (7) sve provjere.
- IZGLED-b, čuvar: testovi 1–9 (vitest), commit RED. Graditelj: `LookHint`, `HINT_*`, `lookFromHint`, `lookFor(name, hint?)`,
  dva pozivatelja; provjere; snimak testa 8.
- Pregled (opus): Review Focus niže; Leon na kraju vidi lica u pregledniku (Za profesora → situacije), jer je to proizvodna
  odluka koju student osjeti.

**Provjere** (svaki dio, cijeli popis iz `.github/workflows/ci.yml`; izlazi u `.superpowers/sdd/2026-10-14-popravak-temelja/izgled-provjere/`):
`cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` ·
`node scripts/probe-mutacija.mjs` · `cargo sqlx migrate run --source crates/store/migrations` + `cargo sqlx prepare --workspace
--check` · `cargo test --workspace` · `corepack pnpm check:docs`, `test:docs`, `check:kod`, `check:api`, `lint`, `typecheck`,
`test` · `corepack pnpm --filter @hotelixsim/web build` · `git diff --exit-code` nakon commita · cargo deny samo u CI-ju.
- **Bajt-isto:** `cargo run -q -p balance --release -- --seeds 100 --check` i `--seeds 1000` (v1), `--model v2 --round week
  --seeds 1000 --check` i `--round month`: svaki izlaz `cmp` s osnovicom s a992973 (balans ne ovisi o `app`/`api`/webu, pa je
  razlika znak greške). Pečat: `cargo test -p game --test v2_sealed` i `v1_frozen` zeleni bez izmjene `v2-golden.json`;
  `git diff --stat a992973 -- crates/sim-hotel-v2 crates/game crates/content crates/balance content prompts` prazan.
- `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>` (B12); IZGLED-b ne prevodi Rust, pa ne broji u „dva agenta".

**Rizik i odluke 🟨** (za ADR na `/kraj`):
1. **Rod iz popisa imena**, ne iz predloška: najmanji šav, bez jezgre; ime u dva popisa → `None` (stari izgled). Ako K5 donese
   ponovljena imena, prelazi se na rezervni put (predložak primjerka), opet bez jezgre.
2. **Podrijetlo do K5 iz izvora imena**: osoblje i partneri `regional` (koža 0, 1, 6), gosti `foreign` (0, 1, 2, 6). Gosti s
   hrvatskim imenom (npr. „Ana Horvat", „Ivan Kovač") time su `foreign`, ali raspon je namjerno uzak, pa nikad ne dobiju tamnu
   put. Tamniji tonovi (3, 4, 5, 7) dolaze tek iz banke ljudi sa zemljom (K5); nema popisa „koje je ime strano" u kodu.
3. **Dob**: ključne riječi u opisu uloge (popis gore); osoblje i partneri bez ključne riječi `adult`; gosti bez nje `null` → web
   bira mlad/odrasla iz imena (kao danas). `senior` samo iz opisa (umirovljenica, baka, stariji gost), uz sijedu kosu.
4. **Nazivi u API-ju engleski** (`look`, `gender`, `age_group`, `origin`, `character_look`), kao ostatak API-ja; vrijednosti
   `f`/`m` kao `names.json`.
5. **Stari izgledi bez hinta ostaju bajt-isti** (`lookFor(ime)`): roboti-hoteli na ljestvici i u sezoni nisu likovi s rodom,
   galerija i zamrznuti test „Ana Kovač" ostaju valjani, a stari klijent (ili sidro) ne mijenja lice. Likovi primjeraka **mijenjaju
   lice** jednom (to je i cilj ADR-042 t. 11); nema spremljenih izgleda koje bi to pokvarilo.
6. **Frizure po rodu** (`HINT_HAIR_F/M`) i učestalost brade (1 od 3) i naušnica (1 od 2) su ukusne odluke; Leon ih vidi u
   pregledniku i mijenja se jedan niz.
7. Sidra S1–S4 (Petra, Marko) ostaju ručni `KNOWN_LOOKS`; API im šalje `look: null`.
8. Seljenje razgovornih tijela iz `schema.rs` je preduvjet (dug 1770 ne smije narasti), ne usputno čišćenje.

**Banka ljudi (K5, samo spomen):** kad igra počne vući likove iz banke ([BANKA_LJUDI](../../architecture/BANKA_LJUDI.md) §5.1),
`character_look` čita rod, dob i zemlju iz zapisa banke umjesto iz popisa imena i ključnih riječi; `Origin` tada dobiva
vrijednosti po zemlji, a web bazen kože po tome. Oblik `CharacterLookBody` ostaje; ova cigla to ne gradi.

**Leonov dio:** redoslijed i stil odlučeni (16.10., sesija 7). Pretpostavke 2, 3 i 6 student vidi, ali nemaju pravu alternativu
prije K5; voditelj ih zapisuje kao ADR 🟨 bez ankete. Leon na kraju pogleda 10–20 lica (situacije jednog kruga) i kaže što ne valja.

**Review Focus kandidati:**
1. Ime koje nije u popisu (sidro, budući sadržaj) ili je u dva popisa: `None` i stari izgled, bez pada ekrana.
2. `age_hint` ne pogađa podnizove („ostarjelom", „starim psom") ni opis tuđe osobe; senior ima prednost pred young.
3. Robot-hotel na ljestvici (`Leaderboard.tsx:45`) i sezoni (`HotelCard.tsx:26`) crta se kao prije.
4. `openapi.json` nakon selidbe (korak a3) bajt-isti prije dodavanja polja; `DUG` spušten, `check:kod` bez upozorenja.
5. `lookFromHint` troši rng istim redom bez obzira na granu (inače dodavanje opcije pomiče druga polja).
6. Hrvatsko ime (staff/partner i gost) nikad koža 3, 4, 5 ili 7 — test 6 prolazi kroz sve kombinacije.

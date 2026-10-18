Cigla L1-13 je **jedna** zajednička kalibracija brojki v2 igre na kraju faze B, po Leonovim odlukama od 17.10.
([ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)) i 18.10. (anketa sesije 9, dno ove kartice).
**Puna kartica za izvođače** (čuvar → graditelj → provjeravatelj), po uzoru na [PRAVILO](task-12-pravilo-vlastiti-odgovor.md): ovo je
pregled (odluke, redoslijed, verzija, zajednička sučelja, provjere, rizici), a svaki dio ima svoju karticu (tablica „Kartice dijelova“);
retci su provjereni 18.10. na f81a622 (`model-v2/sloj1`). Pitanja: [Leonov pregled, dio A](../2026-10-17-leonov-pregled/01-pravila-i-brojke.md)
(A1–A13) i [dio B](../2026-10-17-leonov-pregled/02-tekstovi.md) (B2).

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
| 1 · kvar i popravak (A1) | [task-14-1-kvar.md](task-14-1-kvar.md) |
| 1b · klik nosi ~75 % (18.10. t. 1) | [task-14-1b-klik.md](task-14-1b-klik.md) |
| 2 · tri razine značaja (B2) | [task-14-2-znacaj.md](task-14-2-znacaj.md) |
| 3 · situacije s gostima i popunjenost (A7) | [task-14-3-gosti.md](task-14-3-gosti.md) |
| 4 · kalibracija (A2, A3, A4, A8) i 2.6.0 | [task-14-4-kalibracija.md](task-14-4-kalibracija.md) |

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

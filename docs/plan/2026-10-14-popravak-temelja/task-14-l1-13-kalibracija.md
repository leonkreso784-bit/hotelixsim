Cigla L1-13 je **jedna** zajednička kalibracija brojki v2 igre na kraju faze B. Ova kartica zapisuje Leonove odluke od
17.10.2026. (odgovori na 43 pitanja i razgovor; [ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md))
i iz njih izvodi što L1-13 mora napraviti. **Nije još puna kartica za izvođača:** datoteke i retke, testove čuvara i
interfejse voditelj upisuje kad L1-13 krene (po uzoru na [PRAVILO](task-12-pravilo-vlastiti-odgovor.md)), na svježem
stanju grane. Pitanja: [Leonov pregled, dio A](../2026-10-17-leonov-pregled/01-pravila-i-brojke.md) (A1–A13) i
[dio B](../2026-10-17-leonov-pregled/02-tekstovi.md) (B2).

### Task L1-13: Jedna kalibracija brojki (kvar i popravak, značaj situacija, popunjenost, težina ~20 %)

**Veličina:** 🟨 [PROC] veća od 5 h iz [ROADMAP/igra](../ROADMAP/igra.md), jer su 17.10. dodane dvije mehanike (A1, B2) i
jedno pravilo broja (A7); voditelj je dijeli na dijelove 1–4 niže, svaki na svojoj kratkoj grani (ADR-038 t. 6) ·
**Ovisi o:** B4c ✅, PRAVILO-a ✅ (9a275d8), PRAVILO-b (put „klik najbolji" u balansu; težina se mjeri na vlastitom putu i na
kliku) · **Slijedi ga:** L1-13b, L1-14, L1-15 · **Model:** čuvar i graditelj sonnet, provjeravatelj **opus** (jezgra, balans) ·
**Prevodi Rust:** da (sva četiri dijela; nikad dva dijela usporedo, jer svi diraju napredak).

**Uvjet za svaku izmjenu (ne samo na kraju):** `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check`
i isto s `--round month`, prije i poslije, s opisanim pomakom. Tjedna težina hotela B danas je **48,9 % uz granicu 50 %**
(margina 1,1 bod): svaka mehanika koja dodaje učinak situacijama može je prebaciti. Dijelovi 1–3 smiju prolaziti s
pojasom 5–50 %; tek dio 4 sužava pojas na 15–25 % (ADR-042 t. 15, [ADR-049](../../records/DECISIONS/ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md)).

## Leonove odluke 17.10. (ADR-056)

| # | Leon (doslovno, skraćeno) | što to znači za L1-13 |
|---|---|---|
| A1 | *„Hitni popravak ne rješava stanje tog odjela, već samo privremeno popravlja klimu."* Kvar u sobi = **−1 bod**; popravak odmah i gost kasnije zadovoljan = **+2** (*„miču prethodni −1, rezultat +1"*). Anketa: bodovi su na **napretku odjela 0–100** | hitni popravak **ne diže** stanje odjela kao obnova; kvar spušta stanje Smještaja za 1 bod (100 bp), dobro riješen kvar ga diže za 2 (neto +1). **Privremeni +1 iz [ADR-050](../../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md) se ukida** (`economy.repair_urgent_condition_pts` 100 → 0) i zamjenjuje ovim pravilom. „Oko +10" više nije cilj |
| A2 | *„Prepuštam tebi da pravedno odlučiš brojke."* | čestoću kvarova bira voditelj, uz **tablicu mjerenja** (dobro održan prema zapuštenom hotelu, 3–4 inačice) koju Leon vidi prije spajanja; nije anketa |
| A3 | *„20 %"*; anketa: **isti broj situacija, slabiji učinak svake** | težina ~20 % (pojas 15–25 %) postiže se manjim učinkom po situaciji (`situation_effect_bp` u `periods.json`, novac i bodovi odgovora), ne manjim brojem situacija |
| A4 | ok | jednom slabijem odgovoru u predlošcima gdje „samo najbolji troši novac" dati trošak (npr. pretjerana naknada, kao u P-KLIMA) |
| A7 | *„Broj bi trebao ovisiti o popunjenosti… Broj situacija koje nisu vezane za goste ostaje isti."* | broj situacija **vezanih za goste raste s popunjenošću**, ostale su stalne; novo pravilo izvlačenja i **ponovni balans**. Dio *„to sve ne treba biti riješeno od nas, već voditelja"* je po anketi voditelj odjela iz faze LJUDI (L2a-5) i u L1-13 se ne gradi |
| A8 | *„Da, slažem se, super."* | 16 predložaka koji danas krše „šutnja nije bolja" (`SilenceNotBetter`, dug P4) popraviti; dug pada na 0 |
| B2 | *„Tablica situacija (značajne, polu značajne, ne toliko značajne), bolje imenovati… Značajnije situacije i pametnije odluke nose više bodova."* | **tri razine značaja**, imenovane; bodovi po razini; student vidi razinu |

**Tumačenje voditelja (🟨, provjeriti s Leonom uz tablicu prije spajanja):**
1. **„Zadovoljan gost" = ocjena odgovora ≥ 4** u situaciji kvara s hitnim popravkom. Ocjena 1–3 uz hitni popravak ostavlja −1
   (kvar je riješen, gost nije). To mijenja [ADR-046](../../records/DECISIONS/ADR-046-leonove-odluke-15-10-za-fazu-b.md) t. 1
   (popravak diže odjel „i uz ocjenu 1"); ADR-056 ga nadjačava.
2. **„Kvar u sobi"** = situacija vrste kvar čiji je prostor Smještaj (odjel `rooms` ili `condition_of: "rooms"`, B1b), ne svaki
   kvar u hotelu. Kvarovi drugih prostora (praonica, dizalo) slijede isto pravilo na svom prostoru 🟨.
3. **−1 dolazi kad se kvar pojavi**, ne kad ga student loše riješi; šutnja ostavlja −1 i posljedice „nitko nije reagirao" (A8).
4. Pravi lijek za stanje sobe ostaje održavanje i obnova; **sučelje soba** (svaka soba zasebno vidljiva i obnovljiva, Leon u A1)
   nova je mehanika u fazi C ili kasnije ([ROADMAP/igra](../ROADMAP/igra.md)), **ne** u L1-13.

## Dijelovi i redoslijed

| dio | što | dira (provjereno 17.10. na 847d165) | Leonov dio |
|---|---|---|---|
| 1 · kvar i popravak (A1) | kvar Smještaja −100 bp pri izvlačenju; hitni popravak uz ocjenu ≥ 4 +200 bp; `repair_urgent_condition_pts` 0 (polje ostaje radi starih snimaka) | `crates/sim-hotel-v2/src/resolve/situations.rs` (`situation_effect`, :25-49), `crates/sim-hotel-v2/src/catalog.rs` (`economy` :48, provjera :392), `content/hotel/v2/economy.json` | tablica s tri ishoda (šutnja, popravak uz 3, popravak uz 5) |
| 2 · značaj situacija (B2) | tri razine i bodovi po razini; razina vidljiva studentu | `crates/content/src/templates.rs` (`weight` 1–4 :78-80, `major` :83-85), `content/hotel/v2/situation_templates.json`, web ekran situacija | imena triju razina (prijedlog niže) |
| 3 · popunjenost (A7) | broj situacija vezanih za goste raste s popunjenošću, ostale stalne; prve 3 i dalje obvezne | `crates/sim-hotel-v2/src/situations/count.rs` i `draw.rs`, postavka broja iz B2a (`GameCreated.situation_counts`) | raspon po krugu (npr. lipanj 3, kolovoz 6), u tablici |
| 4 · kalibracija (A2, A3, A4, A8) | težina 15–25 % tjedno i mjesečno za A/B/C; čestoća kvarova; trošak slabijem odgovoru; `SilenceNotBetter` 16 → 0; brisanje `ORDER_DEBT_FLOOR`, testa `order_debt` i grane `clean(Month)` | `content/hotel/v2/periods.json` (`situation_effect_bp`), `situation_templates.json` (`chance`, odgovori), `crates/balance/src/v2.rs` (`SHARE_BAND_BP`), `crates/balance/src/v2_situations.rs`, `crates/balance/tests/answer_gates_v2.rs`, `crates/content/src/templates_principles_debt.rs` | tablica mjerenja prije spajanja (A2: voditelj bira, Leon vidi) |

Redoslijed 1 → 2 → 3 → 4: mehanike prvo, **jedna** kalibracija na kraju (inače se brojke štimaju triput). `CORE_VERSION` se
diže **jednom** za cijeli L1-13 (🟨 2.5.0 → 2.6.0), na zadnjem spajanju; hoće li dijelovi 1–3 do tada ići iza postavke ili se
spojiti zajedno s dijelom 4, voditelj odlučuje kad piše punu karticu. Probne v2 igre nakon toga daju 409 (Leonu se kaže).

### Dio 2: tri razine značaja (B2)

**Nalaz voditelja 17.10.:** predlošci **nemaju** polje ozbiljnosti `major`/`minor`, kako je pisalo u razgovoru. Imaju dvije
druge stvari: `weight` 1–4 (važnost: redoslijed na ekranu i prednost u izvlačenju; danas 2 × 1, 27 × 2, 17 × 3, 9 × 4) i
`major: high|low` (5 predložaka: „veća situacija" za izvrstan ili vrlo loš hotel). Student ni jedno ni drugo ne vidi kao razinu.

**Prijedlog 🟨:** razina se **izvodi iz `weight`**, bez novog polja: 4 → **ključna**, 3 → **važna**, 1–2 → **svakodnevna**
(danas 9 / 17 / 29 predložaka). Imena su prijedlog za Leonovo „bolje imenovati". Bodovi po razini: učinak situacije množi se
faktorom razine (npr. 1,5 / 1,0 / 0,6 [PROC]), pa značajnija situacija i pametnija odluka nose više; točne faktore daje
kalibracija (dio 4). `major` ostaje kako jest (uvjet izvlačenja, ne razina). Student vidi razinu na kartici situacije
(oznaka uz naslov; boja nikad sama, [UI_PRAVILA](../../product/UI_PRAVILA.md)).
**Inače:** novo polje `significance` u predlošku (čisto, ali 55 predložaka + provjera + prepis; ~2 h više [PROC]).

### Dio 3: popunjenost (A7)

**Prijedlog 🟨:** situacija je „vezana za goste" ako ima ulogu `"from": "guest"` ili je u skupini `*guest_facing` (koliko ih je,
provjeriti na sadržaju). Broj gostinjskih u krugu = osnova + korak po popunjenosti prošlog kruga [PROC], ostale stalne.
Ukupan broj ostaje u granicama postavke igre (B2a: tjedno 3–5, mjesečno 4/5/6) dok Leon ne kaže drukčije; više od gornje
granice je odluka o igri (anketa). Rizik: srpanj i kolovoz teži, lipanj i rujan prazniji (A7 „Inače" u pregledu).

## Testovi čuvara (obris; punu listu piše čuvar iz pune kartice)

- kvar Smještaja: izvlačenje spusti stanje za točno 100 bp; hitni popravak uz ocjenu 5 vrati +200 (neto +100); uz ocjenu 3
  ostane −100; šutnja ostane −100 i nikad nije bolja od najlošijeg odgovora;
- `repair_urgent_condition_pts` 0 u sadržaju; stari snimak bez ključa igra isto kao prije (pečat A3, `v2_sealed`);
- razina: svaki predložak ima točno jednu od tri razine; ekran pokazuje naziv razine;
- popunjenost: veća popunjenost nikad ne daje manje gostinjskih situacija; negostinjskih jednako u svakom krugu;
- dug: `SilenceNotBetter` 0, `ORDER_DEBT_FLOOR` i `order_debt` obrisani;
- balans: težina 15–25 % za A, B i C, tjedno i mjesečno, na vlastitom putu; klik najbolji ispod vlastitog (ADR-047 t. 3).

## Provjere

Cijeli CI popis (fmt, clippy, core-deps, testovi, tri balansa, check:docs, test:docs, check:kod, check:api, lint, typecheck,
test, web build), ne samo kartične. Tri balansa 1000 `--check` **nakon svakog dijela**; tablica prije/poslije za Leona.

## Rizik i odluke

- 🟨 „Zadovoljan gost" = ocjena ≥ 4 (tumačenje 1). Ako Leon misli na kasnije zadovoljstvo gosta (recenzija, indeks), to čeka
  recenzije (R4) i ne stane u L1-13.
- 🟨 −1 na svakom kvaru i +2 samo uz dobar odgovor mogu **spustiti** napredak lošeg studenta više nego danas; tablica to
  pokazuje po profilu (A/B/C), jer zapušten hotel ima više kvarova.
- 🟨 Razine iz `weight` vežu važnost na ekranu i značaj bodova u jedno polje; ako ih Leon želi razdvojiti, dolazi polje
  `significance` (Inače u dijelu 2).
- B1a je izmjerila +200 po svakom popravku kao preveliko (tjedno B 57,6 %); novo pravilo daje +200 samo uz ocjenu ≥ 4, a −100
  svakom kvaru, pa se učinak mjeri, ne pretpostavlja.

**Leonov dio:** tablica mjerenja prije spajanja (A2: brojke bira voditelj „pravedno", Leon vidi); imena triju razina (B2);
potvrda tumačenja 1 („zadovoljan gost").

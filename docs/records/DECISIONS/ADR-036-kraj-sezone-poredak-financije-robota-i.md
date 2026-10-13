# ADR-036 — Kraj sezone: poredak, financije robota i sezonski mentor
**Datum:** 2026-10-13 · **Status:** 🟨 **ČEKA LEONOV PREGLED**, osim Leonovih odluka ✅ 12. i 13.10.
(t. 1, t. 2) · **Vezano:** [ADR-024](ADR-024-modeli-presuda-na-sonnetu-5-5.md),
[ADR-023](ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md), ARHITEKTURA §9.4 i §12,
[MODEL_V2.md](../../architecture/MODEL_V2.md) §6.3, §6.5

**Kontekst.** Leon 12.10.: *„na kraju se moraju vidjeti rezultati s botovima, kako je tko odigrao"*. Do sada je na
kraju pisalo samo „Sezona je završena.", a ljestvica (Tvoj hotel + 3 robota) bila je na ekranu svakog kruga. KS-1 je
dodao API kraja sezone, KS-2 sezonskog mentora; ekran kraja sezone (KS-3) je na zasebnoj grani.

**Odluka.**
1. ✅ **Rezultati robota nakon zadnjeg kruga** (Leon 13.10.): `GET /v1/games/{id}/season` daje poredak, krivulju
   napretka, financije sezone i odluke robota s brojkama, ali tek kad su obračunati svi krugovi (409
   `season_not_over` prije, 409 `model_v1` za v1). Tuđi **ljudski** hotel i dalje daje samo indekse i krivulju
   (`finances: null`, `decisions: null`). `GameBody`/`PublicHotelBody` nepromijenjeni.
2. ✅ **Pobjednik** je prvi po napretku (Leon 13.10.): `winner` po hotelu = `rank == 1`; kod dijeljenog prvog mjesta
   ima ih više, a rečenicu bira web.
3. 🟨 **Mjesto:** jednak konačni napredak dijeli mjesto (1, 2, 2, 4), unutar istog mjesta redom slota.
4. 🟨 **Odluke sezone:** `maintenance_deferred_rounds` broji krug u kojem je održavanje **barem jednog** odjela
   odgođeno; `unanswered` broji situacije bez ikakvog odgovora; `seasonal_peak` = najviše sezonaca na kraju kruga;
   `avg_price_cents` = običan prosjek po krugovima (`div_round`), ne ponderiran danima. Prihod i GOP sezone računaju
   se istim zbrojem kao `/pl?scope=season`, bez budžeta.
5. **Sezonski mentor** (dopuna [ADR-024](ADR-024-modeli-presuda-na-sonnetu-5-5.md)): uputa
   `prompts/mentor-sezona.v1.md` (verzija u `AiMeta.prompt`), model kao mentor kruga (`models.mentor` = Sonnet,
   `HOTELIXSIM_AI_MODEL_MENTOR`); jedan poziv po ljudskom hotelu po igri. Događaj `SeasonDebriefRecorded` (jedan po
   hotelu, samo v2, tek nakon zadnjeg kruga; nije u feedu) i posao `season_debrief` iza mentora kruga. Bez ključa →
   odmah rezerva (`source: fallback`); ostale greške AI-ja do zadnjeg pokušaja, pa rezerva.
6. 🟨 **`SeasonBody.mentor` ostaje `{ text }`**: „u izradi" je `null` (web ispituje), a rezerva se prepoznaje po
   završnoj rečenici kao kod mentora kruga; `openapi.json` se zato ne mijenja.
7. 🟨 **Činjenice za sezonskog mentora** (`crates/game/src/season_mentor.rs`): napredak i mjesto u cijelim bodovima
   (kao ekran), s desetinkama samo kad bi hoteli s različitim napretkom zaokruženo izgledali jednako (+8,4 i +7,6);
   najbolji i najslabiji krug u desetinkama boda, samo kad je pomak bar desetinka; odluke riječima („obuka u 6
   tjedana", „nijednom odgođeno"); prosječna cijena uz uobičajenu. Robotima se šalju pravilo, napredak, mjesto, indeksi
   i odluke, **ne financije** („napredak, ne zarada"). Preuzeto stanje se ne pripisuje studentu (isto pravilo kao
   `mentor.v3`, [ADR-023](ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md) dopuna).
8. 🟨 **Rezerva bez AI-ja:** četiri dijela („Kako je prošla sezona / Što je išlo dobro / Što nije valjalo / Što ponijeti
   u sljedeću igru") + „AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki."; usporedba s najboljim
   robotom ispred studenta samo gdje je odmakao; bez roda prema studentu („Tvoj hotel je pobijedio").
9. 🟨 **Nepotpun AI tekst** (prazan ili bez sva četiri podnaslova) ne sprema se kao `ai`: radnik odmah piše rezervu,
   bez ponovnog poziva (jedan poziv po igri). Mentor kruga tu provjeru još nema ([BACKLOG](../BACKLOG.md) `[sloj1]`).

**Cijena ako krivo:** t. 1–4 su `crates/game/src/season.rs` i jedna ruta; t. 5–9 su `season_mentor.rs`, uputa i
radnik. Živi test `season_mentor_writes_four_parts` (`cargo test -p ai --test live -- --ignored`) još nije pokrenut.

# PLATFORMA §5 · Igrini slojevi koje je Leon naveo: gdje sjedaju

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). Ovo nije dizajn igre, samo mjesto u arhitekturi. Svaki sloj ide kroz svoj spec,
balans i Leonove ankete o brojkama. Raspored rada je u kolosijeku IGRA ([ROADMAP/igra](../../plan/ROADMAP/igra.md)).

| sloj | gdje u sustavu | što traži od jezgre | ovisi o | veličina [PROC] |
|---|---|---|---|---|
| **Dobavljači** | katalog dobavljača u `content/` (cijena, kvaliteta, pouzdanost, rok); izbor je **nalog u planu** ([MODEL_V2](../MODEL_V2.md) §6.3) | izravni trošak po upotrebi i zadovoljstvo odjela ovise o dobavljaču; kašnjenje ima vlastiti tok kocke i može pokrenuti situaciju iz predloška | odjeli (sloj 1), predlošci situacija | 15–25 h |
| **Pregled rezervacija** | novi ekran i ruta `GET /v1/hotels/{id}/reservations?period=` | vidi 5.1 | vrste soba, agencije | 25–40 h |
| **Generator gostiju** (ime, zemlja, izgled) | jezgra v2, kozmetički modul: **izvodi se** iz sjemena + krug + redni broj i **ne sprema se** (MODEL_V2 §2 t. 1) | ništa: brojke računa formula, a kocka bira samo tko je gost (MODEL_V2 §2 t. 4) | rezervacije, recenzije | 10–20 h + banka imena |
| **Vrste soba** | opis hotela (tlocrt, tipovi), plan (osnovna cijena + doplata po tipu, ADR-028 t. 10) | raspored prodanih noći po tipu (MODEL_V2 sloj 3) | sloj 1 | prema dizajnu u radu |
| **Pošta s agencijama** | „pošta" je **pogled** na situacije vrste „ponuda" iz predložaka + feed | odgovor je izbor (bez AI-ja, ADR-028 t. 8); prihvaćen ugovor (allotment) puni knjigu rezervacija po nižoj cijeni | rezervacije, predlošci | 15–25 h |

U razrednoj igri svaki od ovih slojeva radi **po hotelu**, a studenti vide jedni drugima samo ono što dopušta
[ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) t. 4 (radnici da, troškovi i
odgovori na situacije ne). Dobavljači i ugovori s agencijama su privatni plan hotela.

## 5.1 Trebaju li rezervacije kao stvari?

Danas jezgra računa **prodane noći kao zbroj** po krugu (`market.rs`). MODEL_V2 §7.3 izričito ne radi „simulaciju po
danu i rezervacije po gostu". Dvije mogućnosti:

| | **A. Knjiga rezervacija kao zbroj** (preporuka) | **B. Svaka rezervacija je stvar** |
|---|---|---|
| model | „Na knjigama" (OTB): za svaki budući krug broj rezerviranih noći po tipu sobe, kanalu i segmentu. Rezervacije stižu unaprijed (krivulja ranog bukiranja), a otkazivanja su postotak | ~200 rezervacija tjedno po hotelu [PROC], ~14 000 po igri, s danima dolaska i odlaska |
| što student vidi | „za idući tjedan rezervirano 68 %, prošli tjedan +9 % (pickup)" + **uzorak kartica** rezervacija (ime, zemlja, tip sobe, noći) izveden iz sjemena | isto, ali svaka je stvarna |
| učenje | revenue management: tempo bukiranja, pickup, rani popust, allotment | isto, uz puno više sitnog posla |
| trošak | jezgra dobiva `na_knjigama` u stanju i fazu „bukiranje" prije tržišta; deterministično i zbrojivo | simulacija po danu, veliko stanje i dnevnik, novi balans; MODEL_V2 to izričito odbija |

Preporuka A. Kartice „pojedinačnih" rezervacija i gosti su kozmetički izvedeni tako da se **zbrajaju točno na broj iz
formule** (šum s nultim zbrojem, MODEL_V2 §2 t. 4). Ako netko jednom zatreba B, A je njezin zbroj, pa A nije slijepa ulica.

## 5.2 Izgled gostiju i zaposlenika (proceduralni avatari)

- **Danas** se izgled računa u TypeScriptu iz teksta (`lookFromSeed`, `apps/web/src/avatar/look.ts:181`). Izgled je
  „podatak" (`AvatarLook`, `look.ts:43-60`), što je dobar temelj.
- **Preporuka za ljude u igri** (gosti, zaposlenici, kandidati): **Rust izvede `look` kao podatak u odgovoru API-ja**
  iz stabilnog ID-a i sjemena igre, a TypeScript samo crta. Jedan izvor istine vrijedi i za budući Expo
  ([ARHITEKTURA §13](../ARHITEKTURA/13-sucelje-isporuka-sigurnost.md)) i za profesorov ekran, i za prikaz **radnika kolega**
  (ADR-037 t. 4). Ništa se ne sprema. Za ručno napisane likove ostaje `lookFromSeed`.
- **ADR-028 vrijedi i za izgled:** nacionalnost određuje jezik i zemlju, **nikad izgled** (boju kože, odjeću) ni
  ponašanje. Test je isti kao zamjena podrijetla (MODEL_V2 §2 t. 9): zamjena zemlje dvama gostima mijenja samo zemlju i jezik.
- **Otvoreno:** `AvatarLook.age` (`look.ts:24`) i odluka „dob zaposlenika ne radimo" (MODEL_V2 §7.3). Kozmetička dob
  ne smije ulaziti ni u jednu formulu ni u tekst o vrijednosti osobe.
- Mješavina zemalja gostiju dolazi iz `docs/research/JEZICI_GOSTIJU.md` (ADR-028, odgovor 8).

## 5.3 Natjecanje za ljude: gdje sjeda u platformi

Ponude radnicima među studentima ([ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) t. 5)
su pravilo jezgre (kolosijek IGRA, faza NATJECANJE ZA LJUDE). Platforma dodaje samo tri stvari: **obavijest u aplikaciji**
vlasniku hotela B (zvonce, RZ-3), **postavku razreda** kojom profesor isključuje preotimanje, i **granicu** od 1–2 ponude
po krugu. Odluku radnika računa formula s kockom iz vlastitog toka, a AI ne zapisuje brojke.

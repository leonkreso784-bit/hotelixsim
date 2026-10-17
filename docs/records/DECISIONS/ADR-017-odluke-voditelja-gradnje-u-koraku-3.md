# ADR-017 — Odluke voditelja gradnje u koraku 3 (red poslova, greške, putanje, javne funkcije)
**Datum:** 2026-10-05 · **Status:** ✅ **Potvrdio Leon 17.10.** (F1; pretpostavke Claudea, „radi bez pitanja") · **Vezano:** [ADR-015](ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md), [ADR-016](ADR-016-leonove-proizvodne-odluke-za-korak-3.md)

**Kontekst.** Tijekom gradnje koraka 3 voditelj je presudio više tehničkih pitanja umjesto Leona. Svako je malo i
mijenja se lokalno; zapisano da se ne izgube.
**Odluka (pretpostavke):**
- **Red poslova nosi identitet preuzimanja** (`JobClaim{id, attempts}`): `fail` i `finish` djeluju samo ako se poklapaju
  i posao je `running`; istek zakupa na zadnjem pokušaju → `failed`. Razlog: zakašnjeli radnik ne smije pokvariti posao
  koji je već preuzeo drugi. Cijena ako je krivo: jedan dodatni parametar u `jobs.rs`.
- **Nepoznat profil ili strategija u API-ju = zabilježen 500**, ne tihi zamjenski izbor. Razlog: tihi izbor skriva
  pokvaren dnevnik. Cijena: pokvarena igra se vidi odmah (što je i cilj).
- **Provjere pristupa idu prije provjere tijela** (401 → 404 → 403 prije 422). Razlog: neprijavljen ne smije saznati
  oblik tijela ni postojanje igre. Cijena: nijedna.
- **Odgovor na situaciju nema mjesec u putanji**, jer su šifre situacija jedinstvene po igri. Ako sadržaj ikad ponovi
  šifru kroz mjesece, mjesec mora ući u putanju (nova verzija rute). Cijena ako je krivo: promjena rute i klijenta.
- **`sim_hotel::{euro_bounds, seasonal_range}` postaju javne** da ih `game` koristi umjesto kopije. Cijena: veće javno
  sučelje jezgre.
- **`crates/game` dobiva istu zabranu floatova, sata i `HashMap` kao jezgra** (`clippy.toml` + `check-core-deps`).
  Razlog: obračun runde mora ostati ponovljiv. Cijena: stroža pravila pri pisanju.
- **Pravilo oblika snimka** (nova polja content DTO-a samo uz `#[serde(default)]`; ispravak otiska nad kanonskim
  `Value`) dokumentirano u [ADR-015](ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 3, a kodni ispravak je dug za korak 4 (BACKLOG `[korak 4]`).

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): F1: sve točke vrijede kako su zapisane.

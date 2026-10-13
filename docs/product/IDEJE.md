# HotelixSim — ideje (nakon profesorova pregleda)

> Leon, 13.10.: *„ovo su samo ideje, sve to ide naknadno — nakon profesorova pregleda"*.
> Ovdje se ideje samo skupljaju; ništa se ne gradi dok Leon ne odluči redoslijed (prijedlog: pilot s jednim razredom,
> pa redoslijed koji pokaže pilot). Ovo nije plan: kad ideja dobije odluku, ide u plan ili u
> [BACKLOG](../records/BACKLOG.md). Dodaci puta B koji su već razrađeni: [PROSIRENJA.md](./PROSIRENJA.md),
> [MODEL_V2.md](../architecture/MODEL_V2.md).
> Stupac „Gdje je razrađeno" upućuje na radne bilješke sesije u `.superpowers/sdd/2026-10-10-v2-sloj1/` (lokalno,
> nisu u gitu); kad bilješka postane odluka, prenosi se u `docs/`.

## Igra

| # | Ideja (Leonovim riječima) | Stanje | Gdje je razrađeno |
|---|---|---|---|
| 1 | **Vrste soba** — King Regular, Twin Regular, King Deluxe, Twin Deluxe, Suite, Family, Connecting…; brojevi soba | dizajn gotov; odluke: cijena po vrsti, brojevi za prikaz, stanje po vrsti | `vrste-soba-agencije-dizajn.md` |
| 2 | **Jedno mjesto za sve cijene** — sobe i svi odjeli zajedno (ekran „Cijene") | dizajn gotov | isto |
| 3 | **Cijena parkinga** i **koliko se jednokratno ulaže u obnovu** za svaki odjel | u gradnji (obnova v2, zatim cijena parkinga) na grani `model-v2/sloj1-obnova` | `obnova-v2-spec.md` |
| 4 | **Pošta s turističkim agencijama** — upiti za grupe, sezonski ugovori (allotment), naše ponude, reklamacije; AI agencije | dizajn gotov | `vrste-soba-agencije-dizajn.md` |
| 5 | **Događaji** — firme javljaju evente u hotelu, vjenčanja | dizajn u izradi | isto, poglavlje „Događaji" |
| 6 | **Izbor dobavljača** | ideja | `platforma-arhitektura.md` §5 (samo gdje sjeda) |
| 7 | **Pregled rezervacija** — koliko ih je | ideja; preporuka: knjiga rezervacija kao zbroj + kartice-uzorci | isto §5.1 |
| 8 | **Generator gostiju** — nasumična imena i likovi po izgledu koji se sami stvaraju | ideja | isto §5.2 |
| 9 | **Novine** — ne pišu o hotelu, nego objavljuju **stanje u državi (makroekonomski uvjeti)**: gospodarstvo, inflacija, cijene energije, turistička potražnja, tečaj; student reagira na te uvjete | ideja | — |
| 10 | **Generirani hoteli** — različit broj soba, imena hotela | ideja | [PLATFORMA.md](../architecture/PLATFORMA.md) §4 |

## Platforma

| # | Ideja | Stanje | Gdje |
|---|---|---|---|
| 11 | **Prijava** (različiti načini; kod igre; kasnije AAI@EduHr) | arhitektura napisana, čeka Leonove odluke L1–L10 | [PLATFORMA.md](../architecture/PLATFORMA.md) §3 |
| 12 | **Profesor** se prijavi, cijeli CRUD, prati sve studente | isto | [PLATFORMA.md](../architecture/PLATFORMA.md) §2 |
| 13 | **Produkcija** — server na internetu, vlastita domena (`hotelixsim.com`, Leon kupuje), Supabase (postojeća organizacija) + Railway | isto; trošak ~5–10 €/mj do financiranja | [PLATFORMA.md](../architecture/PLATFORMA.md) §6 |

## Prezentacija za profesora

Što je napravljeno + plan nadogradnje (ideje iz ove datoteke), platforma za razred, pitanja za fakultet, troškovi.

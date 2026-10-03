# HotelixSim — plan

**Status:** 🟩 AKTIVAN · **Otvoren:** 2026-10-03 · **Usklađen:** 2026-10-04 (arhitektura v2, DEMO.md)
Što gradimo: [VIZIJA.md](./VIZIJA.md) · kako: [ARHITEKTURA.md](./ARHITEKTURA.md) · demo: [DEMO.md](./DEMO.md)

> Faza ima ime, ne slovo. Svaka faza ima uvjet „gotovo kad…" izražen kroz ono što **čovjek može napraviti**,
> nikad kroz „test je zelen". Stari plan (Python prototip do 10.10.) je napušten 04.10.: demo se gradi na pravoj
> arhitekturi i ne baca se.

## Faze

| faza | gotovo kad… |
|---|---|
| **TEMELJI** | …arhitektura i dizajn igre su zapisani i Leon ih je pregledao. Arhitektura v2 ✅ (04.10.) · DEMO.md nacrt ✅ (04.10.), čeka Leonovu provjeru domene |
| **DEMO** (lokalno) | …profesor na Leonovom računalu otvori poveznicu, dobije hotel, odigra lipanj i srpanj (situacije, razgovor s likom, slobodan odgovor s AI ocjenom, izbornik radnji), zatvori mjesec i vidi napredak uz tri robota, te u profesorovom pogledu vidi **zašto** je hotel tu gdje jest ([DEMO.md](./DEMO.md) §1.5) |
| **PREZENTACIJA** | …stoji prezentacija (vizija i osi situacija · fer start i relativna ocjena · živi demo · što tražimo od fakulteta) i profesori su dobili [pitanja](./PITANJA_ZA_PROFESORE.md) |
| **PILOT** | …poslije profesorovog „da": grupa igra sezonu po kalendaru na Railwayu, s pravom prijavom, a profesor vidi relativni napredak svakog hotela |
| **VIZIJA** | međusobni utjecaj, profesorova ploča, više hotela ([VIZIJA.md](./VIZIJA.md) §8) |

## DEMO — redoslijed gradnje

Po ARHITEKTURA §20. Svaki korak ima pregled koda prije idućeg. **Sve lokalno; Railway tek u PILOTU.**

1. Kostur od kraja do kraja: workspace, CI, prazna jezgra → API → lokalni Postgres → jedan ekran.
2. Jezgra + alat za balans: stvarni sloj, indeksi, mehanike, obračun, kocka. Provjere iz DEMO §11 (točke 1–6).
3. Dnevnik, runde (demo način: gumb) i red poslova.
4. AI sloj: presuda, likovi, mentor (situacije iz predložaka); provjera DEMO §11, točka 7.
5. Sučelje igrača (mobitel prvi) i gost preko poveznice.
6. Profesorov pogled (trag promjena, presude s kriterijima).

## Otvoreno za Leona

1. **AI pružatelj za demo:** koje besplatne ključeve imaš. Do odgovora se gradi uz lažnog pružatelja iza porta
   (ARHITEKTURA §10.1), pa ništa ne čeka.
2. **Provjera domene u DEMO.md:** šest najnesigurnijih pretpostavki na kraju dokumenta.
3. **Profesori:** pregled [PITANJA_ZA_PROFESORE.md](./PITANJA_ZA_PROFESORE.md) prije slanja.
4. **Prijava u inkubator** (rok 16.11.) i prezentacija dekanu: nosi li se Sokrat ili simulacija.

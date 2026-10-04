# HotelixSim — roadmap

**Status:** 🟩 živ (faze projekta; nije plan taskova) · **Otvoren:** 2026-10-03
Što gradimo: [VIZIJA.md](../product/VIZIJA.md) · kako: [ARHITEKTURA.md](../architecture/ARHITEKTURA.md) · demo: [DEMO.md](../product/DEMO.md)

> Faza ima ime, ne slovo. Svaka faza ima uvjet „gotovo kad…" izražen kroz ono što **čovjek može napraviti**,
> nikad kroz „test je zelen". Stari plan (Python prototip do 10.10.) je napušten 04.10.: demo se gradi na pravoj
> arhitekturi i ne baca se.

## Faze

| faza | gotovo kad… |
|---|---|
| **TEMELJI** | …arhitektura i dizajn igre su zapisani i Leon ih je pregledao. Arhitektura v2 ✅ (04.10.) · DEMO.md nacrt ✅ (04.10.), čeka Leonovu provjeru domene |
| **DEMO** (lokalno) | …profesor na Leonovom računalu otvori poveznicu, dobije hotel, odigra lipanj i srpanj (situacije, razgovor s likom, slobodan odgovor s AI ocjenom, izbornik radnji), zatvori mjesec i vidi napredak uz tri robota, te u profesorovom pogledu vidi **zašto** je hotel tu gdje jest ([DEMO.md](../product/DEMO.md) §1.5) |
| **PREZENTACIJA** | …stoji prezentacija (vizija i osi situacija · fer start i relativna ocjena · živi demo · što tražimo od fakulteta) i profesori su dobili [pitanja](../product/PITANJA_ZA_PROFESORE.md) |
| **PILOT** | …poslije profesorovog „da": grupa igra sezonu po kalendaru na Railwayu, s pravom prijavom, a profesor vidi relativni napredak svakog hotela |
| **VIZIJA** | međusobni utjecaj, profesorova ploča, više hotela ([VIZIJA.md](../product/VIZIJA.md) §8) |

## DEMO — redoslijed gradnje

Po ARHITEKTURA §20. Svaki korak ima pregled koda prije idućeg. **Sve lokalno; Railway tek u PILOTU.**

1. ✅ Kostur od kraja do kraja: workspace, CI, prazna jezgra → API → lokalni Postgres → jedan ekran
   ([CHANGELOG 0.1.0](../records/CHANGELOG.md), plan u [archive/](../archive/2026-10-04-kostur.md)).
2. Jezgra + alat za balans: stvarni sloj, indeksi, mehanike, obračun, kocka. Provjere iz DEMO §11 (točke 1–6).
3. Dnevnik, runde (demo način: gumb) i red poslova.
4. AI sloj: presuda, likovi, mentor (situacije iz predložaka); provjera DEMO §11, točka 7.
5. Sučelje igrača (mobitel prvi) i gost preko poveznice.
6. Profesorov pogled (trag promjena, presude s kriterijima).

## Gdje smo

**TEMELJI** su napisani, ali arhitektura v2 i domena u DEMO.md još čekaju Leonovu potvrdu (ADR-003, BACKLOG).
**DEMO:** korak 1 ✅; korak 2 (jezgra i alat za balans) je izgrađen na grani `gradnja/jezgra` po planu [2026-10-04-jezgra.md](2026-10-04-jezgra.md)
i čeka završni pregled i spajanje; sljedeći je korak 3 (dnevnik, runde, red poslova). Rokovi: demo ciljano **~14.10.**, prvo profesorovih 15 minuta (DEMO §1.5) —
[ADR-012](../records/DECISIONS.md#adr-012); pitanja profesorima idu tijekom tjedna 5.–11.10.; prijava u inkubator 16.11.

Otvorena pitanja za Leona žive na jednom mjestu: [BACKLOG.md](../records/BACKLOG.md) → „Otvoreno za Leona".

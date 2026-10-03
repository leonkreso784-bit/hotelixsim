# HotelixSim — plan

**Status:** 🟩 AKTIVAN · **Otvoren:** 2026-10-03 · Što gradimo: [VIZIJA.md](./VIZIJA.md)

> Faza ima ime, ne slovo. Svaka faza ima uvjet „gotovo kad…" izražen kroz ono što **čovjek može napraviti**,
> nikad kroz „test je zelen". Arhitektura se bira na početku faze PROTOTIP, ne prije.

## Faze

| faza | rok | gotovo kad… |
|---|---|---|
| **PROTOTIP** | idući tjedan (~10.10.2026) | …profesor na mobitelu dobije hotel, vidi pet faktora, riješi 3–4 situacije s ponuđenim odgovorima, **jednu napiše svojim riječima i dobije pravu AI ocjenu s objašnjenjem**, pa se vidi na ljestvici s tri lažna igrača |
| **PREZENTACIJA** | ~mjesec dana (početak studenog) | …stoji prezentacija s četiri teme (vizija i osi situacija · fer start i relativna ocjena · živi demo · što tražimo od fakulteta) i bolji igriv demo: tri fer hotela A/B/C, izbornik radnji, slobodan potez, lik u situaciji, relativna ocjena |
| **PILOT** | poslije profesorovog „da" | …grupa studenata igra istu sezonu na zajedničkom satu, a profesor vidi relativni napredak svakog hotela |
| **VIZIJA** | — | međusobni utjecaj, profesorova ploča, više hotela ([VIZIJA.md](./VIZIJA.md) §8) |

## PROTOTIP — što je unutra, a što namjerno nije

- **Unutra:** stranica koja izgleda kao igra · hotel s pet faktora · situacije unaprijed napisane (po osima) ·
  izbor odgovora → posljedica na faktore · izbornik radnji · ljestvica s tri lažna igrača i relativnom ocjenom ·
  **jedan pravi AI trenutak** (slobodan odgovor → AI ocjena + objašnjenje).
- **Namjerno nije:** prijava, baza, zajednički sat, AI koji smišlja situacije, likovi, međusobni utjecaj.
- **Jedino tehničko pravilo već sad:** AI ključ živi samo na poslužitelju — prototip zato ima mali poslužiteljski
  dio za taj jedan poziv.

## Otvoreno za Leona

1. **Backend:** Python (FastAPI ili drugo) — bira se na početku PROTOTIPA, uz kratku usporedbu.
2. **AI pružatelj** za demo (besplatni ključ) i gdje živi poslužiteljski dio prototipa.
3. **Vrijeme:** što znači „trajanje 1–7 dana" — stvarni kalendarski dani ili nešto drugo.
4. **Pet faktora i brojke:** kako se mjeri svaki faktor i kako se računa „ista početna ukupna ocjena".
5. **Profesor:** ime i uloga (suautor?); podaci za realnost (sezonalnost, tipične brojke hotela).
6. **GitHub repo** `hotelixsim` (privatni) — još nije napravljen; prvi posao nove sesije. ~~Ime~~ → **HotelixSim** (Leon, 04.10.).
7. **Prijava u inkubator** (rok 16.11.) i prezentacija dekanu — nosi li se Sokrat ili simulacija.

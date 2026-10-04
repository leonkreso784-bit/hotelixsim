---
name: newsession
description: Početak nove HotelixSim sesije — pročitaj predaju prijašnje sesije (docs/SLJEDECA_SESIJA.md), aktivni plan, Leonovu memoriju i stanje repoa (git, baza, CI) i javi kratko gdje smo. Koristi se nakon /clear ili u svježem prozoru.
disable-model-invocation: true
---

# Nova sesija — nastavak iz prijašnje

Leon je upravo otvorio novu sesiju (najčešće nakon `/clear`). Tvoj posao je da za par minuta znaš sve što je
prijašnja sesija znala, bez da Leon išta objašnjava.

## 1 · Pročitaj (paralelno, u jednoj poruci)

1. `docs/SLJEDECA_SESIJA.md` — **cijeli**. To je predaja: cilj ove sesije, prvi korak, otvorena pitanja.
2. **Aktivni plan** u `docs/plan/` (datoteka sa `**Status:** 🟩 AKTIVAN`), ako postoji. `ROADMAP.md` samo ako treba.
3. Leonova memorija (samo čitanje), iz `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\`:
   `tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md`.

Velike dokumente (ARHITEKTURA, DEMO, istraživanja) **ne čitaj cijele** — samo odjeljke koje predaja traži.
Kad trebaš nešto naći, kreni od indeksa `docs/README.md`. `CLAUDE.md` i projektna memorija (`MEMORY.md`) već su učitani sami.

## 2 · Provjeri stanje (Git Bash, jedna naredba)

```bash
cd /c/Users/leonk/Documents/hotelixsim && git status -sb | head -3 && git log --oneline -8 && echo "--- nepushano:" && git log origin/main..main --oneline; bash scripts/baza.sh status; "/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3
```

- Ako baza ne radi (nakon restarta računala): `bash scripts/baza.sh start` i potvrdi izlaz.
- CI stanje gledaš sam preko `gh` — Leona ne pitaš da otvori Actions.
- Ako `git status` pokazuje necommitane izmjene ili drugu granu nego što predaja kaže — **ne diraj ih**, javi Leonu.

## 3 · Javi se Leonu (hrvatski, kratko, bez žargona)

U jednoj poruci, najviše ~12 redaka:

1. **Gdje smo** — 2–3 rečenice iz predaje (što je gotovo, gdje se vidi: lokalno / GitHub, stanje CI-a).
2. **Što danas radimo** — cilj iz predaje, prvi konkretan korak.
3. **Otvorena pitanja** iz predaje — na kraju poruke. `AskUserQuestion` samo za jednostavnu proizvodnu odluku
   s pravim opcijama (preporučena prva, svaka s obrazloženjem). **Arhitektura nikad kroz anketu.** Brojke i činjenice
   koje samo on zna pitaj u tekstu, numerirano, da može odgovoriti kratko.

Zatim radi po predaji. Pravila iz `CLAUDE.md` vrijede: radi bez pitanja za tok rada, **push i deploy samo uz
izričit OK**, nikad „gotovo" bez viđenog izlaza.

Na kraju sesije Leon pokreće `/kraj`.

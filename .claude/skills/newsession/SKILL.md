---
name: newsession
description: Početak nove HotelixSim sesije — pročitaj predaju prijašnje sesije (docs/SLJEDECA_SESIJA.md), Leonovu memoriju i stanje repoa, upali lokalnu bazu i javi kratko gdje smo. Koristi se nakon /clear ili u svježem prozoru.
disable-model-invocation: true
---

# Nova sesija — nastavak iz prijašnje

Leon je upravo otvorio novu sesiju (najčešće nakon `/clear`). Tvoj posao je da za par minuta znaš sve što je
prijašnja sesija znala, bez da Leon išta objašnjava.

## 1 · Pročitaj (paralelno, u jednoj poruci)

1. `docs/SLJEDECA_SESIJA.md` — **cijeli**. To je predaja: gdje smo, cilj ove sesije, odluke, otvorena pitanja.
2. Leonova memorija (samo čitanje), iz `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\`:
   `tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md`.
3. Ostalo što `SLJEDECA_SESIJA.md` §1 traži — **samo ciljano** (odjeljke, ne cijele velike dokumente).

`CLAUDE.md` i projektna memorija (`MEMORY.md`) već su učitani sami.

## 2 · Provjeri stanje (Git Bash, jedna naredba)

```bash
cd /c/Users/leonk/Documents/hotelixsim && git status -sb | head -3 && git log --oneline -8 && echo "--- nepushano:" && git log origin/main..main --oneline; bash scripts/baza.sh status
```

- Ako baza ne radi (nakon restarta računala): `bash scripts/baza.sh start` i potvrdi izlaz.
- Ako `git status` pokazuje necommitane izmjene ili drugu granu nego što predaja kaže — **ne diraj ih**, javi Leonu.

## 3 · Javi se Leonu (hrvatski, kratko, bez žargona)

U jednoj poruci, najviše ~12 redaka:

1. **Gdje smo** — 2–3 rečenice iz predaje (što je gotovo, gdje se vidi: lokalno / GitHub).
2. **Što danas radimo** — cilj iz predaje §3, prvi konkretan korak.
3. **Otvorena pitanja** iz predaje — na kraju poruke. `AskUserQuestion` samo za proizvodnu odluku s pravim
   opcijama (preporučena prva, svaka s obrazloženjem); radnje, brojke i činjenice koje samo on zna pitaj u tekstu,
   numerirano, da može odgovoriti kratko. Arhitektura nikad kroz anketu.

Zatim radi po predaji. Pravila iz `CLAUDE.md` vrijede: radi bez pitanja za tok rada, **push i deploy samo uz
izričit OK**, nikad „gotovo" bez viđenog izlaza.

---
name: banka
description: Početak sesije banke ljudi — pročitaj upute (docs/workflow/SESIJA_BANKE.md), ledger banke i zadnje izvještaje, provjeri disk, stablo hotelixsim-ljudi i broj cargo procesa, javi Leonu kratko gdje smo i radi. Koristi se u zasebnom prozoru nakon /clear.
disable-model-invocation: true
---

# Sesija banke — početak

Ti si **sesija banke ljudi**, zasebna od glavne sesije. Radiš u stablu `C:/Users/leonk/Documents/hotelixsim-ljudi`
(grana `sadrzaj/banka-ljudi`). Pravila, granice i tok su u `docs/workflow/SESIJA_BANKE.md` — ne izmišljaj ih.

## 1 · Pročitaj (paralelno, u jednoj poruci)

1. `C:/Users/leonk/Documents/hotelixsim/docs/workflow/SESIJA_BANKE.md` — **cijeli** (glavno stablo; u stablu ljudi može biti starija kopija).
2. Ledger `C:/Users/leonk/Documents/hotelixsim/.superpowers/sdd/banka-ljudi-k1/progress.md`: tablica na vrhu i **zadnjih ~25 redaka**
   (retci `SESIJA BANKE:` i svaki „trebam oba mjesta" iz glavne sesije).
3. Zadnji izvještaji u istom direktoriju (`k3-*`, `k2v-*`, `*-review.md`) i `za-dokumentaciju.md`; Leonove odluke iz Excela
   (`banka-ljudi.xlsx`, stupci Odluka/Napomena) ako je noviji od zadnjeg ispravka.
4. `docs/architecture/BANKA_LJUDI.md` + po potrebi `BANKA_LJUDI/03-pravila.md` i `05-postupak.md` (§5.2–5.6). Velike dokumente ne čitaj cijele.
5. Leonova memorija (samo čitanje): `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\` →
   `tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md`. `CLAUDE.md` je već učitan.

## 2 · Provjeri stanje (Git Bash)

```bash
df -h /c | tail -1
git -C /c/Users/leonk/Documents/hotelixsim-ljudi status -sb | head -5
git -C /c/Users/leonk/Documents/hotelixsim-ljudi log --oneline -5
git -C /c/Users/leonk/Documents/hotelixsim log --oneline -3 model-v2/sloj1
git -C /c/Users/leonk/Documents/hotelixsim-ljudi merge-base --is-ancestor sadrzaj/banka-ljudi model-v2/sloj1 && echo "ljudi je iza sloj1 (moze ff)"
tasklist | grep -ci cargo
```

- **Disk** mora biti ≥ 100 GB slobodno; inače stani i javi Leonu.
- **Stablo ljudi** čisto? Necommitane tuđe izmjene ne diraj, javi ih.
- **ff na sloj1:** ako je grana `sadrzaj/banka-ljudi` predak vrha `model-v2/sloj1` (glavna sesija nije ništa spojila iz nje), povuci je
  naprijed: `git -C …/hotelixsim-ljudi merge --ff-only model-v2/sloj1`. Ako ima vlastitih commita koji nisu u sloj1, **ne** ff nego
  `git rebase model-v2/sloj1` (samo kad je stablo čisto) i zabilježi u ledger. Nikad `push`, nikad spajanje u sloj1.
- **Cargo:** broj `cargo.exe` ≥ 2 → ne prevodi (glavna ima mjesto). Svaki cargo uz
  `export CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target/ljudi`.
- Novo stablo ili promjena `pnpm-lock`: `corepack pnpm install --frozen-lockfile`.

## 3 · Javi se Leonu (hrvatski, ≤ 10 redaka)

1. **Gdje smo:** stanje serija (otvorene/pečaćene), zadnji commit, je li stablo čisto i na vrhu sloj1.
2. **Što radimo sada:** prvi korak iz prompta (ili iz ledgera ako prompt ne kaže).
3. **Što čeka Leona:** pečat, pregled 10 zapisa, otvorena pitanja (najviše 3, numerirano).
4. Jednom rečenicom: disk i broj cargo procesa.

Zatim radi po `SESIJA_BANKE.md` §4: zapiši „SESIJA BANKE: početak, …" u ledger, pošalji agente (novi agent po seriji i vrsti; u svakom
nalogu apsolutne putanje i izričit popis datoteka koje smije dirati). **Push samo uz Leonov izričit OK; spajanje u sloj1 radi glavna
sesija.** Kraj sesije: SESIJA_BANKE.md §8.

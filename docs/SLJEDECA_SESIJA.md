# Sljedeća sesija — plan „Ekrani igrača" (DEMO korak 5)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Danas radimo DEMO korak 5: ekrani igrača (vidi docs/SLJEDECA_SESIJA.md).
Dizajn je moj koncept u docs/product/DIZAJN.md (logo i tri ekrana u docs/product/dizajn/), stilovi u Tailwindu,
mobitel prvi. Rok demoa je ~14.10., pa plan gradi samo ono što treba profesorovih 15 minuta (DEMO §1.5):
gost preko poveznice, Moj hotel, situacije s razgovorom i slobodnim odgovorom, plan mjeseca, Zatvori mjesec,
rezultat mjeseca s mentorom i ljestvicom, tijek (feed).

Redoslijed: provjeri stanje (git, baza, CI, check:docs, balance --check), pogledaj sliku koncepta, riješi
neslaganja iz DIZAJN.md §3 (proizvodne odluke daj mi kao anketu sa svim opcijama), napiši plan kroz
writing-plans, pa gradi kroz subagent-driven-development na novoj grani, s pregledom nakon svakog taska i
završnim pregledom na najjačem modelu. Ekran provjeri i u pravom pregledniku (Playwright, širina mobitela).
Radi bez pitanja, pretpostavke zapiši kao ADR 🟨; push samo uz moj OK.
```

## Cilj

Plan `docs/plan/<datum>-ekrani.md` (`superpowers:writing-plans`, `**Status:** 🟩 AKTIVAN`) i gradnja kroz
`superpowers:subagent-driven-development` ([AGENTI.md](./workflow/AGENTI.md)). Sučelje je `apps/web` (React +
TypeScript + Vite, tipizirani klijent `packages/api-client` iz `openapi.json`); dodati Tailwind. Gotovo kad Leon na
mobitelu (ili uskoj širini preglednika) odigra lipanj i srpanj od poveznice do ljestvice, bez curla.

**Prvi korak:** `/newsession`, zatim `corepack pnpm check:docs`, `cargo run -p balance --release -- --seeds 100 --check`
i CI (`gh run list`) — sve zeleno; baza `bash scripts/baza.sh start`. Zatim pročitaj:

1. [DIZAJN.md](./product/DIZAJN.md) i sliku [koncept-ekrana.png](./product/dizajn/koncept-ekrana.png) — što ekrani pokazuju i §3 neslaganja.
2. [DEMO.md](./product/DEMO.md) §1.4–1.5 — gost i profesorovih 15 minuta.
3. `openapi.json` / `packages/api-client` — što API stvarno vraća (situacije, razgovor 202 pa čitanje, presuda, mentor, feed).
4. [BACKLOG.md](./records/BACKLOG.md) — stavke `[ekrani]` (npr. upozorenje na neodgovorene PRIJE „Zatvori mjesec", `failed` obračun).

## Otvoreno za Leona

- Neslaganja koncepta i igre ([DIZAJN.md](./product/DIZAJN.md) §3): ocjena do 10 ili do 5; „1/4" ili demo s 2 mjeseca;
  kratki nazivi kriterija; avatari likova i mentora (slike ne postoje).
- Ocjene 40 odgovora za mjerilo AI-ja (`content/hotel/mjerilo/*.json`) — prije demoa.
- Pregled ADR-019 (pretpostavke AI sloja) i likova (Petra Horvat, Marko Babić).
- Ostalo: [BACKLOG.md](./records/BACKLOG.md) → „Otvoreno za Leona". Ništa od toga ne blokira početak koraka 5.

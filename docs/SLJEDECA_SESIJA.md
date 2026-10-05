# Sljedeća sesija — plan „AI sloj" (DEMO korak 4)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Danas radimo DEMO korak 4: plan „AI sloj" (vidi docs/SLJEDECA_SESIJA.md).
Rok demoa je ~14.10., pa plan gradi samo ono što treba profesorovih 15 minuta (DEMO §1.5): presuda na slobodni
odgovor za lipanj i srpanj, likovi, mentor. Najprije popravi pravilo otiska snimka (BACKLOG [korak 4]).

Redoslijed: provjeri stanje (git, baza, CI, check:docs, balance --check), napiši plan kroz writing-plans,
pa gradi kroz subagent-driven-development na novoj grani, s pregledom nakon svakog taska i završnim pregledom
na najjačem modelu. Radi bez pitanja, pretpostavke zapiši kao ADR 🟨; proizvodne odluke daj mi kao anketu
sa svim opcijama; push samo uz moj OK. Neovisne poslove pokreći kao paralelne agente.
```

## Cilj

Napisati plan `docs/plan/<datum>-ai-sloj.md` (`superpowers:writing-plans`, zaglavlje `**Status:** 🟩 AKTIVAN`) i graditi
ga kroz `superpowers:subagent-driven-development` na novoj grani ([AGENTI.md](./workflow/AGENTI.md)): AI iza porta
(presuda na slobodni odgovor s rubrikom, likovi, mentor), zakašnjela presuda za zatvorenu rundu, provjera DEMO §11
točka 7. **Prije svake promjene content DTO-a** (rubrike, likovi u `situations.json`) popraviti otisak snimka: računati ga
nad kanonskim `serde_json::Value` spremljenog snimka, ne nad tipiziranim ponovnim zapisom.

**Prvi korak:** `/newsession`, zatim `corepack pnpm check:docs`, `cargo run -p balance --release -- --seeds 100 --check`
i CI (`gh run list`) — sve zeleno. Zatim pročitaj:

1. [ROADMAP.md](./plan/ROADMAP.md) — korak 4.
2. [ARHITEKTURA.md](./architecture/ARHITEKTURA.md) — odjeljci o AI sloju (port, §8.3 zakašnjela presuda, §9.4 red poslova).
3. [DEMO.md](./product/DEMO.md) — dijelovi o AI-ju (presuda, likovi, mentor, §11 točka 7).
4. [BACKLOG.md](./records/BACKLOG.md) — stavke `[korak 4]` (otisak snimka, `core_version`, `Idempotency-Key`, odgoda ponovnog pokušaja, apalis/pgmq).

## Otvoreno za Leona

- Pregled ADR-015 (arhitektonske pretpostavke koraka 3) i [ADR-017](./records/DECISIONS.md#adr-017) (odluke voditelja gradnje), oba 🟨.
- Ishodi neodgovorenih situacija S1–S4 su nacrt: npr. ignorirana ponuda grupi u S3 jednostavno propada bez kazne — je li to realno?
- AI ključ za demo: koje ključeve ima (pružatelj, model) — vidi BACKLOG „Otvoreno za Leona".
- Ostalo otvoreno: [BACKLOG.md](./records/BACKLOG.md) → „Otvoreno za Leona". Ništa od toga ne blokira početak koraka 4.

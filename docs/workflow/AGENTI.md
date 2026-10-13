# AGENTI — kako HotelixSim gradimo s agentima

> Leon (04.10.): *„koristi agente i subagente da ti pomažu"* i *„napravi da je u pixel agentu puno agenata koji
> rade simultano"*. Ovo je protokol rada. Naredbe za pokretanje i provjere prije commita **nisu ovdje** —
> jedino su mjesto [Lokalni razvoj u README-u](../../README.md#lokalni-razvoj).

## 1 · Tok rada

1. **Brainstorming** (`superpowers:brainstorming`) — što i zašto; arhitektura kroz dokument i raspravu, ne kroz anketu.
2. **Plan** (`superpowers:writing-plans`) u `docs/plan/`, zaglavlje `**Status:** 🟩 AKTIVAN` i `**Područje:** kod|dokumentacija`. Najviše jedan aktivni plan **po području**;
   dugoročni smjer je u [ROADMAP-u](../plan/ROADMAP.md). Plan unutar granice živog dokumenta ([tablica](../README.md#granice-veličine)) smije biti jedna datoteka. Veći se **prije
   aktivacije** razdvaja u pregled + mapu taskova: prvo `node scripts/razdvoji-plan.mjs <plan.md> --provjeri`, pa
   `corepack pnpm docs:plan <plan.md>`. Pregled razdvojenog plana ima granicu indeksa iz iste tablice, pa
   velik plan treba pisati tako da pregled stane, inače se ne može aktivirati.
3. **Gradnja** (`superpowers:subagent-driven-development`) na zasebnoj grani (npr. `gradnja/<ime>`): jedan izvođač po tasku.
4. **Pregled nakon svakog taska** — zaseban pregledavatelj, ne izvođač. Nalaz se vraća izvođaču; tek zeleno ide dalje.
5. **Završni pregled** cijele grane na najjačem modelu; odgođeni nalazi idu u [BACKLOG](../records/BACKLOG.md).
6. **`/kraj`** — dnevnik, ADR-ovi, predaja, brana `check:docs`, lokalni commit; push tek uz Leonov izričit OK.

## 2 · Radni ledger, brief i pretinac

Orkestrator vodi ledger u `.superpowers/sdd/<plan>/` (**gitignoriran**): spec i grana na vrhu, „Ruling" za svaku
odluku koju je donio umjesto Leona (s cijenom ako je kriva), „Preflight scan" (tablica parova taskova: što jedan
proizvodi, što drugi troši, nalaz) i stanje po tasku. Primjer oblika: [ledger kostura](../archive/2026-10-04-kostur-ledger.md).
Na kraju plana ga `/kraj` kopira u `docs/archive/<datum>-<ime>-ledger.md`, jer git ledger ne vidi. Ledger se zove po
pregledu plana (`<datum>-<ime>`), i kad je plan razdvojen.

- **Brief = task datoteka.** Izvođač dobiva putanju task datoteke (`docs/plan/<datum>-<ime>/task-NN-….md`) i pregleda.
  Ako je potreban `task-brief`, uvijek s trećim argumentom, inače radni prostor dobije ime taska:
  `task-brief <mapa>/task-NN-x.md N "$(sdd-workspace <pregled>.md)/task-N-brief.md"`
- **Dvije sesije paralelno** samo na različitim područjima (`kod` i `dokumentacija`), svaka sa svojim aktivnim planom.
- **Pretinac.** Dok traje faza A (plan dokumentacije, Protokol paralelnih sesija), sesija koda ne piše u
  `docs/records/`. Svoje bilješke ostavlja u `.superpowers/sdd/<plan>/za-dokumentaciju.md`, u odjeljcima `## ADR`,
  `## BACKLOG`, `## CHANGELOG`, `## PROGRESS`. Pretinac prenosi sesija koda kad spoji `main` (plan dokumentacije →
  „Spajanje u sesiju koda”); sesija dokumentacije ga samo čita.

## 3 · Model po poslu

| posao | model |
|---|---|
| izvođači taskova | sonnet |
| pregled nakon taska | sonnet |
| kratki ponovni pregled ispravka | haiku |
| završni pregled grane | opus (najjači dostupan) |

## 4 · Paralelni agenti

- Neovisne poslove (provjere, revizije, istraživanja, nacrti) pokreći **u jednoj poruci**, u pozadini.
- **Jedan pisac po datoteci**: svaki agent dobiva popis datoteka koje smije dirati i ništa drugo. Eksperimenti nad
  kodom idu u worktree.
- Tvrdnje agenata nisu dokaz: ključni nalaz provjeri sam ili ga daj drugom agentu da ga neovisno ponovi.
- Leon prati rad u VS Code dodatku Pixel Agents: imenovani agenti dobiju vlastiti lik i sjedalo, neimenovani su
  „Subtask" likovi uz glavnog. Broj agenata nije ograničen postavkom.

## 5 · CI i GitHub

CI gledaj **sam**, Leona ne pitaj da otvori Actions. `gh` je u `C:\Program Files\GitHub CLI\gh.exe` (nije na PATH-u
Bash alata):

```bash
"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3
"/c/Program Files/GitHub CLI/gh.exe" run view <id> -R leonkreso784-bit/hotelixsim
"/c/Program Files/GitHub CLI/gh.exe" run view --job=<id> --log-failed -R leonkreso784-bit/hotelixsim
```

GitHub MCP plugin pada („Authorization header is badly formatted", token nije postavljen) — nije potreban.

## 6 · Što agenti smiju, a što ne

Pravila rada s Leonom u [`CLAUDE.md`](../../CLAUDE.md) vrijede i za svakog agenta (radi bez pitanja, push i deploy
samo uz izričit OK, nikad „zeleno" prije viđenog izlaza). Lokalni commit je u redu; agent commita samo ako mu je to
izričito u zadatku.

## 7 · Zamke ovog stroja

- `pnpm` nije na PATH-u: samo `corepack pnpm …`.
- Cijev maskira exit kod: pokreni golu naredbu pa `echo EXIT=$?` (ne `| tail`).
- Bash alat resetira radni direktorij između poziva: u svakoj naredbi `cd` na apsolutnu putanju ili koristi apsolutne putanje.
- Agentima u promptu daj apsolutne putanje i izričit popis dopuštenih datoteka.

Cigla L1-13b je **ostatak Leonovih odluka iz pregleda 14.10.** ([ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md))
koji nije ušao u fazu B ni u L1-13. Leon 19.10. (sesija 11, anketa): **cijela L1-13b prije pilota** i **rad u valovima** (neovisni dijelovi
istodobno, svaki u svom stablu). Ova kartica je pregled i raspored; analize s retcima (samo čitanje, 19.10., na `e4ec659`):
`.superpowers/sdd/2026-10-14-popravak-temelja/l113b-analiza-mijesanje.md`, `l113b-analiza-odrzavanje.md`, `l113b-analiza-web.md`.

### Task L1-13b: Izmiješani odgovori, pojačano održavanje +5, jedan gumb „Spremi plan", opis Smještaja

**Veličina:** 🟨 [PROC] ukupno 11–17 h rada agenata, oko 1–1,5 sesije uz valove · **Ovisi o:** L1-13 ✅ (e4ec659, `CORE_VERSION` 2.6.0) ·
**Slijedi ga:** L1-14 (Leon igra tjednu i mjesečnu sezonu), L1-15 (`sloj1` → `main`), zatim pilot · **Model:** čuvar i graditelj sonnet,
dio 1 provjeravatelj **opus** (pravilo igre u `game`), ostali sonnet · **Prevodi Rust:** dijelovi 1 i 4; nikad oba usporedo
(jedan Rust slot po sesiji, drugi drži sesija banke).

## Što je već napravljeno iz ADR-042 (ne radi se ponovo)

t. 1 i 7 (kvar i stanje) — L1-13 · t. 6 (lik zaključi u 6 poruka, `lik.v3`) · t. 11 (izgled lika) — IZGLED · t. 12–14 (jednom u krugu,
mjesečno 4/5/6 kao postavka) — B2a. t. 3 (raspon potrebnih radnika) čeka banku ljudi (L2a), t. 9 (ime hotela) je HN-1 u fazi C — **nisu u L1-13b**.

## Dijelovi

| dio | što | dira | Rust | veličina | prije gradnje |
|---|---|---|---|---|---|
| **1 · miješanje** (t. 10) | redoslijed ponuđenih odgovora iz sjemena igre + slota hotela + šifre situacije; app prikazuje izmiješano i kliknuti položaj prevodi natrag u izvorni broj; dnevnik (`game/src/feed.rs`) ne odaje izvorni broj | `game`, `app`, opisi u `openapi.json`; web i ugovor API-ja isti | da (`game`, `app`, `api`) | 3–5 h | 🟨 t. 1–2 niže |
| **2 · jedan gumb** (t. 4) | jedan „Spremi plan" sprema plan i odluke svih odjela odjednom (API to već radi jednim PUT-om i događajem `PlanSet`); upozorenje pri zatvaranju kruga ostaje | samo `apps/web` (+ `packages/i18n`) | ne | 5–7 h (~40 vitest testova se prilagođava) | Leon: gdje stoji gumb |
| **3 · opis Smještaja** | 🟨 vidi pitanje 2 | web + i18n (ili objašnjenje kruga) | ne / da | 1–2 h | **Leon: što je točno traženo** |
| **4 · održavanje +5** (t. 2) | pojačano održavanje neto **+5 bodova stanja mjesečno** (tjedno ~+1,17); danas Smještaj +4, Restoran +3, barovi +1,5–3,5, Wellness +2,5 | `content/hotel/v2/departments.json` (`intensive.equipment_pts`), `crates/content/tests/v2_obnova.rs` | da (test + balans 1000) | 2–3 h | **Leonova anketa s brojkama** (pitanje 3) |

**Brane i pečat:** dio 1 ne mijenja `CORE_VERSION`, pečat ni balans (`crates/balance` ne ovisi o `game`; roboti biraju po kvaliteti).
Dio 4 mijenja samo brojke sadržaja (svaka igra nosi svoj snimak pravila, pečat igra na zamrznutom) — 2.6.0 ostaje; zadani plan koristi
redovno održavanje pa brane `--check` (težina, klik, lanac 0,04) ne bi trebale vidjeti promjenu, ali se **mjere** (1000 tjedno i mjesečno, `cmp`).

## Valovi

```text
val 1 (odmah po Leonovu OK-u):   dio 1 (Rust slot glavne)   ∥   dio 2 (web, bez Rusta)   ∥   dio 3 (web, ako Leon odgovori)
val 2 (kad je dio 1 spojen):     dio 4 (Rust slot glavne; anketa iz pitanja 3 već odgovorena)
spajanje redom:                  1 → 2 → 3 → 4, svaki s cijelim CI-jem (scratchpad/nakon-l113b-N.sh), cmp balansa prema prethodnoj osnovici
```

Svaki dio: svoje stablo `../hotelixsim-l113b-N` (apsolutni put, B22), svoj `CARGO_TARGET_DIR=…/target/l113b-N`, tok čuvar → graditelj →
provjeravatelj (ADR-038). Dijelovi 1, 2 i 3 nemaju zajedničkih datoteka osim `packages/i18n/src/hr.ts` (različiti ključevi — trivijalan
sukob pri spajanju). Ubrzanje prema radu redom oko 1,5× (granica od dva Rust procesa).

## 🟨 Pretpostavke (ADR na `/kraj`)

1. **Miješanje živi u `game` + `app`, ne u webu** — odstupa od doslovnog teksta ADR-042 t. 10 („web šalje izvorni broj"), ali čuva smisao:
   API i web i dalje govore „broj = položaj na ekranu", izvorni broj se nigdje ne vidi, isti student nakon osvježavanja vidi isti redoslijed,
   različiti hoteli različit. Pravilo miješanja se ne mijenja bez dizanja verzije (inače bi klik usred kruga zapisao drugi odgovor).
2. **v1 (zamrznuti demo) ostaje nepromiješan.**
3. **Održavanje +5 bez prekidača sadržaja i bez nove verzije jezgre** (mijenjaju se samo brojke).

## Leonovi odgovori 19.10. (anketa, sesija 11)

- **Opis Smještaja = tekst na stranici Smještaja** (zgrada i sobe određuju kvarove; ADR-042 t. 7) — samo web, ~1 h.
- **Cijena pojačanog ista**, **+5 prije trošenja zbog gužve** (u gužvi +4).
- **Odjeli:** „to se mora izbalansirati isto s financijama, jer ponekad se moraju financije uzeti u obzir, ponekad održavanje".
  Tumačenje voditelja (🟨, potvrditi): **nijedna razina održavanja ne smije biti najbolja uvijek** — dio 4 prije spajanja daje Leonu
  tablicu (kao A2): napredak pojačano / zadano / odgođeno po hotelu, tjedno i mjesečno, za šest operativnih odjela i za svih osam,
  uz to koji hotel „mora štedjeti" (financije) a koji „mora ulagati" (stanje). Polazno šest operativnih; parking i lokal samo ako tablica pokaže potrebu.
- **Gumb „Spremi plan"** (🟨 voditelj): traka vidljiva i na Planu i na stranici odjela.

## Pitanja za Leona (prije vala 1; odgovori gore)

1. **Jedan gumb — gdje stoji?** Preporuka: traka „Spremi plan" vidljiva i na Planu i na stranici odjela (anketa).
2. **„Opis Smještaja" — što si točno htio?** Fraza stoji samo u planu (14.10.), bez zapisa odluke. Dvije mogućnosti:
   (a) tekst na stranici Smještaja da zgrada i sobe određuju kvarove (klima, curenje, brave, dizalo, praonica — ADR-042 t. 7);
   (b) objašnjenje kruga: redak „Održavanje" danas nosi i dio promjene Smještaja, što zbunjuje (nalaz Sloja 1, Task 12 minor).
3. **Održavanje +5 — anketa s brojkama:** (1) koji odjeli (šest operativnih — preporuka; parking i lokal ostaju); (2) cijena pojačanog
   ista (preporuka; pojačano postaje jeftinije po bodu) ili ista po bodu (Smještaj 33.000 €, Bar uz bazen +67 %); (3) +5 prije trošenja
   zbog gužve (preporuka; u gužvi +4) ili poslije (+6 u mirnom krugu); (4) hotel B i uz +5 ostaje ~14–15 kvarova (sobe ~62 < pragovi 65/80)
   — potvrditi „ostaviti" (odluka 19.10.).

**Nalaz uz dio 4:** uz istu cijenu pojačano dobiva oko +1 napretka — u hotelu B tjedno bi se „odgođeno bolje od pojačanog" vjerojatno
okrenulo, u A ne (razlika 3,2). Stavka u [BACKLOG](../../records/BACKLOG.md) ostaje dok se ne izmjeri.

## Stanje 19.10. (kraj sesije 11): svi dijelovi spojeni
Spojeno redom 2 → 3 → 4 → 1, svaki `--no-ff` i pregledan (Approved): **dio 2** jedan gumb 13db069, **dio 3** opis Smještaja 4be1db4, **dio 4**
održavanje +5 bdbdebe (tablica: A štedi, B i C ulažu; šest odjela), **dio 1** miješanje 7556d80. Cijeli CI `nakon-l113b/` zelen, balans bajt-isti
s ispisom dijela 4; pushano, GitHub CI zelen. Pretpostavke 🟨 1–3 i nove su u [ADR-062](../../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md)
(t. 10–17); odgođeni minori u [BACKLOG](../../records/BACKLOG.md). Sljedeće: L1-14.

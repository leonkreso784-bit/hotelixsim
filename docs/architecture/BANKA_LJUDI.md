# Banka ljudi — radnici, gosti i vanjski kao sadržaj igre

> **Status:** 🟨 NACRT (14.10.); K0 ✅ (odgovori §6 u ADR-042 t. 8); **K1a spojen (adf5a4d) i K1b spojen** (16.10.; serija-000 = 10 radnika + 10 gostiju); **K2** (serija-001, 50 + 50, stablo ljudi 918e8e8); **pečat serija 000 i 001 (120 ljudi) Leon je odlučio 17.10.** (*„super su"*; pečat izvodi paralelna sesija banke, [ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)); **treća vrsta VANJSKI** (Leon 16.10., §1.5, korak K2v), **podvrste radnika** (Leon 17.10., §1.2) i **paralelna sesija banke** (§5.6) (usporedo s popravkom: [ADR-052](../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md), kartica u ledgeru `.superpowers/sdd/banka-ljudi-k1/`; rod = Eurostat LFS, zemlje = MUP, uloga smije ovisiti o podrijetlu). Povod: Leonova odluka
> [ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 5 — *„jako puno likova, radnike i goste,
> ~2000 ili više, strukturno, korak po korak, oprezno"*, uz dva agenta posvećena samo tome. Ovo je prvi korak: **format i
> pravila**. Slijede provjera sadržaja u Rustu (`crates/content`) i punjenje u serijama (§5). Arhitektura ide kroz dokument i
> raspravu ([ADR-006](../records/DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md)); pitanja su u §6.

**Riječ „baza".** Banka ljudi **nije PostgreSQL**. To su JSON datoteke sadržaja u repou (`content/hotel/v2/ljudi/`), kao
predlošci situacija. Ulaze u git, pregledavaju se kao tekst, a provjerava ih Rust pri učitavanju sadržaja i u testovima. PostgreSQL čuva samo
igre (dnevnik događaja), a igra iz banke pamti **koje** je likove uzela (ID), ne njihov tekst.

**Danas.** Likovi situacija dobivaju ime iz `content/hotel/v2/names.json` (grana situacija): 3 izvora (gost, zaposlenik,
partner) × 2 roda × 20 imena = 120 imena, samo „Ime Prezime", bez ičega drugog. Provjera traži samo neprazno i
neponovljeno ime (`crates/content/src/templates_check.rs`, `check_names`). Izgled lika izvodi web iz imena
(`apps/web/src/avatar/look.ts`, `lookFromSeed`) i ne zna rod ni dob, pa gošća može dobiti bradu. Opis „tko je lik" daje
predložak (`types`, npr. „umirovljenica na godišnjem odmoru"), a ne ime.

## Dijelovi

| § | dio |
|---|---|
| 1 · što je lik: načelo „banka čuva tko je osoba", RADNIK, GOST, primjeri, **VANJSKI** (§1.5) | [01-lik.md](./BANKA_LJUDI/01-lik.md) |
| 2 · koliko i kakvih: brojevi M1–M3, raspodjele radnika i gostiju, **vanjski** (§2.4) | [02-koliko.md](./BANKA_LJUDI/02-koliko.md) |
| 3 · pravila kvalitete: jedinstvenost, dosljednost, bez stereotipa, rod i padeži, zabranjeno, tko provjerava, **vanjski** (§3.7) | [03-pravila.md](./BANKA_LJUDI/03-pravila.md) |
| 4 · gdje živi i kako se koristi: datoteke, izbor po sjemenu, verzije i pečati, **vanjski** (§4.4) | [04-gdje-zivi.md](./BANKA_LJUDI/04-gdje-zivi.md) |
| 5 · postupak: koraci K0–K5 i **K2v**, agenti i datoteke, „serija gotova", kada se staje, **tri agenta** (§5.5), **paralelna sesija banke** (§5.6) | [05-postupak.md](./BANKA_LJUDI/05-postupak.md) |
| 6 · pitanja za Leona (1–11 odgovorena 14.10., ADR-042 t. 8; 12–16 o vanjskima odgovorena 17.10.) i odluke C1–C11 od 17.10. | [06-pitanja.md](./BANKA_LJUDI/06-pitanja.md) |
| 7 · 🟨 nacrt rizika: likovi nalik poznatim osobama (C3, Leon 17.10.), tri varijante s cijenom, pitanja | [07-poznate-osobe-rizik.md](./BANKA_LJUDI/07-poznate-osobe-rizik.md) |

## Sažetak

- **Tri vrste zapisa** u JSON serijama od najviše 50: **radnik** `R-` (početni ljudi, burza, agencija), **gost** `G-`
  (uloge gosta u situacijama, recenzije), **vanjski** `V-` (inspekcija, policija, vatrogasci, dobavljači, agencije, novinari,
  susjedi; Leon 16.10.). Banka čuva tko je osoba; brojke (vještine, osobine, plaća) računa jezgra (§1.1).
- **Odluke:** ADR-042 t. 8 (odgovori 14.10.), [ADR-052](../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md)
  (banka usporedo s popravkom, rod po Eurostatu, zemlje po MUP-u, uloga smije ovisiti o podrijetlu); Leonove odluke 16.10.
  (sesija 7) o vanjskima, trima agentima i sesiji banke zapisane su u §1.5; odluke 17.10. (pečat, podvrste radnika, C2–C11,
  paralelna sesija) u ADR-056 i §6. Likovi nalik poznatim osobama (C3) **čekaju Leona** (§7); dok ne odluči, banka ih ne dodaje.
- **Gdje se vidi:** samo u repou (`content/hotel/v2/ljudi/`, `crates/content`); igra banku još ne čita (K5). Pregled za Leona:
  `pregled-svi.html` u ledgeru banke.

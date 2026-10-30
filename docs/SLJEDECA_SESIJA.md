# Sljedeća sesija — raspodjela posla u timu, jedan plan do ožujka, gradnja

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 15 (28.–30.10.): prezentacija i demo za **odbor FMTU-a** (izvan repoa, `Documents/HotelixSim-prezentacija`). U tim ulaze
**Rino Tomić** i **Simon Seifert** ([TEAM.md](../TEAM.md), [ADR-067](./records/DECISIONS/ADR-067-tim-rino-tomic-i-simon-seifert-simpolis-kao-izvor.md); pozivi u repo poslani, još „pending"). Simpolis (`riihack`, RI-HACK 4.0,
3. mjesto) je izvor ideja i izgleda. Pilot je **1. godina Hospitality Managementa na engleskom smjeru**, igra na **hr i en**
([ADR-068](./records/DECISIONS/ADR-068-dva-jezika-hrvatski-i-engleski-pilot-na-engleskom.md)). Analize: `.superpowers/sdd/2026-10-29-tim/` (engleski 110–200 h, izgled ~110–180 h web).
Disk očišćen: **187 GB slobodno**; `target/debug` je obrisan, pa je prvo prevođenje dugo (na punjaču).
Ništa od koda nije gradeno; plan [Pilot s profesorom](./plan/2026-10-24-pilot-s-profesorom.md) i dalje 🟨.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 16) — raspodjela posla, jedan plan, gradnja. Pročitaj docs/SLJEDECA_SESIJA.md, TEAM.md, ADR-066, ADR-067, ADR-068, plan docs/plan/2026-10-24-pilot-s-profesorom.md (Pregled, Pitanja za Leona, Global Constraints, Tokovi, VAL 0) i .superpowers/sdd/2026-10-29-tim/ (engleski, izgled).
Laptop je na punjaču. Push, spajanje u main, živi AI test i deploy samo uz moj izričit OK.
1) `df -h /c`, `bash scripts/baza.sh status` (ako ne radi: start), `tasklist | grep -i cargo`; jesu li rtomic20 i simonseifert prihvatili poziv (gh api .../collaborators).
2) Pitaj me kako je prošao odbor FMTU-a (šest odluka: kolegij, nositelj, privatnost, tko odobrava, tko plaća, stručna pomoć) i zapiši u TEAM.md i BACKLOG.
3) Raspodjela posla: prođi sa mnom „Cilj i raspodjela posla" u TEAM.md (što vrijedi za Rina i Simona, njihovo računalo, AI ključevi, sastanak) i otvorena pitanja 1–7; odluke upiši u ADR-067.
4) Postavi mi pitanja plana („Pitanja za Leona" 1, 2, 3, 8 anketom, preporuka prva; 4–7 i 9 tekstom, numerirano) i pretpostavke ADR-068 t. 3–8 (objašnjenja pojmova za prvu godinu anketom).
5) Spoji sve u JEDAN plan do ožujka 2027. (pilot + engleski + izgled po Simpolisu + tko radi što, po mapama); stari plan ide u arhivu ili se prepisuje; najviše 1000 redaka. Pokaži mi ga prije gradnje.
6) Na moje „kreni": novi ledger, pa val 0 po planu — čuvar testova prije graditelja, svaki tok svoje stablo ../hotelixsim-<tok> i svoj target/<tok>, najviše 2 Rust toka, pregled prije spajanja, lint + CI prije pusha.
Na kraju /kraj (check:docs, test:docs i lint EXIT 0).
```

## Cilj sesije

1. **Tim radi:** podjela potvrđena, Rino i Simon pokrenuli igru lokalno i imaju prvi mali PR.
2. **Jedan plan do ožujka** s tri kolosijeka: pilot (val 0–4), engleski, izgled po Simpolisu; svaki s vlasnikom i mapama.
3. **Val 0 = testovi** kreće na Leonov „kreni".

## Upozorenja

- **Rino dira `apps/web/src`, a plan ima tok WEB u istim datotekama.** Prije WEB-T dogovoriti redoslijed: testovi ekrana provjeravaju DOM,
  pa preuređenje izgleda ruši oko 8–15 h testova (izgled-simpolis-29-10.md).
- **`accent` znači drugu boju u HotelixSimu (plava) i shadcn-u (svijetla):** riješiti prije prvog kopiranja iz Simpolisa.
- **Kod Simpolisa je suvlasništvo sve trojice:** kratka pisana suglasnost prije prvog prijenosa (ADR-067 t. 5).
- Prijevod sadržaja: iste šifre, ključevi, redoslijed i broj stavki, inače se mijenjaju brojke; zapečaćene serije banke se ne diraju (`ljudi_sealed.rs`).
- `avatar/look.ts:366,384` traži izgled robota po hrvatskom imenu.
- README je pisan za Windows; provjeriti što imaju Rino i Simon.
- Demo: „HotelixSim demo.bat" uzima `target/debug/server.exe`, a ako ga nema `AppData/Local/hotelixsim/bin/server.exe` (stari kod, 23.10.).
- Lint crven samo pod `.superpowers/` = B19. Najmanji korak lanca 0,04. `grep -r` zapinje na `target/`.
- Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- Ishod odbora FMTU-a. Pitanja plana 1–9. Vlasništvo i licenca (prije inkubatora). Jedna prijava u inkubator do 16.11. (HotelixSim ili Simpolis).
- Zaštita grane `main`. Disk: `kres-lab` 36,6 GB, Downloads 5,7 GB, Temp 5,6 GB, ostatak `Documents/hotelixsim-ljudi` 81 MB.
- Pilot: tekst privatnosti, Railway, `pg_dump`. Pretpostavke 🟨 po popisu u CLAUDE.md. Nalaz E4, C3, uzorci banke, B4a–c, ocjene 40 odgovora.

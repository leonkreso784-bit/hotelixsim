# Sljedeća sesija — HotelixSim: od arhitekture do demoa

> Zapisano 2026-10-04 na kraju sesije. Radimo u **Claude Code terminalu** (Windows: Git Bash + PowerShell).
> Repo: `C:\Users\leonk\Documents\hotelixsim` · GitHub: `leonkreso784-bit/hotelixsim` (privatni).

## 1 · Prvo pročitaj — samo ovo i ciljano

1. `CLAUDE.md` (učita se sam).
2. Memoriju o Leonu (samo čitanje) iz `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\`:
   `tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md`.
   Projektna memorija ovog repoa (`MEMORY.md`) učita se sama.
3. `docs/ARHITEKTURA.md` (nacrt v1). Cijeli pročitaj samo §0 i sadržaj, a ostale odjeljke kad ti zatrebaju.
4. `docs/RASPRAVA_ARHITEKTURE.md`: šest popravaka iz kruga 1 koji **još nisu ugrađeni** i Leonovi odgovori.
5. `docs/PITANJA_ZA_PROFESORE.md`: što čeka profesore. Ništa od toga ne blokira demo.
6. `docs/VIZIJA.md`. **`docs/PLAN.md` je zastario** (stari prototip, rok 10.10., Python), ne slijedi ga.
7. `git status`: provjeri jesu li dokumenti iz 04.10. commitani.

## 2 · Gdje smo (04.10.)

- **Dogovoreno:**
  - runde;
  - pravila računaju brojke, a AI samo prevodi;
  - hotel ima dva sloja: stvarne brojke i pet indeksa;
  - raspored situacija, splet okolnosti i profesorove postavke;
  - dnevnik događaja kao izvor istine;
  - **Rust** za jezgru i backend, **TypeScript** za sučelje;
  - AI ostaje u simulaciji za demo.
- **Rok demoa je pomaknut.** Demo se gradi na pravoj arhitekturi i ne baca se.
- **Alati:** Rust 1.98 i Node 24 su instalirani. Nema pnpm, Dockera ni Railway CLI-ja.

## 3 · Cilj sesije: ekspanzivna izrada demoa, redom

Nijedan korak ne počinje dok Leon ne potvrdi prethodni.

1. **Arhitektura v2.**
   - Ugradi krug 1 iz `RASPRAVA_ARHITEKTURE.md` u `ARHITEKTURA.md`. Prvo pitaj Leona za upit 2: što student vidi o drugima.
   - Prepravi obrazloženje za Rust (§5, §18): determinizam dolazi iz dizajna, a Rust daje stroži prevoditelj, brzinu i prenosivost.
   - Pilot na Railwayu ide kao jedan servis.
   - Upiši novi krug u `RASPRAVA_ARHITEKTURE.md`. Leon odobrava.
2. **Opseg demoa** u novoj datoteci `docs/DEMO.md`:
   - **Demo način vremena:** rundu zatvara gumb, ne kalendar.
   - **Tri robota** sa strategijama koja stvarno igraju kroz jezgru.
   - Demo **bez prave prijave**: poveznica stvori gosta.
   - Što profesor vidi i radi u 15 minuta.
3. **Dizajn igre za demo.** Ovo je najveći komad i srce projekta. Ide u `DEMO.md`:
   - stvarni sloj hotela;
   - formule za pet indeksa;
   - profili A, B i C s **jednakim prostorom za napredak**;
   - katalog mehanika;
   - 3–4 situacije s rubrikama 1–5;
   - formula relativne ocjene;
   - okidači spleta okolnosti.

   Sve je zapisana pretpostavka dok profesori ne odgovore. **Leon je student hotelijerstva i stručnjak za domenu.** Ti slažeš pravila i brojke i pitaš njega.
4. **AI za demo:** pružatelj, model i ključ. Leon ima besplatne ključeve, pitaj koje.
5. **Uskladi** `CLAUDE.md` i `PLAN.md`, pa commitaj. Push samo uz Leonov OK.
6. **Plan izgradnje** (`superpowers:writing-plans`), zatim gradnja kroz agente (`superpowers:subagent-driven-development`). Prvi korak gradnje je kostur od kraja do kraja (ARHITEKTURA §20).

## 4 · Kako koristiti agente

- **Paralelno istraživanje dok raspravljaš s Leonom.** Neovisne agente pokreni u istoj poruci, na primjer:
  - hotelski pokazatelji i formule (ADR, RevPAR, GOPPAR, fluktuacija) za stvarni sloj;
  - kako HOTS i Cesim Hospitality modeliraju potražnju i odluke;
  - stanje Rust alata na trenutnim verzijama: Axum, utoipa, SQLx, apalis ili vlastiti red poslova.
- **Nacrti sadržaja:** agent predloži situacije s rubrikama po osima. Ti ih provjeriš prije nego što idu Leonu.
- **Gradnja:** jedan agent po zadatku iz plana, s jasnim ulazom i izlazom. Nakon svakog zadatka slijedi pregled koda, tek onda idući zadatak.
- **Traženje u kodu:** koristi `izvidnik` ili `Explore` umjesto čitanja datoteka u glavnoj sesiji.
- Ne provjeravaj agente u petlji, obavijest stiže sama.
- **Workflow** (višeagentna orkestracija) pokreći samo ako Leon izričito kaže „use a workflow" ili „ultracode".

## 5 · Pametna potrošnja tokena

- Agent vraća **sažetak** (do ~30 redaka i putanje), ne sirove datoteke.
- **Model po poslu:** jeftiniji (sonnet ili haiku) za pretrage i mehanički posao, najjači za arhitekturu, jezgru i pregled koda.
- `ARHITEKTURA.md` ima oko 680 redaka. Ne čitaj ga cijelog više puta, čitaj odjeljke.
- Ne čitaj datoteku ponovno nakon vlastite izmjene. Koristi Edit umjesto prepisivanja cijelog dokumenta.
- Istu provjeru ne pokreći dvaput bez promjene.
- Na prirodnoj granici, kad je neki korak odobren, predloži Leonu `/compact`.
- **Piši u kraćim komadima.** Leon ne voli dugo čekanje, zato napiši jedan odjeljak, javi se, pa idi dalje.

## 6 · Pravila s Leonom (sažetak; puna su u CLAUDE.md i memoriji)

- Hrvatski, kratko, bez žargona. Pojam koji možda ne zna objasni od početka.
- **Arhitektura se ne nudi kao anketa**, nego kao obrazložen dokument i rasprava. Jednostavne proizvodne odluke smiju ići kroz anketu: preporuka prva, bez lažnih opcija.
- Ne biraj tehnologiju po tome što Leon zna. Misli široko.
- Push i deploy samo uz njegov izričit OK.
- Nikad „gotovo" ili „zeleno" prije viđenog izlaza. „Jesi siguran?" znači novi prolaz provjere.
- Uz svako „gotovo" reci gdje se vidi.
- Temelj prije brzine.
- Pitanja idu u zadnju poruku ture.

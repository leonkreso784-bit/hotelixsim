# ADR-022 — Ekrani igrača: pretpostavke plana koraka 5
**Datum:** 2026-10-10 · **Status:** ✅ **Potvrdio Leon 17.10.** (A13 c; pretpostavke Claudea, „radi bez pitanja") · **Vezano:** [ADR-021](ADR-021-koncept-ekrana-v2-i-odluke-o.md),
[DIZAJN.md](../../product/DIZAJN.md), DEMO §1.4–1.5 · **Plan:** [2026-10-10-ekrani.md](../../archive/2026-10-10-ekrani.md)

**Odluka (pretpostavke):**
1. **Vlastiti mali usmjerivač** (`src/router.tsx`, ~60 redaka, History API) umjesto `react-router` 8 (nova glavna
   verzija, a trebamo sedam ruta bez učitavača). Cijena ako zatreba više: zamjena jedne datoteke i `Link`-ova.
2. **Izgled samo kroz tokene:** boje, slova i zaobljenja su `@theme` varijable u `apps/web/src/styles.css`; ekrani
   koriste imena tokena. Ikone `lucide-react`. Ilustracija obale i „Jadran · 4★" iz koncepta izostaju (nema slike ni
   regije u API-ju); zaglavlje pokazuje broj soba.
3. **Gost:** `/` i `/demo` su isti ekran; tko nema igru vidi „Započni igru" (jedan klik → `POST /v1/demo`). Igra se ne
   stvara sama pri otvaranju, jer bi osvježavanje ili dvostruki efekt u Reactu stvorili dvije igre.
4. **Opis situacije** = prva rečenica lika (`opening`); sadržaj nema polje opisa. S3 (bez lika) pokazuje samo naslov i
   ponuđene odgovore; njezin naslov još nosi „(nacrt)" iz sadržaja — to je Leonov sadržaj, ekran ga ne skriva.
5. **Slobodan odgovor prvi, ponuđeni ispod** („ili izaberi jedan od ponuđenih"); jedno isključuje drugo (poslužitelj
   ionako vraća `already_answered`).
6. **Likovi:** izgled (`AvatarLook`) danas daje tablica poznatih likova u kodu ili deterministički izgled iz imena; poslije
   demoa seli u sadržaj (`character.look`) i profil igrača (API + baza), a uređivač mijenja ta polja. Boje lika su
   podatak (`src/avatar/palette.ts`), ne tema; novi element palete ide na kraj niza da stari izgledi ostanu isti.
7. **Promjene indeksa:** na „Moj hotel" od **početka** (ocjena je napredak od početka), na „Mjesec završen" **u tom
   mjesecu** (prema prethodnom mjesecu, za lipanj prema početku); oba su označena riječima. Uzroci se ne vežu uz
   indekse (API ih ne veže) nego se prikazuju po varijabli („Prodane noći: cijena −120…").
8. **Hrvatski tekstovi uzroka, varijabli i grešaka** (`cause.*`, `var.*`, `problem.*`) napisao je Claude.
9. **Ispitivanje umjesto SSE-a:** razgovor svakih 1,5 s dok lik piše, presuda svake 2 s dok je `pending`, obračun
   svake 2 s dok je runda `closing`; nakon 150 s obračuna ekran kaže da traje dulje nego inače.
10. **Provjera u pregledniku** je ručna (Playwright MCP, 390 px) na pravom poslužitelju i bazi; e2e u CI-ju traži
   Postgres i poslužitelj u poslu weba — BACKLOG `[ekrani]`.

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): A13 c: slobodan odgovor prvi, promjene indeksa „od početka"; ostale točke uz isto „ok sve".

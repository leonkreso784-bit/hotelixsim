# ADR-039 — Brana veličine koda: upozorenje 600, granica 1000, popis duga
**Datum:** 2026-10-13 · **Status:** ✅ **Potvrdio Leon 17.10.** (F1; pretpostavka voditelja, [ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)) ·
**Vezano:** [ADR-030](ADR-030-raspored-dokumentacije-granice-velicine-indeks-mapa.md) Dopuna 2, [ADR-038](ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) t. 4,
[kvaliteta.md](../../plan/ROADMAP/kvaliteta.md) K0-3

**Kontekst.** Leon je za dokumente (`.md`) odlučio: neki smiju preko 1000, ali nijedan preko **1200** (ADR-030 Dopuna 2).
Za **kod** nije odlučio ništa. Datoteke koda ipak tiho rastu (`schema.rs` 1687, `mentor.rs` 1367, `tests/mentor.rs` 1670), a
ADR-038 t. 4 traži da nijedna dirana datoteka ne pređe granicu veličine. Faza TEMELJ RADA (K0-3) zato je izgradila branu
`check:kod` s pretpostavkom koju Leon nije potvrdio.

**Odluka (pretpostavka).**
1. `corepack pnpm check:kod` broji retke `.rs .ts .tsx .mjs .js` (kao `wc -l`, CRLF isto).
2. **Iznad 600** redaka je upozorenje (izlaz 0); **iznad 1000** je greška (izlaz 1, pada i CI).
3. **Popis duga** (`DUG` u `scripts/check-kod.mjs`): tri postojeće datoteke preko 1000 smiju ostati dok ne prerastu
   upisani maksimum. Rast je greška, pad je upozorenje da se unos spusti, a nestala datoteka znači zastario unos.
   Rezanje tih datoteka su cigle K0-4, K0-5 i K0-6.
4. Izuzeto je samo generirano (`packages/api-client/src/schema.d.ts`). `openapi.json` i `content/**/*.json` nisu kod pa se
   ne broje. Ne ulaze ni `node_modules`, `target`, `dist`, `.git`, `.superpowers` i `.claude` (agentske radne kopije).
5. Brojke 600 i 1000 su konstante (`UPOZORENJE`, `TVRDA`), a popis duga jedan objekt; promjena je jedan redak u istom commitu.

**Posljedice.**
- Ako Leon želi drukčije granice za kod (npr. 1200 kao za dokumente), mijenjaju se dvije konstante i popis duga, bez drugog
  posla. Testovi u `scripts/check-kod*.test.mjs` vrijede dok god brana postoji.
- Upozorenja iznad 600 su danas šum (10 istih u svakom pokretanju); kad ih Leon prihvati, valja ih svesti na nova.
- Do Leonove potvrde brana vrijedi kao pravilo rada; stavka je u [BACKLOG](../BACKLOG.md) „Otvoreno za Leona".

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): F1: upozorenje 600, greška 1000 za kod (za `.md` vrijedi 1200).

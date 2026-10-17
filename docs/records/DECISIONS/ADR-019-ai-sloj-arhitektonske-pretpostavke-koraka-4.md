# ADR-019 — AI sloj: arhitektonske pretpostavke koraka 4
**Datum:** 2026-10-09 · **Status:** ✅ **Potvrdio Leon 17.10.** (t. 4–5 A13, ostalo F1; pretpostavke Claudea, „radi bez pitanja") · **Vezano:** [ADR-018](ADR-018-leonove-proizvodne-odluke-za-ai-sloj.md),
[ADR-015](ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md), ARHITEKTURA §8.3–8.4, §9.4, §10 · **Plan:** [2026-10-09-ai-sloj.md](../../archive/2026-10-09-ai-sloj.md)

**Kontekst.** Korak 4 dodaje AI (presuda, lik, mentor) na igru iz koraka 3. ARHITEKTURA opisuje cilj, ali ne i kako se
to uklapa u dnevnik, red poslova i rok demoa ~14.10. Ovo su pretpostavke s kojima plan gradi.
**Odluka (pretpostavke):**
1. **Otisak pravila nad kanonskim `Value`** spremljenog snimka; igre iz koraka 3 prolaze preko starog otiska
   (`legacy_fingerprint`). Nova polja sadržaja uvijek uz `serde(default, skip_serializing_if)`. `core_version` se
   provjerava pri obračunu (druga verzija jezgre = 500, ne tihi obračun). Cijena: jedna dodatna grana u foldu.
2. **Kvalitetu računa kod, ne AI:** AI za svaki od pet kriterija kaže ispunjen/ne uz razlog i bira najbliži ponuđeni
   odgovor; kvaliteta = `max(1, broj ispunjenih)`, kao kod ponuđenih (DEMO §7). Presuda bez pet obrazloženih kriterija
   je nevaljana i ponavlja se.
3. **Sve AI uloge idu kroz postojeći red poslova** (lik, presuda, mentor), ne u HTTP zahtjevu: upis poruke i posao u
   istoj transakciji, AI se zove izvan zaključavanja igre, rezultat se upisuje kroz `decide` koji ponovno provjerava
   stanje runde. Cijena: ekran mora pitati za odgovor lika (202 pa čitanje), sekunda-dvije kašnjenja.
4. **Zadnji pokušaj (5.) piše rezervu:** lik napisanu rečenicu, presuda neutralnu (kvaliteta 3, mehanike `unanswered`,
   „na pregled profesoru"), mentor tekst iz brojki. Igra se nikad ne zaglavi zbog AI-ja.
5. **„Zatvori mjesec" čeka presude najviše 120 s** (posao obračuna se odgađa po 5 s), zatim neutralna presuda. Presuda
   koja stigne kasnije zapisuje se kao `VerdictLate` i **ne mijenja obračun**; ponovni obračun / kompenzacija (§8.3 t. 6)
   i profesorova ispravka su korak 6 / drugi val.
6. **Red poslova dobiva odgodu** (`run_after`): neuspio posao čeka 2, 4, 8, 16 s (najviše 60 s); zakup 180 s jer presuda
   može trajati desetke sekundi. Probni test apalis/pgmq i `Idempotency-Key` ostaju u BACKLOG-u (demo ima prirodnu
   idempotenciju: jedan odgovor po situaciji, poruka tek nakon odgovora lika).
7. **Novi crate `ai`** (tanki HTTP klijent, `reqwest` + rustls) implementira port `AiPort` iz `app`; `game` ostaje čist.
   Upute su datoteke `prompts/*.v1.md`, ugrađene pri prevođenju; uz svaki izlaz u dnevnik idu model, verzija upute i
   tokeni. Bez ključa server radi s `DisabledAi` (sve rezerve).
8. **Odgođeno:** tablica `ai_usage` i tvrdi limit troška po igri (prije Railwaya), zaključavanje modela i uputa u snimak
   pravila (prije pilota), AI koji smišlja situacije (pilot).
9. **Likovi (Petra Horvat, gošća; Marko Babić, šef kuhinje), njihovo ponašanje i 40 nacrta odgovora za mjerilo** su
   nacrt; ocjene u mjerilu upisuje Leon.
10. **AI izlaz koji ulazi u drugu uputu (replike lika u presudi) je također podatak, ne uputa** — ide kroz istu zaštitu
   oznaka kao studentov tekst (svaka replika u vlastitoj oznaci, `<` i `>` razoružani, novi red kao `⏎`); upute presude
   zato postaju `presuda.v2` (v1 ostaje u repou radi povijesti).
**Posljedice.** Novi crate `ai`, migracija `0004`, šest novih događaja (`schema_ver` ostaje 1). Ako Leon odbije t. 3,
lik prelazi na izravan poziv u zahtjevu (mijenja se `app/talk.rs` i ruta); ako odbije t. 5, mijenja se jedna konstanta.

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): t. 4–5 (A13 b: rezervni tekst, obračun čeka presude najviše 120 s), ostale točke F1.

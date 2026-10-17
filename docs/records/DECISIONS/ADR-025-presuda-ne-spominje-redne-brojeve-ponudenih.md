# ADR-025 — Presuda ne spominje redne brojeve ponuđenih odgovora ni kriterija
**Datum:** 2026-10-10 · **Status:** ✅ **Potvrdio Leon 17.10.** (A13 e) · **Vezano:** [ADR-018](ADR-018-leonove-proizvodne-odluke-za-ai-sloj.md) t. 10, [ADR-023](ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md)

**Kontekst.** U prolazu s pravim AI-jem 10.10. presuda je studentu napisala *„Odgovor odgovara ponuđenom odgovoru 0"*.
Korisnička poruka presude nosi `[0]`, `[1]`… ispred ponuđenih odgovora (model mora vratiti `nearest_answer` kao broj), a
student na ekranu vidi ponuđene odgovore i kriterije bez brojeva. Leonovo pravilo: svaka brojka za studenta ima ime
i jedinicu, bez internih oznaka.
**Odluka.**
1. **Uputa `presuda.v3`** (v2 ostaje radi povijesti): ni obrazloženje ni razlozi kriterija ne spominju redni broj,
   oznaku ni redoslijed ponuđenog odgovora ili kriterija; ponuđeni odgovor se opisuje riječima.
2. **Brana u kodu (temelj, pretpostavka):** `game::validate_verdict` odbija presudu čiji tekst spominje „odgovor/kriterij
   + broj", „N. kriterij" ili `[N]`/`(N)` (`VerdictError::InternalNumber`; samo padežni oblici imenica, pa
   „odgovornost 5" prolazi). Odbijena presuda ide istim putem kao svaki krivi oblik:
   ponovni AI poziv, a nakon zadnjeg pokušaja neutralna ocjena 3 na pregled profesoru.
**Cijena ako krivo:** rijetko lažno odbijanje (npr. „u odgovoru 2 puta spominješ…") stoji jedan dodatni poziv presude
(≈ 0,008 $); ako se ponovi do zadnjeg pokušaja, student dobije neutralnu ocjenu umjesto AI-jeve. Alternativa (prepisati
broj u tekst ponuđenog odgovora) bila bi tiha izmjena AI-jeva teksta s rizikom krive gramatike.

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): A13 e: presuda ne spominje redne brojeve.

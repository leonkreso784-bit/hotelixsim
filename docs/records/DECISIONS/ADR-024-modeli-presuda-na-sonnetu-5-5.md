# ADR-024 — Modeli: presuda na Sonnetu 5.5, lik na Haikuu 5.5, mentor na Sonnetu 5.5
**Datum:** 2026-10-10 · **Status:** ✅ **Leonova odluka 10.10.** · **Vezano:** [ADR-018](ADR-018-leonove-proizvodne-odluke-za-ai-sloj.md) t. 4 (zamijenjeno),
[ADR-023](ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md)

**Kontekst.** Plan AI sloja imao je Leonovu „odluku 4": jači model (Opus 5.5) za presude, Sonnet za lika i mentora.
Leon 10.10.: *„stavi Sonnet i Haiku koristi, ne treba nam jače od Sonneta da procjenjuje situaciju"*; u anketi je
odabrao lika na Haikuu, mentor ostaje na Sonnetu.

**Odluka:**
1. **Zadani modeli** (`ai::Models::default()`): presuda `claude-sonnet-5-5` (effort `high`), lik `claude-haiku-5-5`
   (effort `low`), mentor `claude-sonnet-5-5` (effort `low`). Varijable okoline `HOTELIXSIM_AI_MODEL_PRESUDA`,
   `_LIK`, `_MENTOR` i dalje nadjačavaju — promjena modela je jedna varijabla.
2. **Razlog i trošak** (po milijunu tokena, ulaz/izlaz): Opus 4/20 $, Sonnet 2/10 $, Haiku 0,10/0,50 $. Presuda
   po rubrici sa strukturiranim izlazom ne traži više od Sonneta; lik je kratka replika.
3. **Haiku 5.5 nema poslužiteljski rezervni model.** Pravilo je po MODELU, ne po vrsti poziva:
   `request::supports_server_fallback(model)` je istina samo za `claude-opus-5-5`, `claude-sonnet-5-5`,
   `claude-opus-5` i `claude-fable-5-1`. Za ostale (svi `claude-haiku-*`, nepoznati) tijelo nema `"fallbacks"` i ne
   šalje se zaglavlje `anthropic-beta: server-side-fallback-2026-07-01` (`request::beta_header`).

**Cijena ako krivo:** presuda je slabije dosljedna. Provjera: alat `mjerilo` kad Leon ocijeni 40 odgovora
(`content/hotel/mjerilo/*.json`); ako treba, vratiti presudu na Opus varijablom okoline.

**Dopuna 13.10.** Sezonski mentor (kraj sezone, uputa `prompts/mentor-sezona.v1.md`) koristi isti model kao mentor
kruga (`models.mentor`, `HOTELIXSIM_AI_MODEL_MENTOR`), jedan poziv po ljudskom hotelu po igri:
[ADR-036](ADR-036-kraj-sezone-poredak-financije-robota-i.md). Uputa mentora kruga je `mentor.v3`
([ADR-023](ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md) dopuna).

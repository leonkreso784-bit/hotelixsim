# ADR-054 — Leonove odluke 16.10. (sesija 6): prag 80 za tri kvara, kopiranje odgovora, smjer iduće sesije
**Datum:** 2026-10-16 · **Status:** ✅ **Leonove odluke 16.10.** · **Vezano:** [ADR-052](ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md) t. 1,
[ADR-047](ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md), [ADR-046](ADR-046-leonove-odluke-15-10-za-fazu-b.md) t. 5,
[ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 11, [plan popravka](../../plan/2026-10-14-popravak-temelja.md) (B4c, PRAVILO)

**1. Uvjet kvarova P-BRAVE, P-CURENJE-VODE i P-DIZALO = Smještaj ispod 80** (`condition_of: rooms`, `below 8000`; anketa, preporuka voditelja).
Rješava otvoreno pitanje iz ADR-052 t. 1. Razlog: hotel A počinje sa Smještajem 75, a dobro održan nikad ne padne ispod; P-BRAVE i P-CURENJE-VODE
njemu su najčešći tjedni predlošci (3,7 % svaki). Svaki prag 50–70 zato **ruši tjednu branu J-3** (krugu krajem sezone ostanu 2 situacije umjesto
najmanje 3, jer su ostali predlošci potrošili `max_per_season`). Mjereno 1000 sjemena, tjedno i mjesečno `--check`:

| inačica | tjedno J-3 | B3 težina tjedno A / B / C | mjesečno A |
|---|---|---|---|
| bez uvjeta (prije) | prolazi | 41,9 / 48,9 / 41,2 % | 25,7 % |
| prag 50–70 | **pada** (2–5 situacija) | 37,2 / 48,9 / 37,4–41,1 % | 24,2 % |
| samo P-DIZALO | prolazi | 40,7 / 48,9 / 40,7 % | 25,3 % |
| **prag 80** | prolazi (3–5) | **41,6 / 48,9 / 41,2 %** | **25,7 %** |

Samo prag 80 prolazi obje brane i sve testove kapaciteta i sjemena; "samo dizalo" ruši pet testova sjemena i ostavlja dug. Tablica i uzrok:
`.superpowers/sdd/2026-10-14-popravak-temelja/b4c-anketa-brojke.md`. P-INSPEKCIJA je vanjski događaj (`external_cause`, ADR-046). Dug `BreakdownCondition` je 0.
Spojeno s B4c (f7ac7cc). **Cijena:** uvjet je blag (hotel A i B počinju ispod 80, kvar izostaje tek kad student Smještaj popravkom digne iznad 80, što je
i pouka "održavanje sprječava kvar"); strožem pragu treba više predložaka (L1-13).

**2. Kvar B13: kopiranje svih ponuđenih odgovora** (Leon, igrajući). Student zalijepi SVE ponuđene odgovore u polje vlastitog odgovora i najčešće dobije 5;
u situaciji s kuharom koji traži povišicu 20 % (sidro S2) nije. Zahtjev: AI mora uvijek prepoznati kopiranje, **uz pažnju na ulazne i izlazne tokene**.
Odluka: prepisani odgovor prepoznaje **kod** (70 % riječi, kartica PRAVILO-a), ne AI, pa je trošak 0 tokena; AI ne dobiva dodatni ulaz. Pravilo za
"zalijepio više odgovora odjednom" nije odlučeno: **anketa na početku iduće sesije**, prije PRAVILO-a (klik odgovarajućeg odgovora / neutralna 3 / najlošija ocjena).
Upisano kao B13 u [BUGS](../BUGS.md).

**3. Smjer iduće sesije (želja, ne odluka o redoslijedu).** Leon: *"idemo na fazu C i dalje s razvojem likova u igri; u sučelju odmah njihov izgled"*.
Plan ([ADR-046](ADR-046-leonove-odluke-15-10-za-fazu-b.md) t. 5) je PRAVILO → L1-13 → L1-13b (tu je "izgled lika po rodu i dobi", ADR-042 t. 11) → L1-14 → L1-15 → C.
Redoslijed je **Leonova odluka**: na početku iduće sesije kratki nacrt "izgled lika" (deterministički iz roda, dobi i uloge, ili slike: cijena, licenca)
pa anketa o redoslijedu s cijenom preskakanja (faza C prije L1-13 ostavlja bonus popravka na +1 i dug `SilenceNotBetter` 16).

**Cijena ako krivo:** (1) jedna konstanta u `condition_of` triju predložaka; (2) pravilo za zalijepljene odgovore je jedna grana koda; (3) redoslijed faza mijenja se u planu.

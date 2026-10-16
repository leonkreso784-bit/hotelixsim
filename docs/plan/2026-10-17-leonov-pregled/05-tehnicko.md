# Leonov pregled · F — tehničko (jedno „ok") i već riješeno

Dio indeksa [2026-10-17-leonov-pregled.md](../2026-10-17-leonov-pregled.md). Ovo su pretpostavke koje student ne osjeti:
kako je kod složen, kako se mjeri, kako rade agenti. Svaka ima zapisanu cijenu ako je kriva; mijenja se lokalno.

### F1 · Tehničke pretpostavke (predlažem „F1 ok" bez čitanja)

| izvor | u jednoj rečenici |
|---|---|
| [ADR-008](../../records/DECISIONS/ADR-008-odluke-iz-gradnje-kostura-koje-je.md) | sedam sitnica iz gradnje kostura (grana, brana ovisnosti, `cargo fmt`, pnpm skripta, README za Windows) |
| [ADR-013](../../records/DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 2–4, 6–9 | pravila prve igre (v1, zamrznuta): početni F, sezonci po planu, redoslijed obračuna, grupna cijena, drugi val |
| [ADR-015](../../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) osim t. 2 | igra kao dnevnik događaja, zaključavanje igre po naredbi, vlastiti red poslova, server ne mijenja bazu sam, tko što smije vidjeti |
| [ADR-017](../../records/DECISIONS/ADR-017-odluke-voditelja-gradnje-u-koraku-3.md) | red poslova pamti tko je preuzeo posao, nepoznat profil = greška, provjera pristupa prije provjere sadržaja |
| [ADR-019](../../records/DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md) osim t. 4–5 | ocjenu računa kod (AI samo kaže koji je kriterij ispunjen), AI ide kroz red poslova, odgođeni limit troška |
| [ADR-026](../../records/DECISIONS/ADR-026-put-b-hotel-od-brojeva-postaje.md) t. 5 | v2 je zaseban paket uz zamrznuti v1 i uvijek daje i sažetak v1 (indeksi, mentor, ekrani rade dalje) |
| [ADR-029](../../records/DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md) P5, P7–P12, P14–P17 | kako se bira v2, budžet, kamo se knjiže troškovi, nazivi upita; robot Brižni planira sezonce unaprijed |
| [ADR-030](../../records/DECISIONS/ADR-030-raspored-dokumentacije-granice-velicine-indeks-mapa.md) | pravila dokumentacije (indeks + mapa, granice veličine, ADR po datoteci); tvoje dopune 1000 i 1200 već su ✅ |
| [ADR-031](../../records/DECISIONS/ADR-031-sloj-1-tjedni-krug-u-jezgri.md) t. 1, 3, 5 | odlasci u tjednu = dio mjesečnih, mjesec situacije, verzija jezgre se diže pri izdanju |
| [ADR-032](../../records/DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 1–8 | kako odjeli daju brojke cijelog hotela (plan sezonaca, troškovi, prosjeci, odlasci po odjelu) |
| [ADR-033](../../records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md) t. 1, 8 | kako balans mjeri i koliko sezona vrti CI |
| [ADR-039](../../records/DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md) | datoteka koda: upozorenje iznad 600 redaka, greška iznad 1000 (za `.md` si rekao 1200; predlažemo 1000 za kod) |
| [ADR-051](../../records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md) t. 1, 4–13 | mjesečni krug kao jedan blok, granica odjela, kako se mjeri težina, popravak u odjelu situacije |
| [ADR-053](../../records/DECISIONS/ADR-053-pretpostavke-16-10-oznake-testovi-b4.md) t. 1–5 | test „sastojak samo u najboljem", testovi čitaju oznake iz sadržaja, tekstove piše najjači model |
| [ADR-055](../../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 2, 4, 5 | voditelj sam popravi jedan redak testa, svako radno stablo svoja mapa prevođenja, mjerenje prije ankete |
| kartica IZGLED, rizik 1, 4, 5, 7, 8 | rod iz popisa imena, engleski nazivi u API-ju, stari izgled bez podataka ostaje isti, sidra S1–S4 ručno |
| nacrt VANJSKI, pretpostavke 1, 5–7, 9–13 | vanjski kreću odmah, polje ustanove, raspodjele, sesija banke, nazivi u kodu (`nacrt-vanjski-report.md`) |
| banka ljudi, provjere stroja | usporedba razine, dobi i roda unutar uloge (ćelija 40 zapisa; `k2-review.md` pitanje 5), pravila banke u šest dijelova |
| web | bočni izbornik na računalu može se preklopiti sa zaglavljem kod velikog teksta: voditelj popravi bez pitanja |

- **Preporuka:** F1 ok. Ako te jedan redak zanima, napiši „F1 ok osim ADR-039" i dobit ćeš ga objašnjenog.

### F2 · Već riješeno: brisanje iz popisa „Otvoreno za Leona"

| stavka | zašto je zatvorena |
|---|---|
| „Put B — 12 odluka prije gradnje" | odgovorio si 10.10. ([ADR-028](../../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md)) |
| „Pravilo za zalijepljene odgovore (B13)" | odlučeno 16.10., sesija 7: neutralna 3 (ADR piše `/kraj`); ostaje samo A6 |
| „Redoslijed: faza C s izgledom likova prije L1-13?" | odlučeno 16.10., sesija 7: IZGLED sada usporedo s PRAVILO-om, plan dalje L1-13 → L1-15 → C |
| ADR-028: „ljude drugih hotela student ne vidi" | zamijenilo [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md): u razredu su tuđi radnici vidljivi i preotimaju se |
| ADR-053 t. 6: prag 65 za tri kvara | odlučeno 16.10.: prag 80 ([ADR-054](../../records/DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md)) |
| ciljna marža GOP 35–45 % (ADR-013 t. 5) | tvoj odgovor 10.10. (ADR-028, odgovor 9); ostaje samo profesorova potvrda (E2) |
| pretpostavke ADR-013, 015, 017, 019 i Sloja 1 kao zasebne stavke | pokrivene s A8, A13 i F1 |

- **Preporuka:** F2 ok; voditelj ih na `/kraj` briše iz BACKLOG-a i u CLAUDE.md skraćuje popis 🟨.

# BANKA_LJUDI §2 · Koliko i kakvih

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md).

## 2.1 Koliko — obrazloženje brojkom

**Radnici u jednoj razrednoj igri** ([ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md):
jedna igra, do ~30 studenata, svaki svoj hotel; tuđi radnici su vidljivi i mogu se preoteti, pa **ista osoba ne smije biti
u dva hotela iste igre**):

| izvor | račun | radnika |
|---|---|---|
| početni ljudi | 30 hotela × ~65 (profili v2 danas: A 67, B 62, C 68, `content/hotel/v2/profiles.json`) | ~1 950 |
| burza rada | ~27 novih kandidata po razdoblju (2–4 po ulozi × 9 uloga, MODEL_V2 §5.1) × 17 tjedana [PROC] | ~450 |
| agencija | 30 hotela × ~6 stranih radnika po sezoni [PROC] | ~180 |
| **ukupno u igri** | | **~2 600** |
| igra za vježbu | 4 hotela × 65 + burza 450 + agencija 25 | ~735 |

**Gosti:** ~65 situacija po sezoni po hotelu [PROC] × ~0,87 uloga po predlošku × 54 % uloga su gosti (danas 26 od 48
uloga u 55 predložaka) ≈ **31 gost po hotelu po sezoni**; razred od 30 hotela ≈ **~930**. Recenzije (~80 mjesečno po hotelu)
ne trebaju zasebne likove: prikazuju ime i zemlju, a uzimaju ih iz iste banke gostiju, s ponavljanjem među hotelima.

**Cilj u tri prekretnice** (🟨 pitanje 6):

| prekretnica | radnika | gostiju | zašto baš tada |
|---|---|---|---|
| **M1** — prije L2a-1 (faza LJUDI, P3) | 800 | 300 | igra za vježbu (~735) i prvi mali razred; gosti za situacije jednog hotela kroz više igara |
| **M2** — prije razrednog pilota (P4) | 2 000 | 1 000 | razred od ~20 hotela bez ponavljanja; ~930 gostiju razreda |
| **M3** — puni razred | 3 000 | 1 500 | 30 hotela (~2 600) uz ~15 % zalihe; gosti uz filtre segmenta i dobi |

Zaliha nije ukras: filtri dijele banku na male bazene („umirovljenica" = 1 500 × 50 % × 18 % ≈ 135 zapisa).

## 2.2 Raspodjele radnika [PROC; izvor u stupcu]

| dimenzija | cilj (udio banke) | izvor ili pretpostavka |
|---|---|---|
| **uloga** | sobar 28 % · kuhar 14 % · konobar 14 % · recepcioner 11 % · uprava 9 % · tehničar 8 % · šanker 7 % · terapeut 5 % · prodaja 4 % | profil C (68 ljudi, `profiles.json`), pomaknuto prema sezonskim ulogama jer ih burza i agencija traže više |
| **razina** | početnik 35 % · iskusan 45 % · stručnjak 20 % | [PROC]; početnik je jeftin i slab (ADR-028 t. 4), stručnjaka treba ~1 po odjelu za voditelja |
| **dob** | 18–24: 22 % · 25–34: 30 % · 35–49: 32 % · 50–65: 16 % | [PROC]; najmanje 10 % početnika starijih od 35 (promjena zanimanja), da „početnik" ne znači „mlad" |
| **rod** | ukupno ž 55 % · m 45 %; **u svakoj ulozi najmanje 25 % svakog roda** | [PROC]; stvarni omjer u domaćinstvu je jače ženski — pitanje 5 |
| **podrijetlo** | lokalno 50 % · regija 25 % · inozemstvo 25 % | [PROC]; lokalno = Kvarner i Istra, regija = ostatak Hrvatske; inozemstvo puni agenciju (pitanje 2) |
| **zemlje (inozemstvo)** | BiH, Srbija, Sj. Makedonija, Nepal, Indija, Filipini, ostalo [PROC] | najčešće zemlje izdanih dozvola za rad u turizmu — **izvor treba potvrditi** (MUP, uz L2b-1) |
| **jezici (drugi)** | engleski: 0–4 = 10/20/35/30/5 % · njemački: 45/20/20/12/3 % · talijanski: 70/15/10/4/1 % | [PROC]; recepcioner ima engleski ≥ 2; isti udjeli za svako podrijetlo osim materinjeg jezika |

## 2.3 Raspodjele gostiju

| dimenzija | cilj | izvor |
|---|---|---|
| **zemlja** | AT 20 · DE 18 · HR 14 · HU 7 · SI 7 · IT 6 · US 4 · NL 3 · RS 3 · PL 3 · UA 2 · SK 2 · BA 2 · CH 2 · CZ 1,5 · UK 1,5 · ostale 4 (%) | [JEZICI_GOSTIJU.md](../../research/JEZICI_GOSTIJU.md), noćenja TZG Opatija 2023.; „ostale" srezane s ~10 na 4 % jer izvorni udjeli s rasponima prelaze 100 % |
| **segment** | par 35 · obitelj 25 · wellness 15 · poslovni 15 · grupa 10 (%) | [PROC]; MODEL_V2 §3.6 nema udjele |
| **dob** | 18–29: 15 · 30–49: 40 · 50–64: 27 · 65+: 18 (%) | [PROC] |
| **rod** | 50 : 50 | predlošci danas 23 ž / 25 m uloga |
| **jezici** | materinji 4; engleski po istoj tablici kao radnici; HR gosti hrvatski 4 | činjenica zemlje; ostalo [PROC] |

Raspodjela je **cilj cijele banke**, ne svake serije. Serija smije odstupati da popuni manjak (§5.3).

## 2.4 Vanjski: koliko i kakvih

**Koliko — obrazloženje brojkom.** U 55 predložaka je 48 uloga, od toga **10 vanjskih** (21 %). Uz ~65 situacija po sezoni
[PROC] i ~0,87 uloga po predlošku to je 65 × 0,87 × 10/48 ≈ **12 vanjskih po hotelu po sezoni**; gornja granica je zbroj
`repeat.max_per_season` vanjskih predložaka = **17**. Kad policija, vatrogasci i hitna dobiju uloge u predlošcima (§1.5),
dolazi još ~3 po hotelu [PROC], pa ~15. Razred od 30 hotela tada treba ~450 vanjskih ako se nitko ne ponavlja među hotelima.

**Ponavljanje među hotelima** 🟨 (pitanje 12): vanjski se **ne preotimaju** (ADR-037 vrijedi za radnike), a u stvarnosti jedan
sanitarni inspektor pokriva cijelo mjesto. Prijedlog: isti vanjski smije doći u dva hotela iste igre, ali izbor ga izbjegava
dok ima slobodnih (§4.4), a banka je dovoljno velika da se to rijetko dogodi.

| prekretnica | vanjskih | zašto |
|---|---|---|
| **M1** 🟨 | 150 | igra za vježbu: 4 hotela × 17 = 68, uz 14 uloga × 2 roda = 28 malih bazena (~5 po bazenu) |
| **M2** 🟨 | 400 | razred od ~20 hotela × ~15 = 300 + zaliha |
| **M3** 🟨 | 600 | 30 hotela × ~15 = 450 + ~30 % zalihe, jer su bazeni (uloga, rod) mali |

Za M3 to je 12 serija od 50. Ako Leon odluči „nitko dvaput u razredu", M3 raste na ~800 (najgori slučaj 30 × 17 = 510 uz
bazene po ulozi i rodu).

**Raspodjele vanjskih** [PROC; izvor u stupcu] — cilj cijele banke, kao §2.2:

| dimenzija | cilj (udio banke) | izvor ili pretpostavka |
|---|---|---|
| **uloga** | dobavljač 14 · poslovni klijent 14 · zastupnik agencije 11 · organizator 7 · susjed 7 · novinar 6 · sanitarni inspektor 6 · policajac 6 · vatrogasac 5 · djelatnik hitne 5 · lokalna uprava 5,5 · turistički, rada i porezni inspektor po 4,5 (%) | 🟨 težina = zbroj `max_per_season` predložaka po ulozi (dobavljač 4, klijent 4, agencija 3…), nove uloge ~1 po sezoni |
| **rod po ulozi** | stvarni omjer, kao radnici (ADR-052 t. 3) | 🟨 isti izvor: Eurostat LFS Hrvatska, ISCO-08 × spol (inspektori 3257, 3352, 3359; policija 5412; vatrogasci 5411; hitna 2211, 3258; novinari 2642; događanja 3332; agencije 4221; lokalna uprava 2422); do mjerenja prijedlog s `"potvrdeno": false`, susjed 50 : 50 |
| **dob** | 20–29: 10 · 30–44: 40 · 45–59: 40 · 60–80: 10 (%) | [PROC]; službe su rijetko vrlo mlade, susjed i klijent mogu biti stariji |
| **zemlja** | službe, dobavljač, susjed: 100 % HR; u tržišnim ulogama ~30 % stranih (≈ 11 % banke) | 🟨; strane zemlje po raspodjeli gostiju (§2.3) bez HR — agencije i novinari dolaze s tržišta gostiju |
| **jezici** | iz HR: `["hr", 4]` svi, drugi jezici po tablici radnika (§2.2); strani: materinji 4, hrvatski 0–2 | 🟨; AI lik ionako govori hrvatski (kao gost) |

**Bazeni koje predlošci traže.** Predložak bira ulogu i rod, pa banka za svaki par koji predlošci danas koriste (inspektor m,
novinarka ž, dobavljač m, organizatorica ž, zastupnica ž, poslovni klijent ž i m, susjed m) na M1 ima **najmanje 5 zapisa**
🟨, bez obzira na omjer roda u ulozi. To provjerava izvještaj prekretnice (K4), ne svaka serija.

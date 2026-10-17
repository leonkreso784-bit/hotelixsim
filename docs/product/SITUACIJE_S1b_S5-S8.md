# Situacije S1b, S5–S8 — nacrt ponuđenih odgovora i rubrika

> **Zatvoreno 17.10. — povijest.** Leon je nacrt zatvorio ([ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md) B5):
> v1 je zamrznut, a v2 ima vlastite predloške. Tekst niže se više ne provjerava ni ne gradi.
> Ništa ovdje nije potvrđeno. Brojke su [PROC] osim ako piše drukčije. Oblik je isti kao S1–S4 u
> [DEMO.md](DEMO.md) §7. Datoteka DEMO.md nije mijenjana.

## Kako čitati

- **Ocjena (q):** broj ispunjenih kriterija; 0 ili 1 ispunjen daje q = 1 (isto kao S1–S4). Ponuđeni odgovori
  ocijenjeni su po istoj rubrici kao slobodan odgovor.
- **Posljedice** koriste samo jedinice iz DEMO §3–§6: `ServiceRecovery` Δ zadovoljstva = 2 × (q − 3);
  `StaffRetention` Δ morala = 3 × (q − 3); `CrisisCommunication` Δ = 3 × (q − 3); kompenzacije i popravci su trošak u
  mjesecu; tragovi `PositiveRecovery` (q ≥ 4), `AngryGuest` (q ≤ 2), `KeyStaffUnhappy` (q ≤ 2), `DeferredMaintenance`,
  `AgencyRelation` (q ≥ 4). Granice (§6.2): zadovoljstvo i moral najviše ±8, ocjena portala najviše ±0,15.
- **Što nije iz kataloga**, pa je posebno označeno: učinak odlaska ključne osobe, mjesečni prosjek soba izvan
  funkcije, promjena ocjene portala iz javnog odgovora. Za te brojke jezgra još nema formulu.
- Stupac „trošak" je zbroj izravnih troškova u mjesecu situacije (ono što bi robot **Štediša** gledao).
- **Pravila robota (§10) primijenjena na ove tablice:** Štediša = najjeftiniji odgovor (kod jednakog troška onaj s nižim
  q) [PROC]; Brižni = najviši q; Lovac = „srednji" = drugi po redu po q (kod S8: prihvaća grupu).

---

## S1b · „Novi ljudi, nove greške"

**Srpanj–rujan** (okida T5, fluktuacija > 6 %) · individualna · recenzije + radna snaga · problem · težina 2 · lik:
gost (AI glumi) · mehanika: `ServiceRecovery`

Obitelj s djetetom čekala je doručak 50 minuta, narudžba je dvaput pogrešno donesena, a na računu je minibar koji nisu
koristili. Voditelj smjene zna da je pola ekipe u hotelu manje od mjesec dana. Gost traži voditelja i kaže da će
„napisati ono što misli".

| # | kriterij |
|---|---|
| 1 | Isprika i preuzimanje odgovornosti u ime hotela, bez prebacivanja krivnje na „novog konobara" |
| 2 | Greške ispravljene odmah, pred gostom (račun, doručak), ne obećanjem |
| 3 | Razmjerna kompenzacija (doručak, piće, mali popust), ne besplatan boravak |
| 4 | Uzrok u sustavu: uhodavanje, mentor uz novog čovjeka, kratak briefing prije smjene, popis provjere; ne kažnjavanje pojedinca |
| 5 | Zaštita zaposlenika (bez javnog prekora) i provjera s gostom kasnije |

| ponuđeni odgovor | kriteriji | q | trošak | posljedice |
|---|---|---|---|---|
| Ispričati se u ime hotela, odmah ispraviti račun i doručak, ponuditi doručak za obitelj, uz svakog novog zaposlenika odrediti mentora i uvesti 10-minutni briefing prije smjene, sutra pitati gosta je li sve u redu | 1 2 3 4 5 | 5 | 120 € kompenzacija + 600 € mentorstvo = 720 € | zadovoljstvo +4; moral +1; iskustvo +1 idući mjesec; trag `PositiveRecovery` |
| Ispričati se, ispraviti račun i doručak, ponuditi doručak za obitelj | 1 2 3 | 3 | 120 € | zadovoljstvo 0; bez traga; uzrok ostaje (T5 se može ponoviti dok fluktuacija ne padne) |
| Ispričati se, ali objasniti da je konobar nov i da će dobiti ukor; ispraviti račun; ponuditi piće | 2 3 | 2 | 60 € | zadovoljstvo −2; moral −2 (tim čuje); trag `AngryGuest` |
| Ispričati se, ispraviti račun i ponuditi besplatan ostatak boravka (3 noći) | 1 2 | 2 | 525 € | zadovoljstvo −2; trag `AngryGuest` |

Dileme: drugi odgovor je „dovoljno dobar" i jeftin, ali ostavlja uzrok. Treći djeluje odlučno, a plaća se
moralom. Četvrti ostavlja gosta zadovoljnim, ali je nerazmjeran (isti q kao u S1 „vratiti cijeli boravak" bi bio 1; ovdje
je 2 jer su ispričan i ispravljen račun).

**Nesigurno:** zašto gost koji je dobio besplatan boravak dobiva trag `AngryGuest`? Pravilo kaže q ≤ 2, ali stvarno
gost je zadovoljan. Možda treba druga naknada ili drugi trag [PROC, pitanje za Leona].

---

## S5 · „Pukao je bojler — nema tople vode u krilu B"

**Srpanj–rujan** (okida T1, trag `DeferredMaintenance`) · individualna · operativa + gosti · problem · težina 3 · lik:
više nezadovoljnih gostiju, recepcija (AI glumi voditelja recepcije) · mehanike: `EquipmentRepair` +
`ServiceRecovery`

U krilu B (12 soba) puknuo je bojler. Recepcija prima pozive od jutra, a tehničar kaže da stari bojler nitko nije
servisirao već dvije sezone. U kolovozu je bazna popunjenost 83 %, pa se goste teže seli nego u rujnu.

| # | kriterij |
|---|---|
| 1 | Gosti saznaju od hotela prije nego sami nazovu; isprika |
| 2 | Trenutno rješenje za goste: preseljenje, topli tuš drugdje (wellness, bazen), ne obećanje „sutra" |
| 3 | Razmjerna kompenzacija (po sobi i noći), ne popust na cijeli boravak |
| 4 | Odluka o popravku razmjerna riziku, **s cijenom i rokom** (hitno / redovno / privremeno) |
| 5 | Uzrok i prevencija: zašto je pukao (odgođeno održavanje) i provjera ostalih bojlera |

Troškovi popravka [PROC]: hitna zamjena 18 000 € (2 dana, nova oprema); redovni servis 12 000 € (10 dana); privremena
zakrpa 4 000 € (1 dan, zakrpa se vraća kao kvar).

| ponuđeni odgovor | kriteriji | q | trošak | posljedice |
|---|---|---|---|---|
| Obavijestiti goste, preseliti ih odmah, 1 500 € kompenzacije, hitno zamijeniti bojler i provjeriti sve ostale | 1 2 3 4 5 | 5 | 18 000 + 1 500 = 19 500 € | zadovoljstvo +4; oprema +6; sobe izvan funkcije ≈ +1 u mjesecu; trag `PositiveRecovery` |
| Obavijestiti goste, preseliti ih, 800 € kompenzacije, privremeno zakrpati bojler | 1 2 3 | 3 | 4 000 + 800 = 4 800 € | zadovoljstvo 0; oprema +2; sobe izvan funkcije ≈ +0,4; trag `DeferredMaintenance` |
| Obavijestiti goste, preseliti ih, 600 € kompenzacije, ugovorni servis popravlja za 10 dana, krilo B zatvoreno | 1 2 3 | 3 | 12 000 + 600 = 12 600 € | zadovoljstvo 0; oprema +6; sobe izvan funkcije ≈ +3,9 u mjesecu |
| Ostaviti goste u sobama, 30 % popusta na noćenje u krilu B tijekom 10 dana, čekati ugovorni servis | 1 3 | 2 | 12 000 + 6 300 = 18 300 € | zadovoljstvo −2; oprema +6; trag `AngryGuest` |

Dileme: prvi je bolji za goste, ali 19 500 € za profil A (blagajna ≈ 120 000 €) je velik udarac. Drugi je jeftin, ali
ostavlja trag koji može izazvati novi kvar. Treći je skoro jednako skup kao prvi, a vrijedi samo uz slobodne sobe.
Četvrti je „najjeftiniji u glavi", a u računu najskuplji uz lošu ocjenu.

**Nesigurno:** mjesečni obračun ne vidi vršne dane; 3,9 sobe izvan funkcije pri bazi 83 % u modelu ne gubi prodaju,
iako u stvarnoj špici bi (kolovoz). To čini treći odgovor jeftinijim nego što jest. Mogući lijek: prag u jezgri koji
računa prodaju na zadnjem tjednu mjeseca [PROC].

---

## S6 · „Voditeljica recepcije daje otkaz usred sezone"

**Srpanj–rujan** (okida T2, moral < 40 ili `KeyStaffUnhappy`) · individualna · radna snaga · mješovito · težina 3 ·
lik: voditeljica recepcije (AI glumi) · mehanika: `StaffRetention`

Voditeljica recepcije, osam sezona u hotelu, donosi otkaz s mjesečnim otkaznim rokom. Kaže da je „iscrpljena" i da joj
je druga ponuda stigla prije dva tjedna. Recepcija radi 12 sati bez pauze, a ona je istodobno i voditeljica i
zamjena kad netko izostane.

| # | kriterij |
|---|---|
| 1 | Razgovor i razlog prije odluke (plaća, opterećenje, priznanje), bez reagiranja u afektu |
| 2 | Odgovor na pravi razlog, ne samo novcem: raspored, slobodni dani, zamjenik, odgovornost |
| 3 | Kontinuitet recepcije: primopredaja, zamjena iz tima, ostali se ne prebacuju preko granice opterećenja |
| 4 | Pravednost: bez presedana „tko zaprijeti odlaskom, dobiva", jasna pravila |
| 5 | Komunikacija timu: bez glasina, priznanje njezina doprinosa |

| ponuđeni odgovor | kriteriji | q | trošak | posljedice |
|---|---|---|---|---|
| Razgovor; 6 % povišice samo njoj, dva slobodna dana tjedno, dodatni sezonac na recepciji, ona bira zamjenicu i uči je; tim saznaje što se mijenja i zašto | 1 2 3 4 5 | 5 | 130 €/mj + 1 500 € zapošljavanje + 2 200 €/mj = ≈ 3 830 € prvi mjesec | moral +6; ostaje; trag `KeyStaffUnhappy` se briše |
| Razgovor; 15 % povišice i dva slobodna dana, bez ičega drugog | 1 2 | 2 | 330 €/mj | moral −3 (drugi saznaju); ostaje; trag `KeyStaffUnhappy` |
| Prihvatiti odlazak, odmah unaprijediti zamjenicu iz tima uz 8 % povišice, obavijestiti tim | 3 4 5 | 3 | 176 €/mj | moral 0; odlazi; iskustvo −2 (pad iskustva ključne osobe) [PROC] |
| Skinuti je s dužnosti istog dana i odmah angažirati agencijsku recepciju | 3 | 1 | 4 000 €/mj | moral −6; odlazi; trag `KeyStaffUnhappy` |

Dileme: drugi odgovor „kupuje" mir, ali stvara presedan koji tim čita. Treći je razuman, ali gubi iskusnu osobu usred
sezone. Četvrti djeluje „sigurno" jer recepcija ne ostaje bez ljudi.

**Nesigurno:** KeyStaffUnhappy kod q ≤ 2 vrijedi i kad osoba ostaje; stvarno bi trag ovisio o ishodu. Učinak odlaska
ključne osobe (iskustvo −2) nije u katalogu [PROC]. Je li razlika u odnosu na S2 dovoljna (kuhar traži, ona odlazi)?
Moguće spojiti u jednu situaciju s dvije varijante.

---

## S7 · „Loša recenzija se širi"

**Srpanj–rujan** (okida T3, trag `AngryGuest` ili pad ocjene ≥ 0,2) · individualna · recenzije · problem · težina 3 ·
lik: nema (javni odgovor, piše se tekst) · mehanika: `ServiceRecovery` (javni odgovor)

Recenzija 2 / 10 na glavnom portalu: gost piše o „prljavom bazenu, hladnoj kavi i nepristojnoj recepciji". Dio je
točan (bazen je jedan dan bio zatvoren zbog kemije), dio pretjeran. Recenziju je podijelilo još nekoliko ljudi.

| # | kriterij |
|---|---|
| 1 | Odgovor brzo (do 48 sati), javno, uz zahvalu na povratnoj informaciji |
| 2 | Ton bez obrane i prepirke; priznaje što je točno; ne otkriva podatke o gostu |
| 3 | Konkretnost: što je hotel promijenio (ne šablona „žao nam je zbog neugodnosti") |
| 4 | Poziv na privatan razgovor (kontakt osobe, ne opće adrese) |
| 5 | Interni zaključak: uzrok i promjena (bazen, kava, recepcija), ne samo tekst odgovora |

| ponuđeni odgovor | kriteriji | q | trošak | posljedice |
|---|---|---|---|---|
| U 24 h javno odgovoriti: zahvala, priznati bazen, reći što se promijenilo (dvije dnevne provjere kemije), ponuditi razgovor s direktorom; interno: sastanak o bazenu i recepciji | 1 2 3 4 5 | 5 | 1 500 € (odgovaranje) + 200 € = 1 700 € | zadovoljstvo +4; ocjena portala +0,10; trag `PositiveRecovery` |
| Poslati standardni odgovor odmah: isprika, zahvala, poziv da nas kontaktira | 1 2 4 | 3 | 0 € | zadovoljstvo 0; ocjena +0,03 |
| Javno ispraviti činjenice: „bazen je radio, osoblje je ljubazno, gost je zakasnio na doručak" | 1 | 1 | 0 € | zadovoljstvo −4; ocjena −0,05 (javna prepirka); trag `AngryGuest` |
| Privatno ponuditi gostu besplatan boravak za povlačenje recenzije | 4 | 1 | 350 € | zadovoljstvo −4; ocjena −0,05 (portali to zabranjuju, rizik otkrivanja) [PROC]; trag `AngryGuest` |

Dileme: standardni odgovor je brz i neutralan, ali ne uči hotel ništa. Treći odgovor je instinkt vlasnika. Četvrti je
poznata praksa koja je protiv pravila portala [PROC, provjeriti].

**Nesigurno:** katalog nema formulu za promjenu ocjene portala iz javnog odgovora (osim `ReplyToReviews` +0,05 mjesečno);
brojke ocjene su moj prijedlog unutar granice ±0,15. Pravilo Štediše: odgovori 2 i 3 imaju trošak 0, razrješava se nižim q.

---

## S8 · „Gost se vraća s preporukom" — prilika

**Rujan** (okida T4, trag `PositiveRecovery` ili `AgencyRelation`; ako ništa ne okine, dolazi svejedno, DEMO §7
raspored) · individualna · financije · prilika · težina 3 · odgovor **brojkama** (prihvati / odbij / protuponuda) ·
mehanika: `GroupBooking`

Gošća iz ranije situacije (kojoj je hotel pomogao) sada je voditeljica ljudskih resursa tvrtke. Traži **25 soba na 4
noći u rujnu** (100 soba-noći), uz 20 % popusta na referentnu cijenu (135 €), izravno, bez agencije.

Račun (rujan, bazna popunjenost 73 %, 120 sobe): prosječno ≈ 88 soba prodano pojedinačno, ≈ 32 slobodne. Grupa od 25 stane
bez istiskivanja, ali ostaje malo prostora (7 soba). Ta ista ponuda u kolovozu bila bi istiskivanje, kao S3.

| # | kriterij |
|---|---|
| 1 | Prepoznaje da je rujan miran mjesec (bazna popunjenost 73 %), grupa popunjava prazno i ne istiskuje |
| 2 | Vrijednost grupe: marža sobe + ostali prihod (30 %) − CPOR (22 €), u odnosu na dani popust |
| 3 | Protuponuda koja čuva cijenu: manji popust za više noći, ili termin sredinom tjedna, ili paket (doručak, dvorana) |
| 4 | Uvjeti: depozit, rok otkaza, rooming lista, rok za puštanje neprodanih soba |
| 5 | Odnos: zahvala na preporuci, mogućnost godišnje suradnje |

| ponuđeni odgovor (broj soba × noći, popust) | kriteriji | q | prihod soba / doprinos | posljedice |
|---|---|---|---|---|
| Protuponuda: 25 × 4, popust 15 %, utorak–petak sredinom rujna, depozit 30 %, otkaz do 14 dana prije, paket doručka; zahvala i prijedlog godišnjeg ugovora | 1 2 3 4 5 | 5 | q ≥ 4 smanjuje popust za 5 p. b. pa je **efektivni popust 10 %**: 12 150 € + 3 645 € ostalo − 2 200 € CPOR ≈ **13 595 €** | trag `AgencyRelation` |
| Prihvatiti kako je: 25 × 4, popust 20 %, bez uvjeta; zahvala | 1 5 | 2 | 10 800 € + 3 240 € − 2 200 € ≈ **11 840 €** | bez traga |
| Protuponuda: 40 × 4, popust 25 %, zahvala | 1 5 | 2 | 40 soba preko ≈ 32 slobodne istiskuje ≈ 30 soba-noći pojedinačnih (≈ 4 700 €): 16 200 € + 4 860 € − 3 520 € − 4 700 € ≈ **12 900 €** | bez traga |
| Odbiti: „u rujnu smanjujemo osoblje i ne primamo grupe" | — | 1 | 0 € | bez traga; gubi se prilika |

Dileme: treći odgovor izgleda najbolje po bruto prihodu (17 540 €), a po računu je tek blizu drugog. Jezgra ga računa
prema pravilima (§3.1), a rubrika ocjenjuje samo obrazloženje, pa bi dobar student mogao argumentirati i veću grupu.
Četvrti ima smisla kad su sezonci već otpušteni (kapacitet osoblja 45 × 30 × 2,4 = 3 240 soba-noći, ostaje dovoljno).

**Nesigurno:** je li 20 % popusta za direktnu korporativnu grupu u rujnu realno (S3 ima isto pitanje za agenciju)?
Pravilo „q ≥ 4 smanjuje popust 5 p. b." čita se kao smanjenje traženog popusta; ako bi se čitalo na protuponudu,
brojke se mijenjaju [PROC]. Robot Lovac prihvaća drugi odgovor.

---

## Tablica za robote (§10)

| situacija | Štediša (najjeftiniji) | Brižni (najviši q) | Lovac (srednji / prihvaća) |
|---|---|---|---|
| S1b | treći (60 €) | prvi (q5, 720 €) | drugi (q3) |
| S5 | drugi (4 800 €) | prvi (q5, 19 500 €) | treći (q3, 12 600 €) |
| S6 | treći (176 €/mj) | prvi (q5) | treći (q3) |
| S7 | drugi (0 €, q3) | prvi (q5) | drugi (q3) |
| S8 | četvrti (0 €) | prvi (q5) | drugi (prihvaća) |

Provjera §11.3: u ovim situacijama Brižni troši više, a Štediša plaća trag (S1b, S5, S7). Ne bih ovo smatrao
balansom, samo ulazom u alat za balans.

## Najnesigurnije odluke (za Leona)

1. **Brojke posljedica izvan kataloga:** odlazak ključne osobe (S6), javni odgovor na ocjenu (S7), prosjek soba izvan
   funkcije (S5). Sve su moj prijedlog.
2. **q ≤ 2 uvijek daje `AngryGuest`**, čak i kad je gost „kupljen" (S1b, četvrti odgovor). Je li to željeno?
3. **Kompenzacije i cijene popravka** (120 €, 18 000 €, 12 000 €, 4 000 €): nisu iz izvora, [PROC].
4. **Kriteriji svih pet situacija** su tvoje područje: osobito S6 (odlazak, ne povišica) i S7 (je li „poziv na
   privatni razgovor" u stvarnosti standard) i S8 (20 % za izravnu grupu u rujnu).
5. **Dileme u S5 i S8** računaju se po prosjeku mjeseca; prave špice nema (§3.1).

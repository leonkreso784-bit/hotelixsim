# DEMO §7 — situacije s rubrikama

Dio [DEMO.md](../DEMO.md)

## 7 · Situacije s rubrikama

Četiri osnovne situacije. Rubrika ima **pet kriterija**, a ocjena je broj ispunjenih (0 ili 1 ispunjen → 1).
Ponuđeni odgovori ocijenjeni su po istoj rubrici (ARHITEKTURA §10.3). Imena likova su izmišljena.

### S1 · „Klima ne radi, a vani je 34 °C"

**Lipanj** · individualna · recenzije + održavanje · problem · težina 2 · lik: gošća u sobi 214 (AI glumi)
· mehanike: `ServiceRecovery` + `EquipmentRepair`

Gošća s dvoje djece zove recepciju treći put. Klima u sobi ne radi od jutra i prijeti recenzijom.

| # | kriterij |
|---|---|
| 1 | Brza isprika i preuzimanje odgovornosti, bez prebacivanja krivnje (vrućina, tehničar) |
| 2 | Trenutno rješenje: druga soba ili prijenosni uređaj, odmah |
| 3 | Razmjerna kompenzacija (večera, popust na noćenje), ne povrat cijelog boravka |
| 4 | Uzrok se popravlja i prijavljuje održavanju |
| 5 | Provjera kasnije (follow-up) |

| ponuđeni odgovor | kriteriji | q |
|---|---|---|
| Ispričati se, odmah premjestiti obitelj, ponuditi večeru, poslati tehničara, sutra nazvati gošću | 1 2 3 4 5 | 5 |
| Ispričati se, premjestiti obitelj i poslati tehničara | 1 2 4 | 3 |
| Vratiti novac za cijeli boravak | 1 | 1 |
| Objasniti da su zbog toplinskog vala svi uređaji opterećeni i poslati tehničara kad stigne | 4 | 1 |

### S2 · „Glavni kuhar traži povišicu ili odlazi"

**Srpanj** · individualna · radna snaga · mješovito · težina 3 · lik: glavni kuhar (AI glumi) · mehanika:
`StaffRetention`

Usred sezone glavni kuhar traži 20 % veću plaću. Konkurent mu je dao ponudu.

| # | kriterij |
|---|---|
| 1 | Saslušati ga i otkriti pravi razlog (plaća, opterećenje, priznanje) |
| 2 | Odluka u okviru budžeta, s obrazloženjem |
| 3 | Pravednost prema ostatku tima: bez presedana koji ruši moral drugih |
| 4 | Nenovčani elementi: raspored, odgovornost, priznanje, razvoj |
| 5 | Plan B: zamjenik ili nasljednik iz tima |

| ponuđeni odgovor | kriteriji | q |
|---|---|---|
| Razgovor; 8 % povišice i bonus na kraju sezone; on bira i obučava zamjenika | 1 2 3 4 5 | 5 |
| Bonus na kraju sezone ako ostane, bez razgovora | 2 5 | 2 |
| Odmah dati 20 % | 1 | 1 |
| Odbiti: „nitko nije nezamjenjiv" | — | 1 |

### S3 · „Agencija nudi grupu za kolovoz"

**Lipanj** · individualna · financije · prilika · težina 3 · odgovor **brojkama** (prihvati / odbij / protuponuda) ·
mehanika: `GroupBooking`

Agencija nudi 25 soba na 7 noći **u kolovozu** (175 soba-noći) uz 25 % popusta na referentnu cijenu.

| # | kriterij |
|---|---|
| 1 | Prepoznaje da je kolovoz vrh sezone (hotel je rasprodan), pa grupa istiskuje skuplje goste |
| 2 | Uspoređuje vrijednost grupe s vrijednošću istisnutih individualnih gostiju |
| 3 | Protuponuda: manji popust, manje soba ili pomak na lipanj ili rujan |
| 4 | Uvjeti: depozit, rok otkaza, rok za puštanje neprodanih soba |
| 5 | Ukupna vrijednost: potrošnja grupe u hotelu i budući posao s agencijom |

Brojke računaju **pravila** (§3.1), ne rubrika. Rubrika ocjenjuje obrazloženje i daje mali bonus na uvjete. Lekcija
koju pravila sama pokažu: ista ponuda je dobra u lipnju, a loša u kolovozu.

### S4 · „Toplinski val i ograničenja vode" — kolektivna

**Kolovoz** · **kolektivna** (svi hoteli dobiju istu) · operativa + sigurnost · problem · težina 4 · mehanika:
`CrisisCommunication`

Vodovod najavljuje ograničenja potrošnje vode na tjedan dana usred toplinskog vala.

| # | kriterij |
|---|---|
| 1 | Zdravlje gostiju i osoblja prvo (voda za piće, ranjive skupine, rashlađivanje) |
| 2 | Jasna i iskrena obavijest gostima, prije nego što sami primijete |
| 3 | Štednja koja ne uništava boravak (praonica, ručnici, bazen), a ne gašenje tuševa |
| 4 | Rezervni plan s cijenom (cisterna, dobavljač) |
| 5 | Briga o osoblju (smjene, pauze, vrućina) |

Ista situacija pogađa hotele **različito**: hotel s lošom opremom ili preopterećenim ljudima trpi više. To je pouka
iz postojećih simulacija: zajednički šok, različit učinak [IZVOR].

### Situacije iz spleta okolnosti

Ove situacije nastaju samo kad ih okine trag (§9). Rubrike se pišu poslije prve četiri, istim oblikom.

| | situacija | okidač | mehanike |
|---|---|---|---|
| S5 | „Pukao je bojler — nema tople vode u krilu B" (12 soba izvan funkcije) | T1 | `EquipmentRepair` + `ServiceRecovery` |
| S6 | „Voditeljica recepcije daje otkaz usred sezone" | T2 | `StaffRetention` |
| S7 | „Loša recenzija se širi" | T3 | `ServiceRecovery` (javni odgovor) |
| S8 | „Gost se vraća s preporukom" — **prilika** (poslovna grupa u rujnu) | T4 | `GroupBooking` |
| S1b | „Novi ljudi, nove greške" — pritužba zbog neiskusnog osoblja | T5 | `ServiceRecovery` |

### Raspored

| runda | osnovni utori | utori iz spleta |
|---|---|---|
| lipanj | S1, S3 | — |
| srpanj | S2 | najviše jedan |
| kolovoz | S4 | najviše jedan |
| rujan | — | najviše dva; ako ništa ne okine, dolazi S8 kao prilika |

Najviše dvije situacije po rundi po hotelu. Kad se okine više tragova nego što ima mjesta, prednost ima veća
težina, a kod jednake težine kocka iz sjemena.

---

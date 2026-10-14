# ADR-042 — Leonove odluke iz pregleda igre 14.10. (kvarovi, održavanje, osoblje, spremanje, banka ljudi)
**Datum:** 2026-10-14 · **Status:** ✅ **Leonova odluka 14.10.** (anketa i chat) · **Vezano:** [ADR-028](ADR-028-leonove-odluke-za-put-b-ljudi.md),
[ADR-038](ADR-038-nacin-rada-dva-kolosijeka-faze-i.md), [MODEL_V2.md](../../architecture/MODEL_V2.md) §3

**Kontekst.** Leon je odigrao mjesečnu v2 igru na grani situacija i javio pet nalaza. (1) Kvar „Stala pumpa" došao je dva mjeseca
zaredom iako je u prvom poslao tehničara: hitan popravak (`EquipmentRepair { urgent: true }`, `crates/sim-hotel/src/mechanic.rs`)
stoji 250 €, a stanje odjela ne mijenja (privremeni ga spušta 1 bod), pa uzrok ostaje ispod praga 60. (2) Pojačano održavanje
Bara uz bazen daje neto +1,5 boda mjesečno (+6 prema trošenju 4,5) — prespor za izlazak iz zone kvarova u sezoni. (3) Igrač ne
vidi koliko ljudi odjel treba prije odluke (jezgra zna kapacitet po radniku i preporučeni broj sezonaca; ekran pokazuje samo
opterećenje nakon kruga). (4) Svaki odjel ima svoj gumb „Spremi". (5) Plaće su jedan postotak za cijeli hotel, bez povišica i
bez osoba (to je faza LJUDI).

**Odluke.**
1. **Dobar odgovor na kvar popravlja uzrok:** hitan popravak u situaciji odjela diže stanje tog odjela za oko **+10 bodova**
   (trošak ostaje); slab/privremeni ne. Isti kvar se ne vraća dok stanje odjela opet ne padne ispod praga predloška. Samo v2.
2. **Pojačano održavanje jače:** neto oko **+5 bodova mjesečno** (odjel s 50 izađe iznad 60 za ~2 mjeseca). Ide kroz zajednički
   balans (L1-13).
3. **Pregled potrebnih radnika:** raspon uz očekivane goste („Za očekivanih ~180 gostiju dnevno treba oko 14–16 ljudi; imaš 12.")
   — ali **najprije** se po odjelu mora vidjeti koji su to radnici (osobe), dakle nakon temelja ljudi.
4. **Jedan gumb „Spremi plan"** sprema plan i odluke svih odjela odjednom; zatvaranje kruga i dalje upozori na nespremljeno.
5. **Banka ljudi (Leon, chat):** napraviti **jako puno** likova — radnike i goste, **~2000 ili više** — i to **strukturno, korak po
   korak, oprezno**, kroz cijeli razvoj; ne moraju se odmah svi koristiti. Rade **dva agenta posvećena samo tome**. Prvi korak je
   dokument formata i pravila (arhitektura kroz dokument, ADR-006), pa provjera sadržaja, pa punjenje u serijama.

**Posljedice.** Odluke 1, 2 i 4 diraju jezgru, balans i ekrane Plana koje mijenja i grana obnove — rade se **nakon spajanja**
grana (L1-10, L1-11), prije igrive verzije (L1-14). Odluka 3 čeka banku ljudi i fazu LJUDI. Banka ljudi (5) je sadržaj i
smije krenuti odmah, paralelno, jer ne dira jezgru igre; njen format mora pratiti `Zaposlenik` iz MODEL_V2 §3.3 i pravilo o
nacionalnosti iz ADR-028.

**Dopuna 14.10. (Leon, chat, nakon druge igre).**
6. **Razgovor s likom završava unutar 6 poruka.** Leon je u lipnju (P-CURENJE-VODE) poslao 6 poruka, a gošća je u svakom odgovoru
   postavila novo pitanje i razgovor nije završio. Uzrok: prompt `lik.v2` i `CharacterInput` ne znaju za granicu
   `MAX_CHAT_MESSAGES = 6` (`crates/game/src/talk.rs:16`). Popravak: nova verzija prompta (`lik.v3`) koja dobiva redni broj poruke
   i granicu; u zadnje dvije poruke lik ne otvara nova pitanja, a u zadnjoj zaključi (prihvati rješenje ili jasno kaže što ga i
   dalje muči). Živi test na mjerilu prije zamjene.
7. **Kvar dolazi samo kad je stanje loše — za SVE kvarove, ne samo za pumpu.** Leonovo načelo iz t. 1 („dobro održan odjel nema
   kvarova koji se ponavljaju") vrijedi za svih 15 predložaka s popravkom. Danas uvjet stanja imaju samo 4 (bojler, pumpa, lobby
   kava, dotrajalo; klima nema), a P-CURENJE-VODE, P-BRAVE, P-PRAONICA, P-DIZALO, P-TERMINAL, P-DOBAVLJAC dolaze bez obzira
   na održavanje. ✅ Veza (Leon potvrdio 14.10.): curenje, brave, klima, praonica, dizalo → stanje Smještaja (zgrada i sobe); terminal → bez
   uvjeta (oprema, nije prostor); dobavljač nije kvar prostora (ostaje bez uvjeta); oluja, toplinski
   val i nestanak struje su vanjski događaji (bez uvjeta). Ide uz t. 1 i 2, nakon spajanja, kroz balans.
8. **Banka ljudi — Leonovi odgovori na pitanja iz [BANKA_LJUDI.md](../../architecture/BANKA_LJUDI.md) §6 (14.10.):**
   gosti s imenom i prezimenom (1); 25 % stranih radnika za sada, kasnije moguća **agencija za strane radnike** (2, vidi L2b-3);
   dob radnika 18–65, gostiju 18–85 (3); kratka priča do 2 rečenice, samo činjenice (4); **rod po ulozi po stvarnom omjeru u
   hotelijerstvu**, ne najmanje 25 % (5 — preporuka odbijena; izvor omjera treba zabilježiti); tri koraka M1–M3 (6);
   **nitko u razredu nema istog radnika** — isti profil dobiva iste razine, različite ljude (7); pitanje 8 (tko određuje
   vještine i osobine): **mješavina (c)** — agent uz priču upiše razinu iskustva (početnik / iskusan / stručnjak), točne
   vještine i osobine unutar razine izvlači igra po istim pravilima za sve; pregled: svih 100 prve serije, zatim uzorak 10 po seriji (9); partneri kao
   treća vrsta zapisa nakon M1 (10); radnici iz BiH, Srbije i Crne Gore razumiju hrvatski na razini **4** (11).
9. **Nova želja:** student može **sam nazvati svoj hotel** (ime vidljivo kolegama u razredu i na kraju sezone) — cigla za
   kolosijek PLATFORMA/IGRA uz razred (ADR-037); provjera imena (duljina, pristojnost) 🟨.
10. **Ponuđeni odgovori moraju biti izmiješani (Leon 14.10.).** Provjereno: u svih 55 predložaka i u sidrima S1–S4 najbolji
    odgovor (ocjena 5) je **prvi**, a u 53 od 55 predložaka odgovori idu silazno po ocjeni — student nauči „uvijek prvi".
    Popravak: redoslijed prikaza miješa se **po hotelu** (stalan za isti hotel i situaciju, različit među kolegama u razredu,
    iz sjemena igre), a događaji i dalje pamte izvorni broj odgovora — pa se balans, roboti i stari zapisi ne mijenjaju. API
    vraća odgovore u prikaznom redu s izvornim brojem; web šalje izvorni broj. ✅ Druga pojava (Leon 14.10.: ujednačiti): najbolji odgovor je često i
    **najdulji** (npr. 5 kriterija u jednoj rečenici prema 1) — studentu to odaje odgovor i nakon miješanja; **duljine oznaka se
    ujednačuju** u sadržaju (slabiji odgovori zvuče jednako uvjerljivo; provjera sadržaja mjeri razliku duljina).
11. **Izgled lika mora odgovarati liku (Leon 14.10.).** Danas `lookFor(ime)` (`apps/web/src/avatar/look.ts`) računa izgled samo iz
    imena: ne zna rod, dob ni podrijetlo, pa gošća može dobiti bradu, gost žensku frizuru, a lik s hrvatskim imenom tamnu put.
    Popravak: API šalje uz lika **rod, dobnu skupinu i podrijetlo** (već postoje u izvlačenju uloga i u banci imena), a izgled
    se računa iz njih: rod određuje bradu, frizuru i naušnice; dob skupinu (mlad / odrasla osoba / stariji); podrijetlo
    **raspon boje kože prema stvarnom stanovništvu** (hrvatska i regionalna imena — svijetli tonovi; strani gosti po zemlji).
    Ime i dalje bira detalje unutar toga (stalan izgled za istog lika). Ide uz banku ljudi ([BANKA_LJUDI.md](../../architecture/BANKA_LJUDI.md)):
    rod, dob i podrijetlo su obvezna polja svakog lika.
12. **Ista situacija najviše jednom u krugu — i u tjednu i u mjesecu (Leon 14.10.).** U mjesečnoj igri izvlačenje ide po
    virtualnim tjednima (`crates/sim-hotel-v2/src/situations/draw.rs`, `virtual_weeks`), a razmak ponavljanja od 4 tjedna
    dopušta da isti predložak dođe dvaput u istom mjesecu (Leonova igra: `P-PRAONICA.jul` i `P-PRAONICA.jul.2`). Popravak:
    predložak koji je već izvučen u krugu ne ulazi u izbor do kraja tog kruga (mjesečni krug = jedan mjesec). U tjednoj igri
    to već vrijedi. **Iz kruga u krug ponavljanje je dopušteno** (npr. pukla cijev dva mjeseca zaredom) kad je prostor u
    lošem stanju (t. 1, 7) — zabranjeno je samo dvaput u istom krugu. Mijenja mjesečno izvlačenje, pa ide kroz balans (mjesečni v2 1000 `--check`).
13. **Mjesečna igra ima svoj, manji broj situacija (Leon 14.10., anketa) — mijenja odluku P3 od 13.10.** („razmjerno danima,
    ~13–22 po mjesecu"). Leon: „previše situacija; tjedan i mjesec moraju biti razdvojeni, u mjesecu manje, ne da se računa
    kao da su tjedni prošli". Novo: mjesečni krug **4 / 5 / 6** situacija po očekivanoj popunjenosti (ispod 75 % / 75–89 % /
    od 90 %), sezona ~20; tjedni ostaje 3 / 4 / 5. Prve 3 obvezne, ostale se smiju prepustiti voditelju (P4). Situacije i dalje
    nose ~⅓ napretka sezone (P8), pa učinak jedne situacije u mjesečnoj igri raste — faktor određuje zajednički balans (L1-13).
    Dira `crates/sim-hotel-v2/src/situations/count.rs` (`period_count`, `virtual_weeks`) i izvlačenje; balans v2 mjesečni 1000 `--check`.
14. **Profesor određuje broj situacija (Leon 14.10.).** Brojevi iz t. 13 (tjedno 3/4/5, mjesečno 4/5/6) su **zadani**, ne
    zakucani: profesor ih pri otvaranju igre (i između krugova, za sljedeće krugove) smije promijeniti, isto za sve hotele u
    igri. **Temelj odmah:** kad se gradi t. 13, brojevi idu u **postavke igre** zapisane u događaju stvaranja igre
    (`GameCreated`, `serde(default)` = zadani brojevi, pa stare igre ostaju iste), a ne u konstantu u kodu — kasniji ekran
    profesora (kolosijek PLATFORMA, razred ADR-037) samo ih postavlja. ✅ (Leon potvrdio) Granice: najmanje 3 (obvezne), najviše 8 po krugu;
    učinak jedne situacije se preračuna tako da situacije ostanu ~⅓ napretka (P8) bez obzira na broj — tako profesorov izbor
    mijenja koliko vježbe student dobije, a ne pravednost ocjene.

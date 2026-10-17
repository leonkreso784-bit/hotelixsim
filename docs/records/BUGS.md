# Pogreške i pouke (BUGS)

Ovo je mjesto za **pogreške koje su se dogodile i što smo iz njih naučili**, da se ne ponove (Leon 14.10.). Svaki unos ima:
**što** se vidjelo, **uzrok** (pravi, ne simptom), **pouka** (pravilo rada ili brana koja razred greške čini nemogućim) i
**stanje**. Popravak koji tek treba napraviti živi u [BACKLOG.md](BACKLOG.md) ili u ADR-u — ovdje je samo poveznica.
Najnovije na vrhu. Pouka koja postane pravilo rada seli u [AGENTI.md](../workflow/AGENTI.md) ili u branu, a ovdje ostaje poveznica.

## 2026-10-17 — sesija 8

**B14. Test balansa tražio iste primjerke i „klik ≤ vlastiti" po sjemenu** (PRAVILO-b; graditelj BLOCKED na čuvarovu testu, kasnije isto u testu ispisa nad medijanom 3 sjemena).
*Uzrok:* izvlačenje primjeraka ovisi o stanju hotela, pa putovi s različitim ishodom imaju iste primjerke samo do prvog kruga u kojem se stanje razdvoji; po pojedinom sjemenu
klik zna ispasti bolji (mjesečno C, sjeme 1). *Pouka:* testovi koji uspoređuju putove balansa **ne traže iste primjerke ni monotoniju po sjemenu ili medijanu** kad put
mijenja stanje; isti su samo primjerci prvog kruga, a redoslijed se tvrdi nad **zbrojem** po vrsti kruga (i branom na 1000 sjemena). *Stanje:* popravio voditelj (d8a7cc7, b0335d4), [ADR-059](DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md) t. 3.

**B15. Lažno crveno pod opterećenjem: vitest „Timeout waiting for worker" i cargo 0xC0000142** (CI nakon IZGLED-a i nakon PRAVILO-a/-b, dok su istodobno gradile tri stvari).
*Uzrok:* okolina (Windows ne stigne podići proces ili radnika), ne test. *Pouka:* takav EXIT (vitest timeout, `cargo test` EXIT 101/127 s `STATUS_DLL_INIT_FAILED`) **samo se ponovi**,
kad opterećenje padne; ne popravlja se kod ni test. Pravi pad ima poruku testa. *Stanje:* pravilo u nalozima (`upute-izvodacima.md` §2).

**B16. Disk pao na 45 GB** (ispod Leonove granice 50) dok su istodobno gradili CI na `main`, PRAVILO-b i paralelna sesija banke. *Uzrok:* svako stablo ima svoj
`target/<stablo>` od više GB (B12), a stari `target/` spojenih cigli i privremene mape (`sqlx-tmp`, `tmp`) nisu obrisani odmah. *Pouka:* **najviše dva `cargo` ukupno,
jedan Rust slot po sesiji** (glavna i banka); `df -h /c` prije svake gradnje; `target/<stablo>` se briše odmah nakon spajanja, ne na kraju dana. *Stanje:* obrisano šest mapa → 77 GB;
[ADR-059](DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md) t. 6.

## 2026-10-16 — sesija 6

**B12. Zajednički `CARGO_TARGET_DIR` među stablima daje zastarjelu biblioteku** (simptom: `unresolved import content::ljudi` bez retka "Compiling").
*Uzrok:* za isti crate cargo daje isti hash artefakta (put je relativan prema korijenu), a svježinu gleda po vremenu izmjene datoteka; stablo čiji su izvori
stariji od artefakta koji je izgradilo drugo stablo dobije tuđu biblioteku. Lažno zeleno je moguće. *Pouka:* **svako stablo svoj** `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>`
(briše se sa stablom, B11), glavno stablo `target/`; ispravlja B11 i B8 ("zajednički target"). Uz to: `pnpm lint` (biome) provjerava i `content/**/*.json`,
JSON formatirati prije commita; `check:api` EXIT 127 = Windows 0xC0000142 pod opterećenjem (okolina, ponoviti). *Druga B12 (15.10., spojena ovamo):* CI je pao na `lint` nakon A3 jer generirani pečat `fixtures_v2/v2-snapshot.json` nije bio izuzet iz `biome` kao v1 pečati, a ni čuvar, ni graditelj, ni provjeravatelj nisu vrtjeli `corepack pnpm lint` (kartica ga nije tražila). *Pouka:* svaka cigla vrti cijeli popis provjera iz CI-ja, ne samo onaj u kartici; `biome.json` izuzima `**/tests/fixtures_v2` (Global Constraints plana popravka).
*Stanje:* pravilo u [AGENTI](../workflow/AGENTI.md) t. 6 i 7, [ADR-055](DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 4.

**B13. Zalijepljeni svi ponuđeni odgovori najčešće dobiju 5** (prijavio Leon igrajući; u situaciji s kuharom koji traži povišicu 20 %, sidro S2, nije).
*Uzrok:* presuda gleda smisao teksta, a ne to da je tekst prepisan iz ponude; AI se na zbroju svih odgovora ponaša nepredvidivo. *Pouka:* prepoznavanje prepisivanja
mora biti u **kodu** (70 % riječi), ne u AI-ju: 0 ulaznih i izlaznih tokena, uvijek isti ishod ([ADR-054](DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 2).
*Druga pouka (pregled PRAVILO-a, 17.10.):* pravilo o **pojedinačnim riječima** (skup riječi bez redoslijeda) lažno je proglasilo zalijepljenim dobar vlastiti odgovor u obveznoj S1 i jednu prepisanu oznaku; **parovi susjednih riječi** (bigrami, isti prag 70 %, barem dvije oznake) daju 0 lažnih od 229 i hvataju svih 59 zalijepljenih. Pravilo o tekstu mjeri se na cijelom sadržaju prije spajanja, ne na primjeru.
*Stanje:* **riješeno u kodu** (PRAVILO-a, spojeno 9a275d8, `CORE_VERSION` 2.5.0): zalijepljena dva ili više ponuđenih odgovora = neutralna 3 bez učinka, sudac se ne zove; Leonove odluke u [ADR-057](DECISIONS/ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 2 i [ADR-056](DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md) A6.

## 2026-10-15 — popravak temelja

**B11. Disk pao na 12 GB slobodnog.** *Uzrok:* svako radno stablo (do 11 istodobno, plus 6 napuštenih mapa) prevodilo je
Rust u svoj `target/` od više GB, a stabla spojenih grana nisu brisana. *Pouka:* jedan zajednički `CARGO_TARGET_DIR`,
stablo se briše isti dan kad se spoji, `df -h /c` prije otvaranja stabla; granica je **50 GB slobodnog** (Leon 16.10., prije 100 GB; [ADR-057](DECISIONS/ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 9).
*Stanje:* pravilo u [AGENTI.md](../workflow/AGENTI.md) §1 t. 6; spojena stabla i stari `target/` obrisani 15.10.

## 2026-10-14 — iz Leonova igranja i spajanja grana

**B1. Najbolji ponuđeni odgovor je uvijek prvi.** Sva 55 predloška i sidra S1–S4 imaju odgovor ocjene 5 na prvom mjestu
(53 od 55 silazno), a ekran ih prikazuje redom. *Uzrok:* sadržaj je pisan od najboljeg prema najlošijem, a nijedna provjera ne
gleda ono što student vidi. *Pouka:* provjera mora mjeriti **izlaz koji student vidi**, ne samo ispravnost sadržaja (redoslijed,
duljina oznaka). *Stanje:* odlučeno, [ADR-042](DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 10.

**B2. Kvar se vraća iako je student dobro odgovorio.** Hitan popravak ne diže stanje odjela, a 11 od 15 kvarova nema uvjet stanja
(curenje vode dolazi i u hotelu sa stanjem 95). *Uzrok:* mehanika popravka naslijeđena iz v1 (samo trošak), predlošci pisani bez
zapisanog načela. *Pouka:* **načelo igre zapisuje se kao provjera sadržaja** („svaki kvar prostora ima uvjet stanja"), inače se
predložak po predložak zaboravi. *Stanje:* ADR-042 t. 1, 7.

**B3. Ista situacija dvaput u mjesecu** (`P-PRAONICA.jul.2`). *Uzrok:* mjesečni krug izvlači kao niz tjedana, a pravilo razmaka
mjeri tjedne. *Pouka:* pravilo koje vrijedi „po krugu" mora se testirati i na tjednom i na mjesečnom krugu. *Stanje:* **popravljeno u B2a** (15.10.): mjesečni krug se izvlači kao jedan blok, predložak se u krugu ne ponavlja ([ADR-042](DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 12–13).

**B4. AI lik ne završi razgovor u 6 poruka.** *Uzrok:* granicu `MAX_CHAT_MESSAGES` provodi igra, ali AI je ne zna (nema je ni u
uputi ni u ulazu). *Pouka:* **svaka granica koju igra provodi mora biti u ulazu AI-ja**. *Stanje:* ADR-042 t. 6.

**B5. Izgled lika ne odgovara liku** (gošća s bradom, hrvatsko ime s tamnom puti). *Uzrok:* izgled se računa samo iz imena.
*Pouka:* izvedeni prikaz ne smije nagađati iz imena ono što podatak već zna (rod, dob, podrijetlo). *Stanje:* ADR-042 t. 11.

**B6. Brana „udio situacija" pala tek pri spajanju grana** (B Brižni 10 % tjedno, cilj 20–40 %). *Uzrok:* grane obnove i situacija
balansirane su **odvojeno** tjedan dana; obnova je robotu digla napredak bez situacija, a to se vidjelo tek kad su se srele. Brana
uz to mjeri **robota**, ne studenta. *Pouka:* grane koje diraju isti napredak spajati rano ili mjeriti zajedno (privremeno
probno spajanje prije svake brane balansa); branu koja ovisi o ponašanju robota opisati kao mjeru robota. *Stanje:* L1-11 na
grani `model-v2/sloj1-l111`, čeka kalibraciju (ADR-042 t. 15).

**B7. Duplicirani kod u glavnom stablu** (14.10. 01:17, dvostruki `import` i funkcija; web se ne bi preveo, demo pao). *Uzrok:*
nepoznat — izmjena iste zakrpe primijenjena dvaput izvan commita. *Pouka:* `/newsession` provjerava `git status` i **ne dira**
tuđe izmjene, nego ih pokaže Leonu s kopijom. *Stanje:* vraćeno uz Leonov OK, kopija u scratchpadu sesije.

**B8. Zastarjeli `balance.exe` u mjerenju osnovice.** *Uzrok:* zajednički `CARGO_TARGET_DIR` za osnovicu i radno stablo.
*Pouka:* osnovica u privremenom stablu uvijek sa **zasebnim** `CARGO_TARGET_DIR`. *Stanje:* pravilo u nalozima graditeljima.

**B9. Poslužitelj jedne grane obrađuje poslove druge** (`renewal_cents`). *Uzrok:* grane obnove i situacija dijele bazu
`hotelixsim_agenti` i tablicu poslova. *Pouka:* istodobni poslužitelji dviju grana → zasebne baze. *Stanje:* pregled grana ima
zasebne baze (`hotelixsim_pregled_sit`, `hotelixsim_pregled_obnova`).

**B10. Baza ugašena nakon restarta** (dvaput 14.10.). *Pouka:* `/newsession` i svaki agent koji treba bazu prvo `baza.sh status`.

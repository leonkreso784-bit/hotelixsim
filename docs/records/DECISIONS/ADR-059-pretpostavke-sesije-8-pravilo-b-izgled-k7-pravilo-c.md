# ADR-059 — Pretpostavke sesije 8: PRAVILO-b u balansu, IZGLED pri spajanju, PRAVILO-c, jedan Rust slot
**Datum:** 2026-10-17 · **Status:** 🟨 ČEKA LEONOV PREGLED · **Vezano:** [ADR-047](ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md),
[ADR-055](ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md), [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md),
[ADR-058](ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md), [kartica PRAVILO](../../plan/2026-10-14-popravak-temelja/task-12-pravilo-vlastiti-odgovor.md),
[kartica IZGLED](../../plan/2026-10-14-popravak-temelja/task-13-izgled-lika.md), [BUGS](../BUGS.md) B14–B16

Pretpostavke voditelja i izvođača bez pitanja ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)). Što je već u ADR-058 (IZGLED t. 1–8,
PRAVILO-a t. 14) ovdje se ne ponavlja. Svaka stavka ima cijenu ako je kriva.

## PRAVILO-b (balans, spojeno ee6de80)
1. **`writable_v2` je zrcalo igre:** balans zna gdje se piše vlastiti odgovor iz istog izvora kao igra (sidro: `free_text` u AI sadržaju; primjerak
   predloška: ima lika). Napisan kao slobodna funkcija `balance::v2::writable_v2` uz tanki član, jer `Situations::writable` koji je kartica pretpostavila
   nije postojao. Cijena ako krivo: ime i mjesto jedne funkcije u balansu, igra se ne mijenja.
2. **`CLICK_MAX_QUALITY` u balansu je zrcalo** `game::own_words::CLICK_MAX_QUALITY` (4), a test pariteta `mirrors_match_the_game` ruši se ako se razidu.
   Balans ne ovisi o crateu `game` u izvoru (samo u testu). Cijena: jedna konstanta.
3. **Testovi putova tvrde nad zbrojem:** „klik najbolji ≤ vlastiti najbolji" vrijedi nad **zbrojem** napretka po vrsti kruga, uz ukupno strogo manje;
   isti primjerci traže se samo u prvom krugu. Ne po sjemenu ni po medijanu 3 sjemena, jer izvlačenje ovisi o stanju hotela ([BUGS](../BUGS.md) B14;
   d8a7cc7, b0335d4 — voditelj u ulozi čuvara po ADR-055 t. 2). Svojstvo na 1000 sjemena čuva brana `--check`. Cijena: test slabiji po sjemenu, brana ostaje.
4. **Ispis klika:** novi stupac „klik najbolji" u bloku B3, četiri koraka lanca (najbolji > klik > srednji > prepušteno > šutnja), svaki korak brana > 0,
   i redak „težina klika" s istim ravnalom plana kao težina najboljeg. **Težina klika je samo ispis, ne brana** (tjedno 38,9 / 45,1 / 37,7 %,
   mjesečno 24,0 / 32,2 / 19,0 %). Hoće li klik dobiti cilj težine, odlučuje Leon u L1-13 zajedno s nalazom I1 pregleda (klik nosi 92–95 % učinka
   vlastitog odgovora jer gubi samo ocjenu, ne mehanike; ADR-047 t. 3 očekivao ~75 %). Cijena: jedna brana više u L1-13.
5. **Put „srednji" se u balansu ne ograničava** (pregled M1): igra uz vlastiti odgovor ograničava svaki klik, ali danas je srednji odgovor svugdje ≤ 4, pa
   je ishod isti. Razilazi se tek uz predložak s dva odgovora. Odgođeno; cijena: jedan uvjet (`ClickBest | Middle`).

## Rad i provjere
6. **Jedan Rust slot po sesiji:** glavna sesija i paralelna sesija banke ljudi smiju imati po jedan `cargo` (ukupno najviše dva); provjera `tasklist cargo.exe`
   prije gradnje, `df -h /c` (ispod 50 GB ne, [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 9); `target/<stablo>` se briše
   odmah nakon spajanja ([BUGS](../BUGS.md) B16). Dopunjuje ADR-058 t. 15. Cijena: cigle čekaju jedna drugu.
7. **Osnovica nakon PRAVILO-b:** za usporedbu balansa v2 vrijedi izlaz `nakon-izgled/` uz razliku **samo u bloku klika** (retci 102–105, 110–113 i novi
   118–121); svi stari brojevi isti, v1 bajt-isti. Pogledano na `/kraj` sesije 8; sljedeće cigle uspoređuju s `nakon-pravilo-b/`. `cargo test --workspace`
   nakon PRAVILO-b treba ponoviti (EXIT 127 = okolina, [BUGS](../BUGS.md) B15). Cijena: jedno ponavljanje.
8. **Nalog izvođaču mora reći da testovi `app`/`api` traže `DATABASE_URL`:** graditelj PRAVILO-b dobio je nalog „nema baze", pa je `cargo test --workspace`
   pao samo na 34 testa s bazom. Od sada nalog daje bazu `hotelixsim_agenti` (upute izvođačima §2). Cijena: nijedna.

## IZGLED pri spajanju (872ff46)
9. **Sukob duga `scripts/check-kod.mjs`** (pregled I1): zadržana su oba spuštena broja (`api/src/schema.rs` 1717 iz IZGLED-a, `game/tests/mentor.rs` 1687
   iz PRAVILO-a). Frizure po rodu, učestalost brade (1 od 3) i naušnica (1 od 2) ostaju ukusna odluka iz kartice (ADR-058, C11); Leon ih mijenja nakon
   pregleda 10–20 lica u igri. Cijena: jedan popis u `look.ts`.

## PRAVILO-c (u tijeku)
10. **Čuvarovo razilaženje:** kartica zamišlja igru s pravilom `Classic` u testu `app`, ali `NewGameSettings` nema `answer_rule`; zato je „klasična igra"
    u testu igra modela v1 (`classic_game_never_marks_click_capped`). Napomena `click_capped` vrijedi gdje se piše vlastiti odgovor (`free_text`, u praksi lik).
    Web testovi traže dijelove teksta (`/Klikom dobivaš najviše 4 od 5/`, `/vrijedi kao klik/`), ne cijeli tekst. Cijena: jedan test više kad nova igra
    dobije izbor pravila.

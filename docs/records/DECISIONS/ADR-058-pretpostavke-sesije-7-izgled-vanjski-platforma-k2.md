# ADR-058 — Pretpostavke sesije 7 i 8: IZGLED, vanjski, plan platforme, K2, PRAVILO-a, sesija banke, banka hotela
**Datum:** 2026-10-17 · **Status:** 🟨 ČEKA LEONOV PREGLED (dio je Leon potvrdio 17.10., označeno ✅) ·
**Vezano:** [ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md), [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md),
[kartica IZGLED](../../plan/2026-10-14-popravak-temelja/task-13-izgled-lika.md), [BANKA_LJUDI](../../architecture/BANKA_LJUDI.md),
[plan platforme](../../plan/2026-11-platforma.md), [SESIJA_BANKE](../../workflow/SESIJA_BANKE.md)

Pretpostavke voditelja i izvođača bez pitanja ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)). Što je Leon 17.10. već
prihvatio u paketu od 43 pitanja ovdje se ne ponavlja, nego samo upućuje na ADR-056. Svaka 🟨 stavka ima cijenu ako je kriva.

## Već potvrđeno 17.10. (✅, vidi ADR-056)
- **Kartica IZGLED, „Rizik i odluke" 1–8** (rod iz popisa imena, podrijetlo do K5 po izvoru imena, dob iz riječi opisa, engleski nazivi u API-ju,
  stari izgled bez podataka bajt-isti, frizure, brada i naušnice, sidra S1–S4 ručno, selidba iz `schema.rs`): F1 (1, 4, 5, 7, 8) i C11 (2, 3, 6).
- **Nacrt VANJSKI, 13 pretpostavki** (`.superpowers/sdd/banka-ljudi-k1/nacrt-vanjski-report.md`): F1 (1, 5–7, 9–13), C9 (2, 8), C10 (3, 4).
- **PRAVILO-a, granica zalijepljenih:** parovi susjednih riječi, barem dvije oznake po 70 % (A6).
- **K2:** Uzbekistan je „Azija" (C8 a); usporedba razine, dobi i roda unutar uloge s ćelijom od 40 zapisa (F1, provjere stroja banke).
- **Stranica pregleda ljudi:** dob na licu mlad do 29, odrasla 30–54, stariji od 55 (C11 b).

## Otvoreno 🟨
**Plan platforme** (16.10., [nacrt](../../plan/2026-11-platforma.md); Leon je od njegovih 6 pitanja odgovorio samo dio E3, ADR-056):
1. C4 i F0-10 su jedna cigla na kolosijeku PLATFORMA (dira `app`, ne jezgru).
2. C2 uključuje F0-8 (više radnika poslova i traka) i seli se iz PILOTA prije P3.
3. C6 (razdvajanje `api/src/schema.rs`) je preduvjet za svaku platformsku rutu; IGRA nakon faze B radi C6, C1, C2 i RAZRED U JEZGRI prije C3, C5 i
   LJUDI (to je pitanje 5 plana, Leon još bira).
4. F1-12 se dijeli: F1-12a (bez radnika) za P3, F1-12b nakon L2a-12.
5. Brojeve migracija dodjeljuje voditelj pri spajanju (platforma od `0005`).
6. Cigle platforme granaju s `model-v2/sloj1` dok L1-15 ne sleti.
7. Za P3 korisnička imena bez pravog imena i prezimena (profesor drži popis), do G-3.
8. P2 iz ROADMAP-a (kraj studenog) realno klizi na sredinu prosinca [PROC].
9. **Dopuna ADR-004:** izlazak na internet za P3 (profesori, prije pilota) uz Leonov izričit OK; pružatelj nije odlučen (E3: možda Cloudflare).
10. **Profesorov pristup (E3 „vlastite informacije za prijavu"):** tumačenje = profesor ima vlastiti račun; oblik je preporuka plana, jednokratna
    poveznica 24 h (pitanje 2 plana). Cijena: jedna cigla F1-4 drukčija.

**K2 i banka ljudi:**
11. Kosovo i Albanija su „Europa" (`azija: false` u `mjesta.json`); nije pitano. Cijena: jedna zastavica.
12. Na stranici pregleda podrijetlo „regija" za lice = lokalno, regija ili HR, BA, RS, ME, MK, SI. Cijena: jedan popis u stranici, ne u igri.
13. „Studenti" kao podvrsta radnika (C1) — tumačenje voditelja: studenti na sezonskom radu ili praksi, najniža razina koja se trenira. Cijena: naziv i jedna razina.

**PRAVILO-a:**
14. Oznaka od jedne riječi u provjeri zalijepljenih koristi samu riječ (nema para); danas ne djeluje, najkraća oznaka ima 13 riječi.

**Sesija banke** (`sesija-banke-report.md`; Leon 17.10.: sesija je paralelna, ADR-057 t. 6):
15. Nema zajedničke brave za Rust: najviše jedan `cargo` po sesiji, provjera `tasklist` (≤ 1), dogovor kroz ledger.
16. Predaja sesije banke je zadnji redak njezina ledgera („SESIJA BANKE: sljedeće …"), ne zasebna datoteka u `docs/`.
17. Čuvar testova u sesiji banke radi samo kad serija uvodi novo pravilo provjere. Cijena: jedan korak više po seriji.

**Iz Leonovih odgovora 17.10. (ADR-056):**
18. **Ime hotela (C4):** najmanja brana za vulgarnosti uz slobodno ime (Leon nije rekao ne). Cijena: kratak popis u kodu ili ništa.
19. **„Dobar popravak sa zadovoljnim gostom" (A1)** = ocjena odgovora 4 ili 5; tada stanje Smještaja +2, kvar −1. L1-13 mjeri. Cijena: jedan prag.
20. **Situacije s gostima i popunjenost (A7)** grade se uz kalibraciju u L1-13 (mijenjaju težinu i balans), ne prije; granice 3–8 i profesorov
    broj ostaju. Cijena: zasebna cigla prije L1-13.
21. **Mjesto banke hotela:** nakon L1-13, prije kraja faze C, umjesto „generatora hotela" (PLATFORMA §4; generator kasnije samo puni banku). Hotele
    pišu agenti u serijama koje Leon pregleda i pečati, kao ljude. Tri komada: sadržaj (10–15 hotela 4\*, provjera stroja, pečat; prije toga
    istraživanje pravih hotela, A10), jezgra (ključ hotela → tekst, referenca napretka po hotelu, balans po svakom hotelu), platforma (profesor
    dodjeljuje hotel po studentu ili nasumce; tek s razredom). Najtanje mjesto: pravednost napretka za 80 prema 160 soba → indeksi po sobi ili
    udjelu, ne u eurima (PLATFORMA §4.3). Cijena: redoslijed u ROADMAP-u.
22. **Redoslijed nakon L1-13 (i L1-14/15):** prvo PLATFORMA do najmanje igrive razredne igre (prijava i pristup, hosting s izmjerenim troškom,
    oporavak od pada na pravoj bazi, profesorovo sučelje u najmanjem obliku: otvori krug, vidi hotele, vidi razgovore; GDPR i brisanje), tek onda
    sobe, voditelji, smještaj radnika i banka hotela. Razlog: igra bogata bez prijave i hostinga nije produkcija (Leonov pogled, ADR-056).
    Cijena: bogatije značajke igre kasne ~mjesec dana.
23. **Poruka za zalijepljene (A6):** točan tekst piše voditelj u smislu Leonova *„kopirao si, podnio sve ponuđene odgovore…"*; Leon ga vidi u igri.

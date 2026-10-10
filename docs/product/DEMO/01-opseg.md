# DEMO §1 — opseg demoa

Dio [DEMO.md](../DEMO.md)

## 1 · Opseg demoa

### 1.1 Što demo jest

Jedna igra: **igrač + tri robota**, svaki sa svojim hotelom, kroz **dvije runde** (lipanj, srpanj; Leon 10.10., [ADR-021](../../records/DECISIONS/ADR-021-koncept-ekrana-v2-i-odluke-o.md)). Demo je prvi rez prave
arhitekture, a ne prototip koji se baca: ista jezgra, isti dnevnik, isti API.

| | demo | pilot (kasnije) |
|---|---|---|
| vrijeme | **demo način**: rundu zatvara **gumb** | kalendar kolegija |
| igrači | 1 čovjek + 3 robota | grupa studenata |
| prijava | **poveznica stvori gosta** | pravi pružatelj prijave (ARHITEKTURA §12) |
| sezona | 2 runde: lipanj, srpanj (kolovoz i rujan poslije demoa) | po dogovoru s profesorom |
| situacije | predlošci + AI za presudu, likove i mentora | i AI-jem smišljene situacije |
| hosting | **lokalno** (Leonovo računalo) | Railway |

### 1.2 Vrijeme — demo način

- **Runda = jedan mjesec sezone.** Mjesec je prirodna jedinica za hotelske izvještaje, a DZS-ovi podaci o
  popunjenosti su mjesečni [IZVOR].
- Runda je otvorena dok igrač ne pritisne **„Zatvori mjesec"**. Tada se obračunaju svi hoteli zajedno, isto kao u
  pilotu. Razlika je samo u tome **tko** zatvara rundu, a to je jedna postavka igre (`time_mode: manual | calendar`).
- Trajanje situacije („1–7 dana") u demo načinu znači: situacija vrijedi do zatvaranja runde. Kalendarsko značenje
  se odlučuje za pilot (otvoreno u VIZIJA §4).
- Situacija bez odgovora do zatvaranja runde dobiva **ocjenu 1** po rubrici, jer je i ignoriranje odluka. To je
  drukčije od AI ispada, gdje presuda ide na pregled (ARHITEKTURA §8.3).

### 1.3 Tri robota

Roboti igraju **kroz jezgru, istim naredbama kao čovjek**: postavljaju radnje i biraju ponuđene odgovore. Ne pišu
slobodan tekst, pa ne troše AI. Strategije su u §10. Iste strategije ulaze i u alat za balans, pa su roboti ujedno
dokaz da obračun radi za više hotela.

### 1.4 Gost umjesto prijave

- `/demo` stvori **gosta**: novi korisnik bez lozinke, kolačić `httpOnly`, i novu igru s tri robota.
- Gost i njegova igra brišu se nakon 7 dana.
- Zaštita troška: najviše 3 nove demo igre po IP adresi na dan i tvrdi AI limit po igri (ARHITEKTURA §10.2).
- Ista poveznica s `?uloga=profesor` otvara i profesorov pogled na tu igru (§1.5). U pilotu to radi prava prijava.

### 1.5 Što profesor vidi i radi u 15 minuta

| min | korak | što pokazuje |
|---|---|---|
| 0–1 | Otvori poveznicu → „Tvoj hotel" | hotel se **dobije**; pet indeksa i stvarne brojke ispod njih |
| 1–6 | **Lipanj:** pritužba gosta (razgovor s likom, odgovor svojim riječima) i ponuda agencije (brojke) | sloboda izraza, AI glumi lik, pravila računaju |
| 6–7 | Izbornik radnji: cijena, sezonci, održavanje | potez u bilo kojem trenutku, svaki ima cijenu |
| 7–9 | **„Zatvori mjesec"** → obračun, mentor, ljestvica s tri robota | posljedice, objašnjenje, napredak umjesto zarade |
| 9–12 | **Srpanj:** zaposlenik traži povišicu; možda i splet okolnosti iz lipnja | odluke ostavljaju tragove |
| 12–15 | Profesorov pogled: **zašto je hotel tu gdje jest** (svaka promjena s uzrokom), presude AI-ja s kriterijima rubrike | prozirnost i pravednost |

Kolovoz i rujan dolaze poslije demoa (sadržaj za njih još nije gotov). U njima se splet okolnosti najjasnije vidi.

### 1.6 Što demo ne radi

Prava prijava · kalendar · profesor stvara igru ili mijenja postavke · AI smišlja situacije · zajedničko tržište ·
timovi · obavijesti na mobitel · engleski (tekstovi su ipak u prijevodima, ARHITEKTURA K9).

---

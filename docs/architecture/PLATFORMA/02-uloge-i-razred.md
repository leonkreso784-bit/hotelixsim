# PLATFORMA §2 · Uloge, razred, tablice, profesorski CRUD, pregled uživo, granica AI troška

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). **Odluka: [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)
t. 1–6.** Ovdje je samo kako se to slaže u sustav; pravila igre ne prepisujemo.

## 2.1 Uloge

| uloga | tko | smije |
|---|---|---|
| **admin** | Leon | sve organizacije; dodaje profesore; vidi zdravlje sustava i AI trošak; ne čita tuđe razgovore bez razloga (zapis pristupa) |
| **profesor** | nastavnik kolegija | CRUD nad svojim kolegijima, razredima i igrama; vidi svakog studenta svog razreda do kraja (brojke, odluke, AI razgovori, presude); otvara krugove; ispravlja presude; izvozi rezultate |
| **asistent** (po želji, pitanje F9) | suradnik na kolegiju | kao profesor, ali ne briše razrede i ne mijenja postavke igre u tijeku |
| **student** | član razreda | svoj hotel u cijelosti; o kolegama vidi listu hotela, ljestvicu napretka, ocjene, cijene i njihove radnike (ADR-037 t. 3–4); tuđi troškovi i odgovori na situacije skriveni |
| **gost (demo / vježba)** | bilo tko s poveznicom | kao danas: vlastita igra „student + 3 robota", bez razreda |

Prava se provjeravaju na jednom mjestu, u sloju `app`, s testom za svaku ulogu
([ARHITEKTURA §12](../ARHITEKTURA/09-podaci-ai-api-prava.md)). To već radi za gosta. Proširuje se `Caller`: danas nosi samo
`guest`, a dobiva korisnika, ulogu u organizaciji i članstva. Jedno pravilo posebno: o kolegi se vraća samo ono što
ADR-037 t. 4 dopušta, i to **po polju testirano** („API ne vraća privatno").

## 2.2 Hijerarhija: razred = jedna igra s N hotela

```text
organizacija (FMTU)
└── kolegij (npr. „Hotelski menadžment 2026/27")      profesori i asistenti su članovi
    └── razred  ── KOD IGRE (npr. K7M-4QX)   postavke: model v2, tjedan/mjesec, veličina, raspored krugova, AI granica
        └── jedna igra s N hotela (slot 1..N), po jedan hotel po studentu
```

**Odluka: ADR-037 t. 1.** Profesor pri stvaranju bira koliko studenata ulazi; studenti ulaze kodom; u razrednoj igri nema
robota. Igra za vježbu (student + 3 robota) ostaje kakva jest i služi za vježbu i usporedbu.

Prijašnji nacrt (12.10.) predlagao je mnogo odvojenih igara, po jednu za svakog studenta; odbačeno, vidi ADR-037.
Posljedice za arhitekturu:

- **Obračun po hotelu i vrata kruga (t. 2).** Hoteli se danas računaju neovisno (svaki iz svog stanja i plana), pa obračun
  po hotelu ne traži zajedničko tržište. Posao reda poslova je zato „obračun hotela H u krugu k", a ne „obračun cijele
  igre". Izvedba je u jezgri: [ROADMAP/igra](../../plan/ROADMAP/igra.md), faza RAZRED U JEZGRI.
- **Jedna igra, mnogo pisaca.** Naredbe se serijaliziraju po igri ([ADR-015](../../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 4), pa 30 studenata piše u isti dnevnik.
  Zato je predmemorija složene igre preduvjet (dio 06, cigla F0-10), a opterećenje se dokazuje probom 30 studenata (G-8).
- **Utjecaj među hotelima samo na prijelazu kruga (ADR-037, posljedice).** Preotimanje radnika (t. 5) i sve buduće veze
  među hotelima djeluju kad se krug zatvori, nikad usred tuđeg kruga. To čuva determinizam i obračun po hotelu.
- **Zajedničko tržište gostiju** ostaje izvan pilota ([VIZIJA](../../product/VIZIJA.md)). Ako ikad dođe, mora
  poštovati isto pravilo, a vrata moraju čekati sve hotele.
- **Veličina igre.** Granica `slot BETWEEN 1 AND 4` (`0003:17`) pada na 1..64; preporučena veličina razreda je 40, a
  stvarnu granicu mjeri proba G-8 (odluka D3).

## 2.3 Platformske tablice (skica)

Dnevnik događaja ostaje izvor istine o igri. Ove tablice su obične, kao danas `guests`/`games`/`hotels`
([ARHITEKTURA §9.3](../ARHITEKTURA/09-podaci-ai-api-prava.md)). Migracija `0005` i dalje.

```text
organizations   id, name
users           id, display_name, username, password_hash (argon2), role, created_at, deleted_at
identities      user_id, provider ('password'|'aai'|'google'|'microsoft'), subject, UNIQUE(provider, subject)
sessions        id, user_id NULL | guest_id NULL, token_hash, created_at, expires_at, last_seen, user_agent_short
memberships     user_id, org_id | course_id, role ('admin'|'professor'|'assistant'|'student')
courses         id, org_id, name, year
classes         id, course_id, join_code UNIQUE, join_open, settings JSONB, seed, created_by, game_id NULL do starta
class_players   class_id, user_id, guest_id (pseudonim), hotel_id NULL do starta        ← veza račun ↔ hotel u jednoj igri
games           + class_id NULL (vježba i demo nemaju razred)
ai_usage        game_id, hotel_id, role, model, input_tokens, output_tokens, cost_microusd, at   (projekcija iz AiMeta)
hotel_progress  game_id, hotel_id, round, indices, progress, answered, open_situations, review_needed  (projekcija)
audit_log       actor_user, action, target, at                       ← tko je što gledao/mijenjao (profesor, admin)
```

Razlika prema prijašnjem nacrtu: `identities` više ne počinje kodom ni magičnom poveznicom, nego lozinkom (dio 03), a
`class_players` vezuje račun na **hotel u zajedničkoj igri**, ne na vlastitu igru. Do starta („Pokreni") igra još ne
postoji: student je u čekaonici (preporuka, lobby D2 je otvoren), a `classes.game_id` je prazan.

**Ključni trik: `guest_id` u dnevniku ostaje pseudonim igrača.** Događaji već nose `guest:<uuid>` (`0002:5-6`) i
`Controller::Human { guest_id }`. Račun se veže na taj pseudonim kroz `class_players`, pa se oblik događaja ne mijenja
i `SCHEMA_VER` ostaje 1. U dnevniku tako nikad nema imena ni e-adrese (važno za dio 07).

## 2.4 Profesorski CRUD

| radnja | kako | napomena |
|---|---|---|
| stvori / uredi / arhiviraj kolegij | obična tablica | arhiviranje, ne brisanje; brisanje po roku čuvanja (dio 07) |
| stvori razred | postavke (model, tjedan/mjesec, veličina, sjeme, AI granica, smije li preotimanje), kod igre | postavke se zaključavaju pri „Pokreni", jer ulaze u snimak igre |
| otvori / zatvori upis | `join_open` | zatvoren upis = kod više ne radi; pridruživanje nakon starta odbijeno |
| generiraj račune za studente | profesor unese popis, sustav izda korisnička imena i početne lozinke | dio 03 |
| „Pokreni" (preporuka, D2 otvoreno) | jedna transakcija stvara igru s prijavljenima i hotele 1..N | drugi „Pokreni" ne radi ništa; pad usred ne ostavlja polovicu |
| **otvori sljedeći krug** | ručno ili po rasporedu | **Odluka: ADR-037 t. 2, 6.** Posao `open:<igra>:<krug>` s `run_after` (`0004`); 17 tjednih krugova otprilike odgovara semestru |
| neaktivan student | kad rok prođe, hotel sam prelazi u sljedeći krug s dotadašnjim planom | **Odluka: ADR-037 t. 6**; mentor ga upozorava |
| isključi preotimanje | postavka razreda | ADR-037 t. 5 |
| ukloni studenta | hotel se smrzne ili igra na zadanim potezima | otvoreno (D7) |
| ispravi presudu | postojeći neutralni zapisi s `review: true` ([ADR-019](../../records/DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md) t. 4–5) | ispravak je kompenzacija u sljedećem krugu, po hotelu (F2-7), ne ponovni obračun |
| izvoz | CSV: napredak, indeksi po krugu, AI trošak | za ocjenjivanje na kolegiju |

**Tko zatvara krug?** Svaki student zatvara svoj hotel kad je spreman; profesor otvara kad se smije dalje. Prijašnja
preporuka „profesor zatvara za sve" odbačena je (ADR-037 t. 2): brzi bi čekali spore, a igra mora raditi i kroz semestar.

## 2.5 Pregled uživo za profesora

**Što profesor vidi i iz kojih događaja:**

| podatak | izvor u dnevniku |
|---|---|
| tko je u kojem krugu, tko je zatvorio, tko čeka obračun | `RoundClosed`, `RoundResolved`, status posla u `jobs` |
| koliko je situacija neodgovoreno, tko je odgovorio kako | `AnswerChosen`, `ResponseSubmitted`, presude |
| pet indeksa i napredak po krugu, poredak u razredu | `RoundResolved` (izvještaj po hotelu) |
| presude za pregled (neutralne, zakašnjele) | presuda s `review: true`, `VerdictLate` |
| AI trošak po studentu i do granice | `AiMeta` uz svaki AI izlaz (`event.rs:140-145`) |
| „zašto je hotel tu" | postojeće objašnjenje (`/v1/hotels/{id}/explanation`) i mentor, uz profesorovo pravo |

**Kako se to čita:** ne radi se fold cijele igre pri svakom osvježavanju. Svako čitanje danas radi puni fold (BACKLOG
`[korak 4]`), a profesorov ekran to množi. Zato **projekcija `hotel_progress`** (jedna igra, N redaka po krugu) koja se
upisuje **u istoj transakciji** kao `RoundResolved`, odgovor ili presuda (unutar `decide`, `ports.rs:107-117`), pa je
uvijek usklađena. Može se ponovno izgraditi iz dnevnika (cigla F2-1).

**Kako „uživo":**

| način | posao | u našem slučaju |
|---|---|---|
| **polling + ETag** | najmanji: preglednik pita svakih 5 s, poslužitelj odgovori 304 ako se `max(events.id)` razreda nije pomaknuo | web već radi polling (`apps/web/src/api/poll.ts:2-9`); radi kroz svaki proxy |
| **SSE** | srednji: Axum ima SSE; za više procesa treba Postgres `LISTEN/NOTIFY` | [ARHITEKTURA §11](../ARHITEKTURA/09-podaci-ai-api-prava.md) ga već predviđa |
| **WebSocket** | najveći: dvosmjerno, stanje veza | ne treba: naredbe idu običnim POST-om |

**Preporuka:** polling s ETagom (5 s) za profesora i za listu kolega. SSE tek ako proba 30 studenata (G-8) pokaže
potrebu: jedan kanal `NOTIFY class:<id>` i jedna SSE ruta, podaci su isti, mijenja se samo dostava.
**Obavijesti** (novi krug, ponuda za radnika, reset lozinke) su lagane, u aplikaciji, bez e-pošte (ADR-037 t. 6).

## 2.6 Granica AI troška

- **Leon je odlučio:** 2 $ po studentu, a iznad toga ponuđeni odgovori ([ADR-028](../../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md), odgovor 7).
  **Granica vrijedi po igri, odnosno po semestru, ne po satu** (ADR-037 t. 6).
- **Gdje se provjerava:** u `app`, **prije** stavljanja AI posla u red. Zbroji se `AiMeta` za hotel (projekcija
  `ai_usage`) i pomnoži s cijenom modela. Iznad granice odmah ide rezerva, ista kao kad AI ne radi (ADR-019 t. 4).
- **Druga granica, po razredu:** npr. 30 studenata × 2 $ = 60 $. Profesor je vidi i može je povisiti (RZ-4). Treća,
  globalna granica u adminu je osigurač protiv greške u petlji (F2-9).
- **Okvirni trošak:** tjedna igra 0,7–1,7 $ po studentu, mjesečna 0,2–0,4 $ [PROC]. Za 3 grupe × 30 studenata to je
  20–150 $ po sezoni [PROC]. Prije obećanja profesoru treba izmjeriti stvarne tokene
  (`cargo test -p ai --test live -- --ignored`). AI trošak ne kreće sa stvarnim razredom dok nije dogovoreno tko ga plaća (F8).

# Simulacija vođenja hotela — poslovna igra za FMTU

> **Status: IDEJA, nije projekt.** Zabilježeno 2026-08-22 (Leon: *„ovo za simulaciju hotela mi
> se jako sviđa"*). Ništa se ne gradi dok frontend redizajn nije gotov — ovo je materijal za
> **prijedlog dekanu/profesorima**, i kandidat za diplomski, ne cigla u redu čekanja.
> Kad ideja sazri → dobiva vlastiti spec u `plan/` i milestone u [ROADMAP.md](../plan/ROADMAP.md).

## Što je

Student vodi virtualni hotel kroz sezonu. Svaka runda = jedan tjedan (ili mjesec) sezone;
donese odluke, sustav razriješi potražnju, student vidi posljedicu i ide dalje. Cijela
generacija igra **istu sezonu** i natječe se.

**Odluke po rundi:** cijena po segmentu (individualci / poslovni / grupe / OTA), alokacija
kapaciteta, kadrovi, F&B, marketinški budžet, održavanje i capex.

**Ishodi koje vidi:** popunjenost, ADR, **RevPAR**, GOPPAR, zadovoljstvo gosta, fluktuacija
osoblja.

**Što dolazi izvana:** krivulja sezonalnosti (parametrizirana **stvarnim kvarnerskim
podacima**), vrijeme, događaji u gradu, potez konkurencije, makro šok.

## Dvije odluke koje ovo dijele od tablice u Excelu

**1. Zajednički bazen potražnje.** Cijena jednog igrača mijenja potražnju ostalima. Bez toga
igra uči da je cijena izolirana varijabla — što je najveća neistina u revenue managementu.
S time uči da je cijena **potez u tržišnoj igri**. To je cijela pedagoška vrijednost.

**2. Bodovanje NIJE samo profit.** Višekriterijski ishod (profit + zadovoljstvo gosta +
održivost/kadrovi) jer inače pobjeđuje strategija „digni cijenu, otpusti pola ljudi", a to je
točno navika koju kolegij pokušava razgraditi.

## Zašto bi baš ovo ostavilo dojam

- **Tržište već postoji i skupo je.** Hotelijerski fakulteti kupuju **HOTS**, **Cesim
  Hospitality**, **Shadow Manager** — licence po studentu, strani kontekst, sučelja iz prošlog
  desetljeća. Besplatna, **jadranski parametrizirana** simulacija je „ovo nema nijedan drugi
  fakultet u Hrvatskoj", i to je rečenica koju dekan može ponoviti dalje.
- **Postaje profesorova infrastruktura, ne demo.** Na njoj se vodi kolegij i ocjenjuje. To je
  najjači oblik usvajanja — alat preživi autora.
- **Deterministička sezona = poštena ocjena.** Isto sjeme → ista sezona za sve grupe →
  usporedivo i ponovljivo. (Isti zahtjev koji već nosi engine vježbi.)
- **Natjecanje se dobro fotografira.** Generacija protiv generacije, pa Rijeka–Zagreb–Dubrovnik.

## Zašto baš Leon — ~70 % primitiva već postoji

| treba simulaciji | već izgrađeno u Sokratu |
|---|---|
| deterministička generacija sa sjemenkom | engine vježbi (7 tipova, seed → isti zadatak) |
| korisnici, uloge, izolacija podataka | Supabase auth + RLS + owner-check RPC |
| stanje po korisniku kroz vrijeme | `progress` + cloud-sync (offline-first merge) |
| autorstvo scenarija bez deploya | admin CRUD / Studio + `publish_document` |
| dvojezičnost | `js/i18n.js` (HR/EN) |
| mjerenje da se ništa nije slomilo | preflight + Playwright kultura |

Simulacija je u biti **sjeme + stanje + pravila razrješenja + ljestvica**. Nitko ne kreće s ove
pozicije.

## Modul koji prodaje demo u 30 sekundi — AI gost

Role-play situacije na recepciji (ljutit gost, prebooking, reklamacija) na **EN/DE/IT**, uz
ocjenu *service recovery* postupka. Kao samostalan proizvod je tanak; **kao modul unutar
simulacije je ono što sastanak pretvara u „kako da ovo dobijemo".** Trošak po studentu je
stvaran i mora se izmjeriti prije obećanja.

## Iskreni rizici

- **Balans je teži od koda.** Ugađanje krivulja potražnje je pravi posao; softver je lakši dio.
- **Bez profesora kao SUAUTORA ispada zabavno ali pedagoški neobranjivo.** To je ujedno i put
  do usvajanja — suautor brani svoj alat, recenzent ga samo komentira.
- **Runde su nova backend-forma** (zakazano razrješenje: pg_cron ili Edge Function), za razliku
  od svega dosad što je bilo zahtjev-odgovor.
- **Anti-cheat** (višestruki računi u natjecanju s ljestvicom).
- **Ako to nitko ne vodi na kolegiju, to je demo.** Usvajanje je uvjet, ne posljedica.

## Timing — izričito

**Ne sada.** Prvo faza TELEFON + POLICA + C4–C7 na Sokratu; druga platforma pokrenuta sada je
najpouzdaniji način da prva umre.

Ovo je **materijal za pregovor**: uz gotov Sokrat položiti i konkretan prijedlog — *„sljedeće
bih radio ovo, tražim mentora i jedan kolegij za pilot"*. Time sastanak prestaje biti
demonstracija. Usput je i obranjiva tema diplomskog (*ozbiljna igra za učenje upravljanja
prihodima u hotelijerstvu*) — objavljivo, a fakultet dobiva artefakt.

⚠️ **Napomena o kalendaru:** za 10. mjesec 2026. već je planirana prezentacija dekanu za
**zaseban** projekt ([sokrat-ai/DEAN_PITCH.md](../sokrat-ai/DEAN_PITCH.md)). Ovo je **treća**
stvar na istom stolu — odlučiti nose li se zajedno ili odvojeno, jer tri poruke u istom
sastanku znače nula zapamćenih.

## Što se mora odlučiti prije ijednog retka koda

1. Koga primarno gađamo — dekana, profesore ili studente? (Mijenja opseg prve verzije.)
2. Ima li profesor koji ovo hoće voditi na kolegiju? **Ako nema — ne gradi se.**
3. Je li ovo dio Sokrata (isti auth/repo) ili zaseban proizvod? *(Preporuka: zaseban proizvod,
   posuđeni primitivi.)*
4. Odakle podaci za sezonalnost i smije li ih se koristiti.

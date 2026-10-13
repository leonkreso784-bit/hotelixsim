# HotelixSim kao platforma: prijava, razred, profesor, generirani hoteli, produkcija

> **Nacrt za raspravu (ADR-006); odluke su u [ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)**
> (razred, obračun po hotelu, vrata kruga, natjecanje za ljude, sat i semestar, računi, engleski) i
> [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) (dva kolosijeka). Ovdje je kako i zašto
> to slažemo u sustav i koje su mogućnosti; odluke se ne prepisuju. Prepisano 13.10.2026. iz nacrta od 12.10.
> (grana `model-v2/sloj1`) po ADR-037 i dopunama; što je u nacrtu bilo zastarjelo, zamijenjeno je važećim stanjem.
> Brojke koje nisu iz koda označene su **[PROC]**, a što treba provjeriti izvan repoa **[PROVJERITI]**.
> Cigle i faze: [ROADMAP/platforma](../plan/ROADMAP/platforma.md) · igra: [ARHITEKTURA](./ARHITEKTURA.md), [MODEL_V2](./MODEL_V2.md).

## Dijelovi

| § | dio |
|---|---|
| 1 · pojmovi (svaki jednom) i odgovor „možemo li nastaviti" | [01-pojmovi-i-odgovor.md](./PLATFORMA/01-pojmovi-i-odgovor.md) |
| 2 · uloge, razred = jedna igra s N hotela, tablice, profesorski CRUD, pregled uživo, granica AI troška | [02-uloge-i-razred.md](./PLATFORMA/02-uloge-i-razred.md) |
| 3 · prijava: usporedba, računi pa AAI@EduHr, Supabase Auth kao posrednik | [03-prijava.md](./PLATFORMA/03-prijava.md) |
| 4 · generirani hoteli: danas, oblik, pravednost, ovisnosti | [04-generirani-hoteli.md](./PLATFORMA/04-generirani-hoteli.md) |
| 5 · igrini slojevi (dobavljači, rezervacije, gosti, vrste soba, pošta), avatari, natjecanje za ljude | [05-igrini-slojevi.md](./PLATFORMA/05-igrini-slojevi.md) |
| 6 · produkcija i trošak: jedna adresa, hosting (Railway Hobby), kopije, što mora biti gotovo, CI/CD, opterećenje | [06-produkcija-i-trosak.md](./PLATFORMA/06-produkcija-i-trosak.md) |
| 7 · GDPR i sigurnost: podaci, uloge, AI razgovori, anonimizacija u zajedničkoj igri | [07-gdpr-i-sigurnost.md](./PLATFORMA/07-gdpr-i-sigurnost.md) |
| 8 · faze (imena iz ROADMAP-a), temelj ili zakrpa, Leonove odluke i pitanja za fakultet | [08-faze-i-pitanja.md](./PLATFORMA/08-faze-i-pitanja.md) |

## Sažetak na jednoj stranici

**Što je to:** igra ostaje jezgra koja se ne prepisuje; oko nje rastu tri sloja: **identitet** (računi, uloge, razredi),
**profesorov pregled** (projekcija, ekrani) i **produkcija** (jedna adresa, Railway, kopije, nadzor).

**Što je odlučeno** (ADR-037, dopune 13.10.):

- razred je **jedna igra s N studentskih hotela**; studenti ulaze kodom i igraju jedni protiv drugih; roboti samo za vježbu
  i usporedbu (ADR-037 t. 1);
- profesor otvara krug, student zatvara svoj hotel, nitko ne ide iza otvorenog kruga; obračun je po hotelu (t. 2);
- student vidi listu kolega i njihove radnike (t. 3–4); radnike se može preotimati ponudom (t. 5);
- ista igra na satu i kroz semestar; neaktivnom hotelu rok sam prenosi plan; obavijesti u aplikaciji (t. 6);
- prijava: korisničko ime + lozinka koje izdaje profesor, pa AAI@EduHr; identitet odvojen od korisnika (t. 7);
- igra kasnije i na engleskom (t. 8).

**Trošak (Leon plaća sam):** do profesorova OK **0 €** (sve lokalno, ADR-004); izlazak na internet samo uz izričit OK. **Hosting nije
odlučen, ovo je preporuka koja čeka Leonovu potvrdu:** Railway Hobby sa servisom i Postgresom ~5–10 €/mj [PROC], Supabase samo kao
posrednik prijave, kasnije. Isto vrijedi za sesije umjesto JWT-a (L10). Pri širenju najveći trošak je AI, ne hosting.

**Što je još otvoreno:** kako se profesor prijavljuje za prvu probu (D4), veličina igre (D3), imena ili nadimci (D8), uklonjeni
student (D7), lobby/čekaonica (D2), brojke preotimanja radnika (limit ponuda, trošak zapošljavanja, vidljivost plaće: anketa kad faza
krene), koliko različitih generiranih hotela (L7), potvrda hostinga (L4) i sesija (L10), a od fakulteta AAI@EduHr, voditelj obrade, rok čuvanja i tko plaća
([08](./PLATFORMA/08-faze-i-pitanja.md)).

**Gdje se vidi:** samo u dokumentaciji (lokalno / GitHub nakon pusha); ništa u kodu ni na poslužiteljima nije mijenjano.

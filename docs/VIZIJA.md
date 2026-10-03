# hotel.sim — vizija

> **Izvor:** Leonov opis i ankete u sesiji planiranja 2026-10-03 (Sokratova sesija). Ovo je **vizija i dogovor**,
> ne arhitektura — tehnički izbori (backend, baza, AI pružatelj) **nisu doneseni**.
> Izvorna ideja iz kolovoza (runde, revenue management, AI gost): [izvor/HOTEL_SIM_2026-08-22.md](./izvor/HOTEL_SIM_2026-08-22.md).
> Gdje se ova vizija razlikuje od izvorne ideje, **vrijedi ova**.

## 1 · Što je

Platforma na kojoj se student prijavi i **dobije hotel** koji vodi. Više studenata vodi svoje hotele u istom
vremenu i **natječu se** — ali ne u tome tko je više zaradio, nego **tko je svoj hotel najviše unaprijedio**.
Uči se kroz situacije koje stižu i kroz odluke koje student sam donosi. U simulaciju je **uključen AI**.

## 2 · Hotel se DOBIJE, ne bira

Razlog (Leon): natjecanje mora biti fer, a izbor bi to narušio.

- **Svi hoteli kreću s istom ukupnom početnom ocjenom**, ali s **drukčije raspoređenim snagama**.
  - Hotel A: super recenzije, loše financije, nezadovoljni zaposlenici.
  - Hotel B: dobre financije, slabije recenzije, lošija operativa.
  - Hotel C: prosječan u svemu.
- **Zvjezdice ne znače da je netko jači** — znače drugačiji standard.
- Načelo: *svi kreću jednako, ali s drukčije raspoređenim snagama.*

## 3 · Situacije — pet osi

| os | vrijednosti |
|---|---|
| **doseg** | individualna · kolektivna · globalna |
| **područje** | radna snaga · recenzije · financije · nabava · održavanje · sigurnost |
| **priroda** | prilika · problem · mješovito |
| **težina** | 1 – 5 |
| **trajanje** | 1 – 7 dana |

- **Kolektivna situacija (pilot):** svi studenti imaju **isti problem**, svatko ga rješava u svom hotelu.
- **Vizija (ne pilot):** studenti **utječu jedni na druge** — dijele isto tržište gostiju ili radnika, pa odluka jednoga
  mijenja stanje drugima (Leon: *„bolesno"*). Ulazi tek kad osnovna petlja radi i balans je dokazano fer.

## 4 · Kako student igra

- **Odgovor na situaciju je kombinacija:** ponuđene opcije · slobodan tekst · brojčane odluke (cijena, budžet, smjene…).
- **Sloboda kao u BitLifeu:**
  - **izbornik radnji u svakom trenutku** — po područjima (zaposlenici, financije, marketing, održavanje, nabava,
    sigurnost), svaka radnja ima cijenu i posljedicu;
  - **slobodan potez tekstom** — student napiše nešto čega nema u izborniku, AI procijeni izvedivost, cijenu i posljedicu;
  - razgovor s likovima **ne otvara se kad student hoće** — likovi dolaze **uz situacije**.
- **Ključno:** student donosi više odluka, a **posljedice za budućnost su razgranate i mogu biti iznenađujuće**.
- **Vrijeme je zajedničko svima** i teče po kalendaru kolegija; **nitko ga ne ubrzava**.
  ⚠️ Što točno znači „trajanje 1–7 dana" u tom modelu (stvarni dani?) — **još nije odlučeno**.

## 5 · Uloga AI-ja (sve četiri)

1. **Smišlja situacije** po pet osi, prilagođene hotelu.
2. **Ocjenjuje odluke** — posebno slobodan tekst — i određuje posljedice na faktore.
3. **Glumi likove** — gost, zaposlenik, dobavljač, inspektor — unutar situacija.
4. **Mentor** — nakon situacije objasni što je bilo dobro, što loše i zašto.

AI za pilot plaćamo mi (za demo postoje besplatni API ključevi, Leon 03.10.); ključ **nikad u pregledniku**.

## 6 · Ocjena — napredak, ne zarada

- Gleda se **početno stanje** svakog hotela i **napredak u odnosu na taj početni potencijal**.
- Faktori: **zadovoljstvo gostiju · recenzije · zadovoljstvo zaposlenika · operativna učinkovitost · financije**
  (popis se smije proširiti).
- Ocjena je **relativna**: koliko se svaki hotel popravio u odnosu na to s čime je krenuo.

## 7 · Za koga i zašto sad

- **Prvi korisnik je profesor s FMTU-a** koji je zainteresiran, ali **prvo želi vidjeti demo**.
- Pilot = **vođena prezentacija + jedan igriv hotel s tri lažna igrača**. Jezik: **hrvatski**, s tekstom odvojenim tako
  da engleski kasnije ne traži prepravak.

## 8 · Što je izvan pilota (vizija)

Međusobni utjecaj studenata · profesorova ploča (pokreće, zadaje šokove, ocjenjuje) · više hotela i tržišta ·
natjecanje među generacijama i fakultetima. Iz zamrznutog Sokratovog smjera smiju se posuditi ideje
**exam modula** i **organizacije** (fakultet kao kupac).

# Izgled lika — nacrt (16.10.)

**Status:** nacrt za Leonovu odluku o REDOSLIJEDU; što izgled MORA pokazivati već je odlučeno
([ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 11: rod, dob i podrijetlo). Ovdje: kako, kada i koliko košta.
Vezano: [BANKA_LJUDI.md](./BANKA_LJUDI.md) (rod, dob, zemlja svakog lika), [ADR-021](../records/DECISIONS/ADR-021-koncept-ekrana-v2-i-odluke-o.md) / ADR-022 (lik kao podatak, uređivač kao u BitLifeu).

## 1. Što danas imamo

- **Lik je podatak, ne slika.** `AvatarLook` ima 16 polja (koža, frizura, boja kose, oči, usta, brada, naočale, kapa, boja odjeće, pozadina,
  obrve, nos, dobna skupina mlad/odrasla osoba/stariji, naušnice, ovratnik, vrsta odjeće). Crta ga SVG u `apps/web/src/avatar/Avatar.tsx`
  (528 r.), podatak je u `look.ts` (357 r.). Igrač ima svoj lik (`PLAYER_LOOK`).
- **Lik situacije** dobiva izgled iz `lookFor(ime)`: nekoliko poznatih imena ima ručno zadan izgled, svi ostali nasumičan izgled iz sjemena = ime.
  Zato gošća može dobiti bradu, a lik s hrvatskim imenom tamnu put (ADR-042 t. 11).
- **Jezgra već zna rod**: uloga u predlošku situacije ima `gender` (ž/m), a ime se vuče iz `content/hotel/v2/names.json` (popis ž i m,
  po 20 za gosta, osoblje i partnera). API šalje webu samo `character_name` i opis uloge — rod se gubi na putu.
- **Banka ljudi** (K1, 16.10.) ima rod, dob, podrijetlo i zemlju za svaki zapis, ali igra je još ne koristi (korak K5 u BANKA_LJUDI §5.1).

## 2. Dvije mogućnosti

### A. Isti crtić, ali iz roda, dobi i podrijetla (preporuka)

- API uz lika šalje tri polja: **rod** (iz popisa imena ž/m — bez promjene jezgre i dnevnika), **dobna skupina** (danas iz opisa uloge ili
  sjemena; od K5 točna dob iz banke), **podrijetlo** (od K5 iz banke; do tada iz izvora imena: hrvatska i regionalna imena svijetli tonovi).
- Web: `lookFor(ime, rod, dob, podrijetlo)` — rod određuje bradu, frizuru i naušnice; dob skupinu; podrijetlo raspon boje kože prema
  stvarnom stanovništvu. Ime i dalje bira detalje unutar toga, pa je isti lik uvijek isti.
- **Cijena:** ~4–6 h (API jedno polje i jedan upit u popis imena; web jedna funkcija i testovi; pregled). **0 €**, bez licence — sve je naše.
  Dira samo API i web: balans, pečat jezgre i `CORE_VERSION` ostaju isti. Može ići **usporedo** s PRAVILO-om (drugi crateovi).
- Ograničenje: ostaje stil crtića; nema fotorealizma.

### B. Slike (generirane AI-jem ili kupljeni paket)

- **AI slike** (jedna slika po liku, unaprijed, u repou): ~0,03–0,08 € po slici → 1 000 likova ≈ 30–80 € jednokratno. Stil među slikama
  nije jednak; svaka greška = nova slika i ručni pregled; repo naraste ~50 MB. Prava: uvjeti pružatelja dopuštaju uporabu, ali
  AI slika u EU nema autorsku zaštitu (nitko je ne može zabraniti ni tebi, ni ti drugima).
- **Kupljeni paket avatara:** 20–200 € jednokratno; licenca za web-aplikaciju i obrazovnu uporabu mora se provjeriti; broj kombinacija ograničen.
- Obje: lik prestaje biti podatak → nema uređivača, nema „isti lik u drugoj odjeći ili raspoloženju"; proturječi ADR-021/022.
- **Cijena izvedbe:** ~8–15 h (cjevovod, generiranje, pregled tisuću slika) + novac gore.

## 3. Preporuka

Mogućnost **A** kao zasebna cigla **IZGLED** (web + API, ~4–6 h), neovisna o PRAVILO-u i L1-13, pa smije usporedo s njima. L1-13b tada
zadrži samo ono što treba jezgra (rod u `RoleFill`, ako se pokaže potrebnim). B samo ako Leon želi fotorealizam — onda nova rasprava, ne sad.

## 4. Što Leon odlučuje (anketa 16.10.)

1. **Redoslijed:** IZGLED kao zasebna cigla sada (usporedo s PRAVILO-om), ili po planu unutar L1-13b, ili faza C odmah.
2. **Stil:** ostaje crtić (A). Ako ne — B kroz raspravu, uz cijenu iz §2.

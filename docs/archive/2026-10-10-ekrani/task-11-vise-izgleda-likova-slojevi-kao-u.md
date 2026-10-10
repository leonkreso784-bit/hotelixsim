### Task 11: Više izgleda likova (slojevi kao u BitLifeu)

Leon (10.10.): *„trebat ce prosirit likove“* → anketa: više izgleda, više likova, uređivač; „dio odmah“
([ADR-022](../../records/DECISIONS/ADR-022-ekrani-igraca-pretpostavke-plana-koraka-5.md) t. 6, Ruling 9 u ledgeru). Ovaj task proširuje **samo** `src/avatar/` i galeriju:
nove osobine lika i nove opcije postojećih. Gradi se **paralelno** s ekranima u radnom stablu `gradnja/likovi-izgled`
(od commita 4e66e9e) i spaja u `gradnja/ekrani` kad prođe pregled — zato ne dira ništa izvan svojih datoteka.

**Files:**
- Modify: `apps/web/src/avatar/palette.ts`, `apps/web/src/avatar/look.ts`, `apps/web/src/avatar/Avatar.tsx`,
  `apps/web/src/avatar/avatar.test.tsx`, `apps/web/src/screens/AvatarGallery.tsx`
- NE dirati: `packages/i18n/src/hr.ts` (galerija za nove retke koristi postojeći obrazac sirovih id-ova kao frizure), ekrane.

**Interfaces:**
- Consumes (Task 3): `AvatarLook`, `HAIR_STYLES…HATS`, `SEED_*` zamrznuti skupovi, `lookFromSeed`, `lookFor`, `isValidLook`,
  `PLAYER_LOOK`, `MENTOR_LOOK`, `KNOWN_LOOKS`, `Avatar`.
- Produces:
  - Nove osobine u `AvatarLook` (na kraju tipa): `brows: Brows`, `nose: Nose`, `age: Age`, `earrings: Earrings`,
    `neckwear: Neckwear`, `outfit: Outfit` s nizovima `BROWS = ["straight", "arched", "thick", "worried"]`,
    `NOSES = ["button", "long", "wide"]`, `AGES = ["young", "adult", "senior"]`, `EARRINGS = ["none", "studs", "hoops"]`,
    `NECKWEAR = ["none", "tie", "scarf", "bowtie"]`, `OUTFITS = ["shirt", "blazer", "uniform", "apron", "tshirt"]`.
  - Nove opcije **na kraj** postojećih nizova: `HAIR_STYLES` += `"afro", "wavy", "bob", "slick"`; `FACIAL_HAIR` +=
    `"goatee", "full"`; `HATS` += `"beanie"`. Palete na kraj: `SKIN` += `"#F1D3BE", "#3F2A1D"`; `HAIR` += `"#E8CFA0"`
    (svijetloplava), `"#6B2E1F"` (kestenjasta); `CLOTHES` += `"#00897B", "#C2185B", "#8D6E63"`.
  - `LOOK_SCHEMA: readonly LookField[]` gdje `LookField = { key: keyof AvatarLook; kind: "option"; values: readonly string[] }
    | { key: keyof AvatarLook; kind: "palette"; size: number }` — redoslijed polja za budući uređivač (koža, lice…).
  - `DEFAULT_LOOK: AvatarLook` i `normalizeLook(raw: unknown): AvatarLook` — temelj za izgled koji dolazi izvana
    (poslužitelj, uređivač): nedostajuće ili neispravno polje dobiva vrijednost iz `DEFAULT_LOOK`, višak polja se odbacuje.

**Pravila (vrijede i za pregledavatelja):**
1. **Postojeći izgledi se ne mijenjaju.** `SEED_*` skupovi iz Taska 3 i redoslijed izvlačenja u `lookFromSeed` ostaju
   kakvi jesu; nove osobine se izvlače **nakon** svih postojećih, iz novih zamrznutih skupova (`SEED_BROWS`, `SEED_NOSES`,
   `SEED_AGES` = samo `"young", "adult"`, `SEED_EARRINGS`, `SEED_NECKWEAR`, `SEED_OUTFITS` = bez `"apron"` i `"uniform"`).
   Nove opcije postojećih osobina (npr. `"afro"`) **ne** ulaze u stare `SEED_*` skupove. Test: zakucani izgled „Ana Kovač“
   iz Taska 3 zadržava sve stare vrijednosti; proširi očekivani objekt samo novim poljima.
2. `isValidLook` provjerava i nova polja; `LOOK_SCHEMA` sadrži **svaki** ključ `AvatarLook` točno jednom (test).
3. Svaka opcija svake osobine ima svoj sloj `data-part="<osobina>-<opcija>"`, osim „praznih“: `none` (gdje postoji),
   `age-young` i `age-adult` (bez bora). Obrve i nos se uvijek crtaju (zamjenjuju dosadašnje fiksne obrve i nos).
   Test petljom po svim nizovima, kao postojeći test frizura.
4. Odjeća (`outfit`) crta oblik preko boje `clothes`: `shirt` = dosadašnji oblik s ovratnikom; `blazer` = revers tamnijom
   nijansom (`rgba(0,0,0,0.25)` preko boje) i bijela košulja u V-u; `uniform` = stojeći ovratnik, dva gumba i mala pločica s
   imenom; `apron` = bijela pregača preko odjeće s vrpcom oko vrata; `tshirt` = okrugli izrez bez ovratnika.
   `neckwear` se crta iznad odjeće (kravata, šal, leptir-mašna); `earrings` na ušima; `senior` = dvije bore na čelu i
   linije uz oči (tanko, `rgba(0,0,0,0.18)`).
5. Poznati likovi dobivaju nova polja svjesno: `PLAYER_LOOK` (`outfit: "blazer"`, `neckwear: "tie"`, `age: "adult"`,
   `brows: "straight"`, `nose: "button"`, `earrings: "none"`), `MENTOR_LOOK` (`age: "senior"`, `outfit: "blazer"`,
   `neckwear: "scarf"`, `brows: "arched"`, `nose: "long"`, `earrings: "studs"`), Petra Horvat (`brows: "worried"`,
   `nose: "button"`, `age: "adult"`, `earrings: "studs"`, `neckwear: "none"`, `outfit: "tshirt"`), Marko Babić
   (`brows: "thick"`, `nose: "wide"`, `age: "adult"`, `earrings: "none"`, `neckwear: "none"`, `outfit: "apron"`).
6. `normalizeLook` testovi: (a) izgled starog oblika (bez novih polja) → stara polja ista, nova iz `DEFAULT_LOOK`;
   (b) `{ skin: 999, hair: "x" }` → `skin` i `hair` iz `DEFAULT_LOOK`; (c) `null`, broj, niz → `DEFAULT_LOOK`;
   (d) višak polja (`{ ...PLAYER_LOOK, evil: 1 }`) ne prolazi u rezultat. `DEFAULT_LOOK` je valjan.
7. Galerija dobiva po red za svaku novu osobinu (obrve, nos, dob, naušnice, oko vrata, odjeća) kao postojeći red frizura;
   u red frizura ulaze i nove frizure (petlja po `HAIR_STYLES` to već radi).
8. **Vizualna provjera je dio taska:** nakon testova izrenderiraj galeriju (`renderToStaticMarkup` u HTML u scratchpadu,
   posluži lokalno i snimi Playwrightom, kao u Tasku 3) i pogledaj snimku. Sloj koji „pluta“, viri izvan lica ili prekriva
   oči popravi u `Avatar.tsx`. Snimku (PNG) ostavi u scratchpadu i navedi put u izvještaju — kontrolor je pokazuje Leonu.

- [ ] **Step 1:** Testovi za pravila 1, 2, 3, 6 (pad). **Step 2:** palete i nizovi, `LOOK_SCHEMA`, `DEFAULT_LOOK`,
  `normalizeLook`, `lookFromSeed` proširenje, poznati likovi. **Step 3:** slojevi u `Avatar.tsx`. **Step 4:** galerija.
  **Step 5:** testovi zeleni; provjere (sve četiri). **Step 6:** vizualna provjera (pravilo 8). **Step 7:** commit
  „Ekrani 11/13: više izgleda likova — obrve, nos, dob, nakit, odjeća“.

---


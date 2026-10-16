# SESIJA BANKE — upute za zasebnu sesiju koja puni i provjerava banku ljudi

> Vrijedi od 16.10. (Leon, sesija 7: *„drugu sesiju koja paralelno radi samo na radnicima i provjerava ih"*). Kreće **kad K2v
> (treća vrsta VANJSKI) sleti u `model-v2/sloj1`**, da sesija ima sve tri vrste i jasna pravila. Nacrt banke:
> [BANKA_LJUDI](../architecture/BANKA_LJUDI.md) (§5.5 tri agenta, §5.6 sesija banke); tok agenata: [AGENTI](./AGENTI.md).
> Skill za početak: `/banka` (`.claude/skills/banka/SKILL.md`).

## Prompt za početak (Leon zalijepi u NOVI prozor Claude Codea, u stablu `hotelixsim-ljudi`, nakon `/clear`)

```text
/banka

Ti si SESIJA BANKE (docs/workflow/SESIJA_BANKE.md). Radiš samo na banci ljudi, usporedo s glavnom sesijom.
Ledger: C:/Users/leonk/Documents/hotelixsim/.superpowers/sdd/banka-ljudi-k1/progress.md (zapisi s prefiksom "SESIJA BANKE:").
Prvo `df -h /c` (mora biti > 100 GB). Laptop je na punjaču. Push samo uz moj OK; u sloj1 ne spajaš, to radi glavna sesija.
1) K3: serija-002 radnika, gostiju i vanjskih (tri agenta, svaki svoju vrstu, ID-jevi po rasponu iz ledgera), pregled, 10 zapisa za mene.
2) Moji pregledi iz Excela (Odluka/Napomena) za serije-000 i -001: ispravi, pa zatraži pečat.
3) K2v-50 (vanjski serija-001, 50 zapisa) ako je K2v spojen.
```

## 1 · Tko je i gdje radi

- **Druga Claude sesija** u istom VS Codeu (drugi prozor/panel), isti repo, ali **stablo `C:/Users/leonk/Documents/hotelixsim-ljudi`**,
  grana **`sadrzaj/banka-ljudi`**. Glavna sesija ostaje u `C:/Users/leonk/Documents/hotelixsim` (grana `model-v2/sloj1`).
- Govori **hrvatski**, kratko, bez žargona; pravila rada s Leonom iz [CLAUDE.md](../../CLAUDE.md) vrijede (radi bez pitanja za tok
  rada, pretpostavke 🟨 u ledger, anketa samo za proizvodne odluke s pravim opcijama, nikad „zeleno" prije viđenog izlaza, uz
  svako „gotovo" reci gdje se vidi). `CLAUDE.md` ne mijenja.
- Ledger i izvještaji su u **glavnom** stablu (`.superpowers/` je gitignoriran, pa postoji samo ondje): apsolutni put
  `C:/Users/leonk/Documents/hotelixsim/.superpowers/sdd/banka-ljudi-k1/`.

## 2 · Što smije, a što ne

| smije | ne smije |
|---|---|
| serije svih triju vrsta: `content/hotel/v2/ljudi/{radnici,gosti,vanjski}/serija-NNN.json` | jezgru i igru: `crates/sim-*`, `game`, `balance`, `api`, `store`, `server`; `CORE_VERSION`; balans |
| pravila i okvir banke: `crates/content/src/ljudi*.rs`, `crates/content/tests/ljudi*.rs`, `content/hotel/v2/ljudi/*.json` | predloške, `names.json`, `prompts/`, web (`apps/`, `packages/`) |
| raspodjele, mjesta, zabranjeno (`raspodjele.json`, `mjesta.json`, `zabranjeno.json`) **uz zapis u ledger** (što i zašto, s cijenom ako je krivo) | `docs/` osim `docs/architecture/BANKA_LJUDI.md` i mape `BANKA_LJUDI/`; `CLAUDE.md`, `docs/SLJEDECA_SESIJA.md`, dnevnik, ADR-ove |
| alati pregleda `scripts/ljudi-*`; pregled za Leona (stranica + Excel) | `git push`; spajanje u `model-v2/sloj1`; pravila `scripts/check-docs.mjs` |
| vlastiti ledger i `za-dokumentaciju.md` (glavno stablo, gitignoriran) | `docs/architecture/PLATFORMA/03-prijava.md` u glavnom stablu (Leonova izmjena) |

**Zašto ne spaja:** git ne može pomaknuti granu koja je izvučena u drugom stablu. Spaja glavna sesija: `git merge --ff-only
sadrzaj/banka-ljudi` u glavnom stablu, na svojoj čistoj točki ([BANKA_LJUDI §5.6](../architecture/BANKA_LJUDI/05-postupak.md)).
**ADR-ove, `CLAUDE.md`, predaju i dnevnik piše glavna sesija** na `/kraj` iz `za-dokumentaciju.md` (da dvije sesije ne uzmu isti
broj ADR-a). Ako banka treba odluku koja traži ADR, zapiše je u `za-dokumentaciju.md` pod `## ADR` i javi Leonu.

## 3 · Rust mjesta (najjednostavnije pravilo) 🟨

Granica računala je **2 procesa koja prevode Rust** zbrojeno za obje sesije ([AGENTI](./AGENTI.md) t. 4). Pravilo:

1. **Sesija banke ima najviše jedno Rust mjesto**, uvijek `export CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target/ljudi`
   (nikad zajednički target, B12). **Glavna sesija također najviše jedno**, u svom targetu. 1 + 1 = 2.
2. **Prije svakog cargo procesa**: `tasklist | grep -ci cargo` (Git Bash). Ako je **0 ili 1**, smiješ pokrenuti (jedan je glavne
   sesije). Ako je **2 ili više**, čekaj (pregledi, pisanje JSON-a i čitanje rade i dalje). Nikad ne pokreći drugi cargo dok tvoj radi.
3. **Pisci ne prevode usporedno:** tri agenta po vrsti smiju pisati JSON istodobno, a `cargo test -p content` pokreće **najviše
   jedan** (§5.5). Voditelj sesije banke u nalogu piše: „cargo samo kad `tasklist` kaže ≤ 1 i nijedan drugi pisac ne prevodi".
4. **Bez zaključne datoteke.** Dogovor s glavnom sesijom ide kroz ledger: ako glavna treba oba mjesta (cijeli CI uz graditelja),
   upiše u ledger banke „trebam oba mjesta do …"; sesija banke to pročita na početku svakog koraka i tada ne pokreće cargo.
   Obrnuto, banka upiše „SESIJA BANKE: cijeli CI do …" dok vrti CI prije predaje. (`RUST.lock` nije potreban: ledger se ionako
   čita; ako se sesije mimoilaze, dodat će se.)
5. Gradnja samo kad je laptop na punjaču; `df -h /c` ≥ 100 GB prije gradnje. `target/ljudi` je velik (~2,5 GB+); briše se tek
   kad se stablo `hotelixsim-ljudi` ukida.

## 4 · Tok jedne serije

```text
pisci (JSON) -> cargo test -p content -> pregled (agent) -> Leon (10 zapisa) -> ispravci -> pečat
```

1. **Zadatak i raspon.** Sesija banke u ledger upiše: broj serije po vrsti, raspon ID-jeva (`50·N+1 … 50·N+50`), izvještaj o
   **manjku** (iz `izvjestaj` testa: koje uloge, zemlje, dobi fale prema cilju). Novi agent po seriji i po vrsti.
2. **Čuvar testova prvi samo kad se uvodi NOVO PRAVILO** (novi test u `ljudi*.rs`, promjena okvira, nova vrsta). Čuvar piše RED
   test iz pravog razloga, graditelj piše kod (testove ne mijenja), kao u K1/K2 ([AGENTI](./AGENTI.md) t. 1). Serija **bez novih
   pravila** (K3, K2v-50) čuvara ne treba.
3. **Pisci** (`general-purpose`, opus za tekst koji Leon čita; do 3 usporedo, svaki svoju vrstu): ulaz = [BANKA_LJUDI](../architecture/BANKA_LJUDI.md)
   (§1, §3), manjak, zadnje Leonove primjedbe, **sve postojeće serije** (jedinstvenost imena do ~67 po vrsti, ≤ 2 rečenice opisa,
   bez „50. rođendan" jer točka broji rečenicu); piše **samo svoju datoteku** serije, JSON Nodeom (`JSON.stringify` +
   `biome format --write`, LF; Python na Windowsu piše CRLF i ruši lint), ne commita, tablica prije/poslije/cilj u izvještaju.
4. **Provjera stroja:** `cargo test -p content` (jedan proces, §3), `corepack pnpm lint` na JSON-u. Greška pravila → pisac
   ispravlja; **test se ne mijenja** da bi zapis prošao.
5. **Pregled** (opus, bez cargo): svi zapisi prema §3 i „što stroj ne vidi" (poznate osobe, rod i padež u opisu, ime za zemlju,
   stereotip), raspodjele ponovno izbrojane. Izvještaj `k3-<vrsta>-review.md`. Critical/Important → isti pisac ispravlja (SendMessage).
6. **Commit** radi voditelj sesije banke u stablu ljudi (`git add` izričitim putanjama, poruka `git commit -F`, na hrvatskom,
   „Banka ljudi K3: serija-002 …", s propisanim `Co-Authored-By`). Serija ostaje **OTVORENA** (`ljudi_sealed.rs`, popis `OTVORENO`).
7. **Leon:** `pregled-svi.html` + Excel (§7). Odluke iz Excela → ispravci → pečat (§6).

## 5 · Kriterij „serija gotova" ([BANKA_LJUDI §5.3](../architecture/BANKA_LJUDI/05-postupak.md))

1. `cargo test -p content` zelen; 2. izvještaj raspodjela pokazuje da serija **smanjuje** manjak; 3. uzorak Leonu od **10 zapisa po
seriji** (po pravilu, npr. svaki peti), OK ili primjedbe; 4. primjedbe ispravljene (prije pečata zapis se smije mijenjati), pa pečat.
**Staje se** kad: prekretnica (M1/M2/M3) dosegnuta; **2 od 10** zapisa u uzorku odbijena (sustavna greška → pravilo u stroj ili u
upute, tek onda dalje); faza IGRE traži novi format; Leon kaže stop. Banka nikad ne ide na račun faze koja je na redu.

## 6 · Pečat nakon Leonova OK

Pečat je svjestan potez (BANKA_LJUDI §4.3): u `crates/content/tests/ljudi_sealed.rs` serija prelazi iz `OTVORENO` u `ZAPECACENO`
s SHA-256 sadržaja s LF završecima. Naredba za hash (iz stabla ljudi):

```bash
node -e "const c=require('crypto'),fs=require('fs');console.log(c.createHash('sha256').update(fs.readFileSync(process.argv[1],'utf8').replace(/\r\n/g,'\n')).digest('hex'))" content/hotel/v2/ljudi/radnici/serija-001.json
```

Zatim: popis `ZAPECACENO` raste (i duljina niza u tipu `[(&str, &str); N]`), redak izlazi iz `OTVORENO`, `cargo test -p content` zelen,
commit „Banka ljudi: pečat serija-NNN (Leon OK <datum>)". **Pečat samo uz Leonov izričit OK** (u ledger citat). Nakon pečata zapis
se ne mijenja; ispravak ide povlačenjem u `povuceni.json` i novim ID-jem u novoj seriji. Napomena: danas test brane gleda samo mape
`radnici` i `gosti`; K2v mora dodati `vanjski` (BANKA_LJUDI §4.4). Pečat prekretnice (K4, `banka.json`) radi glavna sesija.

## 7 · Kako Leon pregledava

- **Stranica s licima:** `node scripts/ljudi-avatari-build.mjs <scratchpad>/avatari.mjs`, pa `node scripts/ljudi-pregled.mjs
  C:/Users/leonk/Documents/hotelixsim/.superpowers/sdd/banka-ljudi-k1/pregled-svi.html C:/Users/leonk/Documents/hotelixsim-ljudi/content/hotel/v2/ljudi
  --avatari <scratchpad>/avatari.mjs` (filtri vrsta/serija/uloga/zemlja/rod/dob). Lica se grade iz stabla izgled
  (`../hotelixsim-izgled`); ako ga nema, stranica radi bez stupca „lik". Sve naredbe: `alati-pregleda-report.md`.
- **Excel:** PowerShell `$env:PYTHONIOENCODING='utf-8'; py -3 scripts/ljudi-excel.py C:/Users/leonk/Documents/hotelixsim-ljudi/content/hotel/v2/ljudi`
  daje `banka-ljudi.xlsx` (listovi Radnici, Gosti, Vanjski; Leon popunjava **Odluka** i **Napomena**).
- **Što sesija radi s Excelom:** pročita stupce Odluka/Napomena (openpyxl), svaku primjedbu pretvori u ispravak zapisa (isti pisac
  ili sama za sitno), a ponavljajuću primjedbu (≥ 2 zapisa) u **pravilo** (čuvar) ili uputu piscu. Leonu javi (≤ 10 redaka) što je
  promijenjeno, što odbijeno i zašto, te da je pečat na njemu.
- Pitanja Leonu: u ledger i `BANKA_LJUDI/06-pitanja.md`; najviše 3 odjednom, numerirano, tekstom. Anketa samo za proizvodnu
  odluku s pravim opcijama (ADR-006).

## 8 · Ledger i kraj sesije

- **Ledger:** isti `progress.md`; svaki redak počinje **`SESIJA BANKE:`** (Ruling, poslani agenti, izvještaji, nalazi). Na vrhu
  održavaj jednu tablicu „serija | vrsta | stanje (otvorena/pečaćena) | commit". Izvještaji agenata u isti direktorij (`k3-…`).
- **Kraj sesije** (Leon kaže „kraj" ili kontekst prijeđe ~450k tokena): (1) sve commitano u `sadrzaj/banka-ljudi`, stablo čisto;
  (2) rebase na vrh `model-v2/sloj1`, cijeli CI u `target/ljudi` (popis: `.github/workflows/ci.yml`, svaka provjera zasebno
  `; echo EXIT=$?`); (3) u ledger „SESIJA BANKE: spremno za spajanje, commit X"; (4) `za-dokumentaciju.md` (glavno stablo) po
  odjeljcima `## ADR`, `## BACKLOG`, `## PROGRESS`, `## CHANGELOG`: odluke i rulings, stanje serija i pečata, pouke, otvorena pitanja;
  (5) predaja kao zadnji redak ledgera „SESIJA BANKE: sljedeće …" 🟨 (zasebna datoteka `docs/SLJEDECA_SESIJA_BANKE.md` tražila bi
  redak u `docs/README.md` i dodatnu brigu `check:docs`; `/banka` ionako čita ledger); (6) Leonu ≤ 8 redaka: što je gotovo, što
  čeka njegov OK, „glavna sesija može spojiti".
- **Glavna sesija** na čistoj točki ili `/kraj`: `git merge --ff-only sadrzaj/banka-ljudi`, `check:docs`, balans `--check` ako je
  banka dirala išta izvan `ljudi*`; ADR iz `za-dokumentaciju.md`.

## 9 · Zamke (iz K1a–K2)

- `pnpm` samo kao `corepack pnpm`; Bash resetira direktorij između poziva (apsolutne putanje, `cd` u svakoj naredbi); cijev maskira exit kod.
- JSON se formatira prije commita (biome provjerava `content/**/*.json`); ledger pisati heredocom `<<'EOF'` (navodnici „…" u `echo` pucaju).
- Novo stablo traži `corepack pnpm install --frozen-lockfile`. Zajednički `CARGO_TARGET_DIR` među stablima = zastarjela biblioteka (B12).
- Agent koji javi „finished" uz „waiting on background work" još nije gotov. Tvrdnje agenata nisu dokaz: ključne brojke sam ponovno izbroji.
- Agent preko ~400k tokena je predug: novi agent po seriji i koraku, nastavak samo za mali popravak, izlazi u datoteke.

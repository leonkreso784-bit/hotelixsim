# BANKA_LJUDI §5 · Postupak punjenja korak po korak

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md).

## 5.1 Koraci

| korak | što | tko | gotovo kada |
|---|---|---|---|
| **K0** | ovaj dokument → Leonovi odgovori (§6) → ADR | glavna sesija, Leon | ADR s odlukama |
| **K1** ✅ 16.10. (K1a adf5a4d, K1b) | format u Rustu (`ljudi.rs`, `ljudi_check.rs`), `raspodjele.json`, `mjesta.json`, `zabranjeno.json`, `ljudi_sealed.rs`; **20 ručnih primjera** (10 + 10) u `serija-000` i po **jedan namjerno loš primjer za svako pravilo** (test pada s jasnom porukom) | jedan agent (Rust) | `cargo test -p content` zelen; svako pravilo §3.6 ima test |
| **K2** (u tijeku 16.10.: serija-001 u stablu ljudi, 918e8e8, OTVORENA) | prva serija: **50 radnika + 50 gostiju** | agent RADNICI, agent GOSTI | Leon pregleda **svih 100**; nalazi postaju pravila (stroj) ili upute agentima |
| **K2v** (Leon 16.10.) | treća vrsta VANJSKI (§1.5): format, pravila i testovi u Rustu + **10 primjera** u `vanjski/serija-000.json`; kartica `.superpowers/sdd/banka-ljudi-k1/kartica-k2v.md` | čuvar, graditelj (jedan Rust proces), pisac VANJSKI za 10 primjera | `cargo test -p content` zelen; svako pravilo §3.7 ima test; Leon pregleda 10 primjera |
| **K2v-50** | prva serija vanjskih: **50 zapisa** (`vanjski/serija-001.json`) | agent VANJSKI | Leon pregleda svih 50 (kao K2) |
| **K3** | serije po 50, svaka uz provjeru i izvještaj raspodjela | tri agenta, paralelno (§5.5) | „serija gotova" (§5.3) |
| **K4** | pečat prekretnice (M1, M2, M3): `banka.json` nova verzija, raspodjela banke ≤ 3 p. b. od cilja | glavna sesija | izvještaj zelen; balans `--check` |
| **K5** | korištenje u igri: L2a-1/L2a-2 (početni ljudi), L2a-6 (burza), L2b-3 (agencija), R4-1 (recenzije), gosti i vanjski situacija (§4.4) | faze IGRE ([ROADMAP/igra](../../plan/ROADMAP/igra.md)) | po ciglama faza, ne ovdje |

## 5.2 Agenti i datoteke
- **Agent RADNICI** piše samo `radnici/serija-NNN.json`, **agent GOSTI** samo `gosti/serija-NNN.json`, **agent VANJSKI** samo
  `vanjski/serija-NNN.json` (§5.5). Zajedničke datoteke (`banka.json`, `raspodjele.json`, `povuceni.json`, `ljudi_sealed.rs`,
  Rust kod) mijenja **samo sesija koja vodi banku** (od 17.10. paralelna sesija banke, §5.6).
- **Raspon ID-jeva i broj serije** dodjeljuje glavna sesija u zadatku; **sudar imena** dvaju agenata hvata provjera pri
  spajanju, a ispravlja ga agent kasnije serije.
- **Novi agent za svaku seriju** (pouka „agenti i tokeni": jedan agent ne smije narasti preko ~400k tokena). Ulaz: ovaj
  dokument, ciljevi, izvještaj o **manjku** u banci (koje uloge, zemlje, dobi fale) i zadnje Leonove primjedbe. Izlaz:
  datoteka serije i kratki izvještaj u datoteci. Najviše dva agenta koja prevode Rust u isto vrijeme.
- Zasebna grana `sadrzaj/banka-ljudi` u stablu `../hotelixsim-ljudi` (od K1): nove datoteke + retci modula u
  `crates/content/src/lib.rs`, bez sudara sa slojem 1; svako spajanje je ff na `model-v2/sloj1` (§5.6).

## 5.3 Kriterij „serija gotova"
1. `cargo test -p content` zelen (oblik, jedinstvenost, dosljednost, zabranjene riječi, raspodjele po podrijetlu);
2. izvještaj raspodjela pokazuje da serija **smanjuje** manjak prema cilju, ne povećava ga;
3. uzorak Leonu: od K3 **10 zapisa po seriji** izabranih po pravilu (npr. svaki peti), Leon kaže OK ili daje primjedbe;
4. primjedbe su ispravljene (prije pečata se zapis smije mijenjati), pa se serija pečati.

## 5.4 Kada se staje
**Prekretnica dosegnuta** (M1, M2, M3) — staje se dok faza koja koristi banku ne kaže treba li više. **Sustavna greška**
(2 ili više od 10 zapisa u uzorku odbijeno) — stop, pravilo ide u stroj ili upute, tek onda dalje. **Promjena formata**
koju traži faza IGRE — stop do novog formata (§4.3). **Leon kaže stop** — banka nikad ne ide na račun faze koja je na redu.

## 5.5 Tri agenta, svaki svoju vrstu (Leonova odluka 16.10.)

| agent | piše samo | ulaz osim ovog dokumenta | Rust |
|---|---|---|---|
| **RADNICI** | `radnici/serija-NNN.json` | izvještaj manjka radnika (uloga, razina, dob, podrijetlo, zemlje, rod po ulozi) | jedan `cargo test -p content` na kraju |
| **GOSTI** | `gosti/serija-NNN.json` | izvještaj manjka gostiju (zemlja, segment, dob, rod) | isto |
| **VANJSKI** | `vanjski/serija-NNN.json` | izvještaj manjka vanjskih (uloga, rod po ulozi, dob) i popis `types` po ulozi iz predložaka (§1.5) | isto |

- Pisci **ne commitaju** i ne diraju Rust ni okvir; seriju commita sesija koja vodi banku nakon provjere (kao K2, 918e8e8).
- Tri pisca smiju raditi usporedo jer pišu samo JSON; **cargo pokreće najviše jedan od njih u isto vrijeme** (prije pokretanja
  `tasklist | grep -i cargo`, kao u K2), pa banka nikad ne zauzme više od jednog Rust mjesta.
- Sudar imena među tri serije hvata `cargo test -p content` (puno ime jedinstveno u cijeloj banci, §3.1); ispravlja ga pisac
  kasnije serije. Raspon ID-jeva i broj serije za svaku vrstu dodjeljuje sesija banke u nalogu.

## 5.6 Paralelna sesija banke (Leonove odluke 16. i 17.10.)

Banka ima **svoju sesiju Claude Codea koja radi ISTODOBNO s glavnom** (popravak temelja, faze IGRE); nije nova ni zamjenska
sesija. To je drugi kolosijek (ADR-052 t. 2), ne nova faza. Leon 17.10.: *„što više ljudi to bolje, uvijek se mogu koristiti"*:
nakon pečata serija 000 i 001 i koraka K2v paralelna sesija puni nove serije (K3: 50 + 50 radnika i gostiju, vanjski po
prekretnicama §2.4). Prompt za početak (Leon ga zalijepi u novi prozor, ne `/newsession`):
`.superpowers/sdd/banka-ljudi-k1/PROMPT-PARALELNA-SESIJA.md`. **Spaja glavna sesija** (niže).

| | sesija banke | glavna sesija |
|---|---|---|
| **smije** | pečat serija (prvi: 000 i 001, odlučen 17.10.), serije svih triju vrsta, pravila i okvir banke (`crates/content/src/ljudi*.rs`, `crates/content/tests/ljudi*.rs`, `content/hotel/v2/ljudi/**`), alati pregleda (`scripts/ljudi-*`), pregledi za Leona, ovaj dokument i njegovu mapu, svoj ledger | sve ostalo |
| **ne smije** | jezgru i igru (`crates/sim-*`, `game`, `balance`, `api`), predloške i `names.json`, web, `CORE_VERSION`, balans; `CLAUDE.md`, predaju, dnevnik, ADR-ove (vidi niže) | mijenjati datoteke banke dok sesija banke radi |
| **Rust** | najviše **1** proces, uvijek `CARGO_TARGET_DIR=…/hotelixsim/target/ljudi` | najviše **1** proces, svoj target |

- **Grana i stablo:** `sadrzaj/banka-ljudi` u `../hotelixsim-ljudi`. Prije spajanja sesija banke rebasea granu na vrh
  `model-v2/sloj1` i pusti cijeli CI u `target/ljudi`; **spaja glavna sesija** (`git merge --ff-only sadrzaj/banka-ljudi` u
  glavnom stablu, na svojoj čistoj točki), jer git ne pomiče granu koja je izvučena u drugom stablu.
- **Ledger:** `.superpowers/sdd/banka-ljudi-k1/progress.md` (postoji od K1); bilješke za dokumente u `za-dokumentaciju.md` istog
  direktorija. **ADR-ove piše glavna sesija** na `/kraj` iz tih bilješki, da dvije sesije ne uzmu isti broj ADR-a; isto vrijedi za
  `CLAUDE.md`, predaju i dnevnik.
- **Dva Rust mjesta** 🟨: dok obje sesije rade, svaka ima točno jedno mjesto (1 + 1 = granica 2). Glavna sesija koja treba oba
  (npr. cijeli CI uz graditelja) upiše u ledger banke „trebam oba mjesta do …", a sesija banke tada ne pokreće cargo. Prije
  svakog cargo procesa obje provjere `tasklist` (broj `cargo.exe`); gradnja samo na punjaču, `df` ≥ 100 GB prije gradnje.
- **Leon** vidi banku kroz `pregled-svi.html` i Excel s tri lista (radnici, gosti, vanjski); pitanja sesije banke idu u njezin
  ledger i u §6, a ankete samo za proizvodne odluke (ADR-006).

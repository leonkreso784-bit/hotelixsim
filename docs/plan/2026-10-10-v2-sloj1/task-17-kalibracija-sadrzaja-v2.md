### Task 17: Kalibracija sadržaja v2

**Veličina:** M–L, 3–6 h (iterativno) · **Ovisi o:** Task 16, Task 10 · **Paralelno s:** Task 15 · **Model:** opus
(prosudba o brojkama)

**Cilj:** `--check` prolazi za v2 tjedno i mjesečno (1000 sjemena), a v1 ostaje kakav je.

**Files:**
- Modify: **samo** `content/hotel/v2/{departments,profiles,economy,periods}.json`, `.github/workflows/ci.yml` (dva
  nova retka balansa s `--seeds 100`)
- Ne dirati: nijedan `.rs` (ako provjera ne može proći bez promjene formule ili praga → stani, zapiši u izvještaj s
  brojkama i prijedlogom; voditelj donosi Ruling).

**Postupak:**
- [ ] **Step 1:** Ulaz: popis padova iz Taska 16. Za svaku brojku koju mijenjaš zapiši u izvještaj: staro → novo,
  koja provjera, zašto je nova vrijednost uvjerljiva za hrvatski hotel 4* na moru (oznaka [PROC]).
- [ ] **Step 2: Redoslijed podešavanja** (svaki krug: `cargo run -q -p balance --release -- --model v2 --round week --seeds 200`):
  (1) početni indeksi profila (stanja odjela, zadovoljstva, raspodjela osoblja); (2) marže odjela (prosječni račun,
  udio korištenja, trošak namirnica, plaće odjela); (3) GOP marža 35–45 % (stalni dio neraspoređenih, energija po noći,
  fiksne naknade se **ne** diraju da bi se postigla GOP marža — one su ispod GOP-a); (4) udio rada 30–40 %; (5) tjedni
  koeficijenti (`periods.json` `week`) za tjedno ≈ mjesečno; (6) pravednost i dominantne probe.
- [ ] **Step 3: Završna provjera** (svaka zasebno, EXIT=0):
  `cargo run -q -p balance --release -- --seeds 1000 --check` ·
  `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check` ·
  `cargo run -q -p balance --release -- --model v2 --round month --seeds 1000 --check` ·
  `cargo test --workspace` (testovi s brojkama iz sadržaja v2 — npr. `content --test v2_departments` — ne smiju ovisiti
  o vrijednostima koje si promijenio; ako ovise, test je loš: javi).
- [ ] **Step 4:** U `.github/workflows/ci.yml` iza postojećeg retka balansa dodaj
  `- run: cargo run -q -p balance --release -- --model v2 --round week --seeds 100 --check` i isto s `--round month`.
- [ ] **Step 5: Commit** „Sloj 1/17: kalibracija v2 — balans prolazi za tjedan i mjesec" s tablicom promjena u poruci.

---


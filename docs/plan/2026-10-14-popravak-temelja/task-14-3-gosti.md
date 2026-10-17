Dio 3 cigle [L1-13](task-14-l1-13-kalibracija.md) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 3 · situacije s gostima rastu s popunjenošću (A7) — `l113-3-gosti`

**Nalaz 18.10.:** broj već raste s **očekivanom popunjenošću kruga** (`crates/sim-hotel-v2/src/situations/count.rs:74-81`, niska /
srednja / visoka; zadano tjedno 3/4/5, mjesečno 4/5/6), ali dodatna mjesta puni bilo koji predložak. Novo pravilo (uz ključ):
mjesta **iznad `low`** (`SituationCounts::levels().0`) smiju puniti samo situacije s gostima, pa je očekivani broj ostalih isti na
svakoj popunjenosti. Ukupni broj ostaje u postavci igre; prve 3 ostaju obvezne (`game::MANDATORY_SITUATIONS`). Izvor popunjenosti
ostaje očekivana popunjenost kruga (ne prošli krug kao u obrisu; poznata je na početku kruga).

**„S gostima" 🟨:** uloga `from: guest` ili odjel `*guest_facing` (`SituationSpec::guest_related`: `RoleSource::Guest`, `DeptTarget::GuestFacing`,
`template.rs:59-80`, `:192-200`); danas 27 od 55. Strogo „isti broj ostalih u svakom krugu" ne gradi se (sukob sa sidrima i raznolikošću
vrsta); vrijedi u prosjeku. Cijena: Leon može vidjeti krug s dvije negostinjske uz nisku popunjenost.

**Files:**
- Modify: `crates/sim-hotel-v2/src/situations/template.rs:14-29` (`guest_extra_slots`), `:59-80` (`guest_related`);
  `crates/sim-hotel-v2/src/situations/draw.rs:182-187` (uz `round_count` `round_base`), `:465-558` (kad je ključ uključen i
  `pre + drawn ≥ round_base`, `allowed` zadržava samo `guest_related`); `crates/content/src/templates.rs:12-25` i `:35-54`
  (`guest_extra_slots: Option<bool>`, `serde(default, skip_serializing_if)`), `crates/content/src/templates_core.rs:15-` (`to_core`);
  `crates/balance/src/v2_situations.rs:170-190` (`DrawStats`: gostinjske i ostale po razini popunjenosti), ispis bloka J-3 u
  `crates/balance/src/v2.rs` (jedan redak; v2.rs ima 936 od 1000 dopuštenih redaka, kod ispisa ide u `v2_situations.rs`).
- Ne dirati: `count.rs` (broj se ne mijenja), `situation_templates.json` (ključ tek u dijelu 4), igra (izvlačenje je zajedničko).

**Testovi čuvara** (`crates/sim-hotel-v2/tests/situations_guest_slots.rs` nova; RED: polje i metoda ne postoje):
1. `extra_slots_above_low_are_guest_related` — 1000 sjemena, visoka popunjenost: negostinjskih primjeraka ≤ `low` − sidra u svakom krugu.
2. `more_occupancy_never_fewer_guest_situations` — prosjek gostinjskih po krugu: visoka ≥ srednja ≥ niska.
3. `other_situations_same_on_average` — prosjek ostalih po krugu, niska prema visokoj, razlika ≤ 0,15 🟨.
4. `flag_off_is_todays_draw` — ključ `false`: izlaz `draw_situations_with` jednak današnjem za 200 sjemena (karakterizacija).
5. (content) `guest_related_templates_are_27` (popis); `missing_guest_flag_reads_false`.
6. (balans) `report_prints_guest_and_other_counts` (novi redak u bloku J-3).
Ostaju zeleni: `v2_sealed`, paritet igre i balansa.

**Koraci graditelja:** ključ i metoda → izvlačenje → ispis → `cmp` balansa: razlika **samo novi redak** → probni balans s ključem `true`
(prosjek po krugu i po sezoni, težina) → sve provjere. **Rizik:** gostinjskih kandidata ponekad nema (uvjeti, razmak), pa krug ima manje
situacija; prosjek po sezoni (danas tjedno 64,67) mora se vidjeti u izvještaju.

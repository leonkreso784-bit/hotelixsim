# ADR-009 — Ojačavanje brana determinizma je Task 1 plana „jezgra i alat za balans", prije ijedne linije jezgre
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Claude, nakon završnog pregleda kostura; Leon ne mora potvrđivati) ·
**Vezano:** [ADR-003](ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md), [ADR-008](ADR-008-odluke-iz-gradnje-kostura-koje-je.md) · **Izvori:** [BRANE_DETERMINIZMA](../../research/BRANE_DETERMINIZMA_2026-10-04.md),
[ledger kostura](../../archive/2026-10-04-kostur-ledger.md) (Final #3)

**Kontekst.** Završni pregled kostura: brane determinizma slabije su od ARHITEKTURA §6.2/§7.2 (`indexmap`, `RandomState`,
`as` na float/u16, `#[allow]` zaobilazi `deny`, `env::var`). Jezgra je još prazna, pa ništa ne puca.
**Odluka.** Nalaz ne blokira spajanje kostura, ali je OBAVEZAN prvi task sljedećeg plana: allowlist ovisnosti,
`RandomState` u `clippy.toml`, `forbid` + `as_conversions`/cast lintovi, zabrana `f32`/`f64`, probe mutacija kao skripta
u CI-ju. Pokus izvedivosti je izmjeren (npr. `forbid` radi samo pod `cargo clippy`, pa CI mora vrtjeti clippy).
**Posljedice.** Cijena ako je krivo: kod jezgre napisan prije jačih brana mogao bi proći s nedeterminizmom. Zato prvo
brane, pa jezgra. Plan se piše na temelju istraživanja iznad.

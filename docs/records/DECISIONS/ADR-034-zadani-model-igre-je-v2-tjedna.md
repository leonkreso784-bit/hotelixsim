# ADR-034 — Zadani model igre je v2, tjedna igra zadana
**Datum:** 2026-10-12 · **Status:** ✅ **Leonova odluka 12.10.** (anketa) · t. 3 🟨 (završni val, ZV-2) ·
**Vezano:** [ADR-029](ADR-029-sloj-1-pretpostavke-plana.md) P5, [ADR-028](ADR-028-leonove-odluke-za-put-b-ljudi.md) t. 5

**Kontekst.** P5 ([ADR-029](ADR-029-sloj-1-pretpostavke-plana.md)) je pretpostavio da se novi model bira izričito, a
da web ostaje na v1. Završni val je izbor modela stavio pod sklopivi „Za profesora: postavke nove igre", pa je student
na ekranu „Nova igra" mogao nehotice pokrenuti v1 demo. Leon je 12.10. u anketi odlučio da je novi model zadan.

**Odluka.**
1. **Početni ekran** (`StartScreen`; `/` bez igre i `/nova`) odmah nudi „Tjedna igra" (17 kraćih krugova, zadano) ili
   „Mjesečna igra" (4 kruga); obje šalju `POST /v1/demo` s `{ "model": "v2", "round_kind": "week" | "month" }`.
2. **Stari model (v1)** ostaje samo pod sklopivim „Za profesora" kao „Stari model (usporedba)" (prazno tijelo);
   v1 igre se i dalje zauvijek čitaju. API se ne mijenja: `POST /v1/demo` bez tijela i dalje stvara v1 (P5).
3. 🟨 **„Nova igra" na kraju sezone** je poveznica na `/nova` — isti `StartScreen` uz izlaz „Natrag na igru" koji ne
   šalje zahtjev. Postojeća igra ostaje dok igrač ne pokrene novu; `POST /v1/demo` ide tek tada, s postavkama kao s
   početnog ekrana (ZV-2; prije je „Nova igra" tiho pokretala v1).
4. Izbor je radio grupa s legendom, kartice `min-h-12`, fokus na oznaci ([UI_PRAVILA](../../product/UI_PRAVILA.md) §6).

**Posljedice.** Pitanje iz završnog vala „istaknutost i naziv Za profesora / zadani model v1" je zatvoreno ovom
odlukom. Tko pokreće igru kroz API bez tijela i dalje dobiva v1 demo (lipanj–srpanj).

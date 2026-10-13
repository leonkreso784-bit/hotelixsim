### Task 3 (K0-7): Arhitektura platforme u `docs/`

**Files:**
- Read: `.superpowers/sdd/2026-10-10-v2-sloj1/platforma-arhitektura.md` (720 redaka, nije u gitu)
- Create: `docs/architecture/PLATFORMA.md` (indeks ≤ 200) + `docs/architecture/PLATFORMA/NN-tema.md` (svaki ≤ 300)
- Modify: `docs/README.md` (redak u `architecture/`), `docs/product/IDEJE.md` (stupac „Gdje" za ideje 10–13 → `PLATFORMA.md`)

**Interfaces:** Consumes: ADR-037 t. 1–8, dopune i ISPRAVAK na kraju nacrta. Produces: `PLATFORMA.md` s tablicom „§ → datoteka".

- [ ] **Step 1: Provjera prije (čuvar testova)** — zapiši naredbu koja mora na kraju vratiti 0 pogodaka:
  `grep -rniE "partij[ae] po studentu|i mod 4|3 robota u razredu|zasebn(e|ih) partij|zatvara krug za sve|Supabase kao baza" docs/architecture/PLATFORMA*; echo EXIT=$?`
  (očekivano na kraju: EXIT=1, tj. nema pogodaka).
- [ ] **Step 2: Prijenos i prepis (graditelj)** — podijeli po smislu: 01 pojmovi i odgovor, 02 uloge i razred, 03 prijava,
  04 generirani hoteli, 05 igrini slojevi, 06 produkcija i trošak, 07 GDPR i sigurnost, 08 faze i pitanja. Prepiši
  zastarjelo po ADR-037: razred = jedna igra s N hotela; obračun po hotelu i vrata kruga; lista kolega; vidljivi radnici
  i natjecanje za ljude; sat i semestar, neaktivni, obavijesti; računi studenata (korisničko ime + lozinka) pa AAI;
  hosting po dopunama 13.10. (Railway Hobby + Postgres na Railwayu, ~5–10 €/mj, Supabase samo kao IdP kasnije);
  faze F0–F5 → imena faza iz [ROADMAP](../../plan/ROADMAP.md). Odluke se ne prepisuju iz ADR-a: poveznica na ADR-037. Brojke
  troška ostaju s oznakom [PROC]. Na vrhu indeksa: „nacrt za raspravu (ADR-006); odluke su u ADR-037".
- [ ] **Step 3:** naredba iz Step 1 → EXIT=1; `corepack pnpm check:docs; echo EXIT=$?` → 0; `node scripts/check-docs.mjs --popis`
  pokazuje sve nove dijelove ispod granice.
- [ ] **Step 4: Commit** — `docs/architecture/PLATFORMA.md docs/architecture/PLATFORMA/ docs/README.md docs/product/IDEJE.md`.

---

### Završetak faze

- [x] Provjeravatelj (svjež agent) pregleda sva tri commita naspram ovog plana i ADR-038; nalazi idu graditelju (završni pregled
  13.10.: 5 nalaza, popravljeni u `f5871aa`, ponovni pregled čist).
- [x] Voditelj: plan u `docs/archive/` kad su Taskovi 1–3 gotovi i K0-1 potvrđen u CI-ju (nakon Leonova OK-a za push);
  K0-4/5/6 ostaju u [ROADMAP/kvaliteta](../../plan/ROADMAP/kvaliteta.md) do spajanja grana; K0-8 je Leonova sjednica.

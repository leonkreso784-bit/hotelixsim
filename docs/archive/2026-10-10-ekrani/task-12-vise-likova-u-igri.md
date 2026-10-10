### Task 12: Više likova u igri

Svaki lik koji se u igri pojavi dobiva lice. Ide **nakon** Taskova 5–10 (ekrani) i Taska 11 (spojen), jer lica stavlja u
te ekrane.

**Files:**
- Modify: `apps/web/src/avatar/look.ts` (poznati likovi), `apps/web/src/avatar/avatar.test.tsx`,
  `apps/web/src/screens/month/Leaderboard.tsx`, `apps/web/src/screens/MonthScreen.test.tsx`,
  `apps/web/src/screens/SituationsScreen.tsx`, `apps/web/src/screens/SituationScreen.test.tsx`,
  `apps/web/src/screens/HotelScreen.tsx`, `apps/web/src/screens/HotelScreen.test.tsx`,
  `apps/web/src/screens/FeedScreen.tsx`, `apps/web/src/screens/FeedScreen.test.tsx`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `Avatar`, `lookFor`, `PLAYER_LOOK`, `MENTOR_LOOK`, `KNOWN_LOOKS` (Task 3/11); `Leaderboard` (Task 9),
  `SituationsScreen` (Task 6), `HotelScreen` (Task 5), `FeedScreen` (Task 10).
- Produces: upravitelji robotskih hotela u `KNOWN_LOOKS` pod imenima hotela iz igre: `"Štediša"` (stariji, naočale
  `square`, `outfit: "blazer"`, `neckwear: "tie"`, kosa `slick` sijeda), `"Brižni"` (`mouth: "smile"`, `outfit: "shirt"`,
  `hair: "wavy"`, `earrings: "hoops"`), `"Lovac na popunjenost"` (`hat: "cap"`, `outfit: "tshirt"`, `mouth: "grin"`).

**Zahtjevi (svaki s testom):**
1. **Ljestvica:** svaki redak ima lik (`Avatar`, 32 px) — moj hotel `PLAYER_LOOK`, roboti `lookFor(r.name)`. Test: lista
   „Ljestvica napretka“ ima 4 `svg`-a; redak „Štediša“ ima `data-part="glasses-square"`.
2. **Popis situacija:** situacija s likom pokazuje lik (`lookFor(character.name)`, 40 px) lijevo od naslova; bez lika
   (S3) bez slike. Test: poveznica S1 sadrži `svg`, poveznica S3 ne.
3. **Moj hotel:** uz naslov „Moj hotel“ tvoj lik (`PLAYER_LOOK`, 48 px) i tekst „Ti, upravitelj“ (`hotel.you`). Test.
4. **Tijek:** stavka mentora pokazuje `MENTOR_LOOK` (28 px) uz rečenicu. Test: redak „Mentor: …“ sadrži `svg`.
5. Svi likovi su ukras (`aria-hidden`) — ime stoji kao tekst uz njih.

- [ ] Koraci: testovi (pad) → izmjene → provjere (sve četiri) → commit „Ekrani 12/13: lica za ljestvicu, situacije, hotel i tijek“.

---


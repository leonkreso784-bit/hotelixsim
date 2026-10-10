### Task 9: `api` i `server` — rute razgovora i slobodnog odgovora, AI iz okoline

**Files:**
- Modify: `crates/api/src/hotels.rs`, `crates/api/src/schema.rs`, `crates/api/src/problem.rs`, `crates/api/src/lib.rs`
- Modify: `crates/server/Cargo.toml` (`ai`), `crates/server/src/main.rs`, `.env.example`, `README.md` (Lokalni razvoj: ključ)
- Regenerate: `openapi.json`, `packages/api-client/src/` (`corepack pnpm gen:api`)
- Test: `crates/api/tests/hotels.rs`, `crates/api/tests/http.rs`

**Interfaces:**
- Consumes: `app::{send_message, conversation, submit_response, ConversationView, SituationsView, ResponseView}`.
- Produces (rute; tijela `TextBody { text: String }`, `maxLength` 2000 u OpenAPI-ju):
  - `POST /v1/hotels/{hotel_id}/situations/{code}/messages` → **202** `ConversationBody`
  - `GET  /v1/hotels/{hotel_id}/situations/{code}/conversation` → 200 `ConversationBody`
  - `POST /v1/hotels/{hotel_id}/situations/{code}/response` → **202** `SituationsBody`
  - `SituationBody` dobiva `free_text`, `character` (`{name, role}` ili `null`), `response` (`ResponseBody` ili `null`);
    `ResponseBody.status` je enum `pending | judged | neutral`; `CriterionBody { n, text, met, reason }`.
  - `LastMonthBody.mentor: Option<String>`; `FeedItemBody` dobiva `response`, `verdict`, `mentor` (diskriminator `kind`,
    kao postojeće).
  - `ProblemCode`: `not_free_text`, `chat_limit`, `awaiting_reply`, `bad_text`.

- [ ] **Step 1: Napiši testove koji padaju** — u `crates/api/tests/hotels.rs` (postojeći obrazac s `memory::deps`):
  (a) poruka liku → 202 i `awaiting_reply: true`; (b) 7. poruka (nakon 6 odgovora lika preko `run_next_job`) → 409
  `chat_limit`; (c) poruka dok lik nije odgovorio → 409 `awaiting_reply`; (d) slobodan odgovor na S3 → 422
  `not_free_text`; (e) prazan tekst → 422 `bad_text`; (f) tekst od 2001 znaka → 422 `bad_text`; (g) slobodan odgovor
  pa ponuđeni na S1 → 409 `already_answered`; (h) bez kolačića 401, tuđi hotel 403 za sve tri rute; (i) nakon
  `run_next_job` situacije pokazuju `response.status == "judged"` i 5 kriterija s tekstom rubrike; (j) feed i
  `last_month.mentor` nakon zatvaranja lipnja. U `http.rs`: OpenAPI sadrži tri nove putanje i nove kodove u `ProblemCode`.

- [ ] **Step 2: Pokreni i provjeri da pada** — `cargo test -p api; echo EXIT=$?`.
- [ ] **Step 3: Implementiraj rute i DTO-e** po uzoru na `answer` u `hotels.rs` (redoslijed provjera ADR-017: 401 → 404
  → 403 prije 422). Kvaliteta i mehanike **ponuđenih** odgovora i dalje nikad ne izlaze.
- [ ] **Step 4: Server** — `main.rs`:

```rust
fn ai_port() -> Arc<dyn app::AiPort> {
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.trim().is_empty() => {
            let d = ai::Models::default();
            let pick = |var: &str, def: String| std::env::var(var).ok().filter(|v| !v.trim().is_empty()).unwrap_or(def);
            let models = ai::Models {
                judge: pick("HOTELIXSIM_AI_MODEL_PRESUDA", d.judge),
                character: pick("HOTELIXSIM_AI_MODEL_LIK", d.character),
                mentor: pick("HOTELIXSIM_AI_MODEL_MENTOR", d.mentor),
            };
            tracing::info!(presuda = %models.judge, lik = %models.character, mentor = %models.mentor, "AI uključen");
            Arc::new(ai::Anthropic::new(k, models))
        }
        _ => {
            tracing::warn!("ANTHROPIC_API_KEY nije postavljen: AI je isključen, igra radi na rezervama");
            Arc::new(app::DisabledAi)
        }
    }
}
```

  `deps()` dobiva `ai: ai_port()`. `.env.example` dodaje:

```text
# Claude API ključ za presude, likove i mentora (nikad u git!). Prazno = igra radi na rezervama.
ANTHROPIC_API_KEY=
# HOTELIXSIM_AI_MODEL_PRESUDA=claude-opus-5-5
# HOTELIXSIM_AI_MODEL_LIK=claude-sonnet-5-5
# HOTELIXSIM_AI_MODEL_MENTOR=claude-sonnet-5-5
```

  README „Lokalni razvoj" korak 2: jedna rečenica — ključ upiši u `.env` (`ANTHROPIC_API_KEY=…`); bez njega igra radi na
  rezervama. Provjeri da je `.env` u `.gitignore` (`git check-ignore .env` mora ispisati `.env`).
- [ ] **Step 5: Regeneriraj klijent** — `corepack pnpm gen:api; echo EXIT=$?`, zatim `typecheck`, `test`, `lint`.
- [ ] **Step 6: Pokreni sve provjere** — sve `EXIT=0`.
- [ ] **Step 7: Commit** — „AI sloj 9/11: rute razgovora i slobodnog odgovora, AI iz okoline".

---


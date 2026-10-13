### Task 7: `store` — red poslova s odgodom: `run_after`, povećanje razmaka, `defer`, starost posla

**Files:**
- Create: `crates/store/migrations/0004_jobs_run_after.sql`
- Modify: `crates/store/src/jobs.rs`, `.sqlx/` (regeneracija)
- Test: `crates/store/tests/` (postojeća datoteka testova reda poslova — `grep -ln "claim" crates/store/tests`)

**Interfaces:**
- Consumes: `JobQueue::defer`, `Job.age_secs` (Task 6).
- Produces: Postgres izvedba istog ugovora koji `memory` već ispunjava.

- [ ] **Step 1: Napiši testove koji padaju** (`#[sqlx::test]`, samo `sqlx::query(...)` funkcije u testovima):
  (a) `failed_job_is_not_claimable_before_backoff` — `claim`, `fail` → odmah `claim` vraća `None`; nakon
  `UPDATE jobs SET run_after = now()` vraća isti posao s `attempts = 2`;
  (b) `defer_keeps_attempts_and_delays` — `claim` (attempts 1), `defer(claim, 60)` → status `queued`, `attempts = 0`,
  `claim` vraća `None`; nakon `run_after = now()` vraća posao s `attempts = 1`;
  (c) `defer_with_stale_claim_does_nothing` — drugi radnik je preuzeo posao; `defer` sa starim `JobClaim` ne mijenja red;
  (d) `age_secs_comes_from_created_at` — `UPDATE jobs SET created_at = now() - interval '200 seconds'` → `claim` vraća
  `age_secs >= 200`;
  (e) `decide_enqueues_many_jobs` — `decide` s dva `NewJob` upiše oba; ponovljeni ključ se preskače.

- [ ] **Step 2: Pokreni i provjeri da pada** — `bash scripts/baza.sh status`, pa `cargo test -p store; echo EXIT=$?`.

- [ ] **Step 3: Migracija** — `0004_jobs_run_after.sql`:

```sql
-- Odgoda posla (ARHITEKTURA §9.4, ADR-015 t. 8 → korak 4): ponovni pokušaj s povećanjem razmaka i čekanje presuda.
ALTER TABLE jobs ADD COLUMN run_after TIMESTAMPTZ NOT NULL DEFAULT now();
DROP INDEX jobs_ready_idx;
CREATE INDEX jobs_ready_idx ON jobs (run_after, id) WHERE status IN ('queued', 'running');
```

- [ ] **Step 4: Upiti** — `claim`: uvjet `(status = 'queued' AND run_after <= now()) OR (status = 'running' AND
  locked_until < now())`, i `RETURNING …, EXTRACT(EPOCH FROM (now() - created_at))::bigint AS "age_secs!"`.
  `fail`: kad ide natrag u red, `run_after = now() + make_interval(secs => LEAST(60, power(2, attempts))::float8)`
  (2, 4, 8, 16 s; najviše 60). `defer`:

```sql
UPDATE jobs SET status = 'queued', attempts = attempts - 1, locked_until = NULL,
       run_after = now() + make_interval(secs => $3::float8), updated_at = now()
WHERE id = $1 AND attempts = $2 AND status = 'running'
```

  Zatim (baza radi): `cargo sqlx prepare --workspace` i dodaj nove `.sqlx/query-*.json`, obriši zastarjele.

- [ ] **Step 5: Pokreni sve provjere** — sve `EXIT=0`, uključujući `cargo test -p store` na pravoj bazi.
- [ ] **Step 6: BACKLOG** — iz stavke `[dnevnik] → korak 4` obriši „ponovni pokušaj s odgodom" i „zakašnjela AI presuda
  za zatvorenu rundu" (ostaju `Idempotency-Key` i probni test apalis/pgmq, s napomenom „odgođeno, ADR-019").
- [ ] **Step 7: Commit** — „AI sloj 7/11: red poslova s odgodom, povećanjem razmaka i starošću posla".

---


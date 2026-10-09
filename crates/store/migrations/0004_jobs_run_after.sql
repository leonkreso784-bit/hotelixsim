-- Odgoda posla (ARHITEKTURA §9.4, ADR-015 t. 8 → korak 4): ponovni pokušaj s povećanjem razmaka i čekanje presuda.
ALTER TABLE jobs ADD COLUMN run_after TIMESTAMPTZ NOT NULL DEFAULT now();
DROP INDEX jobs_ready_idx;
CREATE INDEX jobs_ready_idx ON jobs (run_after, id) WHERE status IN ('queued', 'running');

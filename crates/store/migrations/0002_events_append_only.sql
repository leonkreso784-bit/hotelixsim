-- Dnevnik je samo za dopisivanje (ARHITEKTURA §9.1, ADR-015 t. 5): baza odbija izmjenu, brisanje i pražnjenje.
-- Uloga bez prava UPDATE/DELETE dolazi prije Railwaya (BACKLOG); okidač radi već sad, i u sqlx::test bazama.
CREATE INDEX events_game_idx ON events (game_id, id);

ALTER TABLE events ADD CONSTRAINT events_actor_format
    CHECK (actor ~ '^(guest:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|robot:[a-z_]+|system)$');

CREATE FUNCTION events_append_only() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'tablica events je samo za dopisivanje (% odbijen)', TG_OP
        USING ERRCODE = 'insufficient_privilege';
END
$$;

CREATE TRIGGER events_no_update_delete BEFORE UPDATE OR DELETE ON events
    FOR EACH ROW EXECUTE FUNCTION events_append_only();
CREATE TRIGGER events_no_truncate BEFORE TRUNCATE ON events
    FOR EACH STATEMENT EXECUTE FUNCTION events_append_only();

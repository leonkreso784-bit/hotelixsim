-- Dnevnik događaja: izvor istine (ARHITEKTURA §9.1). Samo dopisivanje.
-- Zabrana UPDATE/DELETE preko zasebne uloge dolazi u planu „Dnevnik, runde i red poslova".
CREATE TABLE events (
    id           BIGSERIAL PRIMARY KEY,
    game_id      UUID        NOT NULL,
    stream       TEXT        NOT NULL,
    stream_seq   INTEGER     NOT NULL,
    type         TEXT        NOT NULL,
    schema_ver   INTEGER     NOT NULL,
    payload      JSONB       NOT NULL,
    actor        TEXT        NOT NULL,
    recorded_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (stream, stream_seq)
);

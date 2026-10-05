-- Platformske tablice (ARHITEKTURA §9.3): obične tablice za pretragu; istina o igri je u events.
CREATE TABLE guests (
    id          UUID PRIMARY KEY,
    token_hash  TEXT NOT NULL UNIQUE,          -- "sha256:<hex>" tokena iz kolačića; sam token se ne sprema
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE games (
    id          UUID PRIMARY KEY,
    seq         BIGSERIAL UNIQUE,              -- redoslijed stvaranja („zadnja igra gosta"), neovisno o satu
    created_by  UUID NOT NULL REFERENCES guests (id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE hotels (
    id        UUID PRIMARY KEY,
    game_id   UUID NOT NULL REFERENCES games (id),
    slot      SMALLINT NOT NULL CHECK (slot BETWEEN 1 AND 4),
    guest_id  UUID REFERENCES guests (id),     -- NULL = robot
    UNIQUE (game_id, slot)
);
CREATE INDEX hotels_guest_idx ON hotels (guest_id);

-- Red poslova (ARHITEKTURA §9.4, ADR-015 t. 8).
CREATE TABLE jobs (
    id            BIGSERIAL PRIMARY KEY,
    job_key       TEXT NOT NULL UNIQUE,        -- npr. resolve:<game>:<mjesec>; drugi upis istog ključa se preskače
    kind          TEXT NOT NULL,
    game_id       UUID NOT NULL,
    payload       JSONB NOT NULL,
    status        TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'done', 'failed')),
    attempts      INTEGER NOT NULL DEFAULT 0,
    locked_until  TIMESTAMPTZ,
    last_error    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX jobs_ready_idx ON jobs (id) WHERE status IN ('queued', 'running');

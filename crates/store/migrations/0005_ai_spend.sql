-- Knjiga troška AI-ja (pilot, dio 3): jedan red po odgovoru Anthropica (i odrezanom). Zbrojevi po UTC danu i mjesecu
-- čitaju se po `at_unix`; iznos je u nano-dolarima (1 $ = 10^9) da cijene po tokenu ostanu cijeli brojevi.
CREATE TABLE ai_spend (
    id              BIGSERIAL PRIMARY KEY,
    at_unix         BIGINT NOT NULL,
    model           TEXT NOT NULL,
    input_tokens    BIGINT NOT NULL,
    output_tokens   BIGINT NOT NULL,
    cache_write     BIGINT NOT NULL,
    cache_read      BIGINT NOT NULL,
    cost_nano_usd   BIGINT NOT NULL CHECK (cost_nano_usd >= 0)
);
CREATE INDEX ai_spend_at_idx ON ai_spend (at_unix);

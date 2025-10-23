-- Biggest native integer in Postgres is BIGINT (signed 64-bit).
-- Switch to NUMERIC if you need beyond ~9e18.
CREATE TABLE IF NOT EXISTS transaction_count (
    t_address TEXT PRIMARY KEY,
    count   BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS funds_moved (
    t_address TEXT PRIMARY KEY,
    amount  BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS dark_burned_sats (
    t_address TEXT PRIMARY KEY,
    q_address  TEXT NOT NULL DEFAULT 0,
    amount  BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS dark_minted_sats (
    t_address TEXT PRIMARY KEY,
    q_address  TEXT NOT NULL DEFAULT 0,
    amount  BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS lit_minted_sats (
    t_address TEXT PRIMARY KEY,
    amount  BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS lit_burned_sats (
    t_address TEXT PRIMARY KEY,
    amount  BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS addr_mappings (
    t_address TEXT NOT NULL,
    q_address  TEXT NOT NULL
);
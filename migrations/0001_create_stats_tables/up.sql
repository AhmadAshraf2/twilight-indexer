-- Biggest native integer in Postgres is BIGINT (signed 64-bit).
-- Switch to NUMERIC if you need beyond ~9e18.
CREATE TABLE IF NOT EXISTS transactions (
    t_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    primary key (t_address, block);

CREATE TABLE IF NOT EXISTS funds_moved (
    t_address TEXT,
    amount BIGINT NOT NULL DEFAULT 0,
    denom TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS dark_burned_sats (
    t_address TEXT PRIMARY KEY,
    q_address TEXT NOT NULL DEFAULT '',
    amount BIGINT NOT NULL DEFAULT 0,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS dark_minted_sats (
    t_address TEXT PRIMARY KEY,
    q_address TEXT NOT NULL DEFAULT '',
    amount BIGINT NOT NULL DEFAULT 0,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS lit_minted_sats (
    t_address TEXT PRIMARY KEY,
    amount BIGINT NOT NULL DEFAULT 0,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS lit_burned_sats (
    t_address TEXT PRIMARY KEY,
    amount BIGINT NOT NULL DEFAULT 0,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS addr_mappings (
    t_address TEXT NOT NULL,
    q_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (t_address, q_address)
    
);

CREATE TABLE IF NOT EXISTS gas_used_nyks (
    t_address TEXT NOT NULL,
    gas_amount BIGINT NOT NULL,
    denom TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS qq_tx (
    tx TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Fixed: Added comma after to_address
CREATE TABLE IF NOT EXISTS trading_tx (
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Fixed: Added comma after to_address
CREATE TABLE IF NOT EXISTS order_open_tx (
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Fixed: Added comma after to_address
CREATE TABLE IF NOT EXISTS order_close_tx (
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Remove duplicates from addr_mappings before adding unique constraint
WITH ranked AS (
    SELECT ctid,
           ROW_NUMBER() OVER (PARTITION BY t_address, q_address ORDER BY t_address) AS rn
    FROM addr_mappings
)
DELETE FROM addr_mappings a
USING ranked r
WHERE a.ctid = r.ctid
  AND r.rn > 1;

-- Add unique index to support ON CONFLICT (t_address, q_address)
CREATE UNIQUE INDEX IF NOT EXISTS idx_addr_mappings_t_q_unique
ON addr_mappings (t_address, q_address);
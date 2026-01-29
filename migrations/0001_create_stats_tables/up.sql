-- Biggest native integer in Postgres is BIGINT (signed 64-bit).
-- Switch to NUMERIC if you need beyond ~9e18.

CREATE SCHEMA IF NOT EXISTS chain_indexer;
SET search_path TO chain_indexer;

CREATE TABLE IF NOT EXISTS transactions (
    t_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (t_address, block)
);

CREATE TABLE IF NOT EXISTS funds_moved (
    t_address TEXT NOT NULL,
    amount BIGINT NOT NULL DEFAULT 0,
    denom TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (t_address, denom, block)
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
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (t_address, block)
);

CREATE TABLE IF NOT EXISTS qq_tx (
    tx_hash TEXT NOT NULL,
    tx TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tx_hash, block)
);

CREATE TABLE IF NOT EXISTS trading_tx (
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (to_address, from_address, block)
);

CREATE TABLE IF NOT EXISTS order_open_tx (
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (to_address, from_address, block)
);

CREATE TABLE IF NOT EXISTS order_close_tx (
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (to_address, from_address, block)
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block);
CREATE INDEX IF NOT EXISTS idx_funds_moved_block ON funds_moved(block);
CREATE INDEX IF NOT EXISTS idx_addr_mappings_t_address ON addr_mappings(t_address);
CREATE INDEX IF NOT EXISTS idx_addr_mappings_q_address ON addr_mappings(q_address);
CREATE INDEX IF NOT EXISTS idx_gas_used_nyks_block ON gas_used_nyks(block);
CREATE INDEX IF NOT EXISTS idx_qq_tx_block ON qq_tx(block);
CREATE INDEX IF NOT EXISTS idx_trading_tx_from ON trading_tx(from_address);
CREATE INDEX IF NOT EXISTS idx_order_open_tx_from ON order_open_tx(from_address);
CREATE INDEX IF NOT EXISTS idx_order_close_tx_from ON order_close_tx(from_address);

-- Drop the old qq_tx table and recreate with a hash-based primary key
DROP TABLE IF EXISTS qq_tx;

CREATE TABLE qq_tx (
    tx_hash TEXT NOT NULL,
    tx TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tx_hash, block)
);

CREATE INDEX IF NOT EXISTS idx_qq_tx_block ON qq_tx(block);

-- Revert to original qq_tx table
DROP TABLE IF EXISTS qq_tx;

CREATE TABLE qq_tx (
    tx TEXT NOT NULL,
    block BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tx, block)
);

CREATE INDEX IF NOT EXISTS idx_qq_tx_block ON qq_tx(block);

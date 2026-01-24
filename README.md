# Twilight Indexer

A Rust-based blockchain indexer and REST API for the Twilight Project's ZKOS chain. Connects to a Cosmos-based blockchain, indexes transactions, stores statistics in PostgreSQL, and exposes the data through a REST API with Swagger documentation.

## Overview

Twilight Indexer bridges Bitcoin and Cosmos ecosystems by tracking:
- Bitcoin bridge operations (deposits/withdrawals)
- Satoshi movements between funding and trading accounts
- QuisQuis (QQ) privacy-preserving transactions
- Standard Cosmos transactions (bank, staking, governance)

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                    Twilight Indexer                        │
├────────────────────────┬───────────────────────────────────┤
│   REST API Server      │   Background Indexer              │
│   (Actix-web)          │   (30s polling interval)          │
│                        │                                   │
│ • 10 Query Endpoints   │ • Block fetching                  │
│ • Swagger UI           │ • Transaction parsing             │
│ • OpenAPI Schema       │ • Database inserts                │
├────────────────────────┴───────────────────────────────────┤
│                   PostgreSQL Database                       │
│              (12 tables for blockchain stats)               │
├────────────────────────────────────────────────────────────┤
│            Twilight Cosmos Chain (LCD/RPC API)             │
└────────────────────────────────────────────────────────────┘
```

## Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- libpq development libraries

```bash
# macOS
brew install postgresql libpq

# Ubuntu/Debian
sudo apt-get install postgresql libpq-dev
```

## Quick Start

### 1. Configure Environment

```bash
cp .env.example .env
```

Edit `.env`:
```bash
DATABASE_URL=postgres://postgres:postgres@127.0.0.1/stats

NYKS_BLOCK_SUBSCRIBER_URL=https://lcd.twilight.rest/
NYKS_LCD_BASE_URL=https://lcd.twilight.rest/
NYKS_RPC_BASE_URL=https://rpc.twilight.rest/

ENABLE_API=true
ENABLE_INDEXER=true
API_HOST=127.0.0.1
API_PORT=8449

BLOCK_HEIGHT_FILE=height.txt
```

### 2. Setup Database

```bash
cargo install diesel_cli --no-default-features --features postgres
diesel setup
diesel migration run
```

### 3. Build and Run

```bash
cargo build --release
./target/release/twilight_indexer
```

API available at `http://127.0.0.1:8449`

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `NYKS_BLOCK_SUBSCRIBER_URL` | required | LCD endpoint for block fetching |
| `NYKS_LCD_BASE_URL` | required | LCD base URL |
| `NYKS_RPC_BASE_URL` | required | RPC base URL |
| `ENABLE_API` | `true` | Enable REST API server |
| `ENABLE_INDEXER` | `true` | Enable blockchain indexer |
| `API_HOST` | `127.0.0.1` | API listen address |
| `API_PORT` | `8449` | API listen port |
| `BLOCK_HEIGHT_FILE` | `height.txt` | Persist indexer progress |
| `RUST_LOG` | `info` | Log level |

## API Endpoints

**Swagger UI:** `http://localhost:8449/swagger-ui/`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/decode-transaction` | Decode transaction bytecode |
| GET | `/api/transactions/{t_address}` | Transaction count |
| GET | `/api/funding/{t_address}` | Funding transfers |
| GET | `/api/exchange-withdrawal/{t_address}` | Trading → Funding transfers |
| GET | `/api/exchange-deposit/{t_address}` | Funding → Trading transfers |
| GET | `/api/btc-deposit/{t_address}` | BTC deposits |
| GET | `/api/btc-withdrawal/{t_address}` | BTC withdrawals |
| GET | `/api/qq-account/{t_address}` | QuisQuis account mappings |
| GET | `/api/address/{t_address}/all` | All address statistics |

See [API_DOCUMENTATION.md](API_DOCUMENTATION.md) for detailed documentation.

## Database Schema

| Table | Purpose |
|-------|---------|
| `transactions` | Transaction counts per address |
| `funds_moved` | Funding-to-funding transfers |
| `dark_burned_sats` | Trading → Funding (exchange withdrawals) |
| `dark_minted_sats` | Funding → Trading (exchange deposits) |
| `lit_minted_sats` | BTC deposits to Twilight |
| `lit_burned_sats` | BTC withdrawals from Twilight |
| `addr_mappings` | Twilight ↔ QuisQuis address mappings |
| `gas_used_nyks` | Gas consumption per address |
| `qq_tx` | Raw QuisQuis transactions |
| `trading_tx` | Trading transactions |
| `order_open_tx` | Order opens |
| `order_close_tx` | Order closes |

## Supported Transaction Types

**Cosmos Standard:**
- Bank: MsgSend, MsgMultiSend
- Staking: MsgDelegate, MsgUndelegate, MsgBeginRedelegate
- Distribution: MsgWithdrawDelegatorReward
- Governance: MsgSubmitProposal, MsgVote, MsgDeposit

**NYKS Bridge:**
- MsgConfirmBtcDeposit, MsgWithdrawBtcRequest
- MsgRegisterBtcDepositAddress, MsgRegisterReserveAddress
- MsgWithdrawTxSigned, MsgWithdrawTxFinal
- MsgProposeSweepAddress, MsgSignSweep, MsgSignRefund

**ZKOS:**
- MsgTransferTx (QuisQuis transfers)
- MsgMintBurnTradingBtc (Exchange operations)

## Project Structure

```
twilight-indexer/
├── src/
│   ├── main.rs              # Entry point
│   ├── api.rs               # REST API (Actix-web)
│   ├── db.rs                # Database operations (Diesel)
│   ├── schema.rs            # ORM table definitions
│   ├── transaction_types.rs # Transaction parsing
│   ├── block_types.rs       # Block structures
│   ├── pubsub_chain.rs      # Block polling
│   ├── quis_quis_tx.rs      # QQ transaction decoding
│   └── lib.rs               # Protobuf exports
├── migrations/              # Database migrations
├── proto/                   # Protobuf definitions
│   ├── bridgeTx.proto
│   └── zkosTx.proto
├── build.rs                 # Proto compilation
└── .env                     # Configuration
```

## Development

```bash
# Run with logging
RUST_LOG=debug cargo run

# Create migration
diesel migration generate <name>

# Run migrations
diesel migration run

# Reset indexer to specific block
echo "12345" > height.txt
```

## Troubleshooting

**Database connection failed:**
```bash
psql $DATABASE_URL -c "SELECT 1"
```

**Indexer not starting:**
Check blockchain endpoint accessibility:
```bash
curl https://lcd.twilight.rest/cosmos/base/tendermint/v1beta1/blocks/latest
```

**Re-index from scratch:**
```bash
diesel migration revert
diesel migration run
rm height.txt
cargo run
```

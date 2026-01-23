# Twilight Indexer API Documentation

API for querying Twilight's ZKOS blockchain statistics and transaction data.

**Base URL:** `http://{host}:{port}/api`

**Swagger UI:** `http://{host}:{port}/swagger-ui/`

**OpenAPI Schema:** `http://{host}:{port}/api-docs/openapi.json`

---

## Table of Contents

1. [Health Check](#1-health-check)
2. [Decode Transaction](#2-decode-transaction)
3. [Get Transactions](#3-get-transactions)
4. [Get Funds Moved (Funding to Funding)](#4-get-funds-moved-funding-to-funding)
5. [Exchange Withdrawal (Trading to Funding)](#5-exchange-withdrawal-trading-to-funding)
6. [Exchange Deposit (Funding to Trading)](#6-exchange-deposit-funding-to-trading)
7. [BTC Deposit](#7-btc-deposit)
8. [BTC Withdrawal](#8-btc-withdrawal)
9. [QQ Account Mapping](#9-qq-account-mapping)
10. [Get All Address Data](#10-get-all-address-data)

---

## Common Response Structure

All endpoints return JSON responses with the following base structure:

**Success Response:**
```json
{
  "success": true,
  "t_address": "twilight1...",
  // endpoint-specific data
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "Error description"
}
```

---

## Endpoints

### 1. Health Check

Check if the API service is running and healthy.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/health` |
| **Tag** | Health |

#### Request

No parameters required.

#### Response

**Status:** `200 OK`

```json
{
  "status": "healthy",
  "service": "twilight-indexer-api"
}
```

#### Example

```bash
curl -X GET "http://localhost:8080/api/health"
```

---

### 2. Decode Transaction

Decodes a transaction from its byte code representation. This endpoint parses the raw transaction bytes and returns a structured JSON representation of the transaction data, with scalar values converted to human-readable u64 integers.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **Path** | `/api/decode-transaction` |
| **Content-Type** | `application/json` |

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tx_byte_code` | string | Yes | The hex-encoded transaction byte code to decode |

```json
{
  "tx_byte_code": "0x123abc..."
}
```

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "tx_type": "transaction",
  "data": {
    // Decoded transaction structure with scalar values converted to u64
  }
}
```

**Status:** `400 Bad Request`

```json
{
  "success": false,
  "error": "Failed to decode transaction: <error details>"
}
```

#### Example

```bash
curl -X POST "http://localhost:8080/api/decode-transaction" \
  -H "Content-Type: application/json" \
  -d '{"tx_byte_code": "0x..."}'
```

---

### 3. Get Transactions

Returns the count of transaction blocks associated with a specific Twilight address.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/transactions/{t_address}` |
| **Tag** | Transactions |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query transactions for |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "transaction_count": 42
}
```

**Status:** `500 Internal Server Error`

```json
{
  "success": false,
  "error": "Failed to fetch transactions: <error details>"
}
```

#### Example

```bash
curl -X GET "http://localhost:8080/api/transactions/twilight1abc123..."
```

---

### 4. Get Funds Moved (Funding to Funding)

Returns all funds moved between funding accounts for a specific Twilight address. This tracks transfers within the funding layer of the system.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/funding/{t_address}` |
| **Tag** | Funding to Funding |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query funds moved |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "funds_moved": [
    {
      "amount": 100000,
      "denom": "sats",
      "block": 12345
    },
    {
      "amount": 50000,
      "denom": "sats",
      "block": 12350
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `funds_moved` | array | List of funding transfers |
| `funds_moved[].amount` | integer | Amount transferred (in satoshis) |
| `funds_moved[].denom` | string | Denomination of the transfer |
| `funds_moved[].block` | integer | Block height where the transfer occurred |

#### Example

```bash
curl -X GET "http://localhost:8080/api/funding/twilight1abc123..."
```

---

### 5. Exchange Withdrawal (Trading to Funding)

Returns the total amount of Nyks Sats moved from trading accounts to funding accounts (dark burned sats). This represents withdrawals from the trading/exchange layer back to the funding layer.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/exchange-withdrawal/{t_address}` |
| **Tag** | Trading to Funding |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query exchange withdrawals |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "dark_burned_sats": [
    {
      "q_address": "qq1xyz...",
      "amount": 75000,
      "block": 12346
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `dark_burned_sats` | array | List of trading-to-funding transfers |
| `dark_burned_sats[].q_address` | string | The QuisQuis address involved in the transfer |
| `dark_burned_sats[].amount` | integer | Amount transferred (in satoshis) |
| `dark_burned_sats[].block` | integer | Block height where the transfer occurred |

#### Example

```bash
curl -X GET "http://localhost:8080/api/exchange-withdrawal/twilight1abc123..."
```

---

### 6. Exchange Deposit (Funding to Trading)

Returns the total amount of Nyks Sats moved from funding accounts to trading accounts (dark minted sats). This represents deposits into the trading/exchange layer from the funding layer.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/exchange-deposit/{t_address}` |
| **Tag** | Funding to Trading |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query exchange deposits |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "dark_minted_sats": [
    {
      "q_address": "qq1xyz...",
      "amount": 100000,
      "block": 12340
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `dark_minted_sats` | array | List of funding-to-trading transfers |
| `dark_minted_sats[].q_address` | string | The QuisQuis address involved in the transfer |
| `dark_minted_sats[].amount` | integer | Amount transferred (in satoshis) |
| `dark_minted_sats[].block` | integer | Block height where the transfer occurred |

#### Example

```bash
curl -X GET "http://localhost:8080/api/exchange-deposit/twilight1abc123..."
```

---

### 7. BTC Deposit

Returns the total amount of Nyks Sats deposited from the Bitcoin chain to Twilight (lit minted sats). This tracks BTC bridged into the Twilight ecosystem.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/btc-deposit/{t_address}` |
| **Tag** | BTC Deposited |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query BTC deposits |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "lit_minted_sats": [
    {
      "amount": 500000,
      "block": 12300
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `lit_minted_sats` | array | List of BTC deposit events |
| `lit_minted_sats[].amount` | integer | Amount deposited (in satoshis) |
| `lit_minted_sats[].block` | integer | Block height where the deposit was recorded |

#### Example

```bash
curl -X GET "http://localhost:8080/api/btc-deposit/twilight1abc123..."
```

---

### 8. BTC Withdrawal

Returns the total amount of Nyks Sats withdrawn from Twilight to the Bitcoin chain (lit burned sats). This tracks BTC bridged out of the Twilight ecosystem.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/btc-withdrawal/{t_address}` |
| **Tag** | BTC Withdrawn |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query BTC withdrawals |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "lit_burned_sats": [
    {
      "amount": 250000,
      "block": 12400
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `lit_burned_sats` | array | List of BTC withdrawal events |
| `lit_burned_sats[].amount` | integer | Amount withdrawn (in satoshis) |
| `lit_burned_sats[].block` | integer | Block height where the withdrawal was recorded |

#### Example

```bash
curl -X GET "http://localhost:8080/api/btc-withdrawal/twilight1abc123..."
```

---

### 9. QQ Account Mapping

Returns all QuisQuis (QQ) accounts mapped to a specific Twilight address. QuisQuis accounts are used for privacy-preserving transactions in the trading layer.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/qq-account/{t_address}` |
| **Tag** | Twilight/qq mapping |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query QQ account mappings |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "q_addresses": [
    {
      "qq_account": "qq1xyz789...",
      "block": 12100
    },
    {
      "qq_account": "qq1def456...",
      "block": 12200
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `q_addresses` | array | List of QuisQuis account mappings |
| `q_addresses[].qq_account` | string | The QuisQuis account address |
| `q_addresses[].block` | integer | Block height where the mapping was created |

#### Example

```bash
curl -X GET "http://localhost:8080/api/qq-account/twilight1abc123..."
```

---

### 10. Get All Address Data

Returns aggregated data from all endpoints for a given Twilight address. This is a convenience endpoint that combines transaction count, funding transfers, exchange deposits/withdrawals, and BTC deposits/withdrawals in a single response.

| Property | Value |
|----------|-------|
| **Method** | `GET` |
| **Path** | `/api/address/{t_address}/all` |
| **Tag** | Stats |

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `t_address` | string | Yes | Twilight address to query all stats |

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "t_address": "twilight1abc123...",
  "transaction_count": 42,
  "funds_moved": [
    {
      "amount": 100000,
      "denom": "sats",
      "block": 12345
    }
  ],
  "dark_burned_sats": [
    {
      "q_address": "qq1xyz...",
      "amount": 75000,
      "block": 12346
    }
  ],
  "dark_minted_sats": [
    {
      "q_address": "qq1xyz...",
      "amount": 100000,
      "block": 12340
    }
  ],
  "lit_minted_sats": [
    {
      "amount": 500000,
      "block": 12300
    }
  ],
  "lit_burned_sats": [
    {
      "amount": 250000,
      "block": 12400
    }
  ]
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `transaction_count` | integer | Total number of transactions for this address |
| `funds_moved` | array | Funding-to-funding transfers (see [endpoint 4](#4-get-funds-moved-funding-to-funding)) |
| `dark_burned_sats` | array | Trading-to-funding transfers (see [endpoint 5](#5-exchange-withdrawal-trading-to-funding)) |
| `dark_minted_sats` | array | Funding-to-trading transfers (see [endpoint 6](#6-exchange-deposit-funding-to-trading)) |
| `lit_minted_sats` | array | BTC deposits (see [endpoint 7](#7-btc-deposit)) |
| `lit_burned_sats` | array | BTC withdrawals (see [endpoint 8](#8-btc-withdrawal)) |

#### Example

```bash
curl -X GET "http://localhost:8080/api/address/twilight1abc123.../all"
```

---

## HTTP Status Codes

| Code | Description |
|------|-------------|
| `200 OK` | Request was successful |
| `400 Bad Request` | Invalid request (e.g., malformed transaction bytecode) |
| `500 Internal Server Error` | Database or server error |

---

## Glossary

| Term | Description |
|------|-------------|
| **t_address** | A Twilight blockchain address |
| **q_address / qq_account** | A QuisQuis account address used for privacy-preserving transactions |
| **Nyks Sats** | The native token unit on the Twilight network (pegged to Bitcoin satoshis) |
| **Lit Minted/Burned** | BTC entering/leaving the Twilight ecosystem |
| **Dark Minted/Burned** | Funds moving between funding and trading layers |
| **Funding Account** | The standard account layer for holding funds |
| **Trading Account** | The privacy-preserving layer for trading operations |

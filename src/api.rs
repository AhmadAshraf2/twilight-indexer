use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use crate::quis_quis_tx::decode_transaction;
use crate::db;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// Request payload for decoding a transaction
#[derive(Debug, Deserialize, ToSchema)]
pub struct DecodeRequest {
    pub tx_byte_code: String,
}

/// Response for successful transaction decode (raw)
#[derive(Debug, Serialize, ToSchema)]
pub struct DecodeRawResponse {
    pub success: bool,
    pub tx_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<serde_json::Value>,
    pub data: serde_json::Value,
}

/// Error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

/// Response structs for individual endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionsResponse {
    pub success: bool,
    pub t_address: String,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FundsMovedResponse {
    pub success: bool,
    pub t_address: String,
    pub funds_moved: Vec<FundsMovedData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FundsMovedData {
    pub amount: i64,
    pub denom: String,
    pub block: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DarkBurnedSatsResponse {
    pub success: bool,
    pub t_address: String,
    pub dark_burned_sats: Vec<DarkBurnedSatsData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DarkBurnedSatsData {
    pub q_address: String,
    pub amount: i64,
    pub block: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DarkMintedSatsResponse {
    pub success: bool,
    pub t_address: String,
    pub dark_minted_sats: Vec<DarkMintedSatsData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DarkMintedSatsData {
    pub q_address: String,
    pub amount: i64,
    pub block: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LitMintedSatsResponse {
    pub success: bool,
    pub t_address: String,
    pub lit_minted_sats: Vec<LitMintedSatsData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LitMintedSatsData {
    pub amount: i64,
    pub block: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LitBurnedSatsResponse {
    pub success: bool,
    pub t_address: String,
    pub lit_burned_sats: Vec<LitBurnedSatsData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LitBurnedSatsData {
    pub amount: i64,
    pub block: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QAddressData {
    pub qq_account: String,
    pub block: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QAddressesResponse {
    pub success: bool,
    pub t_address: String,
    pub q_addresses: Vec<QAddressData>,
}

/// Combined response for all address data
#[derive(Debug, Serialize, ToSchema)]
pub struct AddressAllDataResponse {
    pub success: bool,
    pub t_address: String,
    pub transaction_count: i64,
    pub funds_moved: Vec<FundsMovedData>,
    pub dark_burned_sats: Vec<DarkBurnedSatsData>,
    pub dark_minted_sats: Vec<DarkMintedSatsData>,
    pub lit_minted_sats: Vec<LitMintedSatsData>,
    pub lit_burned_sats: Vec<LitBurnedSatsData>,
}

/// Known program types from twilight-client-sdk/relayerprogram.json
const PROGRAM_TYPES: &[(&str, &str)] = &[
    ("060a0402000000060a0e0401000000060a0402000000060a0e1013", "RelayerInitializer"),
    ("020403000000060a0401000000050d0401000000060a0d0401000000050e13", "CreateTraderOrder"),
    ("040300000002040300000002040a0000000603000000000a0b04070000000603000000000a04020000000c04020000000a0b04020000000a0c0404000000060a0b0c0302000000050d0307000000050d0407000000050403000000050b0c0406000000050d0407000000050d0403000000050c0e04010000000b0403000000060a0c0402000000060a0e101302", "SettleTraderOrder"),
    ("0401000000060a0302000000060a0306000000060a0c0e0403000000060a0304000000060a0307000000060a0c0e100401000000050402000000060a0405000000060a0d0c0402000000060a0403000000060a0d0e1013", "CreateLendOrder"),
    ("050304000000060a0307000000060a0d0c0302000000060a0306000000060a0d0e0406000000060a0b0403000000060a0c0402000000060a0e100401000000060a0402000000060a0403000000060a0b0c0e101302", "SettleLendOrder"),
    ("0202020202060a0401000000060a0407000000060a0c0e130202020202", "LiquidateOrder"),
    ("040300000002040300000002040a0000000603000000000a0b04070000000603000000000a04020000000c04020000000a0b04020000000a0c0404000000060a0c0302000000050d0307000000050d0407000000050403000000050b0c0406000000050d0407000000050d0403000000050c0e04010000000b0403000000060a0c0402000000060a0e101302", "SettleTraderOrderNegativeMarginDifference"),
];

/// Convert program bytes array to hex string
fn program_bytes_to_hex(program: &serde_json::Value) -> Option<String> {
    if let serde_json::Value::Array(bytes) = program {
        let hex: String = bytes.iter()
            .filter_map(|v| v.as_u64().map(|n| format!("{:02x}", n as u8)))
            .collect();
        Some(hex)
    } else {
        None
    }
}

/// Match program hex to known program type
fn get_program_type(program_hex: &str) -> &'static str {
    for (hex, name) in PROGRAM_TYPES {
        if *hex == program_hex {
            return name;
        }
    }
    "Unknown"
}

/// Get order type based on program type
fn get_order_type(program_type: &str) -> &'static str {
    match program_type {
        "RelayerInitializer" => "initialize_contract",
        "CreateTraderOrder" | "CreateLendOrder" => "order_open",
        "SettleTraderOrder" | "SettleLendOrder" | "LiquidateOrder" | "SettleTraderOrderNegativeMarginDifference" => "order_close",
        _ => "unknown"
    }
}

/// Extract scalar u64 value from Memo data array at a given index
fn extract_scalar_u64_from_data(data: &serde_json::Value, index: usize) -> Option<u64> {
    let arr = data.as_array()?;
    let item = arr.get(index)?;
    let scalar_obj = item.get("Scalar")?;
    let scalar_inner = scalar_obj.get("Scalar")?;
    let bytes = scalar_inner.as_array()?;
    // Convert first 8 bytes to u64 little-endian
    let mut arr_8 = [0u8; 8];
    for (i, byte_val) in bytes.iter().take(8).enumerate() {
        arr_8[i] = byte_val.as_u64()? as u8;
    }
    Some(u64::from_le_bytes(arr_8))
}

/// Decode program bytes into human-readable opcode list
fn decode_program_opcodes(program: &serde_json::Value) -> Vec<String> {
    let bytes: Vec<u8> = match program.as_array() {
        Some(arr) => arr.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect(),
        None => return vec![],
    };

    let mut opcodes = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let opcode = bytes[i];
        i += 1;

        let name = match opcode {
            0x00 => {
                // Push: read LE32 length, then skip data
                if i + 4 <= bytes.len() {
                    let len = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
                    i += 4 + len;
                    format!("push:{}", len)
                } else {
                    "push".to_string()
                }
            }
            0x01 => {
                // Program: read LE32 length, then skip data
                if i + 4 <= bytes.len() {
                    let len = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
                    i += 4 + len;
                    format!("program:{}", len)
                } else {
                    "program".to_string()
                }
            }
            0x02 => "drop".to_string(),
            0x03 => {
                // Dup: read LE32 index
                if i + 4 <= bytes.len() {
                    let idx = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    format!("dup:{}", idx)
                } else {
                    "dup".to_string()
                }
            }
            0x04 => {
                // Roll: read LE32 index
                if i + 4 <= bytes.len() {
                    let idx = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    format!("roll:{}", idx)
                } else {
                    "roll".to_string()
                }
            }
            0x05 => "scalar".to_string(),
            0x06 => "commit".to_string(),
            0x07 => "alloc".to_string(),
            0x0a => "expr".to_string(),
            0x0b => "neg".to_string(),
            0x0c => "add".to_string(),
            0x0d => "mul".to_string(),
            0x0e => "eq".to_string(),
            0x0f => "range".to_string(),
            0x10 => "and".to_string(),
            0x11 => "or".to_string(),
            0x12 => "not".to_string(),
            0x13 => "verify".to_string(),
            0x14 => "unblind".to_string(),
            0x15 => "issue".to_string(),
            0x16 => "borrow".to_string(),
            0x17 => "retire".to_string(),
            0x19 => "fee".to_string(),
            0x1a => "input".to_string(),
            0x1b => {
                // Output: read LE32 count
                if i + 4 <= bytes.len() {
                    let k = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    format!("output:{}", k)
                } else {
                    "output".to_string()
                }
            }
            0x1c => {
                // Contract: read LE32 count
                if i + 4 <= bytes.len() {
                    let k = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    format!("contract:{}", k)
                } else {
                    "contract".to_string()
                }
            }
            0x1d => "log".to_string(),
            0x1e => "call".to_string(),
            0x1f => "signtx".to_string(),
            0x20 => "signid".to_string(),
            0x21 => "signtag".to_string(),
            0x22 => {
                // InputCoin: read LE32 index
                if i + 4 <= bytes.len() {
                    let idx = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    format!("inputcoin:{}", idx)
                } else {
                    "inputcoin".to_string()
                }
            }
            0x23 => {
                // OutputCoin: read LE32 index
                if i + 4 <= bytes.len() {
                    let idx = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    format!("outputcoin:{}", idx)
                } else {
                    "outputcoin".to_string()
                }
            }
            _ => format!("ext:{:#04x}", opcode),
        };

        opcodes.push(name);
    }

    opcodes
}

/// Convert byte array Value to hex string
fn bytes_array_to_hex(value: &serde_json::Value) -> Option<String> {
    if let serde_json::Value::Array(bytes) = value {
        let hex: String = bytes.iter()
            .filter_map(|v| v.as_u64().map(|n| format!("{:02x}", n as u8)))
            .collect();
        Some(hex)
    } else {
        None
    }
}

/// Transform STATE data array to show meaningful labels for commitments
fn transform_state_data(data: &mut serde_json::Value) {
    if let serde_json::Value::Array(arr) = data {
        for (idx, item) in arr.iter_mut().enumerate() {
            if let serde_json::Value::Object(obj) = item {
                // Check if this is a Commitment
                if obj.contains_key("Commitment") {
                    let label = match idx {
                        0 => "total_locked_value",
                        1 => "total_pool_share",
                        _ => "commitment",
                    };
                    *item = serde_json::json!({
                        label: "(encrypted)"
                    });
                }
            }
        }
    }
}

/// Transform MEMO data array to show meaningful labels and values
fn transform_memo_data(data: &mut serde_json::Value) {
    if let serde_json::Value::Array(arr) = data {
        let mut new_data = Vec::new();

        for (idx, item) in arr.iter().enumerate() {
            if let serde_json::Value::Object(obj) = item {
                // Handle Scalar values
                if let Some(scalar_obj) = obj.get("Scalar") {
                    if let Some(scalar_inner) = scalar_obj.get("Scalar") {
                        if let Some(bytes) = scalar_inner.as_array() {
                            // Convert first 8 bytes to u64 little-endian
                            let mut arr_8 = [0u8; 8];
                            for (i, byte_val) in bytes.iter().take(8).enumerate() {
                                if let Some(b) = byte_val.as_u64() {
                                    arr_8[i] = b as u8;
                                }
                            }
                            let u64_val = u64::from_le_bytes(arr_8);

                            match idx {
                                0 => {
                                    // Position size (divided by 10^8)
                                    let pos_size = u64_val as f64 / 100_000_000.0;
                                    new_data.push(serde_json::json!({
                                        "position_size": pos_size
                                    }));
                                }
                                2 => {
                                    // Entry price
                                    new_data.push(serde_json::json!({
                                        "entry_price": u64_val
                                    }));
                                }
                                3 => {
                                    // Order side (1 = short, other = long)
                                    let side = if u64_val == 1 { "short" } else { "long" };
                                    new_data.push(serde_json::json!({
                                        "order_side": side
                                    }));
                                }
                                _ => {
                                    new_data.push(serde_json::json!({
                                        "scalar": u64_val
                                    }));
                                }
                            }
                        }
                    }
                }
                // Handle Commitment values
                else if obj.contains_key("Commitment") {
                    match idx {
                        1 => {
                            new_data.push(serde_json::json!({
                                "leverage": "(encrypted)"
                            }));
                        }
                        _ => {
                            new_data.push(serde_json::json!({
                                "commitment": "(encrypted)"
                            }));
                        }
                    }
                }
            }
        }

        *data = serde_json::Value::Array(new_data);
    }
}

/// Transform byte arrays in the decoded transaction to hex strings
fn transform_byte_arrays(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            // Convert txid field in utxo
            if let Some(txid) = map.get("txid") {
                if let Some(hex) = bytes_array_to_hex(txid) {
                    map.insert("txid".to_string(), serde_json::Value::String(hex));
                }
            }

            // Convert proof field
            if let Some(proof) = map.get("proof") {
                if let Some(hex) = bytes_array_to_hex(proof) {
                    map.insert("proof".to_string(), serde_json::Value::String(hex));
                }
            }

            // Convert sign field in witness
            if let Some(sign) = map.get("sign") {
                if let Some(hex) = bytes_array_to_hex(sign) {
                    map.insert("sign".to_string(), serde_json::Value::String(hex));
                }
            }

            // Convert value_proof.Dleq arrays to hex
            if let Some(serde_json::Value::Object(vp_map)) = map.get_mut("value_proof") {
                if let Some(serde_json::Value::Array(dleq)) = vp_map.get_mut("Dleq") {
                    for item in dleq.iter_mut() {
                        if let serde_json::Value::Array(inner) = item {
                            if inner.len() == 1 {
                                if let Some(arr) = inner.first() {
                                    if let Some(hex) = bytes_array_to_hex(arr) {
                                        *item = serde_json::Value::String(hex);
                                    }
                                }
                            } else if !inner.is_empty() && inner.first().map(|v| v.is_u64()).unwrap_or(false) {
                                // Direct byte array
                                if let Some(hex) = bytes_array_to_hex(item) {
                                    *item = serde_json::Value::String(hex);
                                }
                            }
                        }
                    }
                }
            }

            // Convert encrypt.c and encrypt.d byte arrays to hex strings
            if let Some(serde_json::Value::Object(encrypt_map)) = map.get_mut("encrypt") {
                if let Some(c) = encrypt_map.get("c") {
                    if let Some(hex) = bytes_array_to_hex(c) {
                        encrypt_map.insert("c".to_string(), serde_json::Value::String(hex));
                    }
                }
                if let Some(d) = encrypt_map.get("d") {
                    if let Some(hex) = bytes_array_to_hex(d) {
                        encrypt_map.insert("d".to_string(), serde_json::Value::String(hex));
                    }
                }
            }

            // Convert neighbors byte arrays to hex strings (in call_proof.path.neighbors)
            if let Some(serde_json::Value::Array(neighbors)) = map.get_mut("neighbors") {
                for item in neighbors.iter_mut() {
                    if let Some(hex) = bytes_array_to_hex(item) {
                        *item = serde_json::Value::String(hex);
                    }
                }
            }

            // Convert commitment.Closed byte array to hex string
            if let Some(serde_json::Value::Object(commitment_map)) = map.get_mut("commitment") {
                if let Some(closed) = commitment_map.get("Closed") {
                    if let Some(hex) = bytes_array_to_hex(closed) {
                        commitment_map.insert("Closed".to_string(), serde_json::Value::String(hex));
                    }
                }
            }

            // Convert Commitment.Closed byte array to hex string (capital C variant)
            if let Some(serde_json::Value::Object(commitment_map)) = map.get_mut("Commitment") {
                if let Some(closed) = commitment_map.get("Closed") {
                    if let Some(hex) = bytes_array_to_hex(closed) {
                        commitment_map.insert("Closed".to_string(), serde_json::Value::String(hex));
                    }
                }
            }

            // Convert witness zero_proof byte arrays to hex strings
            if let Some(serde_json::Value::Array(zero_proof)) = map.get_mut("zero_proof") {
                for item in zero_proof.iter_mut() {
                    if let Some(hex) = bytes_array_to_hex(item) {
                        *item = serde_json::Value::String(hex);
                    }
                }
            }

            // Handle State witness object - convert sign and zero_proof inside
            if let Some(serde_json::Value::Object(state_witness)) = map.get_mut("State") {
                // Only process if this looks like a witness (has sign/zero_proof, not out_state)
                if state_witness.contains_key("sign") && !state_witness.contains_key("out_state") {
                    // Convert sign to hex
                    if let Some(sign) = state_witness.get("sign") {
                        if let Some(hex) = bytes_array_to_hex(sign) {
                            state_witness.insert("sign".to_string(), serde_json::Value::String(hex));
                        }
                    }
                    // Convert zero_proof arrays to hex
                    if let Some(serde_json::Value::Array(zero_proof)) = state_witness.get_mut("zero_proof") {
                        for item in zero_proof.iter_mut() {
                            if let Some(hex) = bytes_array_to_hex(item) {
                                *item = serde_json::Value::String(hex);
                            }
                        }
                    }
                }
            }

            // Convert Dleq value_proof arrays to hex strings (handles nested arrays)
            if let Some(serde_json::Value::Array(dleq)) = map.get_mut("Dleq") {
                for item in dleq.iter_mut() {
                    if let serde_json::Value::Array(inner) = item {
                        // Check if this is a nested array of arrays (like in sender_account_dleq)
                        if !inner.is_empty() {
                            if let Some(first) = inner.first() {
                                if first.is_array() {
                                    // Nested array - convert each inner array to hex
                                    for nested_item in inner.iter_mut() {
                                        if let Some(hex) = bytes_array_to_hex(nested_item) {
                                            *nested_item = serde_json::Value::String(hex);
                                        }
                                    }
                                    continue;
                                }
                            }
                        }
                        // Direct byte array
                        if let Some(hex) = bytes_array_to_hex(item) {
                            if !hex.is_empty() {
                                *item = serde_json::Value::String(hex);
                            }
                        }
                    }
                }
            }

            // Convert comm.c and comm.d byte arrays to hex strings (in proof accounts)
            if let Some(serde_json::Value::Object(comm_map)) = map.get_mut("comm") {
                if let Some(c) = comm_map.get("c") {
                    if let Some(hex) = bytes_array_to_hex(c) {
                        comm_map.insert("c".to_string(), serde_json::Value::String(hex));
                    }
                }
                if let Some(d) = comm_map.get("d") {
                    if let Some(hex) = bytes_array_to_hex(d) {
                        comm_map.insert("d".to_string(), serde_json::Value::String(hex));
                    }
                }
            }

            // Convert pk.gr and pk.grsk byte arrays to hex strings (in proof accounts)
            if let Some(serde_json::Value::Object(pk_map)) = map.get_mut("pk") {
                if let Some(gr) = pk_map.get("gr") {
                    if let Some(hex) = bytes_array_to_hex(gr) {
                        pk_map.insert("gr".to_string(), serde_json::Value::String(hex));
                    }
                }
                if let Some(grsk) = pk_map.get("grsk") {
                    if let Some(hex) = bytes_array_to_hex(grsk) {
                        pk_map.insert("grsk".to_string(), serde_json::Value::String(hex));
                    }
                }
            }

            // Convert range_proof byte arrays to hex strings
            if let Some(serde_json::Value::Array(range_proof)) = map.get_mut("range_proof") {
                for item in range_proof.iter_mut() {
                    if let Some(hex) = bytes_array_to_hex(item) {
                        *item = serde_json::Value::String(hex);
                    }
                }
            }

            // Transform State input/output data with meaningful labels
            if let Some(serde_json::Value::Object(state_map)) = map.get_mut("State") {
                if let Some(data) = state_map.get_mut("data") {
                    transform_state_data(data);
                }
            }

            // Transform Memo input/output data with meaningful labels
            if let Some(serde_json::Value::Object(memo_map)) = map.get_mut("Memo") {
                if let Some(data) = memo_map.get_mut("data") {
                    transform_memo_data(data);
                }
            }

            // Recursively transform nested objects
            for (_, v) in map.iter_mut() {
                transform_byte_arrays(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                transform_byte_arrays(item);
            }
        }
        _ => {}
    }
}

/// API endpoint: POST /api/decode-zkos-transaction
///
/// Returns the raw decoded transaction JSON without any transformations.
/// Useful for debugging or when you need the original data structure.
///
/// Example request:
/// ```json
/// {
///   "tx_byte_code": "0x123abc..."
/// }
/// ```
async fn decode_transaction_raw_endpoint(
    req: web::Json<DecodeRequest>,
) -> impl Responder {
    match decode_transaction(&req.tx_byte_code) {
        Ok(decoded_tx) => {
            let mut data = serde_json::to_value(&decoded_tx).unwrap_or(serde_json::json!({}));

            // Determine tx_type from data structure
            let tx_type = if let Some(tx) = data.get("tx") {
                if tx.get("TransactionTransfer").is_some() {
                    "Transfer"
                } else if tx.get("TransactionScript").is_some() {
                    "Script"
                } else if tx.get("Message").is_some() {
                    "Message"
                } else {
                    "Unknown"
                }
            } else {
                "Unknown"
            };

            // Build summary object
            let mut summary = serde_json::json!({
                "tx_type": tx_type
            });

            // For Script transactions, extract program and determine program_type and order_type
            if let Some(tx) = data.get("tx") {
                if let Some(script_tx) = tx.get("TransactionScript") {
                    if let Some(program) = script_tx.get("program") {
                        if let Some(program_hex) = program_bytes_to_hex(program) {
                            let program_type = get_program_type(&program_hex);
                            let order_type = get_order_type(program_type);
                            summary["program_type"] = serde_json::Value::String(program_type.to_string());
                            summary["order_type"] = serde_json::Value::String(order_type.to_string());

                            // Decode program opcodes into human-readable list
                            let opcodes = decode_program_opcodes(program);
                            summary["program_opcodes"] = serde_json::Value::Array(
                                opcodes.into_iter().map(serde_json::Value::String).collect()
                            );

                            // For order_open, extract position_size and order_side from output Memo (COIN -> MEMO)
                            if order_type == "order_open" {
                                if let Some(outputs) = script_tx.get("outputs") {
                                    if let Some(first_output) = outputs.as_array().and_then(|a| a.first()) {
                                        if let Some(memo) = first_output.get("output").and_then(|o| o.get("Memo")) {
                                            if let Some(data) = memo.get("data") {
                                                // data[0] = position_size (divided by 10^8)
                                                if let Some(pos_size) = extract_scalar_u64_from_data(data, 0) {
                                                    let pos_size_converted = pos_size as f64 / 100_000_000.0;
                                                    if let Some(num) = serde_json::Number::from_f64(pos_size_converted) {
                                                        summary["position_size"] = serde_json::Value::Number(num);
                                                    }
                                                }
                                                // data[3] = order_side (1 = short, other = long)
                                                if let Some(side_val) = extract_scalar_u64_from_data(data, 3) {
                                                    let side = if side_val == 1 { "short" } else { "long" };
                                                    summary["order_side"] = serde_json::Value::String(side.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // For order_close, extract position_size and order_side from input Memo (MEMO -> COIN)
                            if order_type == "order_close" {
                                if let Some(inputs) = script_tx.get("inputs") {
                                    if let Some(first_input) = inputs.as_array().and_then(|a| a.first()) {
                                        if let Some(memo) = first_input.get("input").and_then(|i| i.get("Memo")) {
                                            if let Some(data) = memo.get("data") {
                                                // data[0] = position_size (divided by 10^8)
                                                if let Some(pos_size) = extract_scalar_u64_from_data(data, 0) {
                                                    let pos_size_converted = pos_size as f64 / 100_000_000.0;
                                                    if let Some(num) = serde_json::Number::from_f64(pos_size_converted) {
                                                        summary["position_size"] = serde_json::Value::Number(num);
                                                    }
                                                }
                                                // data[3] = order_side (1 = short, other = long)
                                                if let Some(side_val) = extract_scalar_u64_from_data(data, 3) {
                                                    let side = if side_val == 1 { "short" } else { "long" };
                                                    summary["order_side"] = serde_json::Value::String(side.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Transform byte arrays (proof, sign, value_proof) to hex strings
            transform_byte_arrays(&mut data);

            HttpResponse::Ok().json(DecodeRawResponse {
                success: true,
                tx_type: tx_type.to_string(),
                summary: Some(summary),
                data,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to decode transaction: {:?}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: format!("Failed to decode transaction: {}", e),
            })
        }
    }
}

/// API endpoint: POST /api/decode-zkos-transaction-raw
///
/// Returns the raw decoded transaction JSON without any transformations.
/// No hex conversions, no field renaming, just the pure deserialized output.
///
/// Example request:
/// ```json
/// {
///   "tx_byte_code": "0x123abc..."
/// }
/// ```
async fn decode_zkos_transaction_raw_endpoint(
    req: web::Json<DecodeRequest>,
) -> impl Responder {
    match decode_transaction(&req.tx_byte_code) {
        Ok(decoded_tx) => {
            let data = serde_json::to_value(&decoded_tx).unwrap_or(serde_json::json!({}));

            // Determine tx_type from data structure
            let tx_type = if let Some(tx) = data.get("tx") {
                if tx.get("TransactionTransfer").is_some() {
                    "Transfer"
                } else if tx.get("TransactionScript").is_some() {
                    "Script"
                } else if tx.get("Message").is_some() {
                    "Message"
                } else {
                    "Unknown"
                }
            } else {
                "Unknown"
            };

            HttpResponse::Ok().json(DecodeRawResponse {
                success: true,
                tx_type: tx_type.to_string(),
                summary: None,
                data,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to decode transaction: {:?}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: format!("Failed to decode transaction: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/transactions/{t_address}
#[utoipa::path(
    get,
    path = "/api/transactions/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query transactions for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved transactions", body = TransactionsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Transactions"
)]
async fn get_transactions(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_transactions_by_address(&t_address) {
        Ok(records) => {
            let transaction_count = records.len() as i64;

            HttpResponse::Ok().json(TransactionsResponse {
                success: true,
                t_address,
                transaction_count,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch transactions: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch transactions: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/funding/{t_address}
#[utoipa::path(
    get,
    path = "/api/funding/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query total amount of funds moved between funding accounts")
    ),
    responses(
        (status = 200, description = "Successfully retrieved funds moved", body = FundsMovedResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Funding to Funding"
)]
async fn get_funds_moved(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_funds_moved_by_address(&t_address) {
        Ok(records) => {
            let funds_moved: Vec<FundsMovedData> = records
                .into_iter()
                .map(|r| FundsMovedData {
                    amount: r.amount,
                    denom: r.denom,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(FundsMovedResponse {
                success: true,
                t_address,
                funds_moved,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch funds moved: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch funds moved: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/exchange-withdrawal/{t_address}
#[utoipa::path(
    get,
    path = "/api/exchange-withdrawal/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address against which to query total amount of Nyks Sats moved from trading to funding account")
    ),
    responses(
        (status = 200, description = "Successfully retrieved dark burned sats", body = DarkBurnedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Trading to Funding"
)]
async fn get_dark_burned_sats(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_dark_burned_sats_by_address(&t_address) {
        Ok(records) => {
            let dark_burned_sats: Vec<DarkBurnedSatsData> = records
                .into_iter()
                .map(|r| DarkBurnedSatsData {
                    q_address: r.q_address,
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(DarkBurnedSatsResponse {
                success: true,
                t_address,
                dark_burned_sats,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch dark burned sats: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch dark burned sats: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/exchange-deposit/{t_address}
#[utoipa::path(
    get,
    path = "/api/exchange-deposit/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address against which to query total amount of Nyks Sats moved from funding to trading account")
    ),
    responses(
        (status = 200, description = "Successfully retrieved minted sats", body = DarkMintedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Funding to Trading"
)]
async fn get_dark_minted_sats(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_dark_minted_sats_by_address(&t_address) {
        Ok(records) => {
            let dark_minted_sats: Vec<DarkMintedSatsData> = records
                .into_iter()
                .map(|r| DarkMintedSatsData {
                    q_address: r.q_address,
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(DarkMintedSatsResponse {
                success: true,
                t_address,
                dark_minted_sats,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch dark minted sats: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch dark minted sats: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/btc-deposit/{t_address}
#[utoipa::path(
    get,
    path = "/api/btc-deposit/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address against which to query total amount of Nyks Sats deposited from btc chain to Nyks")
    ),
    responses(
        (status = 200, description = "Successfully retrieved lit minted sats", body = LitMintedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "BTC Deposited"
)]
async fn get_lit_minted_sats(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_lit_minted_sats_by_address(&t_address) {
        Ok(records) => {
            let lit_minted_sats: Vec<LitMintedSatsData> = records
                .into_iter()
                .map(|r| LitMintedSatsData {
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(LitMintedSatsResponse {
                success: true,
                t_address,
                lit_minted_sats,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch lit minted sats: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch lit minted sats: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/btc-withdrawal/{t_address}
#[utoipa::path(
    get,
    path = "/api/btc-withdrawal/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address against which to query total amount of Nyks Sats withdrawn from Nyks to btc chain")
    ),
    responses(
        (status = 200, description = "Successfully retrieved withdrawn sats", body = LitBurnedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "BTC Withdrawn"
)]
async fn get_lit_burned_sats(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_lit_burned_sats_by_address(&t_address) {
        Ok(records) => {
            let lit_burned_sats: Vec<LitBurnedSatsData> = records
                .into_iter()
                .map(|r| LitBurnedSatsData {
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(LitBurnedSatsResponse {
                success: true,
                t_address,
                lit_burned_sats,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch lit burned sats: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch lit burned sats: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/qq-account/{t_address}
#[utoipa::path(
    get,
    path = "/api/qq-account/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address against which to query quis quis accounts")
    ),
    responses(
        (status = 200, description = "Successfully retrieved q addresses", body = QAddressesResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Twilight/qq mapping"
)]
async fn get_q_addresses(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    match db::get_qaddresses_for_taddress(&t_address) {
        Ok(records) => {
            let q_addresses: Vec<QAddressData> = records
                .into_iter()
                .map(|r| QAddressData {
                    qq_account: r.q_address,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(QAddressesResponse {
                success: true,
                t_address,
                q_addresses,
            })
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch q addresses: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Failed to fetch q addresses: {}", e),
            })
        }
    }
}

/// API endpoint: GET /api/address/{t_address}/all
/// Returns all data for a given t_address from all tables
#[utoipa::path(
    get,
    path = "/api/address/{t_address}/all",
    params(
        ("t_address" = String, Path, description = "Twilight address to query general stats for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved all address data", body = AddressAllDataResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Stats"
)]
async fn get_address_all_data(path: web::Path<String>) -> impl Responder {
    let t_address = path.into_inner();

    // Fetch all data in parallel would be ideal, but for simplicity we'll do sequential
    let transactions_result = db::get_transactions_by_address(&t_address);
    let funds_moved_result = db::get_funds_moved_by_address(&t_address);
    let dark_burned_result = db::get_dark_burned_sats_by_address(&t_address);
    let dark_minted_result = db::get_dark_minted_sats_by_address(&t_address);
    let lit_minted_result = db::get_lit_minted_sats_by_address(&t_address);
    let lit_burned_result = db::get_lit_burned_sats_by_address(&t_address);

    match (
        transactions_result,
        funds_moved_result,
        dark_burned_result,
        dark_minted_result,
        lit_minted_result,
        lit_burned_result,
    ) {
        (Ok(txs), Ok(funds), Ok(dark_burned), Ok(dark_minted), Ok(lit_minted), Ok(lit_burned)) => {
            let transaction_count = txs.len() as i64;

            let funds_moved: Vec<FundsMovedData> = funds
                .into_iter()
                .map(|r| FundsMovedData {
                    amount: r.amount,
                    denom: r.denom,
                    block: r.block,
                })
                .collect();

            let dark_burned_sats: Vec<DarkBurnedSatsData> = dark_burned
                .into_iter()
                .map(|r| DarkBurnedSatsData {
                    q_address: r.q_address,
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            let dark_minted_sats: Vec<DarkMintedSatsData> = dark_minted
                .into_iter()
                .map(|r| DarkMintedSatsData {
                    q_address: r.q_address,
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            let lit_minted_sats: Vec<LitMintedSatsData> = lit_minted
                .into_iter()
                .map(|r| LitMintedSatsData {
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            let lit_burned_sats: Vec<LitBurnedSatsData> = lit_burned
                .into_iter()
                .map(|r| LitBurnedSatsData {
                    amount: r.amount,
                    block: r.block,
                })
                .collect();

            HttpResponse::Ok().json(AddressAllDataResponse {
                success: true,
                t_address,
                transaction_count,
                funds_moved,
                dark_burned_sats,
                dark_minted_sats,
                lit_minted_sats,
                lit_burned_sats,
            })
        }
        _ => {
            eprintln!("❌ Failed to fetch complete address data");
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: "Failed to fetch complete address data".to_string(),
            })
        }
    }
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Service is healthy")
    ),
    tag = "Health"
)]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "twilight-indexer-api"
    }))
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        get_transactions,
        get_funds_moved,
        get_dark_burned_sats,
        get_dark_minted_sats,
        get_lit_minted_sats,
        get_lit_burned_sats,
        get_q_addresses,
        get_address_all_data
    ),
    components(
        schemas(
            TransactionsResponse,
            FundsMovedResponse,
            FundsMovedData,
            DarkBurnedSatsResponse,
            DarkBurnedSatsData,
            DarkMintedSatsResponse,
            DarkMintedSatsData,
            LitMintedSatsResponse,
            LitMintedSatsData,
            LitBurnedSatsResponse,
            LitBurnedSatsData,
            QAddressesResponse,
            QAddressData,
            AddressAllDataResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Transactions", description = "Returns transaction blocks for each Twilight address"),
        (name = "Funding to Funding", description = "Returns funds moved between funding accounts"),
        (name = "Funding to Trading", description = "Returns funds moved from funding to trading accounts"),
        (name = "Trading to Funding", description = "Returns funds moved from trading to funding accounts"),
        (name = "BTC Deposited", description = "Returns Btc Deposited to Twilight Reserves"),
        (name = "BTC Withdrawn", description = "Returns Btc Withdrawn from Twilight Reserves"),
        (name = "Twilight/qq mapping", description = "Address mappings between Twilight and quis quis accounts"),
        (name = "Stats", description = "General stats for a given Twilight address")
    ),
    info(
        title = "Twilight Indexer API",
        version = "1.0.0",
        description = "API for querying Twilight's ZKOS blockchain stats"
    )
)]
pub struct ApiDoc;

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/decode-zkos-transaction", web::post().to(decode_transaction_raw_endpoint))
            .route("/decode-zkos-transaction-raw", web::post().to(decode_zkos_transaction_raw_endpoint))
            .route("/transactions/{t_address}", web::get().to(get_transactions))
            .route("/funding/{t_address}", web::get().to(get_funds_moved))
            .route("/exchange-withdrawal/{t_address}", web::get().to(get_dark_burned_sats))
            .route("/exchange-deposit/{t_address}", web::get().to(get_dark_minted_sats))
            .route("/btc-deposit/{t_address}", web::get().to(get_lit_minted_sats))
            .route("/btc-withdrawal/{t_address}", web::get().to(get_lit_burned_sats))
            .route("/qq-account/{t_address}", web::get().to(get_q_addresses))
            .route("/address/{t_address}/all", web::get().to(get_address_all_data))
    );
}

/// Start the API server
pub async fn start_api_server(host: &str, port: u16) -> std::io::Result<()> {
    let openapi = ApiDoc::openapi();

    println!("🚀 Starting API server at http://{}:{}", host, port);
    println!("📚 Swagger UI available at http://{}:{}/swagger-ui/", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
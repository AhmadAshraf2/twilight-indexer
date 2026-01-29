use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::quis_quis_tx::decode_transaction;
use crate::db;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// Request payload for decoding a transaction
#[derive(Debug, Deserialize, ToSchema)]
pub struct DecodeRequest {
    pub tx_byte_code: String,
}

/// Response for successful transaction decode
#[derive(Debug, Serialize, ToSchema)]
pub struct DecodeResponse {
    pub success: bool,
    pub tx_type: String,
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

/// Convert opcode byte to instruction name
fn opcode_to_name(opcode: u8) -> &'static str {
    match opcode {
        0x00 => "push",
        0x01 => "program",
        0x02 => "drop",
        0x03 => "dup",
        0x04 => "roll",
        0x05 => "scalar",
        0x06 => "commit",
        0x07 => "alloc",
        0x0a => "expr",
        0x0b => "neg",
        0x0c => "add",
        0x0d => "mul",
        0x0e => "eq",
        0x0f => "range",
        0x10 => "and",
        0x11 => "or",
        0x12 => "not",
        0x13 => "verify",
        0x14 => "unblind",
        0x15 => "issue",
        0x16 => "borrow",
        0x17 => "retire",
        0x19 => "fee",
        0x1a => "input",
        0x1b => "output",
        0x1c => "contract",
        0x1d => "log",
        0x1e => "call",
        0x1f => "signtx",
        0x20 => "signid",
        0x21 => "signtag",
        0x22 => "inputcoin",
        0x23 => "outputcoin",
        _ => "ext",
    }
}

/// Check if opcode has a u32 argument
fn opcode_has_arg(opcode: u8) -> bool {
    matches!(opcode, 0x03 | 0x04 | 0x1b | 0x1c | 0x22 | 0x23) // dup, roll, output, contract, inputcoin, outputcoin
}

/// Check if opcode has variable-length data (push, program)
fn opcode_has_data(opcode: u8) -> bool {
    matches!(opcode, 0x00 | 0x01) // push, program
}

/// Convert program bytes to human-readable instructions
fn decode_program_bytes(bytes: &[Value]) -> Vec<String> {
    let mut instructions = Vec::new();
    let mut i = 0;

    // Convert Value array to u8 array
    let byte_vec: Vec<u8> = bytes.iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect();

    while i < byte_vec.len() {
        let opcode = byte_vec[i];
        let name = opcode_to_name(opcode);

        if opcode_has_data(opcode) {
            // push or program: read 4-byte length, then skip data
            if i + 5 <= byte_vec.len() {
                let len = u32::from_le_bytes([
                    byte_vec[i + 1],
                    byte_vec[i + 2],
                    byte_vec[i + 3],
                    byte_vec[i + 4],
                ]) as usize;
                instructions.push(format!("{}:{}", name, len));
                i += 5 + len;
            } else {
                instructions.push(name.to_string());
                i += 1;
            }
        } else if opcode_has_arg(opcode) {
            // Instructions with u32 argument
            if i + 5 <= byte_vec.len() {
                let arg = u32::from_le_bytes([
                    byte_vec[i + 1],
                    byte_vec[i + 2],
                    byte_vec[i + 3],
                    byte_vec[i + 4],
                ]);
                instructions.push(format!("{}:{}", name, arg));
                i += 5;
            } else {
                instructions.push(name.to_string());
                i += 1;
            }
        } else {
            instructions.push(name.to_string());
            i += 1;
        }
    }

    instructions
}

/// Convert byte array to hex string
fn bytes_to_hex(bytes: &[Value]) -> String {
    bytes.iter()
        .filter_map(|v| v.as_u64().map(|n| format!("{:02x}", n as u8)))
        .collect()
}

/// Transform the JSON to convert txid, program, and scalars to human-readable formats
fn transform_decoded_tx(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Convert txid byte arrays to hex strings
            if map.contains_key("txid") {
                if let Some(Value::Array(bytes)) = map.get("txid") {
                    let hex_str = bytes_to_hex(bytes);
                    map.insert("txid".to_string(), Value::String(hex_str));
                }
            }

            // Convert program byte arrays to instruction lists
            if map.contains_key("program") {
                if let Some(Value::Array(bytes)) = map.get("program") {
                    let instructions = decode_program_bytes(bytes);
                    map.insert("program".to_string(), Value::Array(
                        instructions.into_iter().map(Value::String).collect()
                    ));
                }
            }

            // Convert Scalar objects to u64 values
            if let Some(Value::Object(scalar_map)) = map.get("Scalar") {
                if let Some(Value::Array(bytes)) = scalar_map.get("Scalar") {
                    if let Some(u64_value) = bytes_to_u64(bytes) {
                        *value = serde_json::json!({
                            "scalar": u64_value
                        });
                        return;
                    }
                }
            }

            // Recursively process all values in the object
            for (_, v) in map.iter_mut() {
                transform_decoded_tx(v);
            }
        }
        Value::Array(arr) => {
            // Recursively process all elements in the array
            for item in arr.iter_mut() {
                transform_decoded_tx(item);
            }
        }
        _ => {}
    }
}

/// Convert a byte array (first 8 bytes) to u64 using little-endian
fn bytes_to_u64(bytes: &[Value]) -> Option<u64> {
    if bytes.len() < 8 {
        return None;
    }

    let mut array_8 = [0u8; 8];
    for (i, byte_value) in bytes.iter().take(8).enumerate() {
        if let Some(byte) = byte_value.as_u64() {
            array_8[i] = byte as u8;
        } else {
            return None;
        }
    }

    Some(u64::from_le_bytes(array_8))
}

/// API endpoint: POST /api/decode-transaction
///
/// Example request:
/// ```json
/// {
///   "tx_byte_code": "0x123abc...",
///   "block_height": 12345
/// }
/// ```
async fn decode_transaction_endpoint(
    req: web::Json<DecodeRequest>,
) -> impl Responder {

    match decode_transaction(&req.tx_byte_code) {
        Ok(decoded_tx) => {
            let (tx_type, mut data) = ("transaction", serde_json::to_value(&decoded_tx).unwrap_or(serde_json::json!({})));

            // Transform txid to hex, program to instructions, scalars to u64
            transform_decoded_tx(&mut data);

            HttpResponse::Ok().json(DecodeResponse {
                success: true,
                tx_type: tx_type.to_string(),
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
            .route("/decode-transaction", web::post().to(decode_transaction_endpoint))
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
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
    pub transactions: Vec<TransactionData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionData {
    pub block: i64,
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

/// Combined response for all address data
#[derive(Debug, Serialize, ToSchema)]
pub struct AddressAllDataResponse {
    pub success: bool,
    pub t_address: String,
    pub transactions: Vec<TransactionData>,
    pub funds_moved: Vec<FundsMovedData>,
    pub dark_burned_sats: Vec<DarkBurnedSatsData>,
    pub dark_minted_sats: Vec<DarkMintedSatsData>,
    pub lit_minted_sats: Vec<LitMintedSatsData>,
    pub lit_burned_sats: Vec<LitBurnedSatsData>,
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
            let (tx_type, data) = ("transaction", serde_json::to_value(&decoded_tx).unwrap_or(serde_json::json!({})));

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
            let transactions: Vec<TransactionData> = records
                .into_iter()
                .map(|r| TransactionData { block: r.block })
                .collect();

            HttpResponse::Ok().json(TransactionsResponse {
                success: true,
                t_address,
                transactions,
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

/// API endpoint: GET /api/funds-moved/{t_address}
#[utoipa::path(
    get,
    path = "/api/funds-moved/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query funds moved for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved funds moved", body = FundsMovedResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Funds"
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

/// API endpoint: GET /api/dark-burned-sats/{t_address}
#[utoipa::path(
    get,
    path = "/api/dark-burned-sats/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query dark burned sats for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved dark burned sats", body = DarkBurnedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Dark Sats"
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

/// API endpoint: GET /api/dark-minted-sats/{t_address}
#[utoipa::path(
    get,
    path = "/api/dark-minted-sats/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query dark minted sats for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved dark minted sats", body = DarkMintedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Dark Sats"
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

/// API endpoint: GET /api/lit-minted-sats/{t_address}
#[utoipa::path(
    get,
    path = "/api/lit-minted-sats/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query lit minted sats for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved lit minted sats", body = LitMintedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Lit Sats"
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

/// API endpoint: GET /api/lit-burned-sats/{t_address}
#[utoipa::path(
    get,
    path = "/api/lit-burned-sats/{t_address}",
    params(
        ("t_address" = String, Path, description = "Twilight address to query lit burned sats for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved lit burned sats", body = LitBurnedSatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Lit Sats"
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

/// API endpoint: GET /api/address/{t_address}/all
/// Returns all data for a given t_address from all tables
#[utoipa::path(
    get,
    path = "/api/address/{t_address}/all",
    params(
        ("t_address" = String, Path, description = "Twilight address to query all data for")
    ),
    responses(
        (status = 200, description = "Successfully retrieved all address data", body = AddressAllDataResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Address"
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
            let transactions: Vec<TransactionData> = txs
                .into_iter()
                .map(|r| TransactionData { block: r.block })
                .collect();

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
                transactions,
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
        get_address_all_data
    ),
    components(
        schemas(
            TransactionsResponse,
            TransactionData,
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
            AddressAllDataResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Transactions", description = "Transaction-related endpoints"),
        (name = "Funds", description = "Funds movement endpoints"),
        (name = "Dark Sats", description = "Dark sats (burned/minted) endpoints"),
        (name = "Lit Sats", description = "Lit sats (burned/minted) endpoints"),
        (name = "Address", description = "Address data endpoints")
    ),
    info(
        title = "Twilight Indexer API",
        version = "1.0.0",
        description = "API for querying Twilight blockchain indexer data"
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
            .route("/funds-moved/{t_address}", web::get().to(get_funds_moved))
            .route("/dark-burned-sats/{t_address}", web::get().to(get_dark_burned_sats))
            .route("/dark-minted-sats/{t_address}", web::get().to(get_dark_minted_sats))
            .route("/lit-minted-sats/{t_address}", web::get().to(get_lit_minted_sats))
            .route("/lit-burned-sats/{t_address}", web::get().to(get_lit_burned_sats))
            .route("/address/{t_address}/all", web::get().to(get_address_all_data))
    );
}

/// Start the API server
pub async fn start_api_server(host: &str, port: u16) -> std::io::Result<()> {
    let openapi = ApiDoc::openapi();

    println!("🚀 Starting API server at http://{}:{}", host, port);
    println!("📚 Swagger UI available at http://{}:{}/swagger-ui/", host, port);

    HttpServer::new(move || {
        let cors = Cors::permissive(); // Or configure more restrictively

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
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use crate::quis_quis_tx::{decode_transaction, DecodedQQTx};

/// Request payload for decoding a transaction
#[derive(Debug, Deserialize)]
pub struct DecodeRequest {
    pub tx_byte_code: String,
}

/// Response for successful transaction decode
#[derive(Debug, Serialize)]
pub struct DecodeResponse {
    pub success: bool,
    pub tx_type: String,
    pub data: serde_json::Value,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
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

/// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "twilight-indexer-api"
    }))
}

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/decode-transaction", web::post().to(decode_transaction_endpoint))
    );
}

/// Start the API server
pub async fn start_api_server(host: &str, port: u16) -> std::io::Result<()> {
    println!("🚀 Starting API server at http://{}:{}", host, port);
    
    HttpServer::new(|| {
        let cors = Cors::permissive(); // Or configure more restrictively
        
        App::new()
            .wrap(cors)
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
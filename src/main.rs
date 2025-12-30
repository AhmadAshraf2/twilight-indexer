mod block_types;
mod db;
mod pubsub_chain;
mod transaction_types;
mod schema;
mod quis_quis_tx;
mod api;

use quis_quis_tx::decode_qq_transaction;

#[actix_web::main]
async fn main() {
    dotenv::dotenv().expect("Failed loading dotenv");
    db::run_migrations().expect("Failed to run database migrations");

    // Get configuration from environment variables
    let api_host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let api_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("API_PORT must be a valid port number");

    let enable_api = std::env::var("ENABLE_API")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let enable_indexer = std::env::var("ENABLE_INDEXER")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    // Run both API server and indexer concurrently
    if enable_api && enable_indexer {
        println!("🚀 Starting both API server and blockchain indexer...");
        
        // Spawn indexer in background thread (since it's blocking)
        let indexer_handle = std::thread::spawn(|| {
            pubsub_chain::subscribe_block();
        });

        // Run API server in the current async runtime
        if let Err(e) = api::start_api_server(&api_host, api_port).await {
            eprintln!("❌ API server error: {}", e);
        }

        // Wait for indexer thread to complete (it runs indefinitely)
        let _ = indexer_handle.join();
    } else if enable_api {
        println!("🚀 Starting API server only...");
        if let Err(e) = api::start_api_server(&api_host, api_port).await {
            eprintln!("❌ API server error: {}", e);
        }
    } else if enable_indexer {
        println!("🚀 Starting blockchain indexer only...");
        pubsub_chain::subscribe_block();
    } else {
        println!("⚠️ Both API and indexer are disabled. Nothing to do.");
    }
}
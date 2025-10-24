mod block_types;
mod db;
mod pubsub_chain;
mod quis_quis_tx;
mod schema;
mod transaction_types;

fn main() {
    dotenv::dotenv().expect("Failed loading dotenv");
    db::run_migrations().expect("Failed to run database migrations");
    pubsub_chain::subscribe_block();
}

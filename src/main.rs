mod block_types;
mod db;
mod pubsub_chain;
mod transaction_types;
mod schema;
mod quis_quis_tx;

fn main() {
    dotenv::dotenv().expect("Failed loading dotenv");
    pubsub_chain::subscribe_block();
}
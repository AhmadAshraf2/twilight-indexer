mod block_types;
mod db;
mod pubsub_chain;
mod transaction_types;
mod schema;
mod quis_quis_tx;

use crate::quis_quis_tx::decode_qq_transaction;
use crate::quis_quis_tx::DecodedQQTx;

fn main() {
    dotenv::dotenv().expect("Failed loading dotenv");
    pubsub_chain::subscribe_block();
}
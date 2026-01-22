//! Block subscription and chain event utilities.
//!
//! This module provides functions and statics for subscribing to new blocks from a Cosmos-based
//! blockchain, as well as utilities for making HTTP requests to the chain's REST API.
//!
//! # Features
//! - Block subscription with threaded processing
//! - Configurable endpoint via environment variable
//! - Utilities for requesting data from the chain
//!
//! # Example
//! ```
//! use twilight_indexer::pubsub_chain::subscribe_block;
//! ```
use crate::{block_types::BlockRaw, schema::transactions::block};

use lazy_static::lazy_static;
use std::time;
// #[macro_use]
// extern crate lazy_static;
lazy_static! {
    /// Defaults to `http://localhost:1317/` if not set.
    pub static ref NYKS_BLOCK_SUBSCRIBER_URL: String =
        std::env::var("NYKS_BLOCK_SUBSCRIBER_URL").unwrap_or("http://localhost:1317/".to_string());
}
 //BlockRaw, ThreadPool};

/// Subscribes to new blocks from the Cosmos chain.
///
/// Spawns a background thread that fetches and processes new blocks, sending them through a channel.
///
/// # Arguments
/// * `empty_block` - If true, includes empty blocks in the subscription.
///
/// # Returns
/// A tuple containing:
/// - An `Arc<Mutex<mpsc::Receiver<Block>>>` for receiving new blocks.
/// - A `JoinHandle` for the background thread.
pub fn subscribe_block(){
    let mut latest_height = match BlockRaw::get_latest_block_height() {
        Ok(height) => height,
        Err(arg) => {
            println!("Can not get latest height \nError: {:?}\nSetting height to 0", arg);
            panic!("Cannot get latest height from chain, check connection settings");
        }
    };
    let mut block_height = BlockRaw::get_local_block_height();

    loop {
        let mut attempt = 0;
        while block_height <= latest_height {
            let block_raw_result = BlockRaw::get_block_data_from_height(block_height);
            match block_raw_result {
                Ok(block_raw) => {
                    println!("Fetched Block at height: {}", block_height);
                    for tx in &block_raw.block.data.txs {
                        let _decoded_tx = crate::transaction_types::decode_tx_base64_standard(tx, block_height);
                    }
                    block_height += 1;
                }
                Err(arg) => {
                    if arg.as_str() == "3"{
                        println!("block fetching at block height :{}, return code=3, fetching next block", block_height);
                        block_height += 1;
                    } else {
                        attempt += 1;
                        println!(
                            "block fetching error at block height : {:?} \nError:{:?}",
                            block_height,
                            arg
                        );
                        if attempt == 3 {
                            println!("block fetching at block height :{} failed after 3 attempts, fethcing next block", block_height);
                            block_height += 1;
                            attempt = 0;
                        }
                    }
                }
            }
            BlockRaw::write_local_block_height(block_height);
        }

        latest_height = match BlockRaw::get_latest_block_height() {
            Ok(height) => height,
            Err(arg ) => {
                println!("Can not get latest height \nError: {:?}\nSetting height to 0", arg);
                panic!("Cannot get latest height from chain, check connection settings");
            }
        };

        BlockRaw::write_local_block_height(block_height);
        println!("Sleeping for 30 seconds before checking for new blocks...");
        std::thread::sleep(time::Duration::from_secs(30));
    }
}

/// Makes a blocking HTTP GET request to the given URL.
///
/// # Arguments
/// * `url` - The URL to request.
///
/// # Returns
/// - `Ok(String)` with the response body if successful.
/// - `Err(String)` with an error message if the request fails
pub fn request_url(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    match client.get(url).send() {
        Ok(res) => match res.text() {
            Ok(text) => Ok(text),
            Err(arg) => Err(arg.to_string()),
        },
        Err(arg) => Err(arg.to_string()),
    }
}

#[cfg(test)]
mod test {
    use crate::block_types::BlockRaw;

    #[test]
    fn get_latest_block_test() {
        let latest_block_height = BlockRaw::get_latest_block_height();
        match latest_block_height {
            Ok(height) => println!("Latest Block Height : {}", height),
            Err(arg) => println!("Got Error finding Latest Height with error : {:?}", arg),
        }
    }

    #[test]
    fn get_block_raw_data_from_height_test() {
        let block_data = BlockRaw::get_block_data_from_height(415156);
        match block_data {
            Ok(block) => println!("Block: {:#?}", block),
            Err(arg) => println!(
                "Got Error finding block from Height: {} with error : {:?}",
                415156, arg
            ),
        }
    }
    #[test]
    fn get_block_raw_data_from_wrong_height_test() {
        let block_data = BlockRaw::get_block_data_from_height(0);
        match block_data {
            Ok(block) => println!("Block: {:#?}", block),
            Err(arg) => println!(
                "\nGot Error finding block from Height: {} with error code: {:?}",
                0, arg
            ),
        }
    }

    #[test]
    fn get_block_decoded_transfer_tx_test() {
        // "/twilightproject.nyks.zkos.MsgTransferTx"
        let block_data = BlockRaw::get_block_data_from_height(415156);

        match block_data {
            Ok(block) => {
                println!("Block: {:#?}", block)
            }
            Err(arg) => println!(
                "Got Error finding block from Height: {} with error : {:?}",
                415156, arg
            ),
        }
    }
    #[test]
    fn get_block_decoded_mint_or_burn_test() {
        // "@type": "/twilightproject.nyks.zkos.MsgMintBurnTradingBtc",
        let block_data = BlockRaw::get_block_data_from_height(380157);
        match block_data {
            Ok(block) => {
                println!("Block: {:#?}", block)
            }
            Err(arg) => println!(
                "Got Error finding block from Height: {} with error : {:?}",
                380157, arg
            ),
        }
    }
}
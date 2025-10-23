//! Types and helpers for parsing and representing Cosmos chain blocks and transactions.
//!
//! This module provides Rust structs for deserializing block and transaction data from
//! Cosmos-based blockchains, as well as helpers for extracting and working with this data.
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
lazy_static! {
    pub static ref BLOCK_HEIGHT_FILE: String =
        std::env::var("BLOCK_HEIGHT_FILE").unwrap_or_else(|_| "height.txt".to_string());
}



/// Error response from block queries
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockError {
    pub code: i64,
    pub message: String,
    pub details: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub block: String,
    pub app: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastBlockId {
    #[serde(deserialize_with = "null_to_empty_string")]
    pub hash: String,
    #[serde(rename = "part_set_header")]
    pub part_set_header: PartSetHeader2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSetHeader2 {
    pub total: i64,
    #[serde(deserialize_with = "null_to_empty_string")]
    pub hash: String,
}





#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSetHeader3 {
    pub total: i64,
    #[serde(deserialize_with = "null_to_empty_string")]
    pub hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    #[serde(rename = "block_id_flag")]
    pub block_id_flag: String,
    #[serde(rename = "validator_address")]
    pub validator_address: String,
    pub timestamp: String,
    pub signature: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub txs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub evidence: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockId2 {
    #[serde(deserialize_with = "null_to_empty_string")]
    pub hash: String,
    #[serde(rename = "part_set_header")]
    pub part_set_header: PartSetHeader3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastCommit {
    pub height: String,
    pub round: i64,
    #[serde(rename = "block_id")]
    pub block_id: BlockId2,
    pub signatures: Vec<Signature>,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub version: Version,
    #[serde(rename = "chain_id")]
    pub chain_id: String,
    #[serde(rename = "height", deserialize_with = "string_to_u64")]
    pub height: u64,
    pub time: String,
    #[serde(rename = "last_block_id")]
    pub last_block_id: LastBlockId,
    #[serde(rename = "last_commit_hash")]
    pub last_commit_hash: String,
    #[serde(rename = "data_hash")]
    pub data_hash: String,
    #[serde(rename = "validators_hash")]
    pub validators_hash: String,
    #[serde(rename = "next_validators_hash")]
    pub next_validators_hash: String,
    #[serde(rename = "consensus_hash")]
    pub consensus_hash: String,
    #[serde(rename = "app_hash")]
    pub app_hash: String,
    #[serde(rename = "last_results_hash")]
    pub last_results_hash: String,
    #[serde(rename = "evidence_hash")]
    pub evidence_hash: String,
    #[serde(rename = "proposer_address")]
    pub proposer_address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block1 {
    pub header: Header,
    pub data: Data,
    pub evidence: Evidence,
    #[serde(rename = "last_commit")]
    pub last_commit: LastCommit,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSetHeader {
    pub total: i64,
    pub hash: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockId {
    pub hash: String,
    #[serde(rename = "part_set_header")]
    pub part_set_header: PartSetHeader,
}

/// Represents a raw block as returned by the Cosmos RPC API.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRaw {
    /// The block's unique identifier (hash and part set header).
    #[serde(rename = "block_id")]
    pub block_id: BlockId,
    /// The block's data, including header, transactions, etc.
    pub block: Block1,
}

impl BlockRaw {
    // pub fn get_txid(&mut self) -> Vec<String> {
    //     let mut txid_vec: Vec<String> = Vec::new();
    //     let txs = self.block.data.txs.clone();
    //     for tx in &txs {
    //         let tx_decode = BASE64_STANDARD.decode(tx).unwrap();
    //         let mut sha256 = Sha256::new();
    //         sha256.update(tx_decode.clone());
    //         let result = sha256.finalize();
    //         // println!("tx data bytes : {:?}", hex::encode(result));
    //         txid_vec.push(hex::encode(result));
    //     }
    //     txid_vec
    // }
    // /// Gets raw byte code for all transactions
    // pub fn get_tx_byte_code(&mut self) -> Vec<String> {
    //     self.block.data.txs.clone()
    // }
    // /// Gets transaction byte code with corresponding transaction hashes
    // pub fn get_tx_byte_code_with_txhash(&mut self) -> Vec<(String, String)> //byte_code, txhash
    // {
    //     let mut txid_vec: Vec<(String, String)> = Vec::new();

    //     let txs = self.block.data.txs.clone();
    //     for tx in txs {
    //         let tx_decode = BASE64_STANDARD.decode(tx.clone()).unwrap();
    //         let mut sha256 = Sha256::new();
    //         sha256.update(tx_decode.clone());
    //         let result = sha256.finalize();
    //         // println!("tx data bytes : {:?}", hex::encode(result));
    //         txid_vec.push((tx, hex::encode(result)));
    //     }
    //     txid_vec
    // }
    // /// Gets the block hash
    // pub fn get_block_hash(&mut self) -> String {
    //     self.block_id.hash.clone()
    // }
    /// Gets the block height
    pub fn get_block_height(&mut self) -> u64 {
        self.block.header.height
    }
    /// Retrieves the latest block height from the chain
    pub fn get_latest_block_height() -> Result<u64, String> {
        let url = format!(
            "{}/cosmos/base/tendermint/v1beta1/blocks/latest",
            *NYKS_BLOCK_SUBSCRIBER_URL
        );
        // println!("url :{:?}", url);
        match request_url(&url) {
            Ok(block_data) => {
                let mut block = match BlockRaw::decode(block_data) {
                    Ok(block) => block,
                    Err(arg) => {
                        println!("Error: {:?}", arg);
                        return Err(arg.to_string());
                    }
                };

                Ok(block.get_block_height())
            }
            Err(arg) => Err(arg.to_string()),
        }
    }
    /// Retrieves block data for a specific height
    pub fn get_block_data_from_height(block_height: u64) -> Result<BlockRaw, String> {
        let url = format!(
            "{}/cosmos/base/tendermint/v1beta1/blocks/{}",
            *NYKS_BLOCK_SUBSCRIBER_URL, block_height,
        );
        match request_url(&url) {
            Ok(block_data) => match BlockRaw::decode(block_data) {
                Ok(block) => Ok(block),
                Err(arg) => Err(arg.to_string()),
            },
            Err(arg) => Err(arg.to_string()),
        }
    }

    pub fn decode(json: String) -> Result<Self, String> {
        match serde_json::from_str(&json) {
            Ok(block) => Ok(block),
            Err(arg) => {
                let block_error: BlockError = match serde_json::from_str(&json) {
                    Ok(block_error_result) => block_error_result,
                    Err(arg) => return Err(arg.to_string()),
                };
                if block_error.code == 3 {
                    return Err("3".to_string());
                }

                Err(arg.to_string())
            }
        }
    }


        pub fn get_local_block_height() -> u64 {
        let block_height: u64 = match fs::read_to_string(BLOCK_HEIGHT_FILE.as_str()) {
            Ok(block_height_str) => match block_height_str.trim().parse::<u64>() {
                Ok(block_height) => block_height,
                Err(_) => {
                    eprintln!("Failed to parse block height");
                    1
                }
            },
            Err(e) => {
                eprintln!("Failed to read block height: {}", e);
                1
            }
        };
        block_height
    }
    pub fn write_local_block_height(block_height: u64) {
        match fs::write(BLOCK_HEIGHT_FILE.as_str(), block_height.to_string()) {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to write block height: {}", e),
        }
    }

}

use serde::{
    de::{self, Visitor},
    Deserializer,
};
use std::fmt;
use std::fs;

use crate::pubsub_chain::request_url;
use crate::pubsub_chain::NYKS_BLOCK_SUBSCRIBER_URL;
/// Custom deserializer for converting a string to a `u64`.
///
/// Used for fields that are serialized as strings in the JSON API.
pub fn string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> Visitor<'de> for StringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representation for u64")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<u64, E> {
            value.parse::<u64>().map_err(E::custom)
        }
    }
    deserializer.deserialize_str(StringVisitor)
}

use serde::{Deserialize as NullDeserialize, Deserializer as NullDeserializer};

pub fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: NullDeserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_raw_decode_height_1() {
        let json_str = r#"{
            "block_id": {
                "hash": "ZxPZxXEc3r9uJVgqCjeqRAnrSWzv5QUg+AVWXO4BcK4=",
                "part_set_header": {
                    "total": 1,
                    "hash": "sBIbv/9Gfqx9MzgmCYETc0Nx32fcclNxz+9R3MSuMF0="
                }
            },
            "block": {
                "header": {
                    "version": {
                        "block": "11",
                        "app": "0"
                    },
                    "chain_id": "nyks",
                    "height": "1",
                    "time": "2025-06-13T12:55:07.001017274Z",
                    "last_block_id": {
                        "hash": null,
                        "part_set_header": {
                            "total": 0,
                            "hash": null
                        }
                    },
                    "last_commit_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                    "data_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                    "validators_hash": "2JXZzU66O/FG00NQMm6Z77VAOWFiqY494wVoQKe47uk=",
                    "next_validators_hash": "2JXZzU66O/FG00NQMm6Z77VAOWFiqY494wVoQKe47uk=",
                    "consensus_hash": "BICRvH3cKD93v7+R1zxE2ljD34qcvIZ0Bdi389qtoi8=",
                    "app_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                    "last_results_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                    "evidence_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                    "proposer_address": "GTahVJYnvUCi88UM6EZDxgJEHVM="
                },
                "data": {
                    "txs": []
                },
                "evidence": {
                    "evidence": []
                },
                "last_commit": {
                    "height": "0",
                    "round": 0,
                    "block_id": {
                        "hash": null,
                        "part_set_header": {
                            "total": 0,
                            "hash": null
                        }
                    },
                    "signatures": []
                }
            }
        }"#;

        let block_raw: BlockRaw = match serde_json::from_str(json_str) {
            Ok(block_raw) => block_raw,
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Error: {:?}", e);
            }
        };
        println!("block_raw: {:?}", block_raw);
        assert_eq!(
            block_raw.block_id.hash,
            "ZxPZxXEc3r9uJVgqCjeqRAnrSWzv5QUg+AVWXO4BcK4="
        );
        assert_eq!(block_raw.block_id.part_set_header.total, 1);
        assert_eq!(block_raw.block.header.chain_id, "nyks");
        assert_eq!(block_raw.block.header.height, 1);
        assert!(block_raw.block.data.txs.is_empty());
    }

    #[test]
    fn test_block_raw_decode_height_2() {
        let json_str = r#"{
            "block_id": {
                "hash": "yK6nk0XNaGa/yxmsPYSo9imvpPm1KKb0Pl1GHeEKgfc=",
                "part_set_header": {
                "total": 1,
                "hash": "QgWBa+3nEyKl3WHjpyt7/W9/ABVfFjYhZZd6etUQRsc="
                }
            },
            "block": {
                "header": {
                "version": {
                    "block": "11",
                    "app": "0"
                },
                "chain_id": "nyks",
                "height": "2",
                "time": "2025-06-13T13:06:36.719861789Z",
                "last_block_id": {
                    "hash": "ZxPZxXEc3r9uJVgqCjeqRAnrSWzv5QUg+AVWXO4BcK4=",
                    "part_set_header": {
                    "total": 1,
                    "hash": "sBIbv/9Gfqx9MzgmCYETc0Nx32fcclNxz+9R3MSuMF0="
                    }
                },
                "last_commit_hash": "frt1X5vTIn8dwMnQBsrlIzwNu+itG/yS+Hpbk1B5jMA=",
                "data_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                "validators_hash": "2JXZzU66O/FG00NQMm6Z77VAOWFiqY494wVoQKe47uk=",
                "next_validators_hash": "2JXZzU66O/FG00NQMm6Z77VAOWFiqY494wVoQKe47uk=",
                "consensus_hash": "BICRvH3cKD93v7+R1zxE2ljD34qcvIZ0Bdi389qtoi8=",
                "app_hash": "cmM8gjlrWjhTB/TfQLIbf1skbm0J8D7ieJr+4XEDKug=",
                "last_results_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                "evidence_hash": "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
                "proposer_address": "GTahVJYnvUCi88UM6EZDxgJEHVM="
                },
                "data": {
                "txs": [ ]
                },
                "evidence": {
                "evidence": [ ]
                },
                "last_commit": {
                "height": "1",
                "round": 0,
                "block_id": {
                    "hash": "ZxPZxXEc3r9uJVgqCjeqRAnrSWzv5QUg+AVWXO4BcK4=",
                    "part_set_header": {
                    "total": 1,
                    "hash": "sBIbv/9Gfqx9MzgmCYETc0Nx32fcclNxz+9R3MSuMF0="
                    }
                },
                "signatures": [
                    {
                    "block_id_flag": "BLOCK_ID_FLAG_COMMIT",
                    "validator_address": "GTahVJYnvUCi88UM6EZDxgJEHVM=",
                    "timestamp": "2025-06-13T13:06:36.719861789Z",
                    "signature": "OXGW+sa3kt7fmUFItAaenP1pMVhTrz5tXKQvNFEhKa8VYHRPjA/YXaDcLFI5hgoW8kmQPSDjXpOvqv19ylY5CA=="
                    }
                ]
                }
            }
            }"#;

        let block_raw: BlockRaw = match serde_json::from_str(json_str) {
            Ok(block_raw) => block_raw,
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Error: {:?}", e);
            }
        };
        println!("block_raw: {:?}", block_raw);
        assert_eq!(
            block_raw.block_id.hash,
            "yK6nk0XNaGa/yxmsPYSo9imvpPm1KKb0Pl1GHeEKgfc="
        );
        assert_eq!(block_raw.block_id.part_set_header.total, 1);
        assert_eq!(block_raw.block.header.chain_id, "nyks");
        assert_eq!(block_raw.block.header.height, 2);
        assert!(block_raw.block.data.txs.is_empty());
    }
}
use anyhow::{bail, Context, Result};
use hex;
use crate::db::insert_qq_tx;

use transaction::{Transaction, TransactionData, TransferTransaction, ScriptTransaction, Message};
/// Decode a string that may be base64 or hex into bytes.
fn decode_str_to_bytes(s: &str) -> Result<Vec<u8>> {    
    let clean = s.trim().strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(clean).context("Failed to decode hex string")?;
    Ok(bytes)
}

/// Deserialize into the *full* Transaction (struct with tx_type + tx data).
/// Tries bincode first; optionally falls back to postcard.
pub fn decode_transaction(tx_byte_code: &str) -> Result<Transaction> {
    let bytes = decode_str_to_bytes(tx_byte_code)?;

    // 1) bincode → Transaction
    match bincode::deserialize::<Transaction>(&bytes) {
        Ok(t) => return Ok(t),
        Err(e) => {
            // If this looks like an enum discriminant error, add a nice hint
            if e.to_string().contains("expected variant index") {
                // This error pops up when bytes aren't from the expected format.
                // We’ll try postcard next if enabled.
                // For now, just return the error.
                bail!("bincode deserialization failed (possible format mismatch): {e}");
            } else {
                // Other bincode errors—return the error.
                bail!("bincode deserialization failed: {e}");
            }
        }
    }
}

/// Convenience: decode and extract the TransferTransaction if present.
#[derive(Debug)]
pub enum DecodedQQTx {
    Transfer(TransferTransaction),
    Script(ScriptTransaction),
    Message(Message),
}

pub fn decode_qq_transaction(tx_byte_code: &str, block_height: u64) -> Result<DecodedQQTx> {
    println!("🔍 decode_qq_transaction: starting decode...");

    let t = match decode_transaction(tx_byte_code) {
        Ok(t) => {
            println!("🔍 decode_qq_transaction: decode_transaction succeeded, tx_type: {:?}",
                match &t.tx {
                    TransactionData::TransactionTransfer(_) => "Transfer",
                    TransactionData::TransactionScript(_) => "Script",
                    TransactionData::Message(_) => "Message",
                });
            t
        },
        Err(e) => {
            eprintln!("⚠️ decode_qq_transaction: decode_transaction failed: {:?}", e);
            return Err(e);
        }
    };

    let ts_json = serde_json::to_string_pretty(&t)
        .context("Failed to serialize Transaction to JSON")?;

    println!("🔍 decode_qq_transaction: inserting to qq_tx table...");
    if let Err(e) = insert_qq_tx(&ts_json, block_height) {
        eprintln!("⚠️ decode_qq_transaction: insert_qq_tx failed: {:?}", e);
        return Err(e.into());
    }
    println!("🔍 decode_qq_transaction: insert succeeded");

    Ok(match t.tx {
        TransactionData::TransactionTransfer(tx) => DecodedQQTx::Transfer(tx),
        TransactionData::TransactionScript(tx)   => DecodedQQTx::Script(tx),
        TransactionData::Message(msg)            => DecodedQQTx::Message(msg),
    })
}
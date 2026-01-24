use anyhow::Result;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use prost::Message;
use prost_types::Any;

// Tx containers from cosmos-sdk-proto
use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, TxBody, TxRaw};

// Common standard messages (add more as you need)
use cosmos_sdk_proto::cosmos::bank::v1beta1::{MsgMultiSend, MsgSend, SendAuthorization};
use cosmos_sdk_proto::cosmos::staking::v1beta1::{MsgBeginRedelegate, MsgDelegate, MsgUndelegate};
use cosmos_sdk_proto::cosmos::distribution::v1beta1::{
    MsgFundCommunityPool, MsgSetWithdrawAddress, MsgWithdrawDelegatorReward,
    MsgWithdrawValidatorCommission,
};
use cosmos_sdk_proto::cosmos::gov::v1beta1::{MsgDeposit, MsgSubmitProposal, MsgVote, MsgVoteWeighted};

use twilight_indexer::twilightproject::nyks::bridge as nyksBridge;
use twilight_indexer::twilightproject::nyks::zkos as nyksZkos;

// Import upsert_transaction_count so it is available in this module
use crate::db::*;
use crate::quis_quis_tx::decode_qq_transaction;
use crate::quis_quis_tx::DecodedQQTx;

// use transaction::

/// Typed envelope for standard Cosmos messages (no Debug/serde derives to avoid trait issues).
#[allow(dead_code)]
#[derive(Debug)]
pub enum StandardCosmosMsg {
    // ----- existing standard msgs -----
    // bank
    BankSend(MsgSend),
    BankMultiSend(MsgMultiSend),
    BankSendAuth(SendAuthorization),

    // staking
    StakingDelegate(MsgDelegate),
    StakingUndelegate(MsgUndelegate),
    StakingBeginRedelegate(MsgBeginRedelegate),

    // distribution
    DistWithdrawDelegatorReward(MsgWithdrawDelegatorReward),
    DistWithdrawValidatorCommission(MsgWithdrawValidatorCommission),
    DistSetWithdrawAddress(MsgSetWithdrawAddress),
    DistFundCommunityPool(MsgFundCommunityPool),

    // gov v1beta1
    GovSubmitProposal(MsgSubmitProposal),
    GovDeposit(MsgDeposit),
    GovVote(MsgVote),
    GovVoteWeighted(MsgVoteWeighted),

    // ----- NEW: NYKS bridge custom msgs -----
    NyksConfirmBtcDeposit(nyksBridge::MsgConfirmBtcDeposit),
    NyksRegisterBtcDepositAddress(nyksBridge::MsgRegisterBtcDepositAddress),
    NyksRegisterReserveAddress(nyksBridge::MsgRegisterReserveAddress),
    NyksBootstrapFragment(nyksBridge::MsgBootstrapFragment),
    NyksWithdrawBtcRequest(nyksBridge::MsgWithdrawBtcRequest),
    NyksWithdrawTxSigned(nyksBridge::MsgWithdrawTxSigned),
    NyksWithdrawTxFinal(nyksBridge::MsgWithdrawTxFinal),
    NyksConfirmBtcWithdraw(nyksBridge::MsgConfirmBtcWithdraw),
    NyksProposeSweepAddress(nyksBridge::MsgProposeSweepAddress),
    NyksUnsignedTxSweep(nyksBridge::MsgUnsignedTxSweep),
    NyksUnsignedTxRefund(nyksBridge::MsgUnsignedTxRefund),
    NyksSignRefund(nyksBridge::MsgSignRefund),
    NyksSignSweep(nyksBridge::MsgSignSweep),
    NyksBroadcastTxRefund(nyksBridge::MsgBroadcastTxRefund),
    NyksBroadcastTxSweep(nyksBridge::MsgBroadcastTxSweep),
    NyksSweepProposal(nyksBridge::MsgSweepProposal),

    NyksZkosMsgTransferTx(nyksZkos::MsgTransferTx),
    NyksZkosMsgMintBurnTradingBtc(nyksZkos::MsgMintBurnTradingBtc),
    /// Fallback
    Unknown { type_url: String, raw_value_hex: String },
}

/// Final decoded transaction: concrete prost structs (no serde).
#[derive(Debug)]
pub struct DecodedTx {
    pub _body: TxBody,
    pub _auth_info: AuthInfo,
    pub _signatures: Vec<Vec<u8>>,
    pub _messages: Vec<StandardCosmosMsg>,
}

/// Extract signer address from a message's Any type (for gas tracking)
fn extract_signer_from_any(any: &Any) -> Option<String> {
    let t = any.type_url.as_str();
    let bytes = any.value.as_slice();

    // cosmos.bank.v1beta1.MsgSend
    if ty(t, "cosmos.bank.v1beta1.MsgSend") {
        if let Ok(tx) = MsgSend::decode(bytes) {
            return Some(tx.from_address);
        }
    }
    // cosmos.staking.v1beta1.MsgDelegate
    if ty(t, "cosmos.staking.v1beta1.MsgDelegate") {
        if let Ok(tx) = MsgDelegate::decode(bytes) {
            return Some(tx.delegator_address);
        }
    }
    // cosmos.staking.v1beta1.MsgUndelegate
    if ty(t, "cosmos.staking.v1beta1.MsgUndelegate") {
        if let Ok(tx) = MsgUndelegate::decode(bytes) {
            return Some(tx.delegator_address);
        }
    }
    // cosmos.staking.v1beta1.MsgBeginRedelegate
    if ty(t, "cosmos.staking.v1beta1.MsgBeginRedelegate") {
        if let Ok(tx) = MsgBeginRedelegate::decode(bytes) {
            return Some(tx.delegator_address);
        }
    }
    // cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward
    if ty(t, "cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward") {
        if let Ok(tx) = MsgWithdrawDelegatorReward::decode(bytes) {
            return Some(tx.delegator_address);
        }
    }
    // cosmos.gov.v1beta1.MsgVote
    if ty(t, "cosmos.gov.v1beta1.MsgVote") {
        if let Ok(tx) = MsgVote::decode(bytes) {
            return Some(tx.voter);
        }
    }
    // twilightproject.nyks.bridge.MsgConfirmBtcDeposit
    if ty(t, "twilightproject.nyks.bridge.MsgConfirmBtcDeposit") {
        if let Ok(tx) = nyksBridge::MsgConfirmBtcDeposit::decode(bytes) {
            return Some(tx.twilight_deposit_address);
        }
    }
    // twilightproject.nyks.bridge.MsgWithdrawBtcRequest
    if ty(t, "twilightproject.nyks.bridge.MsgWithdrawBtcRequest") {
        if let Ok(tx) = nyksBridge::MsgWithdrawBtcRequest::decode(bytes) {
            return Some(tx.twilight_address);
        }
    }
    // twilightproject.nyks.zkos.MsgMintBurnTradingBtc
    if ty(t, "twilightproject.nyks.zkos.MsgMintBurnTradingBtc") {
        if let Ok(tx) = nyksZkos::MsgMintBurnTradingBtc::decode(bytes) {
            return Some(tx.twilight_address);
        }
    }

    None
}

/// Decode a base64-encoded TxRaw (from `block.txs[i]`) into concrete structs.
pub fn decode_tx_base64_standard(tx_b64: &str, block_height: u64) -> Result<DecodedTx> {
    // 1) base64 → bytes → TxRaw
    let raw_bytes = B64.decode(tx_b64.trim())?;
    let tx_raw = TxRaw::decode(raw_bytes.as_slice())?;

    // 2) TxBody & AuthInfo
    let body = TxBody::decode(tx_raw.body_bytes.as_slice())?;
    let auth = AuthInfo::decode(tx_raw.auth_info_bytes.as_slice())?;

    // 3) Extract signer address from first message (for gas tracking)
    let signer_address = body.messages.first().and_then(extract_signer_from_any);

    // 4) Messages (Any) → typed messages
    let mut msgs = Vec::<StandardCosmosMsg>::new();
    for any in &body.messages {
        msgs.push(decode_standard_any(any, block_height)?);
    }

    // 5) Record gas usage if we have fee info and a signer address
    if let (Some(fee), Some(addr)) = (&auth.fee, &signer_address) {
        if let Some(coin) = fee.amount.first() {
            if let Ok(gas_amount) = coin.amount.parse::<i64>() {
                if let Err(e) = insert_gas_used(&addr, gas_amount, &coin.denom, block_height as i64) {
                    eprintln!("⚠️ Failed to update gas_used_nyks for {}: {:?}", addr, e);
                }
            }
        }
    }

    Ok(DecodedTx {
        _body: body,
        _auth_info: auth,
        _signatures: tx_raw.signatures, // raw bytes; hex when printing
        _messages: msgs,
    })
}

pub fn decode_standard_any(any: &Any, block_height: u64) -> Result<StandardCosmosMsg> {
    let t = any.type_url.as_str();
    let bytes = any.value.as_slice();

    // ---------- cosmos.bank.v1beta1 ----------
    if ty(t, "cosmos.bank.v1beta1.MsgSend") {
        let tx = MsgSend::decode(bytes)?;
        
        if let Err(e) = insert_transaction_count(&tx.from_address, block_height) {
            eprintln!("⚠️ Failed to update transaction_count for {}: {:?}", tx.from_address, e);
        }

        for coin in tx.amount.clone() {
            let amount: i64 = coin.amount.parse::<i64>().expect("Failed to parse amount string to i64");
            if let Err(e) = insert_funds_moved(&tx.to_address, amount, &coin.denom, block_height) {
                eprintln!("⚠️ Failed to update funds_moved for {}: {:?}", tx.to_address, e);
            }
        }
        return Ok(StandardCosmosMsg::BankSend(tx));
    }

    if ty(t, "cosmos.bank.v1beta1.MsgMultiSend") {
        return Ok(StandardCosmosMsg::BankMultiSend(MsgMultiSend::decode(bytes)?));
    }
    if ty(t, "cosmos.bank.v1beta1.SendAuthorization") {
        return Ok(StandardCosmosMsg::BankSendAuth(SendAuthorization::decode(bytes)?));
    }

    // ---------- cosmos.staking.v1beta1 ----------
    if ty(t, "cosmos.staking.v1beta1.MsgDelegate") {
        return Ok(StandardCosmosMsg::StakingDelegate(MsgDelegate::decode(bytes)?));
    }
    if ty(t, "cosmos.staking.v1beta1.MsgUndelegate") {
        return Ok(StandardCosmosMsg::StakingUndelegate(MsgUndelegate::decode(bytes)?));
    }
    if ty(t, "cosmos.staking.v1beta1.MsgBeginRedelegate") {
        return Ok(StandardCosmosMsg::StakingBeginRedelegate(MsgBeginRedelegate::decode(bytes)?));
    }

    // ---------- cosmos.distribution.v1beta1 ----------
    if ty(t, "cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward") {
        return Ok(StandardCosmosMsg::DistWithdrawDelegatorReward(
            MsgWithdrawDelegatorReward::decode(bytes)?,
        ));
    }
    if ty(t, "cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission") {
        return Ok(StandardCosmosMsg::DistWithdrawValidatorCommission(
            MsgWithdrawValidatorCommission::decode(bytes)?,
        ));
    }
    if ty(t, "cosmos.distribution.v1beta1.MsgSetWithdrawAddress") {
        return Ok(StandardCosmosMsg::DistSetWithdrawAddress(
            MsgSetWithdrawAddress::decode(bytes)?,
        ));
    }
    if ty(t, "cosmos.distribution.v1beta1.MsgFundCommunityPool") {
        return Ok(StandardCosmosMsg::DistFundCommunityPool(
            MsgFundCommunityPool::decode(bytes)?,
        ));
    }

    // ---------- cosmos.gov.v1beta1 ----------
    if ty(t, "cosmos.gov.v1beta1.MsgSubmitProposal") {
        return Ok(StandardCosmosMsg::GovSubmitProposal(MsgSubmitProposal::decode(bytes)?));
    }
    if ty(t, "cosmos.gov.v1beta1.MsgDeposit") {
        return Ok(StandardCosmosMsg::GovDeposit(MsgDeposit::decode(bytes)?));
    }
    if ty(t, "cosmos.gov.v1beta1.MsgVote") {
        return Ok(StandardCosmosMsg::GovVote(MsgVote::decode(bytes)?));
    }
    if ty(t, "cosmos.gov.v1beta1.MsgVoteWeighted") {
        return Ok(StandardCosmosMsg::GovVoteWeighted(MsgVoteWeighted::decode(bytes)?));
    }

    // ---------- twilightproject.nyks.bridge (custom) ----------
    if ty(t, "twilightproject.nyks.bridge.MsgConfirmBtcDeposit") {
        let tx = nyksBridge::MsgConfirmBtcDeposit::decode(bytes)?;

        if let Err(e) = insert_lit_minted_sats(&tx.twilight_deposit_address, tx.deposit_amount as i64, block_height) {
            eprintln!("⚠️ Failed to update transaction for {}: {:?}", tx.twilight_deposit_address, e);
        }
        
        return Ok(StandardCosmosMsg::NyksConfirmBtcDeposit(tx));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgRegisterBtcDepositAddress") {
        return Ok(StandardCosmosMsg::NyksRegisterBtcDepositAddress(nyksBridge::MsgRegisterBtcDepositAddress::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgRegisterReserveAddress") {
        return Ok(StandardCosmosMsg::NyksRegisterReserveAddress(nyksBridge::MsgRegisterReserveAddress::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgBootstrapFragment") {
        return Ok(StandardCosmosMsg::NyksBootstrapFragment(nyksBridge::MsgBootstrapFragment::decode(bytes)?));
    }

    if ty(t, "twilightproject.nyks.bridge.MsgWithdrawBtcRequest") {
        let tx = nyksBridge::MsgWithdrawBtcRequest::decode(bytes)?;
        if let Err(e) = insert_lit_burned_sats(&tx.twilight_address, tx.withdraw_amount as i64, block_height) {
            eprintln!("⚠️ Failed to update transaction for {}: {:?}", tx.twilight_address, e);
        }
        return Ok(StandardCosmosMsg::NyksWithdrawBtcRequest(tx));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgWithdrawTxSigned") {
        return Ok(StandardCosmosMsg::NyksWithdrawTxSigned(nyksBridge::MsgWithdrawTxSigned::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgWithdrawTxFinal") {
        return Ok(StandardCosmosMsg::NyksWithdrawTxFinal(nyksBridge::MsgWithdrawTxFinal::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgConfirmBtcWithdraw") {
        return Ok(StandardCosmosMsg::NyksConfirmBtcWithdraw(nyksBridge::MsgConfirmBtcWithdraw::decode(bytes)?));
    }

    if ty(t, "twilightproject.nyks.bridge.MsgProposeSweepAddress") {
        return Ok(StandardCosmosMsg::NyksProposeSweepAddress(nyksBridge::MsgProposeSweepAddress::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgUnsignedTxSweep") {
        return Ok(StandardCosmosMsg::NyksUnsignedTxSweep(nyksBridge::MsgUnsignedTxSweep::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgUnsignedTxRefund") {
        return Ok(StandardCosmosMsg::NyksUnsignedTxRefund(nyksBridge::MsgUnsignedTxRefund::decode(bytes)?));
    }

    if ty(t, "twilightproject.nyks.bridge.MsgSignRefund") {
        return Ok(StandardCosmosMsg::NyksSignRefund(nyksBridge::MsgSignRefund::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgSignSweep") {
        return Ok(StandardCosmosMsg::NyksSignSweep(nyksBridge::MsgSignSweep::decode(bytes)?));
    }

    if ty(t, "twilightproject.nyks.bridge.MsgBroadcastTxRefund") {
        return Ok(StandardCosmosMsg::NyksBroadcastTxRefund(nyksBridge::MsgBroadcastTxRefund::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgBroadcastTxSweep") {
        return Ok(StandardCosmosMsg::NyksBroadcastTxSweep(nyksBridge::MsgBroadcastTxSweep::decode(bytes)?));
    }
    if ty(t, "twilightproject.nyks.bridge.MsgSweepProposal") {
        return Ok(StandardCosmosMsg::NyksSweepProposal(nyksBridge::MsgSweepProposal::decode(bytes)?));
    }

    if ty(t, "twilightproject.nyks.zkos.MsgTransferTx") {
        let cosmos_tx = nyksZkos::MsgTransferTx::decode(bytes)?;
        let decoded = decode_qq_transaction(&cosmos_tx.tx_byte_code, block_height)?;
        match decoded {
                DecodedQQTx::Transfer(tx) => {
                    eprint!("Got transfer tx: {:?}", tx);
                    let inputs = tx.get_input_values();
                    let outputs = tx.get_output_values();
                    if inputs.is_empty() || outputs.is_empty() { 
                        eprintln!("⚠️ TransferTransaction has no inputs or outputs");
                        return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx));
                    }
                    let owner = match inputs[0].as_owner_address() {
                        Some(o) => o.clone(),
                        None => {
                            eprintln!("⚠️ Failed to get owner address from input");
                            return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx))
                        },
                    };
                    let new_qq_account = outputs[0]
                        .to_quisquis_account()
                        .expect("Failed to convert to quisquis account"
                    );
                    let new_qq_account = hex::encode(
                    bincode::serialize(&new_qq_account)
                        .expect("Failed to serialize account to bytes")
                    );

                    let t_address = match get_taddress_for_qaddress(&owner)?{
                        Some(o) => o.clone(),
                        None => return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx)),
                    };

                    if let Err(e) = insert_addr_mappings(&t_address, &new_qq_account, block_height) {
                        eprintln!("⚠️ Failed to update addr_mappings for {} <-> {}: {:?}", t_address, new_qq_account, e);
                    }

                    if let Err(e) = insert_transaction_count(&t_address, block_height) {
                        eprintln!("⚠️ Failed to update transaction_count for {}: {:?}", t_address, e);
                    }

                    if inputs[0].in_type == zkvm::IOType::Coin && outputs[0].out_type == zkvm::IOType::Memo {
                        if let Err(e) = insert_trading_tx(&new_qq_account, &owner, block_height){
                            eprintln!("⚠️ Failed to update trading tx for {}: {:?}", new_qq_account, e);
                        }
                    }
                }
                DecodedQQTx::Script(script_tx) => {
                    println!("Got script tx: {:?}", script_tx);
                    let inputs = script_tx.get_input_values();
                    let outputs = script_tx.get_output_values();

                    println!("🔍 Script TX - inputs count: {}, outputs count: {}", inputs.len(), outputs.len());

                    if inputs.is_empty() || outputs.is_empty() {
                        eprintln!("⚠️ ScriptTransaction has no inputs or outputs");
                        return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx));
                    }

                    println!("🔍 Script TX - input[0].in_type: {:?}, output[0].out_type: {:?}",
                             inputs[0].in_type, outputs[0].out_type);

                    let owner = match inputs[0].as_owner_address() {
                        Some(o) => {
                            println!("🔍 Script TX - owner address: {}", o);
                            o.clone()
                        },
                        None => {
                            eprintln!("⚠️ Failed to get owner address from input");
                            return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx))
                        },
                    };

                    let new_qq_account = match outputs[0].to_quisquis_account() {
                        Ok(acc) => acc,
                        Err(e) => {
                            eprintln!("⚠️ Failed to convert output to quisquis account: {}", e);
                            return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx));
                        }
                    };
                    let new_qq_account = hex::encode(
                        bincode::serialize(&new_qq_account)
                            .expect("Failed to serialize account to bytes")
                    );

                    println!("🔍 Script TX - new_qq_account: {}", new_qq_account);

                    let is_order_open = inputs[0].in_type == zkvm::IOType::Coin && outputs[0].out_type == zkvm::IOType::Memo;
                    let is_order_close = inputs[0].in_type == zkvm::IOType::Memo && outputs[0].out_type == zkvm::IOType::Coin;

                    println!("🔍 Script TX - is_order_open: {}, is_order_close: {}", is_order_open, is_order_close);

                    if is_order_open {
                        println!("📝 Inserting order_open_tx: to={}, from={}, block={}", new_qq_account, owner, block_height);
                        if let Err(e) = insert_order_open_tx(&new_qq_account, &owner, block_height){
                            eprintln!("⚠️ Failed to insert order_open_tx for {}: {:?}", new_qq_account, e);
                        } else {
                            println!("✅ Successfully inserted order_open_tx");
                        }
                    }

                    if is_order_close {
                        println!("📝 Inserting order_close_tx: to={}, from={}, block={}", new_qq_account, owner, block_height);
                        if let Err(e) = insert_order_close_tx(&new_qq_account, &owner, block_height){
                            eprintln!("⚠️ Failed to insert order_close_tx for {}: {:?}", new_qq_account, e);
                        } else {
                            println!("✅ Successfully inserted order_close_tx");
                        }
                    }

                    if !is_order_open && !is_order_close {
                        println!("⚠️ Script TX did not match order_open or order_close conditions");
                    }
                }
                DecodedQQTx::Message(msg) => {
                    println!("Got message tx: {:?}", msg);
                }
        }
        return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx));
    }

    if ty(t, "twilightproject.nyks.zkos.MsgMintBurnTradingBtc") {
        let tx = nyksZkos::MsgMintBurnTradingBtc::decode(bytes)?;
        if tx.mint_or_burn == true {
            if let Err(e) = insert_dark_minted_sats(&tx.twilight_address, &tx.qq_account, tx.btc_value as i64, block_height) {
                eprintln!("⚠️ Failed to update dark minted sats for {}: {:?}", tx.twilight_address, e);
            }
            if let Err(e) = insert_addr_mappings(&tx.twilight_address, &tx.qq_account, block_height) {
                eprintln!("⚠️ Failed to update addr_mappings for {} <-> {}: {:?}", tx.twilight_address, tx.qq_account, e);
            }
        }
        else if tx.mint_or_burn == false {
            if let Err(e) = insert_dark_burned_sats(&tx.twilight_address, &tx.qq_account, tx.btc_value as i64, block_height) {
                eprintln!("⚠️ Failed to update dark burned sats for {}: {:?}", tx.twilight_address, e);
            }
        }

        if let Err(e) = insert_transaction_count(&tx.twilight_address, block_height) {
            eprintln!("⚠️ Failed to update transaction_count for {}: {:?}", tx.twilight_address, e);
        }

        return Ok(StandardCosmosMsg::NyksZkosMsgMintBurnTradingBtc(tx));
    }

    // ---------- Fallback ----------
    Ok(StandardCosmosMsg::Unknown {
        type_url: any.type_url.clone(),
        raw_value_hex: hex::encode(&any.value),
    })
}


// /// Simple printer so you can see what's inside without serde/Debug derives.
// pub fn print_tx(tx: &DecodedTx) {
//     println!("memo: {}", tx.body.memo);
//     println!("timeout_height: {}", tx.body.timeout_height);
//     if let Some(fee) = &tx.auth_info.fee {
//         println!("gas_limit: {}", fee.gas_limit);
//         for c in &fee.amount {
//             println!("fee amount: {} {}", c.amount, c.denom);
//         }
//     }
//     println!("signatures: {}", tx.signatures.len());
//     for (i, sig) in tx.signatures.iter().enumerate() {
//         println!("  sig[{i}]: {}", hex::encode(sig));
//     }
//     println!("messages: {}", tx.messages.len());
//     for (i, m) in tx.messages.iter().enumerate() {
//         match m {
//             StandardCosmosMsg::BankSend(msg) => {
//                 let amts = msg.amount.iter()
//                     .map(|c| format!("{} {}", c.amount, c.denom))
//                     .collect::<Vec<_>>()
//                     .join(", ");
//                 println!("  [{i}] bank.MsgSend {} -> {} [{}]", msg.from_address, msg.to_address, amts);
//             }
//             StandardCosmosMsg::StakingDelegate(msg) => {
//                 if let Some(coin) = &msg.amount {
//                     println!("  [{i}] staking.MsgDelegate {} to {} ({} {})",
//                         msg.delegator_address, msg.validator_address, coin.amount, coin.denom);
//                 } else {
//                     println!("  [{i}] staking.MsgDelegate {} to {} (no amount?)",
//                         msg.delegator_address, msg.validator_address);
//                 }
//             }
//             StandardCosmosMsg::Unknown { type_url, .. } => {
//                 println!("  [{i}] <UNKNOWN> {}", type_url);
//             }
//             _ => {
//                 // add more pretty cases as you need
//                 println!("  [{i}] {}", type_name(m));
//             }
//         }
//     }
// }

// fn type_name(m: &StandardCosmosMsg) -> &'static str {
//     match m {
//         // ---- Cosmos standard ----
//         StandardCosmosMsg::BankSend(_) => "cosmos.bank.v1beta1.MsgSend",
//         StandardCosmosMsg::BankMultiSend(_) => "cosmos.bank.v1beta1.MsgMultiSend",
//         StandardCosmosMsg::BankSendAuth(_) => "cosmos.bank.v1beta1.SendAuthorization",

//         StandardCosmosMsg::StakingDelegate(_) => "cosmos.staking.v1beta1.MsgDelegate",
//         StandardCosmosMsg::StakingUndelegate(_) => "cosmos.staking.v1beta1.MsgUndelegate",
//         StandardCosmosMsg::StakingBeginRedelegate(_) => "cosmos.staking.v1beta1.MsgBeginRedelegate",

//         StandardCosmosMsg::DistWithdrawDelegatorReward(_) => "cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
//         StandardCosmosMsg::DistWithdrawValidatorCommission(_) => "cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission",
//         StandardCosmosMsg::DistSetWithdrawAddress(_) => "cosmos.distribution.v1beta1.MsgSetWithdrawAddress",
//         StandardCosmosMsg::DistFundCommunityPool(_) => "cosmos.distribution.v1beta1.MsgFundCommunityPool",

//         StandardCosmosMsg::GovSubmitProposal(_) => "cosmos.gov.v1beta1.MsgSubmitProposal",
//         StandardCosmosMsg::GovDeposit(_) => "cosmos.gov.v1beta1.MsgDeposit",
//         StandardCosmosMsg::GovVote(_) => "cosmos.gov.v1beta1.MsgVote",
//         StandardCosmosMsg::GovVoteWeighted(_) => "cosmos.gov.v1beta1.MsgVoteWeighted",

//         // ---- Twilight NYKS bridge ----
//         StandardCosmosMsg::NyksConfirmBtcDeposit(_) => "twilightproject.nyks.bridge.MsgConfirmBtcDeposit",
//         StandardCosmosMsg::NyksRegisterBtcDepositAddress(_) => "twilightproject.nyks.bridge.MsgRegisterBtcDepositAddress",
//         StandardCosmosMsg::NyksRegisterReserveAddress(_) => "twilightproject.nyks.bridge.MsgRegisterReserveAddress",
//         StandardCosmosMsg::NyksBootstrapFragment(_) => "twilightproject.nyks.bridge.MsgBootstrapFragment",

//         StandardCosmosMsg::NyksWithdrawBtcRequest(_) => "twilightproject.nyks.bridge.MsgWithdrawBtcRequest",
//         StandardCosmosMsg::NyksWithdrawTxSigned(_) => "twilightproject.nyks.bridge.MsgWithdrawTxSigned",
//         StandardCosmosMsg::NyksWithdrawTxFinal(_) => "twilightproject.nyks.bridge.MsgWithdrawTxFinal",
//         StandardCosmosMsg::NyksConfirmBtcWithdraw(_) => "twilightproject.nyks.bridge.MsgConfirmBtcWithdraw",

//         StandardCosmosMsg::NyksProposeSweepAddress(_) => "twilightproject.nyks.bridge.MsgProposeSweepAddress",
//         StandardCosmosMsg::NyksUnsignedTxSweep(_) => "twilightproject.nyks.bridge.MsgUnsignedTxSweep",
//         StandardCosmosMsg::NyksUnsignedTxRefund(_) => "twilightproject.nyks.bridge.MsgUnsignedTxRefund",

//         StandardCosmosMsg::NyksSignRefund(_) => "twilightproject.nyks.bridge.MsgSignRefund",
//         StandardCosmosMsg::NyksSignSweep(_) => "twilightproject.nyks.bridge.MsgSignSweep",

//         StandardCosmosMsg::NyksBroadcastTxRefund(_) => "twilightproject.nyks.bridge.MsgBroadcastTxRefund",
//         StandardCosmosMsg::NyksBroadcastTxSweep(_) => "twilightproject.nyks.bridge.MsgBroadcastTxSweep",

//         StandardCosmosMsg::NyksSweepProposal(_) => "twilightproject.nyks.bridge.MsgSweepProposal",

//         StandardCosmosMsg::NyksZkosMsgTransferTx(_) => "twilightproject.nyks.zkos.MsgTransferTx",
//         StandardCosmosMsg::NyksZkosMsgMintBurnTradingBtc(_) => "twilightproject.nyks.zkos.MsgMintBurnTradingBtc",

//         // ---- Fallback ----
//         StandardCosmosMsg::Unknown { .. } => "<UNKNOWN>",
//     }
// }


fn ty(t: &str, want: &str) -> bool {
    // Accept both "/pkg.MsgType" and "pkg.MsgType"
    t == want || t.strip_prefix('/') == Some(want)
}
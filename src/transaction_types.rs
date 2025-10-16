use anyhow::Result;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use cosmos_sdk_proto::cosmos::authz::v1beta1::msg_server::Msg;
use diesel::upsert;
use prost::Message;
use prost_types::Any;

// Import the Message trait so that decode is available for prost types
use prost::Message as _;

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
    pub body: TxBody,
    pub auth_info: AuthInfo,
    pub signatures: Vec<Vec<u8>>,
    pub messages: Vec<StandardCosmosMsg>,
}

/// Decode a base64-encoded TxRaw (from `block.txs[i]`) into concrete structs.
pub fn decode_tx_base64_standard(tx_b64: &str) -> Result<DecodedTx> {
    // 1) base64 → bytes → TxRaw
    let raw_bytes = B64.decode(tx_b64.trim())?;
    let tx_raw = TxRaw::decode(raw_bytes.as_slice())?;

    // 2) TxBody & AuthInfo
    let body = TxBody::decode(tx_raw.body_bytes.as_slice())?;
    let auth = AuthInfo::decode(tx_raw.auth_info_bytes.as_slice())?;

    // 3) Messages (Any) → typed messages
    let mut msgs = Vec::<StandardCosmosMsg>::new();
    for any in &body.messages {
        msgs.push(decode_standard_any(any)?);
    }

    Ok(DecodedTx {
        body,
        auth_info: auth,
        signatures: tx_raw.signatures, // raw bytes; hex when printing
        messages: msgs,
    })
}

pub fn decode_standard_any(any: &Any) -> Result<StandardCosmosMsg> {
    let t = any.type_url.as_str();
    let bytes = any.value.as_slice();

    // ---------- cosmos.bank.v1beta1 ----------
    if ty(t, "cosmos.bank.v1beta1.MsgSend") {
        let tx = MsgSend::decode(bytes)?;
        
        if let Err(e) = upsert_transaction_count(&tx.from_address, 1) {
            eprintln!("⚠️ Failed to update transaction_count for {}: {:?}", tx.from_address, e);
        }

        let amount = tx.amount.iter().map(|c| c.amount.parse::<i64>().unwrap_or(0)).sum();
        if let Err(e) = upsert_funds_moved(&tx.from_address, amount) {
            eprintln!("⚠️ Failed to update funds_moved for {}: {:?}", tx.from_address, e);
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

        if let Err(e) = upsert_lit_minted_sats(&tx.twilight_deposit_address, tx.deposit_amount as i64) {
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
        if let Err(e) = upsert_lit_burned_sats(&tx.twilight_address, tx.withdraw_amount as i64) {
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
        let decoded = decode_qq_transaction(&cosmos_tx.tx_byte_code)?;
        match decoded {
                DecodedQQTx::Transfer(tx) => {
                    let inputs = tx.get_input_values();
                    let outputs = tx.get_output_values();
                    if inputs.is_empty() || outputs.is_empty() {
                        eprintln!("⚠️ TransferTransaction has no inputs or outputs");
                        return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx));
                    }
                    let owner = match inputs[0].as_owner_address() {
                        Some(o) => o.clone(),
                        None => return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx)),
                    };
                    let new_qq_account = outputs[0]
                        .to_quisquis_account()
                        .expect("Failed to convert to quisquis account"
                    );
                    let new_qq_account = hex::encode(
                    bincode::serialize(&new_qq_account)
                        .expect("Failed to serialize account to bytes")
                    );

                    let tAddress = match get_taddress_for_qaddress(&owner)?{
                        Some(o) => o.clone(),
                        None => return Ok(StandardCosmosMsg::NyksZkosMsgTransferTx(cosmos_tx)),
                    };

                    if let Err(e) = upsert_addr_mappings(&tAddress, &new_qq_account) {
                        eprintln!("⚠️ Failed to update addr_mappings for {} <-> {}: {:?}", tAddress, new_qq_account, e);
                    }

                    if let Err(e) = upsert_transaction_count(&tAddress, 1) {
                        eprintln!("⚠️ Failed to update transaction_count for {}: {:?}", tAddress, e);
                    }
                }
                DecodedQQTx::Script(script) => {
                    println!("Got script tx: {:?}", script);
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
            if let Err(e) = upsert_dark_minted_sats(&tx.twilight_address, &tx.qq_account, tx.btc_value as i64) {
                eprintln!("⚠️ Failed to update dark minted sats for {}: {:?}", tx.twilight_address, e);
            }
            if let Err(e) = upsert_addr_mappings(&tx.twilight_address, &tx.qq_account) {
                eprintln!("⚠️ Failed to update addr_mappings for {} <-> {}: {:?}", tx.twilight_address, tx.qq_account, e);
            }
        }
        else if tx.mint_or_burn == false {
            if let Err(e) = upsert_dark_burned_sats(&tx.twilight_address, &tx.qq_account, tx.btc_value as i64) {
                eprintln!("⚠️ Failed to update dark burned sats for {}: {:?}", tx.twilight_address, e);
            }
        }

        if let Err(e) = upsert_transaction_count(&tx.twilight_address, 1) {
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


/// Simple printer so you can see what's inside without serde/Debug derives.
pub fn print_tx(tx: &DecodedTx) {
    println!("memo: {}", tx.body.memo);
    println!("timeout_height: {}", tx.body.timeout_height);
    if let Some(fee) = &tx.auth_info.fee {
        println!("gas_limit: {}", fee.gas_limit);
        for c in &fee.amount {
            println!("fee amount: {} {}", c.amount, c.denom);
        }
    }
    println!("signatures: {}", tx.signatures.len());
    for (i, sig) in tx.signatures.iter().enumerate() {
        println!("  sig[{i}]: {}", hex::encode(sig));
    }
    println!("messages: {}", tx.messages.len());
    for (i, m) in tx.messages.iter().enumerate() {
        match m {
            StandardCosmosMsg::BankSend(msg) => {
                let amts = msg.amount.iter()
                    .map(|c| format!("{} {}", c.amount, c.denom))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("  [{i}] bank.MsgSend {} -> {} [{}]", msg.from_address, msg.to_address, amts);
            }
            StandardCosmosMsg::StakingDelegate(msg) => {
                if let Some(coin) = &msg.amount {
                    println!("  [{i}] staking.MsgDelegate {} to {} ({} {})",
                        msg.delegator_address, msg.validator_address, coin.amount, coin.denom);
                } else {
                    println!("  [{i}] staking.MsgDelegate {} to {} (no amount?)",
                        msg.delegator_address, msg.validator_address);
                }
            }
            StandardCosmosMsg::Unknown { type_url, .. } => {
                println!("  [{i}] <UNKNOWN> {}", type_url);
            }
            _ => {
                // add more pretty cases as you need
                println!("  [{i}] {}", type_name(m));
            }
        }
    }
}

fn type_name(m: &StandardCosmosMsg) -> &'static str {
    match m {
        // ---- Cosmos standard ----
        StandardCosmosMsg::BankSend(_) => "cosmos.bank.v1beta1.MsgSend",
        StandardCosmosMsg::BankMultiSend(_) => "cosmos.bank.v1beta1.MsgMultiSend",
        StandardCosmosMsg::BankSendAuth(_) => "cosmos.bank.v1beta1.SendAuthorization",

        StandardCosmosMsg::StakingDelegate(_) => "cosmos.staking.v1beta1.MsgDelegate",
        StandardCosmosMsg::StakingUndelegate(_) => "cosmos.staking.v1beta1.MsgUndelegate",
        StandardCosmosMsg::StakingBeginRedelegate(_) => "cosmos.staking.v1beta1.MsgBeginRedelegate",

        StandardCosmosMsg::DistWithdrawDelegatorReward(_) => "cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
        StandardCosmosMsg::DistWithdrawValidatorCommission(_) => "cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission",
        StandardCosmosMsg::DistSetWithdrawAddress(_) => "cosmos.distribution.v1beta1.MsgSetWithdrawAddress",
        StandardCosmosMsg::DistFundCommunityPool(_) => "cosmos.distribution.v1beta1.MsgFundCommunityPool",

        StandardCosmosMsg::GovSubmitProposal(_) => "cosmos.gov.v1beta1.MsgSubmitProposal",
        StandardCosmosMsg::GovDeposit(_) => "cosmos.gov.v1beta1.MsgDeposit",
        StandardCosmosMsg::GovVote(_) => "cosmos.gov.v1beta1.MsgVote",
        StandardCosmosMsg::GovVoteWeighted(_) => "cosmos.gov.v1beta1.MsgVoteWeighted",

        // ---- Twilight NYKS bridge ----
        StandardCosmosMsg::NyksConfirmBtcDeposit(_) => "twilightproject.nyks.bridge.MsgConfirmBtcDeposit",
        StandardCosmosMsg::NyksRegisterBtcDepositAddress(_) => "twilightproject.nyks.bridge.MsgRegisterBtcDepositAddress",
        StandardCosmosMsg::NyksRegisterReserveAddress(_) => "twilightproject.nyks.bridge.MsgRegisterReserveAddress",
        StandardCosmosMsg::NyksBootstrapFragment(_) => "twilightproject.nyks.bridge.MsgBootstrapFragment",

        StandardCosmosMsg::NyksWithdrawBtcRequest(_) => "twilightproject.nyks.bridge.MsgWithdrawBtcRequest",
        StandardCosmosMsg::NyksWithdrawTxSigned(_) => "twilightproject.nyks.bridge.MsgWithdrawTxSigned",
        StandardCosmosMsg::NyksWithdrawTxFinal(_) => "twilightproject.nyks.bridge.MsgWithdrawTxFinal",
        StandardCosmosMsg::NyksConfirmBtcWithdraw(_) => "twilightproject.nyks.bridge.MsgConfirmBtcWithdraw",

        StandardCosmosMsg::NyksProposeSweepAddress(_) => "twilightproject.nyks.bridge.MsgProposeSweepAddress",
        StandardCosmosMsg::NyksUnsignedTxSweep(_) => "twilightproject.nyks.bridge.MsgUnsignedTxSweep",
        StandardCosmosMsg::NyksUnsignedTxRefund(_) => "twilightproject.nyks.bridge.MsgUnsignedTxRefund",

        StandardCosmosMsg::NyksSignRefund(_) => "twilightproject.nyks.bridge.MsgSignRefund",
        StandardCosmosMsg::NyksSignSweep(_) => "twilightproject.nyks.bridge.MsgSignSweep",

        StandardCosmosMsg::NyksBroadcastTxRefund(_) => "twilightproject.nyks.bridge.MsgBroadcastTxRefund",
        StandardCosmosMsg::NyksBroadcastTxSweep(_) => "twilightproject.nyks.bridge.MsgBroadcastTxSweep",

        StandardCosmosMsg::NyksSweepProposal(_) => "twilightproject.nyks.bridge.MsgSweepProposal",

        StandardCosmosMsg::NyksZkosMsgTransferTx(_) => "twilightproject.nyks.zkos.MsgTransferTx",
        StandardCosmosMsg::NyksZkosMsgMintBurnTradingBtc(_) => "twilightproject.nyks.zkos.MsgMintBurnTradingBtc",

        // ---- Fallback ----
        StandardCosmosMsg::Unknown { .. } => "<UNKNOWN>",
    }
}


fn ty(t: &str, want: &str) -> bool {
    // Accept both "/pkg.MsgType" and "pkg.MsgType"
    t == want || t.strip_prefix('/') == Some(want)
}
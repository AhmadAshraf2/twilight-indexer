use diesel::prelude::*;
use crate::schema::*;
use anyhow::Result;
use diesel::PgConnection;
use sha2::{Sha256, Digest};

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = transactions)]
pub struct Transactions {
    pub t_address: String,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = funds_moved)]
pub struct FundsMoved {
    pub t_address: String,
    pub amount: i64,
    pub denom: String,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dark_burned_sats)]
pub struct DarkBurnedSats {
    pub t_address: String,
    pub q_address: String,
    pub amount: i64,
    pub block: i64
}


#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dark_minted_sats)]
pub struct DarkMintedSats {
    pub t_address: String,
    pub q_address: String,
    pub amount: i64,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = lit_burned_sats)]
pub struct LitBurnedSats {
    pub t_address: String,
    pub amount: i64,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = lit_minted_sats)]
pub struct LitMintedSats {
    pub t_address: String,
    pub amount: i64,
    pub block: i64
}


#[derive(Queryable, Insertable, AsChangeset, Selectable, Debug, Clone)]
#[diesel(table_name = addr_mappings)]
pub struct AddrMappings {
    pub t_address: String,
    pub q_address: String,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = gas_used_nyks)]
pub struct GasUsedNyks {
    pub t_address: String,
    pub gas_amount: i64,
    pub denom: String,
    pub block: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = qq_tx)]
pub struct QQTx {
    pub tx_hash: String,
    pub tx: String,
    pub block: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = trading_tx)]
pub struct TradingTx {
    pub to_address: String,
    pub from_address: String,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = order_open_tx)]
pub struct OrderOpenTx {
    pub to_address: String,
    pub from_address: String,
    pub block: i64
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = order_close_tx)]
pub struct OrderCloseTx {
    pub to_address: String,
    pub from_address: String,
    pub block: i64
}


fn establish_connection() -> Result<PgConnection> {
    // Usually stored in .env as DATABASE_URL=postgres://user:pass@localhost/stats
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let conn = PgConnection::establish(&database_url)?;
    Ok(conn)
}
/// Add a transaction count (increment existing or insert new)
pub fn insert_transaction_count(twilight_address: &str, block_height: u64) -> Result<()> {
    use crate::schema::transactions::dsl::*;

    let mut conn = establish_connection()?;

    let new_entry = Transactions {
        t_address: twilight_address.to_string(),
        block: block_height as i64,
    };

    diesel::insert_into(transactions)
        .values(&new_entry)
        .on_conflict((t_address, block))
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

/// Add funds moved (increment existing or insert new)
pub fn insert_funds_moved(twilight_address: &str, amount_delta: i64, denom_str: &str, block_height: u64) -> Result<()> {
    use crate::schema::funds_moved::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = FundsMoved {
        t_address: twilight_address.to_string(),
        amount: amount_delta,
        denom: denom_str.to_string(),
        block: block_height as i64,
    };

    diesel::insert_into(funds_moved)
        .values(&new_entry)
        .on_conflict((t_address, denom, block))
        .do_update()
        .set(amount.eq(amount + amount_delta))
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_dark_burned_sats(twilight_address: &str, quis_address: &str, amount_delta: i64, block_height: u64) -> Result<()> {
    use crate::schema::dark_burned_sats::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = DarkBurnedSats {
        t_address: twilight_address.to_string(),
        q_address: quis_address.to_string(),
        amount: amount_delta,
        block: block_height as i64,
    };
    diesel::insert_into(dark_burned_sats)
        .values(&new_entry)
        .on_conflict(t_address)
        .do_update()
        .set(amount.eq(amount + amount_delta))
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_dark_minted_sats(twilight_address: &str, quis_address: &str, amount_delta: i64, block_height: u64) -> Result<()> {
    use crate::schema::dark_minted_sats::dsl::*;

    let mut conn = establish_connection()?;
    let new_entry = DarkMintedSats {
        t_address: twilight_address.to_string(),
        q_address: quis_address.to_string(),
        amount: amount_delta,
        block: block_height as i64,
    };
    diesel::insert_into(dark_minted_sats)
        .values(&new_entry)
        .on_conflict(t_address)
        .do_update()
        .set(amount.eq(amount + amount_delta))
        .execute(&mut conn)?;

    Ok(())
}


pub fn insert_lit_minted_sats(twilight_address: &str, amount_delta: i64, block_height: u64) -> Result<()> {
    use crate::schema::lit_minted_sats::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = LitMintedSats {
        t_address: twilight_address.to_string(),
        amount: amount_delta,
        block: block_height as i64,
    };
    diesel::insert_into(lit_minted_sats)
        .values(&new_entry)
        .on_conflict(t_address)
        .do_update()
        .set(amount.eq(amount + amount_delta))
        .execute(&mut conn)?;

    Ok(())
}


pub fn insert_lit_burned_sats(twilight_address: &str, amount_delta: i64, block_height: u64) -> Result<()> {
    use crate::schema::lit_burned_sats::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = LitBurnedSats {
        t_address: twilight_address.to_string(),
        amount: amount_delta,
        block: block_height as i64,
    };
    diesel::insert_into(lit_burned_sats)
        .values(&new_entry)
        .on_conflict(t_address)
        .do_update()
        .set(amount.eq(amount + amount_delta))
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_addr_mappings(twilight_address: &str, quis_address: &str, block_height: u64) -> Result<()> {
    use crate::schema::addr_mappings::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = AddrMappings {
        t_address: twilight_address.to_string(),
        q_address: quis_address.to_string(),
        block: block_height as i64,
    };

    diesel::insert_into(addr_mappings)
        .values(&new_entry)
        .on_conflict((t_address, q_address)) // composite key / unique pair
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

pub fn get_taddress_for_qaddress(quis_address: &str) -> Result<Option<String>> {
    use crate::schema::addr_mappings::dsl::*;
    let mut conn = establish_connection()?;

    let mapping = addr_mappings
        .filter(q_address.eq(quis_address))
        .select(AddrMappings::as_select())
        .first::<AddrMappings>(&mut conn)
        .optional()?;

    Ok(mapping.map(|m| m.t_address))
}

pub fn get_qaddresses_for_taddress(t_addr: &str) -> Result<Vec<AddrMappings>> {
    use crate::schema::addr_mappings::dsl::*;
    let mut conn = establish_connection()?;

    let results = addr_mappings
        .filter(t_address.eq(t_addr))
        .select(AddrMappings::as_select())
        .load::<AddrMappings>(&mut conn)?;

    Ok(results)
}

pub fn insert_gas_used(addr: &str, gas: i64, denom_str: &str, height: i64) -> Result<()> {
    use crate::schema::gas_used_nyks::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = GasUsedNyks {
        t_address: addr.to_string(),
        gas_amount: gas,
        denom: denom_str.to_string(),
        block: height,
    };
    diesel::insert_into(gas_used_nyks)
        .values(&new_entry)
        .on_conflict((t_address, denom, block))
        .do_update()
        .set(gas_amount.eq(gas_amount + gas))
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_qq_tx(tx_str: &str, block_height: u64) -> Result<()> {
    use crate::schema::qq_tx::dsl::*;
    let mut conn = establish_connection()?;

    // Generate SHA256 hash of the transaction JSON for the primary key
    let mut hasher = Sha256::new();
    hasher.update(tx_str.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(hash_bytes);

    let new_entry = QQTx {
        tx_hash: hash_hex,
        tx: tx_str.to_string(),
        block: block_height as i64,
    };
    diesel::insert_into(qq_tx)
        .values(&new_entry)
        .on_conflict((tx_hash, block))
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_trading_tx(to_addr: &str, from_addr: &str, block_height: u64) -> Result<()> {
    use crate::schema::trading_tx::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = TradingTx {
        to_address: to_addr.to_string(),
        from_address: from_addr.to_string(),
        block: block_height as i64,
    };
    diesel::insert_into(trading_tx)
        .values(&new_entry)
        .on_conflict((to_address, from_address, block))
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_order_open_tx(to_addr: &str, from_addr: &str, block_height: u64) -> Result<()> {
    use crate::schema::order_open_tx::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = OrderOpenTx {
        to_address: to_addr.to_string(),
        from_address: from_addr.to_string(),
        block: block_height as i64,
    };
    diesel::insert_into(order_open_tx)
        .values(&new_entry)
        .on_conflict((to_address, from_address, block))
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

pub fn insert_order_close_tx(to_addr: &str, from_addr: &str, block_height: u64) -> Result<()> {
    use crate::schema::order_close_tx::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = OrderCloseTx {
        to_address: to_addr.to_string(),
        from_address: from_addr.to_string(),
        block: block_height as i64,
    };
    diesel::insert_into(order_close_tx)
        .values(&new_entry)
        .on_conflict((to_address, from_address, block))
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

pub fn run_migrations() -> Result<()> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    // Embed migrations from the migrations/ directory
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    let mut conn = establish_connection()?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

// Query functions for API endpoints

pub fn get_transactions_by_address(addr: &str) -> Result<Vec<Transactions>> {
    use crate::schema::transactions::dsl::*;
    let mut conn = establish_connection()?;

    let results = transactions
        .filter(t_address.eq(addr))
        .select((t_address, block))
        .load::<Transactions>(&mut conn)?;

    Ok(results)
}

pub fn get_funds_moved_by_address(addr: &str) -> Result<Vec<FundsMoved>> {
    use crate::schema::funds_moved::dsl::*;
    let mut conn = establish_connection()?;

    let results = funds_moved
        .filter(t_address.eq(addr))
        .select((t_address, amount, denom, block))
        .load::<FundsMoved>(&mut conn)?;

    Ok(results)
}

pub fn get_dark_burned_sats_by_address(addr: &str) -> Result<Vec<DarkBurnedSats>> {
    use crate::schema::dark_burned_sats::dsl::*;
    let mut conn = establish_connection()?;

    let results = dark_burned_sats
        .filter(t_address.eq(addr))
        .select((t_address, q_address, amount, block))
        .load::<DarkBurnedSats>(&mut conn)?;

    Ok(results)
}

pub fn get_dark_minted_sats_by_address(addr: &str) -> Result<Vec<DarkMintedSats>> {
    use crate::schema::dark_minted_sats::dsl::*;
    let mut conn = establish_connection()?;

    let results = dark_minted_sats
        .filter(t_address.eq(addr))
        .select((t_address, q_address, amount, block))
        .load::<DarkMintedSats>(&mut conn)?;

    Ok(results)
}

pub fn get_lit_minted_sats_by_address(addr: &str) -> Result<Vec<LitMintedSats>> {
    use crate::schema::lit_minted_sats::dsl::*;
    let mut conn = establish_connection()?;

    let results = lit_minted_sats
        .filter(t_address.eq(addr))
        .select((t_address, amount, block))
        .load::<LitMintedSats>(&mut conn)?;

    Ok(results)
}

pub fn get_lit_burned_sats_by_address(addr: &str) -> Result<Vec<LitBurnedSats>> {
    use crate::schema::lit_burned_sats::dsl::*;
    let mut conn = establish_connection()?;

    let results = lit_burned_sats
        .filter(t_address.eq(addr))
        .select((t_address, amount, block))
        .load::<LitBurnedSats>(&mut conn)?;

    Ok(results)
}
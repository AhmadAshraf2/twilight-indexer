use diesel::prelude::*;
use crate::schema::{dark_burned_sats, dark_minted_sats, funds_moved, transaction_count, lit_burned_sats, lit_minted_sats, addr_mappings};
use anyhow::Result;
use diesel::{prelude::*, PgConnection};

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = transaction_count)]
pub struct TransactionCount {
    pub tAddress: String,
    pub count: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = funds_moved)]
pub struct FundsMoved {
    pub tAddress: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dark_burned_sats)]
pub struct DarkBurnedSats {
    pub tAddress: String,
    pub qAddress: String,
    pub amount: i64,
}


#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dark_minted_sats)]
pub struct DarkMintedSats {
    pub tAddress: String,
    pub qAddress: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = lit_burned_sats)]
pub struct LitBurnedSats {
    pub tAddress: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = lit_minted_sats)]
pub struct LitMintedSats {
    pub tAddress: String,
    pub amount: i64,
}


#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = addr_mappings)]
pub struct AddrMappings {
    pub tAddress: String,
    pub qAddress: String,
}


fn establish_connection() -> Result<PgConnection> {
    // Usually stored in .env as DATABASE_URL=postgres://user:pass@localhost/stats
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let conn = PgConnection::establish(&database_url)?;
    Ok(conn)
}
/// Add a transaction count (increment existing or insert new)
pub fn upsert_transaction_count(t_address: &str, delta: i64) -> Result<()> {
    use crate::schema::transaction_count::dsl::*;

    let mut conn = establish_connection()?;

    // Check if exists
    if let Ok(existing) = transaction_count
        .filter(tAddress.eq(t_address))
        .first::<(String, i64)>(&mut conn)
    {
        // Exists: increment count
        diesel::update(transaction_count.filter(tAddress.eq(t_address)))
            .set(count.eq(count + delta))
            .execute(&mut conn)?;
    } else {
        // Insert new
        let new_entry = TransactionCount {
            tAddress: t_address.to_string(),
            count: delta,
        };
        diesel::insert_into(transaction_count)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

/// Add funds moved (increment existing or insert new)
pub fn upsert_funds_moved(address_: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::funds_moved::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = funds_moved
        .filter(tAddress.eq(address_))
        .first::<FundsMoved>(&mut conn)
    {
        diesel::update(funds_moved.filter(tAddress.eq(address_)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = FundsMoved {
            tAddress: address_.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(funds_moved)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_dark_burned_sats(tAddress_: &str, qAddress_: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::dark_burned_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = dark_burned_sats
        .filter(tAddress.eq(tAddress_))
        .first::<DarkBurnedSats>(&mut conn)
    {
        diesel::update(dark_burned_sats.filter(tAddress.eq(tAddress_)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = DarkBurnedSats {
            tAddress: tAddress_.to_string(),
            qAddress: qAddress_.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(dark_burned_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_dark_minted_sats(tAddress_: &str, qAddress_: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::dark_minted_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = dark_minted_sats
        .filter(tAddress.eq(tAddress_))
        .first::<DarkMintedSats>(&mut conn)
    {
        diesel::update(dark_minted_sats.filter(tAddress.eq(tAddress_)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = DarkMintedSats {
            tAddress: tAddress_.to_string(),
            qAddress: qAddress_.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(dark_minted_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}


pub fn upsert_lit_minted_sats(tAddress_: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::lit_minted_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = lit_minted_sats
        .filter(tAddress.eq(tAddress_))
        .first::<LitMintedSats>(&mut conn)
    {
        diesel::update(lit_minted_sats.filter(tAddress.eq(tAddress_)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = LitMintedSats {
            tAddress: tAddress_.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(lit_minted_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}


pub fn upsert_lit_burned_sats(tAddress_: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::lit_burned_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = lit_burned_sats
        .filter(tAddress.eq(tAddress_))
        .first::<LitBurnedSats>(&mut conn)
    {
        diesel::update(lit_burned_sats.filter(tAddress.eq(tAddress_)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = LitBurnedSats {
            tAddress: tAddress_.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(lit_burned_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_addr_mappings(tAddress_: &str, qAddress_: &str) -> Result<()> {
    use crate::schema::addr_mappings::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = AddrMappings {
        tAddress: tAddress_.to_string(),
        qAddress: qAddress_.to_string(),
    };

    diesel::insert_into(addr_mappings)
        .values(&new_entry)
        .on_conflict((tAddress, qAddress)) // composite key / unique pair
        .do_nothing()
        .execute(&mut conn)?;

    Ok(())
}

pub fn get_taddress_for_qaddress(qAddress_: &str) -> Result<Option<String>> {
    use crate::schema::addr_mappings::dsl::*;
    let mut conn = establish_connection()?;

    let mapping = addr_mappings
        .filter(qAddress.eq(qAddress_))
        .first::<AddrMappings>(&mut conn)
        .optional()?;

    Ok(mapping.map(|m| m.tAddress))
}
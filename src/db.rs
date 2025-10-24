use crate::schema::{
    addr_mappings, dark_burned_sats, dark_minted_sats, funds_moved, lit_burned_sats,
    lit_minted_sats, transaction_count,
};
use anyhow::Result;
use diesel::prelude::*;
use diesel::PgConnection;

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = transaction_count)]
pub struct TransactionCount {
    pub t_address: String,
    pub count: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = funds_moved)]
pub struct FundsMoved {
    pub t_address: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dark_burned_sats)]
pub struct DarkBurnedSats {
    pub t_address: String,
    pub q_address: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = dark_minted_sats)]
pub struct DarkMintedSats {
    pub t_address: String,
    pub q_address: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = lit_burned_sats)]
pub struct LitBurnedSats {
    pub t_address: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = lit_minted_sats)]
pub struct LitMintedSats {
    pub t_address: String,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = addr_mappings)]
pub struct AddrMappings {
    pub t_address: String,
    pub q_address: String,
}

fn establish_connection() -> Result<PgConnection> {
    // Usually stored in .env as DATABASE_URL=postgres://user:pass@localhost/stats
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = PgConnection::establish(&database_url)?;
    Ok(conn)
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
/// Add a transaction count (increment existing or insert new)
pub fn upsert_transaction_count(twilight_address: &str, delta: i64) -> Result<()> {
    use crate::schema::transaction_count::dsl::*;

    let mut conn = establish_connection()?;

    // Check if exists
    if let Ok(_existing) = transaction_count
        .filter(t_address.eq(twilight_address))
        .first::<(String, i64)>(&mut conn)
    {
        // Exists: increment count
        diesel::update(transaction_count.filter(t_address.eq(twilight_address)))
            .set(count.eq(count + delta))
            .execute(&mut conn)?;
    } else {
        // Insert new
        let new_entry = TransactionCount {
            t_address: twilight_address.to_string(),
            count: delta,
        };
        diesel::insert_into(transaction_count)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

/// Add funds moved (increment existing or insert new)
pub fn upsert_funds_moved(twilight_address: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::funds_moved::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = funds_moved
        .filter(t_address.eq(twilight_address))
        .first::<FundsMoved>(&mut conn)
    {
        diesel::update(funds_moved.filter(t_address.eq(twilight_address)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = FundsMoved {
            t_address: twilight_address.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(funds_moved)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_dark_burned_sats(
    twilight_address: &str,
    quis_address: &str,
    amount_delta: i64,
) -> Result<()> {
    use crate::schema::dark_burned_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = dark_burned_sats
        .filter(t_address.eq(twilight_address))
        .first::<DarkBurnedSats>(&mut conn)
    {
        diesel::update(dark_burned_sats.filter(t_address.eq(twilight_address)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = DarkBurnedSats {
            t_address: twilight_address.to_string(),
            q_address: quis_address.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(dark_burned_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_dark_minted_sats(
    twilight_address: &str,
    quis_address: &str,
    amount_delta: i64,
) -> Result<()> {
    use crate::schema::dark_minted_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = dark_minted_sats
        .filter(t_address.eq(twilight_address))
        .first::<DarkMintedSats>(&mut conn)
    {
        diesel::update(dark_minted_sats.filter(t_address.eq(twilight_address)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = DarkMintedSats {
            t_address: twilight_address.to_string(),
            q_address: quis_address.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(dark_minted_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_lit_minted_sats(twilight_address: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::lit_minted_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = lit_minted_sats
        .filter(t_address.eq(twilight_address))
        .first::<LitMintedSats>(&mut conn)
    {
        diesel::update(lit_minted_sats.filter(t_address.eq(twilight_address)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = LitMintedSats {
            t_address: twilight_address.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(lit_minted_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_lit_burned_sats(twilight_address: &str, amount_delta: i64) -> Result<()> {
    use crate::schema::lit_burned_sats::dsl::*;

    let mut conn = establish_connection()?;
    if let Ok(_) = lit_burned_sats
        .filter(t_address.eq(twilight_address))
        .first::<LitBurnedSats>(&mut conn)
    {
        diesel::update(lit_burned_sats.filter(t_address.eq(twilight_address)))
            .set(amount.eq(amount + amount_delta))
            .execute(&mut conn)?;
    } else {
        let new_entry = LitBurnedSats {
            t_address: twilight_address.to_string(),
            amount: amount_delta,
        };
        diesel::insert_into(lit_burned_sats)
            .values(&new_entry)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub fn upsert_addr_mappings(twilight_address: &str, quis_address: &str) -> Result<()> {
    use crate::schema::addr_mappings::dsl::*;
    let mut conn = establish_connection()?;

    let new_entry = AddrMappings {
        t_address: twilight_address.to_string(),
        q_address: quis_address.to_string(),
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
        .first::<AddrMappings>(&mut conn)
        .optional()?;

    Ok(mapping.map(|m| m.t_address))
}

diesel::table! {
    transactions (t_address, block) {
        t_address -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    funds_moved (t_address, denom, block) {
        t_address -> Text,
        amount -> BigInt,
        denom -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    dark_burned_sats (t_address) {
        t_address -> Text,
        q_address -> Text,
        amount -> BigInt,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    dark_minted_sats (t_address) {
        t_address -> Text,
        q_address -> Text,
        amount -> BigInt,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    lit_minted_sats (t_address) {
        t_address -> Text,
        amount -> BigInt,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    lit_burned_sats (t_address) {
        t_address -> Text,
        amount -> BigInt,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    addr_mappings (t_address, q_address) {
        t_address -> Text,
        q_address -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    gas_used_nyks (t_address, block) {
        t_address -> Text,
        gas_amount -> BigInt,
        denom -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    qq_tx (tx_hash, block) {
        tx_hash -> Text,
        tx -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    trading_tx (to_address, from_address, block) {
        to_address -> Text,
        from_address -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    order_open_tx (to_address, from_address, block) {
        to_address -> Text,
        from_address -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    order_close_tx (to_address, from_address, block) {
        to_address -> Text,
        from_address -> Text,
        block -> BigInt,
        created_at -> Timestamp,
    }
}
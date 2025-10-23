diesel::table! {
    transaction_count (t_address) {
        t_address -> Text,
        count -> BigInt,
    }
}

diesel::table! {
    funds_moved (t_address) {
        t_address -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    dark_burned_sats (t_address) {
        t_address -> Text,
        q_address -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    dark_minted_sats (t_address) {
        t_address -> Text,
        q_address -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    lit_minted_sats (t_address) {
        t_address -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    lit_burned_sats (t_address) {
        t_address -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    addr_mappings (t_address, q_address) {
        t_address -> Text,
        q_address -> Text,
    }
}   

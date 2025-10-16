diesel::table! {
    transaction_count (tAddress) {
        tAddress -> Text,
        count -> BigInt,
    }
}

diesel::table! {
    funds_moved (tAddress) {
        tAddress -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    dark_burned_sats (tAddress) {
        tAddress -> Text,
        qAddress -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    dark_minted_sats (tAddress) {
        tAddress -> Text,
        qAddress -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    lit_minted_sats (tAddress) {
        tAddress -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    lit_burned_sats (tAddress) {
        tAddress -> Text,
        amount -> BigInt,
    }
}

diesel::table! {
    addr_mappings (tAddress, qAddress) {
        tAddress -> Text,
        qAddress -> Text,
    }
}   

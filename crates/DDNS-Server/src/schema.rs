diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password_hash -> Text,
    }
}

diesel::table! {
    devices (id) {
        id -> Integer,
        user_id -> Integer,
        device_identifier -> Text,
        token_hash -> Text,
        last_seen_ip -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    domains (id) {
        id -> Integer,
        device_id -> Integer,
        hostname -> Text,
        current_ip -> Nullable<Text>,
        is_active -> Bool,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(devices -> users (user_id));
diesel::joinable!(domains -> devices (device_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    devices,
    domains,
);
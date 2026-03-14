diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password_hash -> Text, // 管理介面用
    }
}

diesel::table! {
    devices (id) {
        id -> Integer,
        user_id -> Integer,
        device_identifier -> Text, // uuid 
        token_hash -> Text,        // 設備 API 認證用的 Token
        last_seen_ip -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    domains (id) {
        id -> Integer,
        device_id -> Integer,      // 關聯到設備
        hostname -> Text,          // 例如: "home.example.com"
        current_ip -> Nullable<Text>,
        is_active -> Bool,
        updated_at -> Timestamp,
    }
}

// 定義關聯
diesel::joinable!(devices -> users (user_id));
diesel::joinable!(domains -> devices (device_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    devices,
    domains,
);
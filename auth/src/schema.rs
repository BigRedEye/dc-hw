table! {
    use diesel::sql_types::*;

    confirmations (id) {
        id -> Int4,
        token -> Text,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        user_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    products (id) {
        id -> Int4,
        name -> Text,
        code -> Text,
        category -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    sessions (id) {
        id -> Int4,
        refresh_token -> Text,
        access_token -> Text,
        expires_at -> Timestamp,
        user_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::Access_level;

    users (id) {
        id -> Int4,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        password -> Text,
        permissions -> Access_level,
    }
}

joinable!(confirmations -> users (user_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    confirmations,
    products,
    sessions,
    users,
);

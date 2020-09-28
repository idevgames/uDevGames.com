table! {
    gh_user_records (id) {
        id -> BigInt,
        login -> Text,
        avatar_url -> Text,
        html_url -> Text,
    }
}

table! {
    permissions (id) {
        id -> Integer,
        gh_user_id -> BigInt,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    gh_user_records,
    permissions,
);

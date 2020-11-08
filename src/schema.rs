table! {
    attachments (id) {
        id -> Integer,
        name -> Text,
        published -> Bool,
        mime_type -> Text,
        md5 -> Binary,
    }
}

table! {
    gh_user_records (id) {
        id -> BigInt,
        login -> Text,
        avatar_url -> Text,
        html_url -> Text,
    }
}

table! {
    jam_entries (id) {
        id -> Integer,
        submitter_user_id -> BigInt,
        approval_state -> Integer,
        title -> Text,
        slug -> Text,
        summary -> Text,
        summary_attachment_id -> Integer,
        rich_text_id -> Integer,
    }
}

table! {
    jam_entry_updates (id) {
        id -> Integer,
        jam_entry_id -> Integer,
        title -> Text,
        slug -> Text,
        summary -> Text,
        rich_text_id -> Nullable<Integer>,
        external_content_url -> Nullable<Text>,
        approval_state -> Integer,
    }
}

table! {
    jams (id) {
        id -> Integer,
        title -> Text,
        slug -> Text,
        summary -> Text,
        summary_attachment_id -> Integer,
        rich_text_id -> Integer,
        start_date -> Text,
        end_date -> Text,
        approval_state -> Integer,
    }
}

table! {
    permissions (id) {
        id -> Integer,
        gh_user_id -> BigInt,
        name -> Text,
    }
}

table! {
    rich_text_attachments (id) {
        id -> Integer,
        rich_text_id -> Integer,
        attachment_id -> Integer,
    }
}

table! {
    rich_texts (id) {
        id -> Integer,
        content -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    attachments,
    gh_user_records,
    jam_entries,
    jam_entry_updates,
    jams,
    permissions,
    rich_text_attachments,
    rich_texts,
);

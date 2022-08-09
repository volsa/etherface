table! {
    use diesel::sql_types::*;
    use crate::model::*;

    etherscan_contract (id) {
        id -> Int4,
        address -> Text,
        name -> Text,
        compiler -> Text,
        compiler_version -> Text,
        url -> Text,
        scraped_at -> Nullable<Timestamptz>,
        added_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    github_crawler_metadata (id) {
        id -> Int4,
        last_user_check -> Timestamptz,
        last_repository_check -> Timestamptz,
        last_repository_search -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    github_repository (id) {
        id -> Int4,
        owner_id -> Int4,
        name -> Text,
        html_url -> Text,
        language -> Nullable<Text>,
        stargazers_count -> Int4,
        size -> Int4,
        fork -> Bool,
        created_at -> Timestamptz,
        pushed_at -> Timestamptz,
        updated_at -> Timestamptz,
        scraped_at -> Nullable<Timestamptz>,
        visited_at -> Nullable<Timestamptz>,
        added_at -> Timestamptz,
        solidity_ratio -> Nullable<Float4>,
        is_deleted -> Bool,
        found_by_crawling -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    github_user (id) {
        id -> Int4,
        login -> Text,
        html_url -> Text,
        is_deleted -> Bool,
        added_at -> Timestamptz,
        visited_at -> Nullable<Timestamptz>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    mapping_signature_etherscan (signature_id, contract_id, kind) {
        signature_id -> Int4,
        contract_id -> Int4,
        kind -> Signature_kind,
        added_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    mapping_signature_fourbyte (signature_id, kind) {
        signature_id -> Int4,
        kind -> Signature_kind,
        added_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    mapping_signature_github (signature_id, repository_id, kind) {
        signature_id -> Int4,
        repository_id -> Int4,
        kind -> Signature_kind,
        added_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    mapping_signature_kind (signature_id, kind) {
        signature_id -> Int4,
        kind -> Signature_kind,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::*;

    signature (id) {
        id -> Int4,
        text -> Text,
        hash -> Text,
        is_valid -> Bool,
        added_at -> Timestamptz,
    }
}

joinable!(github_repository -> github_user (owner_id));
joinable!(mapping_signature_etherscan -> etherscan_contract (contract_id));
joinable!(mapping_signature_etherscan -> signature (signature_id));
joinable!(mapping_signature_fourbyte -> signature (signature_id));
joinable!(mapping_signature_github -> github_repository (repository_id));
joinable!(mapping_signature_github -> signature (signature_id));

allow_tables_to_appear_in_same_query!(
    etherscan_contract,
    github_crawler_metadata,
    github_repository,
    github_user,
    mapping_signature_etherscan,
    mapping_signature_fourbyte,
    mapping_signature_github,
    mapping_signature_kind,
    signature,
);

// @generated automatically by Diesel CLI.

diesel::table! {
    asset_coverage_bitmap (id) {
        id -> Nullable<Integer>,
        manifest_id -> Integer,
        bitmap -> Binary,
        version -> Integer,
    }
}

diesel::table! {
    asset_gaps (id) {
        id -> Nullable<Integer>,
        manifest_id -> Integer,
        start_ts -> Text,
        end_ts -> Text,
        state -> Text,
        lease_owner -> Nullable<Text>,
        lease_expires_at -> Nullable<Text>,
    }
}

diesel::table! {
    asset_manifest (id) {
        id -> Nullable<Integer>,
        symbol -> Text,
        provider -> Text,
        asset_class -> Text,
        timeframe_amount -> Integer,
        timeframe_unit -> Text,
        desired_start -> Text,
        desired_end -> Nullable<Text>,
        watermark -> Nullable<Text>,
        last_error -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    engine_kv (k) {
        k -> Nullable<Text>,
        v -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    asset_coverage_bitmap,
    asset_gaps,
    asset_manifest,
    engine_kv,
);

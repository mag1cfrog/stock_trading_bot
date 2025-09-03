//! upsert statements
use diesel::prelude::*;
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection, insert_into};

use crate::schema::{
    asset_class, provider, provider_asset_class as pac, provider_symbol_map as psm,
};

use crate::models::catalog::{
    NewAssetClass, NewProvider, NewProviderAssetClass, NewProviderSymbolMap,
    ProviderSymbolMapUpdate,
};

/// upsert provider
pub fn upsert_provider(
    conn: &mut SqliteConnection,
    code_: &str,
    name_: &str,
) -> anyhow::Result<usize> {
    let row = NewProvider {
        code: code_,
        name: name_,
    };
    let n = insert_into(provider::table)
        .values(&row)
        .on_conflict(provider::code)
        .do_update()
        .set(provider::name.eq(name_))
        .execute(conn)?;
    Ok(n)
}

/// upsert asset classes
pub fn upsert_asset_class(conn: &mut SqliteConnection, code_: &str) -> anyhow::Result<usize> {
    let row = NewAssetClass { code: code_ };
    let n = insert_into(asset_class::table)
        .values(&row)
        .on_conflict(asset_class::code)
        .do_nothing()
        .execute(conn)?;
    Ok(n)
}

/// provider <--> asset_class link
pub fn upsert_provider_asset_class(
    conn: &mut SqliteConnection,
    p: &str,
    a: &str,
) -> anyhow::Result<usize> {
    let row = NewProviderAssetClass {
        provider_code: p,
        asset_class_code: a,
    };
    let n = insert_into(pac::table)
        .values(&row)
        .on_conflict((pac::provider_code, pac::asset_class_code))
        .do_nothing()
        .execute(conn)?;
    Ok(n)
}

/// symbol map upsert
pub fn upsert_symbol_map(
    conn: &mut SqliteConnection,
    p: &str,
    a: &str,
    canon: &str,
    remote: &str,
) -> anyhow::Result<usize> {
    let row = NewProviderSymbolMap {
        provider_code: p,
        asset_class_code: a,
        canonical_symbol: canon,
        remote_symbol: remote,
    };
    let n = insert_into(psm::table)
        .values(&row)
        .on_conflict((
            psm::provider_code,
            psm::asset_class_code,
            psm::canonical_symbol,
        ))
        .do_update()
        .set(psm::remote_symbol.eq(remote))
        .execute(conn)?;
    Ok(n)
}

/// symbol map update
pub fn update_remote_symbol(
    conn: &mut SqliteConnection,
    provider_code: &str,
    asset_class_code: &str,
    canonical: &str,
    new_remote: &str,
) -> diesel::QueryResult<usize> {
    diesel::update(
        psm::table.filter(
            psm::provider_code
                .eq(provider_code)
                .and(psm::asset_class_code.eq(asset_class_code))
                .and(psm::canonical_symbol.eq(canonical)),
        ),
    )
    .set(ProviderSymbolMapUpdate {
        remote_symbol: new_remote,
    })
    .execute(conn)
}

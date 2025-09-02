use diesel::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub struct Current {
    pub providers: BTreeMap<String, String>,
    pub classes: BTreeSet<String>,
    pub pairs: BTreeSet<(String, String)>,
    pub symbols: BTreeSet<(String, String, String, String)>,
}

pub fn read_current(conn: &mut SqliteConnection) -> anyhow::Result<Current> {
    use crate::schema::{asset_class, provider, provider_asset_class, provider_symbol_map};

    let providers = provider::table
        .select((provider::code, provider::name))
        .load::<(String, String)>(conn)?
        .into_iter()
        .collect();

    let classes = asset_class::table
        .select(asset_class::code)
        .load::<String>(conn)?
        .into_iter()
        .collect();

    let pairs = provider_asset_class::table
        .select((
            provider_asset_class::provider_code,
            provider_asset_class::asset_class_code,
        ))
        .load::<(String, String)>(conn)?
        .into_iter()
        .collect();

    let symbols = provider_symbol_map::table
        .select((
            provider_symbol_map::provider_code,
            provider_symbol_map::asset_class_code,
            provider_symbol_map::canonical_symbol,
            provider_symbol_map::remote_symbol,
        ))
        .load::<(String, String, String, String)>(conn)?
        .into_iter()
        .collect();

    Ok(Current {
        providers,
        classes,
        pairs,
        symbols,
    })
}

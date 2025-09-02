use crate::catalog::repo::*;
use crate::catalog::sync::diff::CatalogDiff;
use diesel::prelude::*;

/// Apply the diff inside the current transaction.
/// Note: delete order honors FKs: symbol_map -> pairs -> providers/classes.
pub fn apply_diff(conn: &mut SqliteConnection, diff: &CatalogDiff) -> anyhow::Result<()> {
    // Upserts
    for (p, name_) in &diff.providers_upsert {
        upsert_provider(conn, p, name_)?;
    }
    for a in &diff.classes_upsert {
        upsert_asset_class(conn, a)?;
    }
    for (p, a) in &diff.pairs_upsert {
        upsert_provider_asset_class(conn, p, a)?;
    }
    for (p, a, canon, remote) in &diff.symbols_upsert {
        upsert_symbol_map(conn, p, a, canon, remote)?;
    }

    // Prune (reverse dependency order)
    use crate::schema::{
        asset_class as ac, provider as pr, provider_asset_class as pac, provider_symbol_map as psm,
    };

    for (p, a, canon, remote) in &diff.symbols_delete {
        diesel::delete(
            psm::table.filter(
                psm::provider_code
                    .eq(p)
                    .and(psm::asset_class_code.eq(a))
                    .and(psm::canonical_symbol.eq(canon))
                    .and(psm::remote_symbol.eq(remote)),
            ),
        )
        .execute(conn)?;
    }

    for (p, a) in &diff.pairs_delete {
        diesel::delete(
            pac::table.filter(pac::provider_code.eq(p).and(pac::asset_class_code.eq(a))),
        )
        .execute(conn)?;
    }

    for p in &diff.providers_delete {
        diesel::delete(pr::table.filter(pr::code.eq(p))).execute(conn)?;
    }
    for a in &diff.classes_delete {
        diesel::delete(ac::table.filter(ac::code.eq(a))).execute(conn)?;
    }

    Ok(())
}

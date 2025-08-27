//! Catalog configuration: parsing, normalization, and loading.
//!
//! This module defines a TOML-backed “provider catalog” that describes:
//! - Providers (canonical lowercase codes + human-readable names)
//! - Which asset classes each provider supports
//! - Optional capability metadata (markets, supported timeframes, flags)
//! - Canonical-to-remote symbol mappings per provider/asset class
//!
//! Key behaviors:
//! - Normalization enforces lowercase provider codes and asset class names,
//!   trims whitespace, and de-duplicates entries while preserving order.
//! - Symbol map entries are de-duplicated by the tuple (asset_class, canonical).
//! - Unknown symbol_map.asset_class can be dropped or treated as an error via
//!   [`UnknownSymbolAssetClassPolicy`].
//!
//! Entrypoints:
//! - Parse + normalize from a TOML string: [`load_catalog_str`]
//! - Parse + normalize from a file path: [`load_catalog_path`]
//! - Normalization with explicit policy: [`normalize_catalog_with_policy`]
//! - Back-compat wrapper (drop unknown symbol asset classes): [`normalize_catalog`]
//!
//! The normalized shape is suitable for seeding relational lookup tables such as
//! [`crate::schema::provider`](crate::schema::provider),
//! [`crate::schema::asset_class`](crate::schema::asset_class),
//! [`crate::schema::provider_asset_class`](crate::schema::provider_asset_class),
//! and [`crate::schema::provider_symbol_map`](crate::schema::provider_symbol_map).

use std::{collections::HashSet, mem};

use anyhow::{Context, bail};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::from_str;

/// Top-level catalog mapping provider codes to their configuration.
///
/// Keys are normalized to lowercase during normalization (e.g., "AlPaCa" -> "alpaca").
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    /// Map of provider code -> configuration.
    ///
    /// The code is normalized (trimmed, lowercase) by [`normalize_catalog_with_policy`].
    pub providers: IndexMap<String, ProviderCfg>,
}

/// Provider configuration payload for one provider code.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCfg {
    /// Human-readable provider name (e.g., "Alpaca Markets").
    pub name: String,
    /// Supported asset classes (e.g., ["us_equity","futures"]).
    ///
    /// This list is normalized to unique, lowercase values while preserving order.
    pub asset_classes: Vec<String>,

    /// Optional markets list (provider-specific semantics).
    pub markets: Option<Vec<String>>,
    /// Optional supported timeframes (amount + unit like "Minute", "Hour", "Day", "Week", "Month").
    pub timeframes: Option<Vec<TimeframeCfg>>,
    /// Whether the provider supports extended trading sessions (if applicable).
    pub supports_extended: Option<bool>,
    /// Whether the provider supports historical backfill.
    pub supports_backfill: Option<bool>,
    /// Optional canonical-to-remote symbol mappings for this provider.
    ///
    /// Normalization trims fields, lowercases `asset_class`, de-duplicates by
    /// (asset_class, canonical), and can drop or error on unknown asset classes
    /// depending on [`UnknownSymbolAssetClassPolicy`].
    pub symbol_map: Option<Vec<SymbolMapCfg>>,
}

/// Timeframe capability descriptor (amount × unit).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeframeCfg {
    /// Magnitude component (e.g., 1, 5, 15).
    pub amount: u32,
    /// Unit component (e.g., "Minute", "Hour", "Day", "Week", "Month").
    pub unit: String,
}

/// Canonical-to-remote symbol mapping within a provider and asset class.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SymbolMapCfg {
    /// Asset class name (normalized lowercase during normalization).
    pub asset_class: String,
    /// Canonical internal symbol (e.g., "AAPL", "ES").
    pub canonical: String,
    /// Provider-specific remote symbol (e.g., "AAPL", "ESZ5").
    pub remote: String,
}

/// Summary of changes performed during normalization.
///
/// All counters are additive for the processed catalog.
#[derive(Debug, Default)]
pub struct NormalizationReport {
    /// Number of provider keys that changed when lowercasing/trimming.
    pub providers_renamed: usize,
    /// Count of removed duplicate asset classes after normalization.
    pub asset_classes_deduped: usize,
    /// Count of removed duplicate (asset_class, canonical) symbol pairs.
    pub symbol_map_pairs_deduped: usize,
    /// Count of symbol_map entries dropped due to unknown asset_class (Drop policy).
    pub symbol_map_unknown_asset_class_dropped: usize,
}

/// Policy to handle symbol_map entries whose asset_class is not declared in the provider’s asset_classes list.
#[derive(Copy, Clone, Debug)]
pub enum UnknownSymbolAssetClassPolicy {
    /// Drop symbol_map entries whose asset_class isn't declared in provider.asset_classes
    Drop,
    /// Treat as an error
    Error,
}

/// Normalize a catalog in-place with an explicit policy for unknown symbol asset classes.
///
/// What normalization does:
/// - Lowercase + trim provider keys; reject duplicates after normalization
/// - Deduplicate and lowercase `asset_classes`, preserving the first occurrence order
/// - For `symbol_map`:
///   - Trim all fields; lowercase `asset_class`
///   - Enforce the asset_class is declared in the provider (`Drop` vs `Error`)
///   - Deduplicate by `(asset_class, canonical)` preserving the first occurrence
///
/// Returns:
/// - A [`NormalizationReport`] detailing the changes made
///
/// Errors:
/// - Empty or duplicate provider codes after normalization
/// - Empty asset class names or empty symbol fields after trimming
/// - Unknown symbol_map.asset_class when policy is [`UnknownSymbolAssetClassPolicy::Error`]
pub fn normalize_catalog_with_policy(
    cat: &mut Catalog,
    policy: UnknownSymbolAssetClassPolicy,
) -> anyhow::Result<NormalizationReport> {
    let mut report = NormalizationReport::default();

    // Rebuild providers map
    let mut rebuilt: IndexMap<String, ProviderCfg> = IndexMap::new();
    let old = std::mem::take(&mut cat.providers);

    for (raw_code, mut cfg) in old {
        let code = raw_code.trim().to_lowercase();
        if code.is_empty() {
            bail!("provider code cannot be empty after trimming");
        }
        if code != raw_code {
            report.providers_renamed += 1;
        }
        if rebuilt.contains_key(&code) {
            bail!("duplicate provider code after normalization: {code}");
        }

        // --- normalize asset_classes (dedupe, preserve order)
        let before_len = cfg.asset_classes.len();
        let mut seen_ac = HashSet::new();
        let mut norm_classes = Vec::with_capacity(before_len);

        for mut ac in mem::take(&mut cfg.asset_classes) {
            ac = ac.trim().to_lowercase();
            if ac.is_empty() {
                bail!("asset class cannot be empty after trimming");
            }
            if seen_ac.insert(ac.clone()) {
                norm_classes.push(ac);
            }
        }
        report.asset_classes_deduped += before_len.saturating_sub(norm_classes.len());

        // For quick membership checks when normalizing symbol map:
        let declared_classes: HashSet<&str> = norm_classes.iter().map(|s| s.as_str()).collect();

        // --- normalize symbol_map (dedupe by (asset_class, canonical))
        let mut norm_symbol_map: Option<Vec<SymbolMapCfg>> = None;
        if let Some(list) = mem::take(&mut cfg.symbol_map) {
            let before_len = list.len();
            let mut out = Vec::with_capacity(before_len);
            let mut seen_pair = HashSet::new();

            for mut sm in list {
                sm.asset_class = sm.asset_class.trim().to_lowercase();
                if sm.asset_class.is_empty() {
                    bail!("symbol_map.asset_class cannot be empty after trimming");
                }
                sm.canonical = sm.canonical.trim().to_string();
                if sm.canonical.is_empty() {
                    bail!("symbol_map.canonical cannot be empty after trimming");
                }
                sm.remote = sm.remote.trim().to_string();
                if sm.remote.is_empty() {
                    bail!("symbol_map.remote cannot be empty after trimming");
                }

                // enforce membership
                if !declared_classes.contains(sm.asset_class.as_str()) {
                    match policy {
                        UnknownSymbolAssetClassPolicy::Drop => {
                            report.symbol_map_unknown_asset_class_dropped += 1;
                            continue;
                        }
                        UnknownSymbolAssetClassPolicy::Error => {
                            bail!(
                                "symbol_map asset_class '{}' is not declared in provider.asset_classes",
                                sm.asset_class
                            );
                        }
                    }
                }

                let key = (sm.asset_class.clone(), sm.canonical.clone());
                if seen_pair.insert(key) {
                    out.push(sm);
                } else {
                    report.symbol_map_pairs_deduped += 1;
                }
            }

            if !out.is_empty() {
                norm_symbol_map = Some(out);
            }
        }

        let mut cfg = cfg;
        cfg.asset_classes = norm_classes;
        cfg.symbol_map = norm_symbol_map;
        rebuilt.insert(code, cfg);
    }

    cat.providers = rebuilt;
    Ok(report)
}

/// This calls [`normalize_catalog_with_policy`] using [`UnknownSymbolAssetClassPolicy::Drop`]
/// so that unknown symbol_map asset classes are silently dropped.
pub fn normalize_catalog(cat: &mut Catalog) -> anyhow::Result<NormalizationReport> {
    normalize_catalog_with_policy(cat, UnknownSymbolAssetClassPolicy::Drop)
}

/// Parse and normalize a catalog from a TOML string.
///
/// Steps:
/// - Deserialize TOML into [`Catalog`]
/// - Normalize via [`normalize_catalog`]
///
/// Errors:
/// - TOML parse failures
/// - Normalization errors (see [`normalize_catalog_with_policy`])
pub fn load_catalog_str(toml_str: &str) -> anyhow::Result<Catalog> {
    let mut cat: Catalog = from_str(toml_str).context("failed to parse catalog TOML")?;
    let _report = normalize_catalog(&mut cat).context("normalize_catalog failed")?;
    // log::info!("{:?}", _report);
    Ok(cat)
}

/// Read a catalog TOML file from disk, parse, and normalize it.
///
/// See [`load_catalog_str`] for details on parsing and normalization.
pub fn load_catalog_path(path: impl AsRef<std::path::Path>) -> anyhow::Result<Catalog> {
    let text = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("read catalog file {}", path.as_ref().display()))?;
    load_catalog_str(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn mk() -> Catalog {
        use crate::catalog::config::{Catalog, ProviderCfg, SymbolMapCfg, TimeframeCfg};
        let mut providers: IndexMap<String, ProviderCfg> = IndexMap::new();
        providers.insert(
            "AlPaCa ".into(),
            ProviderCfg {
                name: "Alpaca".into(),
                asset_classes: vec!["US_Equity".into(), "us_equity".into(), "Futures".into()],
                markets: None,
                timeframes: Some(vec![TimeframeCfg {
                    amount: 1,
                    unit: "Minute".into(),
                }]),
                supports_extended: Some(true),
                supports_backfill: Some(true),
                symbol_map: Some(vec![
                    SymbolMapCfg {
                        asset_class: "US_Equity".into(),
                        canonical: "AAPL".into(),
                        remote: " AAPL ".into(),
                    },
                    SymbolMapCfg {
                        asset_class: "us_equity".into(),
                        canonical: "AAPL".into(),
                        remote: "AAPL".into(),
                    }, // dup pair -> dropped
                    SymbolMapCfg {
                        asset_class: "FUTURES".into(),
                        canonical: "ES".into(),
                        remote: "ESZ5".into(),
                    },
                ]),
            },
        );
        Catalog { providers }
    }

    #[test]
    fn normalizes_codes_and_dedupes() {
        let mut cat = mk();
        normalize_catalog(&mut cat).unwrap();

        let (only_code, cfg) = cat.providers.first().unwrap();
        assert_eq!(only_code, "alpaca"); // lowercased key
        assert_eq!(cfg.asset_classes, vec!["us_equity", "futures"]); // dedup + lowercase

        let sm = cfg.symbol_map.as_ref().unwrap();
        assert_eq!(sm.len(), 2); // duplicate (asset_class,canonical) removed
        assert!(
            sm.iter().any(|x| x.asset_class == "us_equity"
                && x.canonical == "AAPL"
                && x.remote == "AAPL")
        );
        assert!(
            sm.iter()
                .any(|x| x.asset_class == "futures" && x.canonical == "ES" && x.remote == "ESZ5")
        );
    }

    #[test]
    fn duplicate_provider_collision_errors() {
        let mut cat = mk();
        // insert another provider that normalizes to same key
        cat.providers.insert(
            "alpaca".into(),
            cat.providers.get_index(0).unwrap().1.clone(),
        );
        let err = normalize_catalog(&mut cat).unwrap_err();
        assert!(err.to_string().contains("duplicate provider code"));
    }

    #[test]
    fn snapshot_normalized_catalog() {
        let toml_str = r#"
            [providers.AlPaCa]
            name = "Alpaca"
            asset_classes = ["US_Equity", "us_equity", "Futures"]
            [[providers.AlPaCa.symbol_map]]
            asset_class = "US_Equity"
            canonical = "AAPL"
            remote = " AAPL "
            [[providers.AlPaCa.symbol_map]]
            asset_class = "us_equity"
            canonical = "AAPL"
            remote = "AAPL"
        "#;

        let mut cat = toml::from_str::<Catalog>(toml_str).unwrap();
        let _ = normalize_catalog(&mut cat).unwrap();

        // insta compares against a stored snapshot you review+accept.
        insta::assert_json_snapshot!("normalized_catalog", &cat);
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn providers_lowercased_and_unique(
            // up to 5 random provider names with mixed case + whitespace
            names in proptest::collection::vec(".{1,8}", 1..5),
        ) {
            let mut cat = Catalog { providers: IndexMap::new() };
            for (i, n) in names.iter().enumerate() {
                let key = if i % 2 == 0 { n.to_uppercase() } else { format!("  {n} ") };
                cat.providers.insert(key, ProviderCfg {
                    name: "X".into(),
                    asset_classes: vec!["US_Equity".into(), "us_equity".into()],
                    markets: None, timeframes: None,
                    supports_extended: None, supports_backfill: None, symbol_map: None,
                });
            }

            let res = normalize_catalog(&mut cat);
            if res.is_ok() {
                // 1) all keys lowercase
                assert!(cat.providers.keys().all(|k| k.chars().all(|c| !c.is_uppercase())));
                // 2) no duplicates (IndexMap guarantees unique keys)
            } else {
                // a normalization collision is allowed; property still holds for success cases
            }
        }
    }

    #[test]
    fn symbol_map_unknown_asset_class_drops_by_default() {
        let toml_str = r#"
            [providers.alpaca]
            name = "Alpaca"
            asset_classes = ["us_equity"]
            [[providers.alpaca.symbol_map]]
            asset_class = "futures"     # not declared above
            canonical  = "ES"
            remote     = "ESZ5"
        "#;

        let mut cat = toml::from_str::<Catalog>(toml_str).unwrap();
        let rep = normalize_catalog(&mut cat).unwrap();
        assert_eq!(rep.symbol_map_unknown_asset_class_dropped, 1);
        let sm = cat.providers["alpaca"].symbol_map.as_deref().unwrap_or(&[]);
        assert!(sm.is_empty());
    }

    #[test]
    fn symbol_map_unknown_asset_class_as_error() {
        let toml_str = r#"
            [providers.alpaca]
            name = "Alpaca"
            asset_classes = ["us_equity"]
            [[providers.alpaca.symbol_map]]
            asset_class = "futures"
            canonical  = "ES"
            remote     = "ESZ5"
        "#;
        let mut cat = toml::from_str::<Catalog>(toml_str).unwrap();
        let err = normalize_catalog_with_policy(&mut cat, UnknownSymbolAssetClassPolicy::Error)
            .unwrap_err();
        assert!(err.to_string().contains("not declared"));
    }
}

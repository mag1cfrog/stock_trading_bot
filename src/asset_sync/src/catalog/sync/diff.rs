use std::collections::{BTreeMap, BTreeSet};

/// What needs to change to make DB == TOML.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogDiff {
    // Upserts
    pub providers_upsert: BTreeMap<String, String>, // code -> name
    pub classes_upsert: BTreeSet<String>,           // codes
    pub pairs_upsert: BTreeSet<(String, String)>,   // (provider, class)
    pub symbols_upsert: BTreeSet<(String, String, String, String)>, // (p,a,canon,remote)

    // Prunes
    pub providers_delete: BTreeSet<String>,
    pub classes_delete: BTreeSet<String>,
    pub pairs_delete: BTreeSet<(String, String)>,
    pub symbols_delete: BTreeSet<(String, String, String, String)>,
}

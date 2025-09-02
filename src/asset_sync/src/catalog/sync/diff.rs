use std::collections::{BTreeMap, BTreeSet};

use crate::catalog::sync::{read::Current, want::Wanted};

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

pub fn make_diff(w: &Wanted, c: &Current, prune: bool) -> CatalogDiff {
    let mut d = CatalogDiff::default();

    // upserts
    d.providers_upsert = w.providers.clone();
    d.classes_upsert = w.classes.clone();
    d.pairs_upsert = w.pairs.clone();
    d.symbols_upsert = w.symbols.clone();

    // prunes (only when requested)
    if prune {
        for k in &c.providers {
            if !w.providers.contains_key(k.0) {
                d.providers_delete.insert(k.0.clone());
            }
        }
        for k in &c.classes {
            if !w.classes.contains(k) {
                d.classes_delete.insert(k.clone());
            }
        }
        for k in &c.pairs {
            if !w.pairs.contains(k) {
                d.pairs_delete.insert(k.clone());
            }
        }
        for k in &c.symbols {
            if !w.symbols.contains(k) {
                d.symbols_delete.insert(k.clone());
            }
        }
    }

    d
}

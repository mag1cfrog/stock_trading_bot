use crate::catalog::config::Catalog;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Wanted {
    pub providers: BTreeMap<String, String>,
    pub classes: BTreeSet<String>,
    pub pairs: BTreeSet<(String, String)>,
    pub symbols: BTreeSet<(String, String, String, String)>,
}

pub fn wanted_from_catalog(cat: &Catalog) -> Wanted {
    let mut providers = BTreeMap::new();
    let mut classes = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    let mut symbols = BTreeSet::new();

    for (p, pcfg) in &cat.providers {
        providers.insert(p.clone(), pcfg.name.clone());
        for a in &pcfg.asset_classes {
            classes.insert(a.clone());
            pairs.insert((p.clone(), a.clone()));
        }
        if let Some(sm) = &pcfg.symbol_map {
            for s in sm {
                symbols.insert((
                    p.clone(),
                    s.asset_class.clone(),
                    s.canonical.clone(),
                    s.remote.clone(),
                ));
            }
        }
    }

    Wanted {
        providers,
        classes,
        pairs,
        symbols,
    }
}

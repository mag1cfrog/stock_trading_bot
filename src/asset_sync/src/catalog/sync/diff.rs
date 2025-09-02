use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

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

impl CatalogDiff {
    /// True if there is nothing to upsert or delete.
    pub fn is_noop(&self) -> bool {
        self.providers_upsert.is_empty()
            && self.classes_upsert.is_empty()
            && self.pairs_upsert.is_empty()
            && self.symbols_upsert.is_empty()
            && self.providers_delete.is_empty()
            && self.classes_delete.is_empty()
            && self.pairs_delete.is_empty()
            && self.symbols_delete.is_empty()
    }
}

impl fmt::Display for CatalogDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // helper: section header with underline
        let mut wrote_any = false;
        let mut section = |title: &str,
                           body: &mut dyn FnMut(&mut fmt::Formatter<'_>) -> fmt::Result|
         -> fmt::Result {
            if wrote_any {
                writeln!(f)?;
            }
            writeln!(f, "{title}")?;
            for _ in 0..title.len() {
                write!(f, "-")?;
            }
            writeln!(f)?;
            body(f)?;
            wrote_any = true;
            Ok(())
        };

        // UPSERTS
        if !self.providers_upsert.is_empty() {
            section("Providers (UPSERT)", &mut |f| {
                for (code, name) in &self.providers_upsert {
                    writeln!(f, "+ {code}  \"{name}\"")?;
                }
                Ok(())
            })?;
        }
        if !self.classes_upsert.is_empty() {
            section("Asset Classes (UPSERT)", &mut |f| {
                for code in &self.classes_upsert {
                    writeln!(f, "+ {code}")?;
                }
                Ok(())
            })?;
        }
        if !self.pairs_upsert.is_empty() {
            section("Provider - Class (UPSERT)", &mut |f| {
                for (prov, class) in &self.pairs_upsert {
                    writeln!(f, "+ {prov} - {class}")?;
                }
                Ok(())
            })?;
        }
        if !self.symbols_upsert.is_empty() {
            section("Symbol Map (UPSERT)", &mut |f| {
                for (prov, class, canon, remote) in &self.symbols_upsert {
                    if canon == remote {
                        writeln!(f, "+ {prov}/{class}  {canon}")?;
                    } else {
                        writeln!(f, "+ {prov}/{class}  {canon} → {remote}")?;
                    }
                }
                Ok(())
            })?;
        }

        // DELETES
        if !self.providers_delete.is_empty() {
            section("Providers (DELETE)", &mut |f| {
                for code in &self.providers_delete {
                    writeln!(f, "- {code}")?;
                }
                Ok(())
            })?;
        }
        if !self.classes_delete.is_empty() {
            section("Asset Classes (DELETE)", &mut |f| {
                for code in &self.classes_delete {
                    writeln!(f, "- {code}")?;
                }
                Ok(())
            })?;
        }
        if !self.pairs_delete.is_empty() {
            section("Provider - Class (DELETE)", &mut |f| {
                for (prov, class) in &self.pairs_delete {
                    writeln!(f, "- {prov} - {class}")?;
                }
                Ok(())
            })?;
        }
        if !self.symbols_delete.is_empty() {
            section("Symbol Map (DELETE)", &mut |f| {
                for (prov, class, canon, remote) in &self.symbols_delete {
                    writeln!(f, "- {prov}/{class}  {canon} ({remote})")?;
                }
                Ok(())
            })?;
        }

        if !wrote_any {
            write!(f, "No changes")
        } else {
            Ok(())
        }
    }
}

pub fn make_diff(w: &Wanted, c: &Current, prune: bool) -> CatalogDiff {
    let mut d = CatalogDiff {
        providers_upsert: w.providers.clone(),
        classes_upsert: w.classes.clone(),
        pairs_upsert: w.pairs.clone(),
        symbols_upsert: w.symbols.clone(),
        ..Default::default()
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn wanted_min() -> Wanted {
        let providers = BTreeMap::from([("alpaca".to_string(), "Alpaca".to_string())]);
        let classes = BTreeSet::from(["us_equity".to_string()]);
        let pairs = BTreeSet::from([("alpaca".to_string(), "us_equity".to_string())]);
        let symbols = BTreeSet::from([(
            "alpaca".to_string(),
            "us_equity".to_string(),
            "AAPL".to_string(),
            "AAPL".to_string(), // same -> prints without arrow
        )]);

        Wanted {
            providers,
            classes,
            pairs,
            symbols,
        }
    }

    fn current_empty() -> Current {
        Current {
            providers: BTreeMap::new(),
            classes: BTreeSet::new(),
            pairs: BTreeSet::new(),
            symbols: BTreeSet::new(),
        }
    }

    #[test]
    fn display_no_changes() {
        // make_diff with both sides empty -> "No changes"
        let w = Wanted::default();
        let c = Current {
            providers: BTreeMap::new(),
            classes: BTreeSet::new(),
            pairs: BTreeSet::new(),
            symbols: BTreeSet::new(),
        };
        let d = make_diff(&w, &c, false);
        assert_eq!(d.to_string(), "No changes");
    }

    #[test]
    fn display_upserts_expected() {
        // Upserts only; prune=false so no DELETE sections.
        let w = wanted_min();
        let c = current_empty();
        let d = make_diff(&w, &c, false);
        let got = d.to_string();

        // Expected layout (headers underlined to the exact length).
        let expected = "\
Providers (UPSERT)
------------------
+ alpaca  \"Alpaca\"

Asset Classes (UPSERT)
----------------------
+ us_equity

Provider - Class (UPSERT)
-------------------------
+ alpaca - us_equity

Symbol Map (UPSERT)
-------------------
+ alpaca/us_equity  AAPL
";
        assert_eq!(got, expected, "pretty diff did not match");
    }

    // Run this manually to preview how diffs print in your console:
    // cargo test -p asset_sync -- catalog::sync::diff::tests::print_example -- --nocapture --ignored
    #[test]
    #[ignore]
    fn print_example() {
        let w = wanted_min();

        // Current has one extra provider/class/symbol so we exercise DELETEs with prune=true.
        let c = Current {
            providers: BTreeMap::from([
                ("alpaca".to_string(), "Alpaca".to_string()),
                ("intrinio".to_string(), "Intrinio".to_string()),
            ]),
            classes: BTreeSet::from(["us_equity".to_string(), "futures".to_string()]),
            pairs: BTreeSet::from([
                ("alpaca".to_string(), "us_equity".to_string()),
                ("intrinio".to_string(), "futures".to_string()),
            ]),
            symbols: BTreeSet::from([
                (
                    "alpaca".to_string(),
                    "us_equity".to_string(),
                    "AAPL".to_string(),
                    "AAPL".to_string(),
                ),
                (
                    "intrinio".to_string(),
                    "futures".to_string(),
                    "ES".to_string(),
                    "ESZ5".to_string(),
                ),
            ]),
        };

        let d = make_diff(&w, &c, true);
        println!("{d}");
    }
}

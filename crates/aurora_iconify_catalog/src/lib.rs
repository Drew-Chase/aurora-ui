//! Icon names for every [Iconify](https://iconify.design) icon set, and the
//! naming rules shared by the crates that turn them into Rust code.
//!
//! The name lists are a snapshot written by `cargo run -p xtask -- iconify-catalog`.
//! An icon's ID is its position in its set's sorted name list, so IDs only mean
//! something together with the catalog that produced them ([`CATALOG_HASH`]).
//!
//! This crate has no dependencies and does no I/O. The `aurora_iconify` build
//! script and the `icon_sets!` macro both compile it, so they always agree.

mod data;
mod generate;
pub mod index;
mod naming;

use std::str::Lines;

pub use data::{CATALOG_HASH, SETS};
pub use generate::{facade_set_source, sets_module_source};
pub use naming::{
    Fnv1a, accessor_type_name, feature_env_var, is_rust_keyword, is_valid_name, method_names,
    sanitize_ident, to_pascal_case, variant_name,
};

/// One icon set in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetData {
    /// The Iconify prefix, such as `material-symbols`. It is also the Cargo feature name.
    pub prefix: &'static str,
    /// Every icon name in the set, sorted, one per line.
    pub names_text: &'static str,
}

impl SetData {
    /// The icon names, in sorted order.
    pub fn names(&self) -> Lines<'static> {
        self.names_text.lines()
    }
}

/// The set with this Iconify prefix, if the catalog has it.
pub fn set(prefix: &str) -> Option<&'static SetData> {
    SETS.iter().find(|set| set.prefix == prefix)
}

/// Fingerprints a list of sets (each prefix and its sorted names). The result
/// changes whenever any prefix or name does.
pub fn catalog_hash<'a, S, N>(sets: S) -> u64
where
    S: IntoIterator<Item = (&'a str, N)>,
    N: IntoIterator<Item = &'a str>,
{
    let mut hash = Fnv1a::new();
    for (prefix, names) in sets {
        hash.update(prefix.as_bytes());
        hash.update(b"\0");
        for name in names {
            hash.update(name.as_bytes());
            hash.update(b"\n");
        }
        hash.update(b"\0");
    }
    hash.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_sorted_unique_and_matches_its_fingerprint() {
        let mut previous_prefix = "";
        for set in SETS {
            assert!(
                set.prefix > previous_prefix,
                "sets are sorted: {}",
                set.prefix
            );
            previous_prefix = set.prefix;
            let mut previous_name = "";
            for name in set.names() {
                assert!(is_valid_name(name), "{}: invalid name {name}", set.prefix);
                assert!(
                    name > previous_name,
                    "{}: names are sorted: {name}",
                    set.prefix
                );
                previous_name = name;
            }
        }
        assert_eq!(
            catalog_hash(SETS.iter().map(|set| (set.prefix, set.names()))),
            CATALOG_HASH
        );
    }

    #[test]
    fn set_lookup_finds_by_prefix() {
        if let Some(first) = SETS.first() {
            assert_eq!(set(first.prefix), Some(first));
        }
        assert_eq!(set("no-such-set"), None);
    }
}

//! Naming rules shared by the code generator, the macro and xtask.

use std::collections::HashSet;

/// Returns `true` for names that are safe in URLs, file paths and generated
/// code: lowercase ASCII letters and digits, `-` and `_`, starting with a
/// letter or digit.
pub fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Converts an icon or set name to a Rust identifier. Characters that are not
/// alphanumeric become `_`, a leading digit gets a `_` in front, and a keyword
/// gets an `_icon` suffix.
pub fn sanitize_ident(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let s = if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{s}")
    } else {
        s
    };
    if is_rust_keyword(&s) {
        format!("{s}_icon")
    } else {
        s
    }
}

/// Whether `s` is a reserved word in Rust (edition 2024).
pub fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async"
            | "await"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "union"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "yield"
            | "abstract"
            | "become"
            | "final"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
    )
}

/// Converts a set name such as `material-symbols` to `MaterialSymbols`.
pub fn to_pascal_case(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

/// The name of a set's `SetId` variant, such as `MaterialSymbols`. A name that
/// starts with a digit gets a `_` in front.
pub fn variant_name(prefix: &str) -> String {
    let name = to_pascal_case(prefix);
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{name}")
    } else {
        name
    }
}

/// The name of a set's accessor type, such as `MaterialSymbolsIcons`.
pub fn accessor_type_name(prefix: &str) -> String {
    format!("{}Icons", variant_name(prefix))
}

/// The environment variable Cargo sets for an enabled feature, such as
/// `CARGO_FEATURE_MATERIAL_SYMBOLS` for `material-symbols`.
pub fn feature_env_var(feature: &str) -> String {
    format!(
        "CARGO_FEATURE_{}",
        feature.replace('-', "_").to_ascii_uppercase()
    )
}

/// Pairs each name with its method identifier, in input order.
///
/// Names that sanitize to the same identifier share it. The first keeps it and
/// the others get no method, but stay reachable through `by_name`. `by_name` is
/// reserved for that lookup. Each result carries the name's position, which is
/// the icon's ID.
pub fn method_names<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<(usize, &'a str, String)> {
    let mut taken = HashSet::from(["by_name".to_string()]);
    names
        .into_iter()
        .enumerate()
        .filter_map(|(id, name)| {
            let ident = sanitize_ident(name);
            taken.insert(ident.clone()).then_some((id, name, ident))
        })
        .collect()
}

/// 64-bit FNV-1a, which fingerprints data without a dependency.
#[derive(Debug, Clone, Copy)]
pub struct Fnv1a(u64);

impl Fnv1a {
    /// Starts a new hash.
    pub fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    /// Mixes `bytes` into the hash.
    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    /// The hash of everything mixed in so far.
    pub fn finish(self) -> u64 {
        self.0
    }
}

impl Default for Fnv1a {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_names_like_the_readme_says() {
        assert_eq!(sanitize_ident("calendar-2-fill"), "calendar_2_fill");
        assert_eq!(sanitize_ident("arrow-left"), "arrow_left");
        assert_eq!(sanitize_ident("3d-rotate"), "_3d_rotate");
        assert_eq!(sanitize_ident("box"), "box_icon");
        assert_eq!(sanitize_ident("type"), "type_icon");
    }

    #[test]
    fn names_the_set_types_and_features() {
        assert_eq!(variant_name("mage"), "Mage");
        assert_eq!(variant_name("material-symbols"), "MaterialSymbols");
        assert_eq!(variant_name("3d-set"), "_3dSet");
        assert_eq!(
            accessor_type_name("material-symbols"),
            "MaterialSymbolsIcons"
        );
        assert_eq!(
            feature_env_var("material-symbols"),
            "CARGO_FEATURE_MATERIAL_SYMBOLS"
        );
    }

    #[test]
    fn rejects_names_that_could_escape_urls_or_paths() {
        assert!(is_valid_name("material-symbols"));
        assert!(is_valid_name("3d-rotate"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("../etc"));
        assert!(!is_valid_name("a/b"));
        assert!(!is_valid_name("a b"));
        assert!(!is_valid_name("Mage"));
        assert!(!is_valid_name("a?icons=b"));
        assert!(!is_valid_name("-a"));
    }

    #[test]
    fn method_names_keep_the_first_name_for_each_identifier() {
        let names = ["a-b", "a_b", "by-name", "c"];
        assert_eq!(
            method_names(names),
            [(0, "a-b", "a_b".to_string()), (3, "c", "c".to_string())]
        );
    }

    #[test]
    fn fnv1a_matches_the_reference_value() {
        let mut hash = Fnv1a::new();
        hash.update(b"a");
        assert_eq!(hash.finish(), 0xaf63_dc4c_8601_ec8c);
    }
}

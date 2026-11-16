//! Icon/set name validation and conversion to Rust identifiers.

/// Returns `true` for names that are safe to interpolate into URLs, file
/// paths and generated code: lowercase alphanumerics, `-` and `_`, starting
/// with an alphanumeric.
pub fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Converts an icon or set name to a valid Rust identifier.
pub fn sanitize_ident(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    // Ensure it doesn't start with a digit
    let s = if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{s}")
    } else {
        s
    };
    // Escape Rust reserved keywords
    if is_rust_keyword(&s) {
        format!("{s}_icon")
    } else {
        s
    }
}

fn is_rust_keyword(s: &str) -> bool {
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
    fn pascal_cases_set_names() {
        assert_eq!(to_pascal_case("mage"), "Mage");
        assert_eq!(to_pascal_case("material-symbols"), "MaterialSymbols");
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
}

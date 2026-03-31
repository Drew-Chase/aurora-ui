/// Convert a locale tag to a valid Rust module name (snake_case).
/// `"en-us"` → `"en_us"`, `"zh-Hans"` → `"zh_hans"`, `"ar-SA"` → `"ar_sa"`
pub fn locale_to_mod_name(tag: &str) -> String {
    let s: String = tag
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    escape_if_keyword(prefix_if_digit(s))
}

/// Convert a TOML key segment to a valid Rust field name.
/// `"window"` → `"window"`, `"dark-mode"` → `"dark_mode"`
pub fn sanitize_field_name(key: &str) -> String {
    let s: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    escape_if_keyword(prefix_if_digit(s))
}

/// Convert a TOML key segment to PascalCase for struct naming.
/// `"general"` → `"General"`, `"dark_mode"` → `"DarkMode"`
pub fn to_pascal_case(key: &str) -> String {
    key.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect()
}

fn prefix_if_digit(s: String) -> String {
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{s}")
    } else {
        s
    }
}

fn escape_if_keyword(s: String) -> String {
    if is_rust_keyword(&s) {
        format!("{s}_")
    } else {
        s
    }
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
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
            | "type"
            | "union"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "yield"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
    )
}

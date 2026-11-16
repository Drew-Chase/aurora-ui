//! Finds which icons a crate refers to by scanning its source text.
//!
//! A proc macro cannot see where its output is used, so `icon_sets!` reads the
//! invoking crate's `.rs` files and collects every word that could be an icon
//! reference. The result is a deliberate over-approximation: a word that merely
//! happens to equal an icon name embeds one extra icon, but a real reference is
//! never missed for ordinary code.
//!
//! Words are also graded. One written like an icon access (a `.name()` call
//! or a quoted `"name"`) is a *reference*; anything else is only a *mention*.
//! Both get embedded, but only a reference is worth failing the build over
//! when its icon cannot be downloaded.

use crate::names::sanitize_ident;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Every word found in a crate's sources that could name an icon, mapped to
/// whether it was seen written like an icon access.
#[derive(Default)]
pub struct Words(HashMap<String, bool>);

impl Words {
    /// Scans all `.rs` files below `root`, skipping build output, hidden
    /// directories and nested packages.
    pub fn scan_dir(root: &Path) -> Self {
        let mut words = Self::default();
        words.visit_dir(root, true);
        words
    }

    /// Whether the sources contain `icon_name` in any form: its original name
    /// (`calendar-2-fill`) or the generated method name (`calendar_2_fill`).
    pub fn mentions(&self, icon_name: &str) -> bool {
        self.0.contains_key(icon_name) || self.0.contains_key(sanitize_ident(icon_name).as_str())
    }

    /// Whether the sources contain `icon_name` written like an icon access.
    pub fn references(&self, icon_name: &str) -> bool {
        let is_reference = |word: &str| self.0.get(word).copied().unwrap_or(false);
        is_reference(icon_name) || is_reference(&sanitize_ident(icon_name))
    }

    fn visit_dir(&mut self, dir: &Path, is_root: bool) {
        // A nested directory with its own manifest is a different package and
        // gets its own expansion.
        if !is_root && dir.join("Cargo.toml").is_file() {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                self.visit_dir(&path, false);
            } else if name.ends_with(".rs")
                && let Ok(bytes) = fs::read(&path)
            {
                self.add_text(&String::from_utf8_lossy(&bytes));
            }
        }
    }

    /// Collects words from `text`.
    ///
    /// Each maximal run of `[A-Za-z0-9_-]` is recorded whole (matching an
    /// original icon name) and, if it contains dashes, split on them (matching
    /// identifiers on either side of a minus sign).
    pub fn add_text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';

        let mut i = 0;
        while i < bytes.len() {
            if !is_word(bytes[i]) {
                i += 1;
                continue;
            }
            let (mut start, mut end) = (i, i);
            while end < bytes.len() && is_word(bytes[end]) {
                end += 1;
            }
            i = end;

            while start < end && bytes[start] == b'-' {
                start += 1;
            }
            while end > start && bytes[end - 1] == b'-' {
                end -= 1;
            }
            if start == end {
                continue;
            }

            // Word bytes are ASCII, so both ends are character boundaries.
            let run = &text[start..end];
            let has_dash = run.contains('-');
            let is_reference = is_quoted(bytes, start, end)
                || (!has_dash && is_nullary_method_call(bytes, start, end));
            self.insert(run, is_reference);
            if has_dash {
                for part in run.split('-').filter(|part| !part.is_empty()) {
                    self.insert(part, false);
                }
            }
        }
    }

    fn insert(&mut self, word: &str, is_reference: bool) {
        match self.0.get_mut(word) {
            Some(seen) => *seen |= is_reference,
            None => {
                self.0.insert(word.to_string(), is_reference);
            }
        }
    }
}

/// `"name"` or `"set:name"`.
fn is_quoted(bytes: &[u8], start: usize, end: usize) -> bool {
    bytes.get(end) == Some(&b'"') && start > 0 && matches!(bytes[start - 1], b'"' | b':')
}

/// `.name()` — a method call without arguments, which is what every generated
/// icon accessor looks like. Whitespace (such as the line break in a method
/// chain) is allowed between the tokens.
fn is_nullary_method_call(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut before = bytes[..start]
        .iter()
        .rev()
        .skip_while(|b| b.is_ascii_whitespace());
    let mut after = bytes[end..].iter().skip_while(|b| b.is_ascii_whitespace());

    // `..name()` is a range bound, not a member access.
    let after_dot = before.next() == Some(&b'.') && before.next() != Some(&b'.');
    after_dot
        && after.next() == Some(&b'(')
        && after.find(|b| !b.is_ascii_whitespace()) == Some(&b')')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Words {
        let mut words = Words::default();
        words.add_text(text);
        words
    }

    #[test]
    fn method_calls_are_references() {
        let w = words("let svg = Icon::mage().calendar_2_fill();");
        assert!(w.references("calendar-2-fill"));
        assert!(!w.mentions("chart-fill"));
    }

    #[test]
    fn chained_calls_are_references() {
        let w = words("Icon::lucide()\n    .arrow_right ( );");
        assert!(w.references("arrow-right"));
    }

    #[test]
    fn calls_with_arguments_fields_and_paths_are_only_mentions() {
        let w = words("items.map(|x| x); self.filter; LucideIcons::send; fs::copy(a, b);");
        for name in ["map", "filter", "send", "copy"] {
            assert!(w.mentions(name), "{name}");
            assert!(!w.references(name), "{name}");
        }
    }

    #[test]
    fn quoted_names_are_references() {
        let w = words(r#"Icon::mage().by_name("chart-fill"); lookup("mage:home")"#);
        assert!(w.references("chart-fill"));
        assert!(w.references("home"));
    }

    #[test]
    fn bare_words_are_only_mentions() {
        let w = words("my_icon!(arrow_left); let n = total-arrow_up; let search = 0..send();");
        for name in ["arrow-left", "arrow-up", "search", "send"] {
            assert!(w.mentions(name), "{name}");
            assert!(!w.references(name), "{name}");
        }
    }

    #[test]
    fn a_reference_anywhere_outranks_a_mention() {
        let w = words("let save = 1; doc.save(); let save = 2;");
        assert!(w.references("save"));
    }

    #[test]
    fn maps_sanitized_keywords_and_leading_digits() {
        let w = words("Icon::lucide().box_icon(); Icon::mage()._3d_rotate();");
        assert!(w.references("box"));
        assert!(w.references("3d-rotate"));
    }

    #[test]
    fn handles_non_ascii_text() {
        let w = words("// héllo — wörld\nlet s = \"日本語\"; Icon::mage().home();");
        assert!(w.references("home"));
    }

    #[test]
    fn scans_package_but_not_nested_packages_or_target() {
        let root = std::env::temp_dir().join(format!("iconify-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src/ui")).unwrap();
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::create_dir_all(root.join("nested/src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        fs::write(root.join("src/ui/page.rs"), "Icon::lucide().send()").unwrap();
        fs::write(root.join("src/notes.txt"), "Icon::lucide().check()").unwrap();
        fs::write(root.join("target/debug/gen.rs"), "Icon::lucide().save()").unwrap();
        fs::write(root.join("nested/Cargo.toml"), "[package]").unwrap();
        fs::write(root.join("nested/src/lib.rs"), "Icon::lucide().trash()").unwrap();

        let w = Words::scan_dir(&root);
        let _ = fs::remove_dir_all(&root);

        assert!(w.mentions("send"));
        assert!(!w.mentions("check"), "non-Rust files are ignored");
        assert!(!w.mentions("save"), "build output is ignored");
        assert!(!w.mentions("trash"), "nested packages are ignored");
    }
}

//! Generates the code `icon_sets!` expands to.
//!
//! The expansion holds one `static` table and one `match` arm per declared set,
//! the `Icon` and `Provider` items, and the embedded data. Nothing in it depends
//! on how many icons the catalog has; the accessors live in the facade.

use aurora_iconify_catalog::{CATALOG_HASH, accessor_type_name, sanitize_ident, variant_name};
use std::fmt::Write;
use std::path::PathBuf;

/// Where a set's icon index and SVG data are.
pub enum Data {
    /// Written into the expansion. Used for small selections.
    Inline { index: Vec<u8>, blob: String },
    /// Read from cache files with `include_bytes!` and `include_str!`. Used for
    /// whole sets and large selections.
    File { index: PathBuf, blob: PathBuf },
}

impl Data {
    /// The index and the SVG data as Rust expressions of type `&'static [u8]`
    /// and `&'static str`.
    fn expressions(&self) -> (String, String) {
        match self {
            Data::Inline { index, blob } => (byte_string(index), format!("{blob:?}")),
            Data::File { index, blob } => (
                format!("::core::include_bytes!({:?})", index.to_string_lossy()),
                format!("::core::include_str!({:?})", blob.to_string_lossy()),
            ),
        }
    }
}

/// One declared icon set, ready for code generation.
pub struct SetCode {
    pub prefix: &'static str,
    pub data: Data,
}

/// The Rust source that `icon_sets!` expands to for `sets`.
pub fn generate(sets: &[SetCode]) -> String {
    let mut out = String::new();

    // Compile-time checks: the facade must be the release this macro was built
    // with, and each declared set must be enabled as a feature of the facade.
    let _ = writeln!(
        out,
        "const _: () = ::core::assert!(::aurora_iconify::CATALOG_HASH == {CATALOG_HASH:#x}, {:?});",
        "aurora_iconify and aurora_iconify_macros come from different icon catalogs: use the same 0.4 release for both"
    );
    for set in sets {
        let message = format!(
            "icon set `{p}` needs the `{p}` feature of aurora_iconify: add it to the aurora_iconify features in Cargo.toml",
            p = set.prefix
        );
        let _ = writeln!(
            out,
            "const _: () = ::core::assert!(::aurora_iconify::__set_enabled({:?}), {message:?});",
            set.prefix
        );
    }

    // The embedded data, one table per declared set.
    for set in sets {
        let (index, blob) = set.data.expressions();
        let _ = writeln!(
            out,
            "#[allow(dead_code)]\nstatic {}: ::aurora_iconify::IconTable = ::aurora_iconify::IconTable::new({index}, {blob});",
            table_name(set.prefix)
        );
    }

    // The entry point: one method per declared set, and a lookup by prefix.
    out.push_str("#[allow(dead_code)]\npub struct Icon;\n\n#[allow(dead_code)]\nimpl Icon {\n");
    for set in sets {
        let _ = writeln!(
            out,
            "    /// The `{p}` icon set.\n    pub fn {m}() -> ::aurora_iconify::{ty}<Provider> {{\n        ::aurora_iconify::{ty}::__handle()\n    }}",
            p = set.prefix,
            m = sanitize_ident(set.prefix),
            ty = accessor_type_name(set.prefix)
        );
    }
    out.push_str(
        "    /// Looks up a declared icon set by its prefix, such as `\"lucide\"`.\n    \
         pub fn from_set(name: &str) -> ::core::option::Option<::aurora_iconify::DynIconSet<Provider>> {\n        \
         match name {\n",
    );
    for set in sets {
        let _ = writeln!(
            out,
            "            {:?} => ::core::option::Option::Some(::aurora_iconify::DynIconSet::__new(::aurora_iconify::SetId::{})),",
            set.prefix,
            variant_name(set.prefix)
        );
    }
    out.push_str("            _ => ::core::option::Option::None,\n        }\n    }\n}\n\n");

    // The provider that the accessors call. Its arms cover only the declared
    // sets, so the catch-all can be unreachable when every enabled set is declared.
    out.push_str(
        "#[allow(dead_code)]\npub struct Provider;\n\n\
         #[allow(unreachable_patterns)]\n\
         impl ::aurora_iconify::Provider for Provider {\n    \
         fn get(set: ::aurora_iconify::SetId, id: u32) -> ::core::option::Option<&'static str> {\n        \
         match set {\n",
    );
    for set in sets {
        let _ = writeln!(
            out,
            "            ::aurora_iconify::SetId::{} => {}.get(id),",
            variant_name(set.prefix),
            table_name(set.prefix)
        );
    }
    out.push_str(
        "            _ => ::core::option::Option::None,\n        }\n    }\n    \
         fn find(set: ::aurora_iconify::SetId, name: &str) -> ::core::option::Option<&'static str> {\n        \
         match set {\n",
    );
    for set in sets {
        let _ = writeln!(
            out,
            "            ::aurora_iconify::SetId::{} => {}.find(name),",
            variant_name(set.prefix),
            table_name(set.prefix)
        );
    }
    out.push_str("            _ => ::core::option::Option::None,\n        }\n    }\n}\n");
    out
}

/// The static that holds one set's table, such as `__ICONIFY_MATERIAL_SYMBOLS`.
fn table_name(prefix: &str) -> String {
    format!(
        "__ICONIFY_{}",
        prefix.replace('-', "_").to_ascii_uppercase()
    )
}

/// A byte-string literal with every byte escaped, so it is valid in any source encoding.
fn byte_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(3 + bytes.len() * 4);
    out.push_str("b\"");
    for byte in bytes {
        let _ = write!(out, "\\x{byte:02x}");
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_iconify_catalog::index::{Entry, write};

    /// A set with inline data for `icons`, given as (ID, name, SVG).
    fn inline_set(prefix: &'static str, icons: &[(u32, &str, &str)]) -> SetCode {
        let mut blob = String::new();
        let mut entries = Vec::new();
        for &(id, name, svg) in icons {
            entries.push(Entry {
                id,
                name,
                start: blob.len() as u32,
                len: svg.len() as u32,
            });
            blob.push_str(svg);
        }
        let index = write(&entries, blob.len() as u32);
        SetCode {
            prefix,
            data: Data::Inline { index, blob },
        }
    }

    fn file_set(prefix: &'static str) -> SetCode {
        SetCode {
            prefix,
            data: Data::File {
                index: PathBuf::from(format!("cache/{prefix}.idx")),
                blob: PathBuf::from(format!("cache/{prefix}.svgs")),
            },
        }
    }

    #[test]
    fn output_is_valid_rust() {
        let code = generate(&[
            inline_set("lucide", &[(3, "home", "<svg/>")]),
            inline_set("material-symbols", &[]),
        ]);
        if let Err(e) = syn::parse_file(&code) {
            panic!("generated code does not parse: {e}\n{code}");
        }
    }

    #[test]
    fn each_declared_set_gets_a_table_a_method_and_an_arm() {
        let code = generate(&[inline_set("material-symbols", &[(0, "home", "<svg/>")])]);
        assert!(code.contains("static __ICONIFY_MATERIAL_SYMBOLS: ::aurora_iconify::IconTable"));
        assert!(code.contains(
            "pub fn material_symbols() -> ::aurora_iconify::MaterialSymbolsIcons<Provider>"
        ));
        assert!(code.contains(
            "\"material-symbols\" => ::core::option::Option::Some(::aurora_iconify::DynIconSet::__new(::aurora_iconify::SetId::MaterialSymbols))"
        ));
        assert!(code.contains(
            "::aurora_iconify::SetId::MaterialSymbols => __ICONIFY_MATERIAL_SYMBOLS.get(id),"
        ));
        assert!(code.contains("::aurora_iconify::__set_enabled(\"material-symbols\")"));
    }

    #[test]
    fn file_backed_data_is_read_with_includes() {
        let code = generate(&[SetCode {
            prefix: "lucide",
            data: Data::File {
                index: PathBuf::from(r"C:\cache\table-1.idx"),
                blob: PathBuf::from(r"C:\cache\table-1.svgs"),
            },
        }]);
        assert!(code.contains(r#"::core::include_bytes!("C:\\cache\\table-1.idx")"#));
        assert!(code.contains(r#"::core::include_str!("C:\\cache\\table-1.svgs")"#));
        syn::parse_file(&code).unwrap();
    }

    #[test]
    fn each_declared_set_costs_a_fixed_amount_of_code() {
        // Whatever a set holds, it adds one table, one method, one arm and one
        // assertion. Ten file-backed sets therefore stay well under 20 KB.
        let sets: Vec<SetCode> = [
            "lucide",
            "mage",
            "tabler",
            "ph",
            "heroicons",
            "feather",
            "iconoir",
            "basil",
            "solar",
            "mdi",
        ]
        .into_iter()
        .map(file_set)
        .collect();
        let code = generate(&sets);
        assert!(code.len() < 20 * 1024, "{} bytes", code.len());
        syn::parse_file(&code).unwrap();
    }

    #[test]
    fn small_inline_selections_stay_small() {
        // Six 400-byte icons across two sets: the expansion is the embedded data
        // plus a fixed amount of code, with the index escaped byte by byte.
        let big = "x".repeat(400);
        let code = generate(&[
            inline_set("lucide", &[(0, "a", &big), (1, "b", &big), (2, "c", &big)]),
            inline_set("mage", &[(0, "d", &big), (1, "e", &big), (2, "f", &big)]),
        ]);
        assert!(code.len() < 12 * 1024, "{} bytes", code.len());
    }

    #[test]
    fn byte_strings_escape_every_byte() {
        assert_eq!(byte_string(&[0, 255, b'a']), r#"b"\x00\xff\x61""#);
    }
}

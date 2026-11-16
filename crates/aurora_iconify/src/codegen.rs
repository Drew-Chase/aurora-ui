//! Generates the Rust API emitted by `icon_sets!`.
//!
//! The output is built as source text and parsed once by the compiler, which
//! is far cheaper than assembling hundreds of thousands of individual tokens
//! for large sets.
//!
//! Per set, all embedded SVGs live in one string (`BLOB`) with a sorted
//! `INDEX` of `(name, start, len)`. Every icon in the set gets a method so
//! editors can complete it; methods for embedded icons slice the blob, the
//! rest call a shared `missing` function. Nothing here grows the binary or
//! the compile time per *available* icon beyond one trivial inline function.

use crate::cache::Span;
use crate::names::{sanitize_ident, to_pascal_case};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

/// Where the concatenated SVG data of a set comes from.
pub enum Blob<'a> {
    /// Emitted as a string literal — used when only a few icons are embedded.
    Inline(&'a str),
    /// Read with a single `include_str!` — used for sets embedded whole.
    File(&'a Path),
}

pub struct IconSet<'a> {
    pub prefix: &'a str,
    /// Every icon in the set, sorted.
    pub names: &'a [String],
    /// The icons that have data, sorted by name, with their spans in `blob`.
    pub embedded: Vec<(&'a str, Span)>,
    pub blob: Blob<'a>,
}

/// Functions shared by every per-set data module. `__PREFIX__` is replaced
/// with the set's prefix as a string literal.
const MODULE_FUNCTIONS: &str = r#"
    #[inline]
    pub fn get(i: usize) -> &'static str {
        let (_, start, len) = INDEX[i];
        &BLOB[start as usize..(start + len) as usize]
    }

    pub fn find(name: &str) -> ::core::option::Option<&'static str> {
        match INDEX.binary_search_by(|entry| entry.0.cmp(name)) {
            ::core::result::Result::Ok(i) => ::core::option::Option::Some(get(i)),
            ::core::result::Result::Err(_) => ::core::option::Option::None,
        }
    }

    #[cold]
    #[inline(never)]
    #[track_caller]
    pub fn missing(name: &str) -> ! {
        ::core::panic!(
            "icon `{0}:{1}` is not embedded: icon_sets! found no reference to it in this crate's \
             sources. List it explicitly (\"{0}\": [\"{1}\"]) or embed the whole set (\"{0}\": all).",
            __PREFIX__,
            name
        )
    }
"#;

pub fn generate(sets: &[IconSet<'_>], api_base: &str) -> String {
    let method_count: usize = sets.iter().map(|set| set.names.len()).sum();
    let mut out = String::with_capacity(4096 + method_count * 256);

    out.push_str(
        "/// Entry point to the icon sets declared with `icon_sets!`.\n\
         pub struct Icon;\n\
         #[allow(dead_code)]\n\
         impl Icon {\n",
    );
    for set in sets {
        let (method, ty) = (sanitize_ident(set.prefix), struct_name(set.prefix));
        let doc = format!("The `{}` icon set.", set.prefix);
        let _ = writeln!(
            out,
            "#[doc = {doc:?}] #[inline] pub fn {method}() -> {ty} {{ {ty} }}"
        );
    }
    out.push_str(
        "/// Look up an icon set by its prefix string.\n\
         pub fn from_set(name: &str) -> ::core::option::Option<DynIconSet> {\n\
         match name {\n",
    );
    for set in sets {
        let _ = writeln!(
            out,
            "{:?} => ::core::option::Option::Some(DynIconSet::{}),",
            set.prefix,
            variant_name(set.prefix)
        );
    }
    out.push_str("_ => ::core::option::Option::None,\n}\n}\n}\n");

    out.push_str(
        "/// Dynamic icon set handle for runtime set selection.\n\
         #[allow(dead_code)]\n\
         pub enum DynIconSet {\n",
    );
    for set in sets {
        let doc = format!("The `{}` icon set.", set.prefix);
        let _ = writeln!(out, "#[doc = {doc:?}] {},", variant_name(set.prefix));
    }
    out.push_str(
        "}\n\
         #[allow(dead_code)]\n\
         impl DynIconSet {\n\
         /// Look up an embedded icon by its original name within this set.\n\
         pub fn by_name(&self, name: &str) -> ::core::option::Option<&'static str> {\n\
         match self {\n",
    );
    for set in sets {
        let _ = writeln!(
            out,
            "DynIconSet::{} => {}::find(name),",
            variant_name(set.prefix),
            module_name(set.prefix)
        );
    }
    out.push_str("}\n}\n}\n");

    for set in sets {
        write_set(&mut out, set, api_base);
    }
    out
}

fn write_set(out: &mut String, set: &IconSet<'_>, api_base: &str) {
    let prefix = set.prefix;
    let ty = struct_name(prefix);
    let module = module_name(prefix);

    // Data module.
    let _ = write!(
        out,
        "#[doc(hidden)]\n#[allow(dead_code, non_snake_case)]\nmod {module} {{\npub static BLOB: &str = "
    );
    match set.blob {
        Blob::Inline(text) => {
            let _ = write!(out, "{text:?}");
        }
        Blob::File(path) => {
            let _ = write!(out, "::core::include_str!({:?})", path.to_string_lossy());
        }
    }
    let _ = write!(
        out,
        ";\npub static INDEX: [(&str, u32, u32); {}] = [",
        set.embedded.len()
    );
    for (name, (start, len)) in &set.embedded {
        let _ = write!(out, "({name:?},{start},{len}),");
    }
    out.push_str("];\n");
    out.push_str(&MODULE_FUNCTIONS.replace("__PREFIX__", &format!("{prefix:?}")));
    out.push_str("}\n");

    // Accessor type: one method per icon in the set.
    let doc = format!("Icons of the `{prefix}` set.");
    let _ = write!(
        out,
        "#[doc = {doc:?}]\npub struct {ty};\n#[allow(dead_code)]\nimpl {ty} {{\n"
    );

    let slots: HashMap<&str, usize> = set
        .embedded
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (*name, i))
        .collect();
    // Two icon names can sanitize to the same identifier; the first one wins
    // and the other stays reachable through `by_name`.
    let mut taken: HashSet<String> = HashSet::from(["by_name".to_string()]);

    for name in set.names {
        let method = sanitize_ident(name);
        if !taken.insert(method.clone()) {
            continue;
        }
        let doc =
            format!("![{name}]({api_base}/{prefix}/{name}.svg?width=48&height=48&color=white)");
        let _ = write!(out, "#[doc = {doc:?}] #[inline] ");
        let _ = match slots.get(name.as_str()) {
            Some(i) => writeln!(
                out,
                "pub fn {method}(&self) -> &'static str {{ {module}::get({i}) }}"
            ),
            // `track_caller` makes the panic point at the code that asked
            // for the icon rather than at the `icon_sets!` invocation.
            None => writeln!(
                out,
                "#[track_caller] pub fn {method}(&self) -> &'static str {{ {module}::missing({name:?}) }}"
            ),
        };
    }

    let _ = write!(
        out,
        "/// Look up an embedded icon by its original name (e.g. `\"calendar-2-fill\"`).\n\
         ///\n\
         /// Finds icons that this crate references by method or by name in a\n\
         /// string literal, plus any listed in `icon_sets!`. Use `\"{prefix}\": all`\n\
         /// to make every icon of the set available.\n\
         pub fn by_name(&self, name: &str) -> ::core::option::Option<&'static str> {{ {module}::find(name) }}\n\
         }}\n"
    );
}

fn variant_name(prefix: &str) -> String {
    let name = to_pascal_case(prefix);
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{name}")
    } else {
        name
    }
}

fn struct_name(prefix: &str) -> String {
    format!("{}Icons", variant_name(prefix))
}

fn module_name(prefix: &str) -> String {
    format!("__iconify_{}", prefix.replace('-', "_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn demo(names: &[String]) -> String {
        let sets = [
            IconSet {
                prefix: "mage",
                names,
                embedded: vec![("box", (0, 6)), ("chart-fill", (6, 8))],
                blob: Blob::Inline("<box/><chart/>"),
            },
            IconSet {
                prefix: "material-symbols",
                names,
                embedded: names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (name.as_str(), (i as u32, 1)))
                    .collect(),
                blob: Blob::File(Path::new(r"C:\cache\material-symbols\all-0123.svgs")),
            },
        ];
        generate(&sets, "https://api.example")
    }

    #[test]
    fn output_is_valid_rust() {
        let code = demo(&strings(&["3d-rotate", "box", "chart-fill", "home"]));
        if let Err(e) = syn::parse_file(&code) {
            panic!("generated code does not parse: {e}\n{code}");
        }
    }

    #[test]
    fn embedded_icons_slice_the_blob_and_others_are_stubs() {
        let code = demo(&strings(&["3d-rotate", "box", "chart-fill", "home"]));
        assert!(code.contains("pub fn box_icon(&self) -> &'static str { __iconify_mage::get(0) }"));
        assert!(
            code.contains("pub fn chart_fill(&self) -> &'static str { __iconify_mage::get(1) }")
        );
        assert!(
            code.contains(
                r#"pub fn home(&self) -> &'static str { __iconify_mage::missing("home") }"#
            )
        );
        assert!(code.contains(
            r#"pub fn _3d_rotate(&self) -> &'static str { __iconify_mage::missing("3d-rotate") }"#
        ));
        assert!(code.contains(
            r#"pub static INDEX: [(&str, u32, u32); 2] = [("box",0,6),("chart-fill",6,8),];"#
        ));
        assert!(code.contains(r#"pub static BLOB: &str = "<box/><chart/>";"#));
        assert!(code.contains(r#"#[inline] #[track_caller] pub fn home(&self)"#));
        assert!(code.contains(r#"#[inline] pub fn box_icon(&self)"#));
    }

    #[test]
    fn whole_sets_use_one_include() {
        let code = demo(&strings(&["a", "b"]));
        assert_eq!(code.matches("include_str!").count(), 1);
        assert!(
            code.contains(r#"::core::include_str!("C:\\cache\\material-symbols\\all-0123.svgs")"#)
        );
        assert!(code.contains("pub struct MaterialSymbolsIcons;"));
        assert!(code.contains("pub fn material_symbols() -> MaterialSymbolsIcons"));
        assert!(code.contains(
            r#""material-symbols" => ::core::option::Option::Some(DynIconSet::MaterialSymbols),"#
        ));
    }

    #[test]
    fn colliding_method_names_are_emitted_once() {
        let code = demo(&strings(&["a-b", "a_b", "by-name"]));
        assert_eq!(code.matches("pub fn a_b(&self)").count(), 2, "once per set");
        assert_eq!(code.matches("pub fn by_name(&self, name: &str)").count(), 3);
        assert!(!code.contains("pub fn by_name(&self) -> &'static str"));
        syn::parse_file(&code).unwrap();
    }

    #[test]
    fn svg_text_is_escaped() {
        let names = strings(&["a"]);
        let sets = [IconSet {
            prefix: "demo",
            names: &names,
            embedded: vec![("a", (0, 22))],
            blob: Blob::Inline("<path d=\"M0 0\"/>\\{x}\n"),
        }];
        let code = generate(&sets, "https://api.example");
        let file = syn::parse_file(&code).expect("escaped blob parses");
        assert!(!file.items.is_empty());
    }
}

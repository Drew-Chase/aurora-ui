mod codegen;
mod load;
mod names;
mod parse;
mod tree;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::parse_macro_input;

/// Reads TOML locale files at compile time and generates a `lang` module with
/// type-safe, zero-cost `&'static str` constants for every translation key.
///
/// # Input
///
/// Pass a path (relative to `CARGO_MANIFEST_DIR`) to either:
///
/// - A **directory** of per-locale `.toml` files (e.g. `en-us.toml`, `ru.toml`)
/// - A **single `.toml` file** with `[locale-tag]` sections
///
/// # Generated API
///
/// ```ignore
/// aurora_lang::lang!("locales");
///
/// // Default locale (first alphabetically):
/// let title = lang::LANG.title.window;
///
/// // Specific locale:
/// let title_ru = lang::ru::LANG.title.window;
///
/// // Dynamic lookup by tag:
/// let l = lang::by_tag("ru").unwrap();
/// let title = l.title.window;
///
/// // All available tags:
/// for tag in lang::LOCALE_TAGS {
///     println!("{tag}");
/// }
/// ```
#[proc_macro]
pub fn lang(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as parse::LangInput);
    let path_str = input.path.value();

    let full_path = load::resolve_path(&path_str);

    // Load and normalize all locales
    let locale_map = match load::load_locales(&full_path) {
        Ok(m) => m,
        Err(e) => {
            let msg = format!("lang!: {e}");
            return quote! { compile_error!(#msg); }.into();
        }
    };

    // Validate: need at least one locale
    let locale_names: Vec<&String> = locale_map.keys().collect();
    if locale_names.is_empty() {
        return quote! { compile_error!("lang!: no locales found"); }.into();
    }

    // Validate: all locales must have the same key set
    let first_locale = locale_names[0];
    let reference_keys: BTreeSet<&String> = locale_map[first_locale].keys().collect();

    for locale in &locale_names[1..] {
        let keys: BTreeSet<&String> = locale_map[*locale].keys().collect();
        let missing: Vec<_> = reference_keys.difference(&keys).collect();
        let extra: Vec<_> = keys.difference(&reference_keys).collect();
        if !missing.is_empty() {
            let msg = format!(
                "lang!: locale `{locale}` is missing keys present in `{first_locale}`: {missing:?}"
            );
            return quote! { compile_error!(#msg); }.into();
        }
        if !extra.is_empty() {
            let msg = format!(
                "lang!: locale `{locale}` has extra keys not in `{first_locale}`: {extra:?}"
            );
            return quote! { compile_error!(#msg); }.into();
        }
    }

    // Build the key tree
    let all_keys: Vec<String> = reference_keys.into_iter().cloned().collect();
    let key_tree = match tree::build_key_tree(&all_keys) {
        Ok(t) => t,
        Err(e) => {
            let msg = format!("lang!: {e}");
            return quote! { compile_error!(#msg); }.into();
        }
    };

    // Generate struct definitions (shared across all locales)
    let struct_defs = codegen::generate_struct_defs(&key_tree, "Lang");

    // Generate default locale constant (first locale)
    let default_values = &locale_map[first_locale];
    let default_const_expr = codegen::generate_const_expr(&key_tree, default_values, "Lang");

    // Generate per-locale submodules
    let mut locale_modules = Vec::new();
    let mut locale_tag_arms = Vec::new();
    let mut locale_tag_strs = Vec::new();

    for locale_tag in &locale_names {
        let values = &locale_map[*locale_tag];
        let const_expr = codegen::generate_const_expr(&key_tree, values, "Lang");
        let mod_name = format_ident!("{}", names::locale_to_mod_name(locale_tag));
        let tag_str = locale_tag.as_str();

        locale_modules.push(quote! {
            pub mod #mod_name {
                use super::*;
                pub const LANG: Lang = #const_expr;
            }
        });

        locale_tag_arms.push(quote! {
            #tag_str => ::core::option::Option::Some(&#mod_name::LANG)
        });
        locale_tag_strs.push(quote! { #tag_str });
    }

    // Rebuild tracking: include_bytes! triggers Cargo file watching,
    // but we only store a bool (not the file contents) to avoid binary bloat.
    let toml_paths = load::collect_toml_paths(&full_path);
    let track_paths: Vec<proc_macro2::TokenStream> = toml_paths
        .iter()
        .map(|p| {
            let path_str = p.to_string_lossy().replace('\\', "/");
            quote! { const _: bool = !include_bytes!(#path_str).is_empty(); }
        })
        .collect();

    let num_locales = locale_names.len();

    let expanded = quote! {
        #[allow(dead_code, non_upper_case_globals, non_camel_case_types)]
        pub mod lang {
            #(#track_paths)*

            #(#struct_defs)*

            /// Translations for the default locale.
            pub const LANG: Lang = #default_const_expr;

            #(#locale_modules)*

            /// Returns the `Lang` struct for a locale tag, or `None` if unknown.
            ///
            /// Matching is case-insensitive: `"en-US"` and `"en-us"` both work.
            pub fn by_tag(tag: &str) -> ::core::option::Option<&'static Lang> {
                match tag.to_ascii_lowercase().as_str() {
                    #(#locale_tag_arms,)*
                    _ => ::core::option::Option::None,
                }
            }

            /// All locale tags available, in declaration order.
            pub const LOCALE_TAGS: [&str; #num_locales] = [#(#locale_tag_strs),*];
        }
    };

    expanded.into()
}

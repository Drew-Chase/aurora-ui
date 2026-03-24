#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Token};

const API_BASE: &str = "https://api.iconify.design";
const CACHE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days
const BATCH_SIZE: usize = 80;

/// Fetches icon sets from [iconify.design](https://iconify.design) at compile
/// time and generates a type-safe Rust API with all SVGs embedded as string
/// literals.
///
/// # Usage
///
/// ```ignore
/// aurora_iconify::icon_sets!("mage", "lucide");
///
/// let svg: &str = Icon::mage().calendar_2_fill();
/// let svg: Option<&str> = Icon::mage().by_name("calendar-2-fill");
/// let svg: Option<&str> = Icon::from_set("mage").and_then(|s| s.by_name("calendar-2-fill"));
/// ```
///
/// Each icon set is fetched from the Iconify API on first compile. Results are
/// cached locally for 7 days. All SVG data is embedded directly in the binary.
#[proc_macro]
pub fn icon_sets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IconSetsInput);
    let set_names = &input.sets;

    if set_names.is_empty() {
        return quote! { compile_error!("icon_sets! requires at least one icon set name"); }.into();
    }

    let cache_dir = cache_dir();

    let mut icon_struct_methods = Vec::new();
    let mut set_structs = Vec::new();
    let mut from_set_arms = Vec::new();
    let mut dyn_variants = Vec::new();
    let mut dyn_by_name_arms = Vec::new();

    for set_name in set_names {
        let data = match fetch_icon_set(set_name, &cache_dir) {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("Failed to fetch icon set '{}': {}", set_name, e);
                return quote! { compile_error!(#msg); }.into();
            }
        };

        let method_name = format_ident!("{}", sanitize_ident(set_name));
        let struct_name = format_ident!("{}Icons", to_pascal_case(set_name));
        let variant_name = format_ident!("{}", to_pascal_case(set_name));

        // Generate per-icon methods and by_name match arms
        let mut icon_methods = Vec::new();
        let mut by_name_arms = Vec::new();

        for (icon_name, svg_string) in &data.icons {
            let fn_name = format_ident!("{}", sanitize_ident(icon_name));
            let name_str = icon_name.as_str();

            icon_methods.push(quote! {
                pub fn #fn_name(&self) -> &'static str { #svg_string }
            });

            by_name_arms.push(quote! {
                #name_str => Some(self.#fn_name()),
            });
        }

        set_structs.push(quote! {
            pub struct #struct_name;

            impl #struct_name {
                #(#icon_methods)*

                /// Look up an icon by its original name (e.g. `"calendar-2-fill"`).
                pub fn by_name(&self, name: &str) -> Option<&'static str> {
                    match name {
                        #(#by_name_arms)*
                        _ => None,
                    }
                }
            }
        });

        let set_str = set_name.as_str();
        icon_struct_methods.push(quote! {
            pub fn #method_name() -> #struct_name { #struct_name }
        });
        from_set_arms.push(quote! {
            #set_str => Some(DynIconSet::#variant_name),
        });
        dyn_variants.push(variant_name.clone());
        dyn_by_name_arms.push(quote! {
            DynIconSet::#variant_name => #struct_name.by_name(name),
        });
    }

    let expanded = quote! {
        pub struct Icon;

        impl Icon {
            #(#icon_struct_methods)*

            /// Look up an icon set by its prefix string.
            pub fn from_set(name: &str) -> Option<DynIconSet> {
                match name {
                    #(#from_set_arms)*
                    _ => None,
                }
            }
        }

        #(#set_structs)*

        /// Dynamic icon set handle for runtime set selection.
        pub enum DynIconSet {
            #(#dyn_variants,)*
        }

        impl DynIconSet {
            /// Look up an icon by name within this set.
            pub fn by_name(&self, name: &str) -> Option<&'static str> {
                match self {
                    #(#dyn_by_name_arms)*
                }
            }
        }
    };

    expanded.into()
}

// ---------------------------------------------------------------------------
// Input parsing
// ---------------------------------------------------------------------------

struct IconSetsInput {
    sets: Vec<String>,
}

impl Parse for IconSetsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sets = Vec::new();
        while !input.is_empty() {
            let lit: LitStr = input.parse()?;
            sets.push(lit.value());
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { sets })
    }
}

// ---------------------------------------------------------------------------
// API fetching + caching
// ---------------------------------------------------------------------------

struct IconSetData {
    icons: Vec<(String, String)>, // (name, full_svg_string)
}

fn cache_dir() -> PathBuf {
    // Try CARGO_MANIFEST_DIR first (available in proc macros via env)
    let base = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("target").join(".iconify-cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn cache_path(cache_dir: &Path, prefix: &str) -> PathBuf {
    cache_dir.join(format!("{}.json", prefix))
}

fn is_cache_valid(path: &Path) -> bool {
    path.exists()
        && fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.elapsed().unwrap_or(Duration::MAX) < CACHE_TTL)
            .unwrap_or(false)
}

fn fetch_icon_set(prefix: &str, cache_dir: &Path) -> Result<IconSetData, String> {
    let cached = cache_path(cache_dir, prefix);

    // Try cache first
    let json_str = if is_cache_valid(&cached) {
        fs::read_to_string(&cached).map_err(|e| format!("cache read failed: {e}"))?
    } else {
        // Step 1: Get icon names
        let names = fetch_icon_names(prefix)?;
        if names.is_empty() {
            return Err(format!("icon set '{prefix}' has no icons"));
        }

        // Step 2: Fetch all icon data in batches
        let mut all_data = serde_json::Map::new();
        let mut default_width = 24u64;
        let mut default_height = 24u64;

        for chunk in names.chunks(BATCH_SIZE) {
            let icons_param = chunk.join(",");
            let url = format!("{API_BASE}/{prefix}.json?icons={icons_param}");
            let body = http_get(&url)?;
            let parsed: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {e}"))?;

            if let Some(w) = parsed.get("width").and_then(|v| v.as_u64()) {
                default_width = w;
            }
            if let Some(h) = parsed.get("height").and_then(|v| v.as_u64()) {
                default_height = h;
            }
            if let Some(icons) = parsed.get("icons").and_then(|v| v.as_object()) {
                for (k, v) in icons {
                    all_data.insert(k.clone(), v.clone());
                }
            }
        }

        // Build cache object
        let cache_obj = serde_json::json!({
            "width": default_width,
            "height": default_height,
            "icons": all_data,
        });
        let json_str = serde_json::to_string(&cache_obj)
            .map_err(|e| format!("JSON serialize error: {e}"))?;
        let _ = fs::write(&cached, &json_str);
        json_str
    };

    parse_icon_set_json(&json_str)
}

fn fetch_icon_names(prefix: &str) -> Result<Vec<String>, String> {
    let url = format!("{API_BASE}/collection?prefix={prefix}");
    let body = http_get(&url)?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {e}"))?;

    let mut names = Vec::new();

    // Icons can be in "uncategorized" array or in "categories" object
    if let Some(arr) = parsed.get("uncategorized").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(s) = item.as_str() {
                names.push(s.to_string());
            }
        }
    }
    if let Some(cats) = parsed.get("categories").and_then(|v| v.as_object()) {
        for (_cat, arr) in cats {
            if let Some(arr) = arr.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str()
                        && !names.contains(&s.to_string())
                    {
                        names.push(s.to_string());
                    }
                }
            }
        }
    }

    Ok(names)
}

fn parse_icon_set_json(json_str: &str) -> Result<IconSetData, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;

    let default_width = parsed
        .get("width")
        .and_then(|v| v.as_u64())
        .unwrap_or(24);
    let default_height = parsed
        .get("height")
        .and_then(|v| v.as_u64())
        .unwrap_or(24);

    let icons_obj = parsed
        .get("icons")
        .and_then(|v| v.as_object())
        .ok_or("missing 'icons' object")?;

    let mut icons = Vec::new();

    for (name, data) in icons_obj {
        let body = data
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let w = data
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(default_width);
        let h = data
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(default_height);

        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">{body}</svg>"
        );
        icons.push((name.clone(), svg));
    }

    icons.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(IconSetData { icons })
}

fn http_get(url: &str) -> Result<String, String> {
    ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP request failed for {url}: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response from {url}: {e}"))
}

// ---------------------------------------------------------------------------
// Name utilities
// ---------------------------------------------------------------------------

fn sanitize_ident(name: &str) -> String {
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
        "as" | "async" | "await" | "box" | "break" | "const" | "continue" | "crate" | "do"
            | "dyn" | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl"
            | "in" | "let" | "loop" | "macro" | "match" | "mod" | "move" | "mut" | "pub"
            | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" | "trait"
            | "true" | "try" | "type" | "union" | "unsafe" | "use" | "where" | "while"
            | "yield" | "abstract" | "become" | "final" | "override" | "priv" | "typeof"
            | "unsized" | "virtual"
    )
}

fn to_pascal_case(name: &str) -> String {
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

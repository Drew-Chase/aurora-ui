//! Proc macro for compile-time theme configuration from JSON.
//!
//! Add `aurora_theming::config!("themes.json")` to your `main.rs` to generate
//! typed theme accessors and profile management from a JSON file.
//!
//! The JSON file uses `"_"` as the default profile. Other profiles inherit from
//! `"_"` and only need to specify overrides.
//!
//! # JSON format
//!
//! ```json
//! {
//!   "_": {
//!     "colors": {
//!       "background": "#09090b",
//!       "foreground": "#fafafa"
//!     },
//!     "edges": {
//!       "button_padding": { "top": 8, "right": 16, "bottom": 8, "left": 16 }
//!     },
//!     "corners": {
//!       "card_radius": 8
//!     },
//!     "values": {
//!       "font_size": 14
//!     }
//!   },
//!   "light": {
//!     "colors": {
//!       "background": "#ffffff",
//!       "foreground": "#09090b"
//!     }
//!   }
//! }
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;

/// Reads a JSON theme file at compile time and generates profile data, an enum,
/// and typed accessor modules.
///
/// # Usage
/// ```ignore
/// aurora_theming::config!("themes.json");
/// ```
///
/// This generates a `theme` module containing:
/// - `ProfileId` enum with one variant per profile
/// - `set(ProfileId)` to switch the active profile
/// - `theme::colors::*()` accessors for color values
/// - `theme::edges::*()` accessors for edge values
/// - `theme::corners::*()` accessors for corner values
/// - `theme::values::*()` accessors for f32 values
#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let lit: syn::LitStr = match syn::parse(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let path_str = lit.value();

    // Resolve relative to the calling crate's manifest dir
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let full_path = std::path::Path::new(&manifest_dir).join(&path_str);

    let json_str = match std::fs::read_to_string(&full_path) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Failed to read theme file `{}`: {}", full_path.display(), e);
            return syn::Error::new(lit.span(), msg)
                .to_compile_error()
                .into();
        }
    };

    let root: BTreeMap<String, serde_json::Value> = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Invalid theme JSON: {}", e);
            return syn::Error::new(lit.span(), msg)
                .to_compile_error()
                .into();
        }
    };

    // Collect profile names (default first)
    let mut profile_names: Vec<String> = Vec::new();
    if root.contains_key("_") {
        profile_names.push("_".into());
    }
    for key in root.keys() {
        if key != "_" {
            profile_names.push(key.clone());
        }
    }
    if profile_names.is_empty() {
        return syn::Error::new(lit.span(), "Theme JSON must have at least a \"_\" default profile")
            .to_compile_error()
            .into();
    }

    // Collect all unique keys per category from all profiles
    let color_keys = collect_keys(&root, &profile_names, "colors");
    let edge_keys = collect_keys(&root, &profile_names, "edges");
    let corner_keys = collect_keys(&root, &profile_names, "corners");
    let value_keys = collect_keys(&root, &profile_names, "values");

    // Parse default profile values
    let default_profile = root.get("_").cloned().unwrap_or(serde_json::Value::Object(Default::default()));

    // Build per-profile data
    let mut profile_color_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    let mut profile_edge_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    let mut profile_corner_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    let mut profile_value_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();

    for pname in &profile_names {
        let profile = root.get(pname).cloned().unwrap_or_default();

        let colors: Vec<_> = color_keys
            .iter()
            .map(|key| {
                let val = get_category_value(&profile, "colors", key)
                    .or_else(|| get_category_value(&default_profile, "colors", key));
                parse_color_token(val.as_ref())
            })
            .collect();

        let edges: Vec<_> = edge_keys
            .iter()
            .map(|key| {
                let val = get_category_value(&profile, "edges", key)
                    .or_else(|| get_category_value(&default_profile, "edges", key));
                parse_edges_token(val.as_ref())
            })
            .collect();

        let corners: Vec<_> = corner_keys
            .iter()
            .map(|key| {
                let val = get_category_value(&profile, "corners", key)
                    .or_else(|| get_category_value(&default_profile, "corners", key));
                parse_corners_token(val.as_ref())
            })
            .collect();

        let values: Vec<_> = value_keys
            .iter()
            .map(|key| {
                let val = get_category_value(&profile, "values", key)
                    .or_else(|| get_category_value(&default_profile, "values", key));
                parse_f32_token(val.as_ref())
            })
            .collect();

        profile_color_data.push(colors);
        profile_edge_data.push(edges);
        profile_corner_data.push(corners);
        profile_value_data.push(values);
    }

    // Generate ProfileId enum variants
    let enum_variants: Vec<_> = profile_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let variant = if name == "_" {
                format_ident!("Default")
            } else {
                format_ident!("{}", to_pascal_case(name))
            };
            let idx = i;
            quote! { #variant = #idx }
        })
        .collect();

    // Generate profile name strings
    let name_strs: Vec<_> = profile_names
        .iter()
        .map(|n| {
            let s = if n == "_" { "default" } else { n.as_str() };
            quote! { #s }
        })
        .collect();

    let num_profiles = profile_names.len();
    let num_colors = color_keys.len();
    let num_edges = edge_keys.len();
    let num_corners = corner_keys.len();
    let num_values = value_keys.len();

    // Generate static arrays per profile
    let color_arrays: Vec<_> = profile_color_data
        .iter()
        .enumerate()
        .map(|(i, colors)| {
            let name = format_ident!("COLORS_{}", i);
            quote! {
                static #name: [aurora_theme::Color; #num_colors] = [#(#colors),*];
            }
        })
        .collect();

    let edge_arrays: Vec<_> = profile_edge_data
        .iter()
        .enumerate()
        .map(|(i, edges)| {
            let name = format_ident!("EDGES_{}", i);
            quote! {
                static #name: [aurora_theme::Edges; #num_edges] = [#(#edges),*];
            }
        })
        .collect();

    let corner_arrays: Vec<_> = profile_corner_data
        .iter()
        .enumerate()
        .map(|(i, corners)| {
            let name = format_ident!("CORNERS_{}", i);
            quote! {
                static #name: [aurora_theme::Corners; #num_corners] = [#(#corners),*];
            }
        })
        .collect();

    let value_arrays: Vec<_> = profile_value_data
        .iter()
        .enumerate()
        .map(|(i, vals)| {
            let name = format_ident!("VALUES_{}", i);
            quote! {
                static #name: [f32; #num_values] = [#(#vals),*];
            }
        })
        .collect();

    // Refs for the profile table
    let color_refs: Vec<_> = (0..num_profiles)
        .map(|i| {
            let name = format_ident!("COLORS_{}", i);
            quote! { &#name }
        })
        .collect();
    let edge_refs: Vec<_> = (0..num_profiles)
        .map(|i| {
            let name = format_ident!("EDGES_{}", i);
            quote! { &#name }
        })
        .collect();
    let corner_refs: Vec<_> = (0..num_profiles)
        .map(|i| {
            let name = format_ident!("CORNERS_{}", i);
            quote! { &#name }
        })
        .collect();
    let value_refs: Vec<_> = (0..num_profiles)
        .map(|i| {
            let name = format_ident!("VALUES_{}", i);
            quote! { &#name }
        })
        .collect();

    // Generate accessor functions per key
    let color_fns: Vec<_> = color_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let fn_name = format_ident!("{}", key);
            quote! {
                pub fn #fn_name() -> aurora_theme::Color {
                    PROFILE_COLORS[aurora_theme::current_profile()][#i]
                }
            }
        })
        .collect();

    let edge_fns: Vec<_> = edge_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let fn_name = format_ident!("{}", key);
            quote! {
                pub fn #fn_name() -> aurora_theme::Edges {
                    PROFILE_EDGES[aurora_theme::current_profile()][#i]
                }
            }
        })
        .collect();

    let corner_fns: Vec<_> = corner_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let fn_name = format_ident!("{}", key);
            quote! {
                pub fn #fn_name() -> aurora_theme::Corners {
                    PROFILE_CORNERS[aurora_theme::current_profile()][#i]
                }
            }
        })
        .collect();

    let value_fns: Vec<_> = value_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let fn_name = format_ident!("{}", key);
            quote! {
                pub fn #fn_name() -> f32 {
                    PROFILE_VALUES[aurora_theme::current_profile()][#i]
                }
            }
        })
        .collect();

    // Emit the include_str! so cargo tracks the file for rebuilds
    let path_for_tracking = full_path.to_string_lossy().to_string();

    let output = quote! {
        #[allow(dead_code)]
        pub mod theme {
            // Force cargo to rebuild when the JSON changes
            const _THEME_FILE: &str = include_str!(#path_for_tracking);

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[repr(usize)]
            pub enum ProfileId {
                #(#enum_variants),*
            }

            pub fn set(profile: ProfileId) {
                aurora_theme::set_profile(profile as usize);
            }

            pub fn current() -> ProfileId {
                let id = aurora_theme::current_profile();
                // Safety: ProfileId is repr(usize) and all ids are valid
                unsafe { std::mem::transmute::<usize, ProfileId>(id.min(#num_profiles - 1)) }
            }

            pub fn profile_count() -> usize { #num_profiles }

            pub fn profile_name(id: ProfileId) -> &'static str {
                static NAMES: [&str; #num_profiles] = [#(#name_strs),*];
                NAMES[id as usize]
            }

            // Static profile data arrays
            #(#color_arrays)*
            #(#edge_arrays)*
            #(#corner_arrays)*
            #(#value_arrays)*

            static PROFILE_COLORS: [&[aurora_theme::Color; #num_colors]; #num_profiles] = [#(#color_refs),*];
            static PROFILE_EDGES: [&[aurora_theme::Edges; #num_edges]; #num_profiles] = [#(#edge_refs),*];
            static PROFILE_CORNERS: [&[aurora_theme::Corners; #num_corners]; #num_profiles] = [#(#corner_refs),*];
            static PROFILE_VALUES: [&[f32; #num_values]; #num_profiles] = [#(#value_refs),*];

            /// Initialize the aurora_theme runtime with this theme's data.
            /// Call this once at the start of your application.
            pub fn init() {
                aurora_theme::init(aurora_theme::ThemeData {
                    profile_names: vec![#(#name_strs),*],
                    colors: vec![#(Vec::from(#color_refs.as_slice())),*],
                    edges: vec![#(Vec::from(#edge_refs.as_slice())),*],
                    corners: vec![#(Vec::from(#corner_refs.as_slice())),*],
                    values: vec![#(Vec::from(#value_refs.as_slice())),*],
                });
            }

            pub mod colors {
                use super::*;
                #(#color_fns)*
            }

            pub mod edges {
                use super::*;
                #(#edge_fns)*
            }

            pub mod corners {
                use super::*;
                #(#corner_fns)*
            }

            pub mod values {
                use super::*;
                #(#value_fns)*
            }
        }
    };

    output.into()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_keys(
    root: &BTreeMap<String, serde_json::Value>,
    profiles: &[String],
    category: &str,
) -> Vec<String> {
    let mut keys = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for pname in profiles {
        if let Some(profile) = root.get(pname) {
            if let Some(obj) = profile.get(category).and_then(|v| v.as_object()) {
                for key in obj.keys() {
                    if seen.insert(key.clone()) {
                        keys.push(key.clone());
                    }
                }
            }
        }
    }
    keys
}

fn get_category_value(
    profile: &serde_json::Value,
    category: &str,
    key: &str,
) -> Option<serde_json::Value> {
    profile
        .get(category)
        .and_then(|c| c.get(key))
        .cloned()
}

fn parse_color_token(val: Option<&serde_json::Value>) -> proc_macro2::TokenStream {
    match val {
        Some(serde_json::Value::String(hex)) => {
            let hex = hex.trim_start_matches('#');
            let (r, g, b, a) = match hex.len() {
                6 => {
                    let n = u32::from_str_radix(hex, 16).unwrap_or(0);
                    (((n >> 16) & 0xFF) as u8, ((n >> 8) & 0xFF) as u8, (n & 0xFF) as u8, 255u8)
                }
                8 => {
                    let n = u32::from_str_radix(hex, 16).unwrap_or(0);
                    (((n >> 24) & 0xFF) as u8, ((n >> 16) & 0xFF) as u8, ((n >> 8) & 0xFF) as u8, (n & 0xFF) as u8)
                }
                _ => (0, 0, 0, 255),
            };
            quote! { aurora_theme::Color::new(#r, #g, #b, #a) }
        }
        Some(serde_json::Value::Object(obj)) => {
            let r = obj.get("r").or(obj.get("red")).and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let g = obj.get("g").or(obj.get("green")).and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let b = obj.get("b").or(obj.get("blue")).and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let a = obj.get("a").or(obj.get("alpha")).and_then(|v| v.as_u64()).unwrap_or(255) as u8;
            quote! { aurora_theme::Color::new(#r, #g, #b, #a) }
        }
        _ => quote! { aurora_theme::Color::new(0, 0, 0, 255) },
    }
}

fn parse_edges_token(val: Option<&serde_json::Value>) -> proc_macro2::TokenStream {
    match val {
        Some(serde_json::Value::Object(obj)) => {
            let top = obj.get("top").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let right = obj.get("right").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let bottom = obj.get("bottom").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let left = obj.get("left").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            quote! { aurora_theme::Edges::new(#top, #right, #bottom, #left) }
        }
        Some(serde_json::Value::Number(n)) => {
            let v = n.as_f64().unwrap_or(0.0) as f32;
            quote! { aurora_theme::Edges::all(#v) }
        }
        _ => quote! { aurora_theme::Edges::zero() },
    }
}

fn parse_corners_token(val: Option<&serde_json::Value>) -> proc_macro2::TokenStream {
    match val {
        Some(serde_json::Value::Object(obj)) => {
            let tl = obj.get("top_left").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let tr = obj.get("top_right").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let br = obj.get("bottom_right").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let bl = obj.get("bottom_left").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            quote! { aurora_theme::Corners::new(#tl, #tr, #br, #bl) }
        }
        Some(serde_json::Value::Number(n)) => {
            let v = n.as_f64().unwrap_or(0.0) as f32;
            quote! { aurora_theme::Corners::all(#v) }
        }
        _ => quote! { aurora_theme::Corners::zero() },
    }
}

fn parse_f32_token(val: Option<&serde_json::Value>) -> proc_macro2::TokenStream {
    match val {
        Some(serde_json::Value::Number(n)) => {
            let v = n.as_f64().unwrap_or(0.0) as f32;
            quote! { #v }
        }
        _ => quote! { 0.0f32 },
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect()
}

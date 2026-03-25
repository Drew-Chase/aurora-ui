//! Proc macro for compile-time theme configuration from JSON.
//!
//! Add `aurora_theming::config!("themes.json")` to your `main.rs` to generate
//! typed theme accessors and profile management from a JSON file.
//!
//! The JSON file uses `"_"` as the default profile. Other profiles inherit from
//! `"_"` and only need to specify overrides.

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
    let abs_path_str = full_path.to_string_lossy().to_string();

    let json_str = match std::fs::read_to_string(&full_path) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Failed to read theme file `{}`: {}", full_path.display(), e);
            return syn::Error::new(lit.span(), msg)
                .to_compile_error()
                .into();
        }
    };

    let root: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Invalid theme JSON: {}", e);
            return syn::Error::new(lit.span(), msg)
                .to_compile_error()
                .into();
        }
    };

    let root_obj = match root.as_object() {
        Some(o) => o,
        None => {
            return syn::Error::new(lit.span(), "Theme JSON root must be an object")
                .to_compile_error()
                .into();
        }
    };

    // Collect profile names (default "_" first, then others sorted)
    let mut profile_names: Vec<String> = Vec::new();
    if root_obj.contains_key("_") {
        profile_names.push("_".into());
    }
    for key in root_obj.keys() {
        if key != "_" {
            profile_names.push(key.clone());
        }
    }
    if profile_names.is_empty() {
        return syn::Error::new(lit.span(), "Theme JSON must have at least a \"_\" default profile")
            .to_compile_error()
            .into();
    }

    // Collect all unique keys per category across all profiles.
    // Use BTreeMap to get stable sorted order.
    let color_keys = collect_sorted_keys(root_obj, &profile_names, "colors");
    let edge_keys = collect_sorted_keys(root_obj, &profile_names, "edges");
    let corner_keys = collect_sorted_keys(root_obj, &profile_names, "corners");
    let value_keys = collect_sorted_keys(root_obj, &profile_names, "values");

    // Parse default profile
    let default_profile = root_obj.get("_").cloned().unwrap_or(serde_json::Value::Object(Default::default()));

    // Build per-profile data arrays
    let mut profile_color_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    let mut profile_edge_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    let mut profile_corner_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    let mut profile_value_data: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();

    for pname in &profile_names {
        let profile = root_obj.get(pname.as_str()).cloned().unwrap_or_default();

        let colors: Vec<_> = color_keys.iter().map(|key| {
            let val = get_val(&profile, "colors", key).or_else(|| get_val(&default_profile, "colors", key));
            parse_color_token(val.as_ref())
        }).collect();

        let edges: Vec<_> = edge_keys.iter().map(|key| {
            let val = get_val(&profile, "edges", key).or_else(|| get_val(&default_profile, "edges", key));
            parse_edges_token(val.as_ref())
        }).collect();

        let corners: Vec<_> = corner_keys.iter().map(|key| {
            let val = get_val(&profile, "corners", key).or_else(|| get_val(&default_profile, "corners", key));
            parse_corners_token(val.as_ref())
        }).collect();

        let values: Vec<_> = value_keys.iter().map(|key| {
            let val = get_val(&profile, "values", key).or_else(|| get_val(&default_profile, "values", key));
            parse_f32_token(val.as_ref())
        }).collect();

        profile_color_data.push(colors);
        profile_edge_data.push(edges);
        profile_corner_data.push(corners);
        profile_value_data.push(values);
    }

    // Generate ProfileId enum variants
    let enum_variants: Vec<_> = profile_names.iter().enumerate().map(|(i, name)| {
        let variant = if name == "_" { format_ident!("Default") } else { format_ident!("{}", to_pascal_case(name)) };
        let idx = i;
        quote! { #variant = #idx }
    }).collect();

    let name_strs: Vec<_> = profile_names.iter().map(|n| {
        let s = if n == "_" { "default" } else { n.as_str() };
        quote! { #s }
    }).collect();

    let num_profiles = profile_names.len();
    let num_colors = color_keys.len();
    let num_edges = edge_keys.len();
    let num_corners = corner_keys.len();
    let num_values = value_keys.len();

    // Generate static arrays per profile
    let color_arrays: Vec<_> = profile_color_data.iter().enumerate().map(|(i, colors)| {
        let name = format_ident!("COLORS_{}", i);
        quote! { static #name: [aurora_theme::Color; #num_colors] = [#(#colors),*]; }
    }).collect();

    let edge_arrays: Vec<_> = profile_edge_data.iter().enumerate().map(|(i, edges)| {
        let name = format_ident!("EDGES_{}", i);
        quote! { static #name: [aurora_theme::Edges; #num_edges] = [#(#edges),*]; }
    }).collect();

    let corner_arrays: Vec<_> = profile_corner_data.iter().enumerate().map(|(i, corners)| {
        let name = format_ident!("CORNERS_{}", i);
        quote! { static #name: [aurora_theme::Corners; #num_corners] = [#(#corners),*]; }
    }).collect();

    let value_arrays: Vec<_> = profile_value_data.iter().enumerate().map(|(i, vals)| {
        let name = format_ident!("VALUES_{}", i);
        quote! { static #name: [f32; #num_values] = [#(#vals),*]; }
    }).collect();

    // Refs for the profile tables
    let color_refs: Vec<_> = (0..num_profiles).map(|i| { let n = format_ident!("COLORS_{}", i); quote! { &#n } }).collect();
    let edge_refs: Vec<_> = (0..num_profiles).map(|i| { let n = format_ident!("EDGES_{}", i); quote! { &#n } }).collect();
    let corner_refs: Vec<_> = (0..num_profiles).map(|i| { let n = format_ident!("CORNERS_{}", i); quote! { &#n } }).collect();
    let value_refs: Vec<_> = (0..num_profiles).map(|i| { let n = format_ident!("VALUES_{}", i); quote! { &#n } }).collect();

    // Accessor functions per key
    let color_fns: Vec<_> = color_keys.iter().enumerate().map(|(i, key)| {
        let fn_name = format_ident!("{}", key);
        quote! { pub fn #fn_name() -> aurora_theme::Color { PROFILE_COLORS[aurora_theme::current_profile()][#i] } }
    }).collect();

    let edge_fns: Vec<_> = edge_keys.iter().enumerate().map(|(i, key)| {
        let fn_name = format_ident!("{}", key);
        quote! { pub fn #fn_name() -> aurora_theme::Edges { PROFILE_EDGES[aurora_theme::current_profile()][#i] } }
    }).collect();

    let corner_fns: Vec<_> = corner_keys.iter().enumerate().map(|(i, key)| {
        let fn_name = format_ident!("{}", key);
        quote! { pub fn #fn_name() -> aurora_theme::Corners { PROFILE_CORNERS[aurora_theme::current_profile()][#i] } }
    }).collect();

    let value_fns: Vec<_> = value_keys.iter().enumerate().map(|(i, key)| {
        let fn_name = format_ident!("{}", key);
        quote! { pub fn #fn_name() -> f32 { PROFILE_VALUES[aurora_theme::current_profile()][#i] } }
    }).collect();

    // Build well-known slot registration: for each profile, build a 26-color array
    // mapping well-known names to their aurora_theme::slots indices.
    let well_known: &[(&str, usize)] = &[
        ("background", 0), ("foreground", 1), ("muted", 2), ("muted_foreground", 3),
        ("border", 4), ("input_border", 5), ("ring", 6), ("primary", 7),
        ("primary_foreground", 8), ("secondary", 9), ("secondary_foreground", 10),
        ("accent", 11), ("accent_foreground", 12), ("destructive", 13),
        ("destructive_foreground", 14), ("success", 15), ("success_foreground", 16),
        ("warning", 17), ("warning_foreground", 18), ("info", 19),
        ("info_foreground", 20), ("card", 21), ("card_foreground", 22),
        ("popover", 23), ("popover_foreground", 24), ("overlay", 25),
    ];

    let mut slot_colors_per_profile: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
    for pname in &profile_names {
        let profile = root_obj.get(pname.as_str()).cloned().unwrap_or_default();
        let mut slots: Vec<proc_macro2::TokenStream> = Vec::new();
        for &(name, _slot) in well_known {
            let val = get_val(&profile, "colors", name)
                .or_else(|| get_val(&default_profile, "colors", name));
            if let Some(v) = val {
                slots.push(parse_color_token(Some(&v)));
            } else {
                // Use the built-in default from aurora_theme
                let idx = _slot;
                slots.push(quote! { aurora_theme::DEFAULT_COLORS[#idx] });
            }
        }
        slot_colors_per_profile.push(slots);
    }

    let slot_arrays: Vec<_> = slot_colors_per_profile.iter().enumerate().map(|(i, slots)| {
        let name = format_ident!("SLOT_COLORS_{}", i);
        quote! { static #name: [aurora_theme::Color; 26] = [#(#slots),*]; }
    }).collect();
    let slot_names: Vec<_> = (0..num_profiles).map(|i| format_ident!("SLOT_COLORS_{}", i)).collect();

    let output = quote! {
        #[allow(dead_code)]
        pub mod theme {
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
                unsafe { std::mem::transmute::<usize, ProfileId>(id.min(#num_profiles - 1)) }
            }

            pub fn profile_count() -> usize { #num_profiles }

            pub fn profile_name(id: ProfileId) -> &'static str {
                static NAMES: [&str; #num_profiles] = [#(#name_strs),*];
                NAMES[id as usize]
            }

            // Well-known slot color arrays (for aurora_theme registration)
            #(#slot_arrays)*

            /// Register the user theme with aurora_theme so all widgets
            /// pick up the correct colors. Call once at startup.
            pub fn init() {
                aurora_theme::register_colors(
                    vec![#(#slot_names.to_vec()),*]
                );
            }

            // Custom theme color arrays (for direct theme::colors::* access)
            #(#color_arrays)*
            #(#edge_arrays)*
            #(#corner_arrays)*
            #(#value_arrays)*

            static PROFILE_COLORS: [&[aurora_theme::Color; #num_colors]; #num_profiles] = [#(#color_refs),*];
            static PROFILE_EDGES: [&[aurora_theme::Edges; #num_edges]; #num_profiles] = [#(#edge_refs),*];
            static PROFILE_CORNERS: [&[aurora_theme::Corners; #num_corners]; #num_profiles] = [#(#corner_refs),*];
            static PROFILE_VALUES: [&[f32; #num_values]; #num_profiles] = [#(#value_refs),*];

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

/// Collect all unique keys for a category across all profiles, sorted alphabetically.
fn collect_sorted_keys(
    root: &serde_json::Map<String, serde_json::Value>,
    profiles: &[String],
    category: &str,
) -> Vec<String> {
    let mut all_keys = BTreeMap::new();
    for pname in profiles {
        if let Some(profile) = root.get(pname.as_str()) {
            if let Some(cat) = profile.get(category) {
                if let Some(obj) = cat.as_object() {
                    for key in obj.keys() {
                        all_keys.entry(key.clone()).or_insert(());
                    }
                }
            }
        }
    }
    all_keys.into_keys().collect()
}

fn get_val(profile: &serde_json::Value, category: &str, key: &str) -> Option<serde_json::Value> {
    profile.get(category)?.get(key).cloned()
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
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

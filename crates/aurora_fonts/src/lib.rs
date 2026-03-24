#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Token};

const METADATA_URL: &str = "https://fonts.google.com/metadata/fonts";
const CSS_API: &str = "https://fonts.googleapis.com/css2";
const CACHE_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60); // 30 days

/// Fetches font families from [Google Fonts](https://fonts.google.com) at
/// compile time and embeds the `.ttf` bytes directly in your binary.
///
/// Family names are **case-insensitive**: `"Roboto"`, `"roboto"`, and
/// `"ROBOTO"` all work.
///
/// # Usage
///
/// ```rust,ignore
/// aurora_fonts::font_families!("Roboto", "Open Sans");
///
/// fn main() {
///     let bytes: &[u8] = FontFamily::roboto().regular();
///     let bold: &[u8] = FontFamily::roboto().bold();
///     let italic: &[u8] = FontFamily::roboto().italic();
///
///     // Numeric weight access:
///     let w500: Option<&[u8]> = FontFamily::roboto().weight(500);
///
///     // Multiple weights at once:
///     let fonts: Vec<&[u8]> = FontFamily::roboto().with_weights(&[100, 400, 700]);
/// }
/// ```
#[proc_macro]
pub fn font_families(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FamilyInput);
    let family_names = &input.families;

    if family_names.is_empty() {
        return quote! { compile_error!("font_families! requires at least one font family name"); }
            .into();
    }

    let cache_dir = cache_dir();

    // Fetch the global metadata once (cached)
    let metadata = match fetch_metadata(&cache_dir) {
        Ok(m) => m,
        Err(e) => {
            let msg = format!("Failed to fetch Google Fonts metadata: {e}");
            return quote! { compile_error!(#msg); }.into();
        }
    };

    let mut family_struct_methods = Vec::new();
    let mut family_structs = Vec::new();
    let mut by_name_arms = Vec::new();
    let mut dyn_variants = Vec::new();
    let mut dyn_weight_arms = Vec::new();
    let mut dyn_weight_italic_arms = Vec::new();

    for family_input in family_names {
        let family_lower = family_input.to_lowercase();

        // Find the canonical family name from metadata (case-insensitive)
        let family_meta = metadata
            .iter()
            .find(|m| m.family.to_lowercase() == family_lower);

        let family_meta = match family_meta {
            Some(m) => m,
            None => {
                let msg = format!(
                    "Font family '{}' not found on Google Fonts",
                    family_input
                );
                return quote! { compile_error!(#msg); }.into();
            }
        };

        let canonical_name = &family_meta.family;

        // Fetch font files for this family
        let font_data = match fetch_family_fonts(canonical_name, &family_meta.variants, &cache_dir)
        {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("Failed to fetch fonts for '{}': {}", canonical_name, e);
                return quote! { compile_error!(#msg); }.into();
            }
        };

        let method_name = format_ident!("{}", sanitize_ident(&family_lower));
        let struct_name = format_ident!("{}Family", to_pascal_case(canonical_name));
        let variant_name = format_ident!("{}", to_pascal_case(canonical_name));

        // Generate weight methods
        let mut weight_methods = Vec::new();
        let mut weight_match_arms = Vec::new();
        let mut weight_italic_match_arms = Vec::new();

        for variant in &font_data {
            let weight = variant.weight;
            let is_italic = variant.italic;
            let bytes = &variant.bytes;
            let byte_literals: Vec<_> = bytes.iter().map(|b| quote! { #b }).collect();

            let fn_name = weight_method_name(weight, is_italic);
            let fn_ident = format_ident!("{}", fn_name);

            weight_methods.push(quote! {
                pub fn #fn_ident(&self) -> &'static [u8] {
                    static DATA: &[u8] = &[#(#byte_literals),*];
                    DATA
                }
            });

            let weight_lit = weight;
            if is_italic {
                weight_italic_match_arms.push(quote! {
                    #weight_lit => Some(self.#fn_ident()),
                });
            } else {
                weight_match_arms.push(quote! {
                    #weight_lit => Some(self.#fn_ident()),
                });
            }
        }

        family_structs.push(quote! {
            pub struct #struct_name;

            impl #struct_name {
                #(#weight_methods)*

                /// Look up a normal (non-italic) variant by numeric weight.
                pub fn weight(&self, weight: u16) -> Option<&'static [u8]> {
                    match weight {
                        #(#weight_match_arms)*
                        _ => None,
                    }
                }

                /// Look up an italic variant by numeric weight.
                pub fn weight_italic(&self, weight: u16) -> Option<&'static [u8]> {
                    match weight {
                        #(#weight_italic_match_arms)*
                        _ => None,
                    }
                }

                /// Get font bytes for multiple weights at once (non-italic).
                /// Skips weights that are not available.
                pub fn with_weights(&self, weights: &[u16]) -> Vec<&'static [u8]> {
                    weights.iter().filter_map(|&w| self.weight(w)).collect()
                }
            }
        });

        let name_lower = family_lower.as_str();
        let canonical_str = canonical_name.as_str();
        family_struct_methods.push(quote! {
            pub fn #method_name() -> #struct_name { #struct_name }
        });
        by_name_arms.push(quote! {
            #name_lower => Some(DynFontFamily::#variant_name),
        });
        // Also match canonical casing
        if name_lower != canonical_str.to_lowercase().as_str() {
            by_name_arms.push(quote! {
                #canonical_str => Some(DynFontFamily::#variant_name),
            });
        }
        dyn_variants.push(variant_name.clone());
        dyn_weight_arms.push(quote! {
            DynFontFamily::#variant_name => #struct_name.weight(weight),
        });
        dyn_weight_italic_arms.push(quote! {
            DynFontFamily::#variant_name => #struct_name.weight_italic(weight),
        });
    }

    let expanded = quote! {
        pub struct FontFamily;

        impl FontFamily {
            #(#family_struct_methods)*

            /// Look up a font family by name (case-insensitive).
            pub fn by_name(name: &str) -> Option<DynFontFamily> {
                match name.to_lowercase().as_str() {
                    #(#by_name_arms)*
                    _ => None,
                }
            }
        }

        #(#family_structs)*

        /// Dynamic font family handle for runtime selection.
        pub enum DynFontFamily {
            #(#dyn_variants,)*
        }

        impl DynFontFamily {
            /// Look up a normal variant by numeric weight.
            pub fn weight(&self, weight: u16) -> Option<&'static [u8]> {
                match self {
                    #(#dyn_weight_arms)*
                }
            }

            /// Look up an italic variant by numeric weight.
            pub fn weight_italic(&self, weight: u16) -> Option<&'static [u8]> {
                match self {
                    #(#dyn_weight_italic_arms)*
                }
            }
        }
    };

    expanded.into()
}

// ---------------------------------------------------------------------------
// Input parsing
// ---------------------------------------------------------------------------

struct FamilyInput {
    families: Vec<String>,
}

impl Parse for FamilyInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut families = Vec::new();
        while !input.is_empty() {
            let lit: LitStr = input.parse()?;
            families.push(lit.value());
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { families })
    }
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

struct FamilyMeta {
    family: String,
    variants: Vec<String>, // e.g. ["100", "300", "400", "700", "100i", "400i"]
}

struct FontVariant {
    weight: u16,
    italic: bool,
    bytes: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Metadata fetching
// ---------------------------------------------------------------------------

fn cache_dir() -> PathBuf {
    let base = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("target").join(".fonts-cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn is_cache_valid(path: &Path) -> bool {
    path.exists()
        && fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.elapsed().unwrap_or(Duration::MAX) < CACHE_TTL)
            .unwrap_or(false)
}

fn fetch_metadata(cache_dir: &Path) -> Result<Vec<FamilyMeta>, String> {
    let cached = cache_dir.join("metadata.json");

    let json_str = if is_cache_valid(&cached) {
        fs::read_to_string(&cached).map_err(|e| format!("cache read failed: {e}"))?
    } else {
        let body = http_get(METADATA_URL)?;
        let _ = fs::write(&cached, &body);
        body
    };

    parse_metadata(&json_str)
}

fn parse_metadata(json_str: &str) -> Result<Vec<FamilyMeta>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("metadata JSON parse error: {e}"))?;

    let families = parsed
        .get("familyMetadataList")
        .and_then(|v| v.as_array())
        .ok_or("missing 'familyMetadataList' in metadata")?;

    let mut result = Vec::new();
    for entry in families {
        let family = entry
            .get("family")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let fonts = entry.get("fonts").and_then(|v| v.as_object());
        let mut variants = Vec::new();
        if let Some(fonts) = fonts {
            for key in fonts.keys() {
                variants.push(key.clone());
            }
        }

        if !family.is_empty() && !variants.is_empty() {
            result.push(FamilyMeta { family, variants });
        }
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Font file fetching
// ---------------------------------------------------------------------------

fn fetch_family_fonts(
    family: &str,
    variants: &[String],
    cache_dir: &Path,
) -> Result<Vec<FontVariant>, String> {
    let family_dir = cache_dir.join(family.to_lowercase().replace(' ', "_"));
    let _ = fs::create_dir_all(&family_dir);

    // Parse available weights and italics
    let mut axes: Vec<(u16, bool)> = Vec::new();
    for v in variants {
        let italic = v.ends_with('i');
        let weight_str = if italic {
            &v[..v.len() - 1]
        } else {
            v.as_str()
        };
        if let Ok(w) = weight_str.parse::<u16>() {
            axes.push((w, italic));
        }
    }
    axes.sort();

    // Build CSS API URL with all axes
    let axis_params: Vec<String> = axes
        .iter()
        .map(|(w, italic)| format!("{},{}", if *italic { 1 } else { 0 }, w))
        .collect();
    let family_param = family.replace(' ', "+");
    let css_url = format!(
        "{}?family={}:ital,wght@{}&display=swap",
        CSS_API,
        family_param,
        axis_params.join(";")
    );

    // Fetch CSS and parse font URLs
    let css = http_get_with_ua(&css_url)?;
    let url_map = parse_css_font_urls(&css)?;

    // Download each font variant
    let mut result = Vec::new();
    for (weight, italic) in &axes {
        let key = if *italic {
            format!("{}i", weight)
        } else {
            format!("{}", weight)
        };

        let url = match url_map.get(&key) {
            Some(u) => u,
            None => continue, // Skip if CSS didn't include this variant
        };

        let cache_file = family_dir.join(format!("{}.ttf", key));
        let bytes = if cache_file.exists() && is_cache_valid(&cache_file) {
            fs::read(&cache_file).map_err(|e| format!("cache read failed: {e}"))?
        } else {
            let font_bytes = http_get_bytes(url)?;
            let _ = fs::write(&cache_file, &font_bytes);
            font_bytes
        };

        result.push(FontVariant {
            weight: *weight,
            italic: *italic,
            bytes,
        });
    }

    if result.is_empty() {
        return Err(format!("no font files found for '{family}'"));
    }

    Ok(result)
}

fn parse_css_font_urls(css: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let mut map = std::collections::HashMap::new();

    // Parse @font-face blocks from CSS
    let mut current_weight: Option<String> = None;
    let mut current_style: Option<String> = None;
    let mut current_url: Option<String> = None;

    for line in css.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("font-weight:") {
            let val = trimmed
                .trim_start_matches("font-weight:")
                .trim()
                .trim_end_matches(';')
                .trim();
            current_weight = Some(val.to_string());
        } else if trimmed.starts_with("font-style:") {
            let val = trimmed
                .trim_start_matches("font-style:")
                .trim()
                .trim_end_matches(';')
                .trim();
            current_style = Some(val.to_string());
        } else if trimmed.starts_with("src:") {
            // Extract URL from src: url(...) format(...)
            if let Some(start) = trimmed.find("url(") {
                let rest = &trimmed[start + 4..];
                if let Some(end) = rest.find(')') {
                    current_url = Some(rest[..end].to_string());
                }
            }
        } else if trimmed == "}" {
            // End of @font-face block
            if let (Some(weight), Some(style), Some(url)) =
                (&current_weight, &current_style, &current_url)
            {
                let key = if style == "italic" {
                    format!("{}i", weight)
                } else {
                    weight.clone()
                };
                map.insert(key, url.clone());
            }
            current_weight = None;
            current_style = None;
            current_url = None;
        }
    }

    Ok(map)
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

fn http_get(url: &str) -> Result<String, String> {
    ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP request failed for {url}: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response from {url}: {e}"))
}

fn http_get_with_ua(url: &str) -> Result<String, String> {
    // Google Fonts CSS API returns different formats based on User-Agent.
    // A generic UA gets .ttf files (which is what we want).
    ureq::get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; aurora_fonts/0.1.0)",
        )
        .call()
        .map_err(|e| format!("HTTP request failed for {url}: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response from {url}: {e}"))
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut buf = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP request failed for {url}: {e}"))?
        .body_mut()
        .as_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read response from {url}: {e}"))?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Name utilities
// ---------------------------------------------------------------------------

fn weight_method_name(weight: u16, italic: bool) -> String {
    let base = match weight {
        100 => "thin",
        200 => "extra_light",
        300 => "light",
        400 => "regular",
        500 => "medium",
        600 => "semi_bold",
        700 => "bold",
        800 => "extra_bold",
        900 => "black",
        _ => return format!("w{}{}", weight, if italic { "_italic" } else { "" }),
    };

    if italic {
        if weight == 400 {
            "italic".to_string()
        } else {
            format!("{base}_italic")
        }
    } else {
        base.to_string()
    }
}

fn sanitize_ident(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{s}")
    } else {
        s
    }
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

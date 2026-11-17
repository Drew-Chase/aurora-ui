//! `icon_sets!`: embeds the SVGs an application uses from Iconify icon sets.
//!
//! The expansion holds the icons a crate embeds plus a few declarations per
//! set. The accessor for every icon of an enabled set lives in the
//! `aurora_iconify` facade, so an editor indexes it once rather than in every
//! crate that uses icons.

mod cache;
mod codegen;
mod fetch;
mod input;
mod scan;

use aurora_iconify_catalog::{self as catalog, index};
use cache::Cache;
use codegen::{Data, SetCode};
use fetch::Api;
use input::{Mode, SetSpec};
use proc_macro::TokenStream;
use scan::Words;
use std::path::Path;

const API_BASE: &str = match option_env!("ICONIFY_API_URL") {
    Some(v) => v,
    None => "https://api.iconify.design",
};

/// Largest SVG data written into the expansion. Bigger selections go to cache
/// files, so the expansion stays small.
const INLINE_LIMIT: usize = 64 * 1024;

/// Embeds the SVGs your crate uses from [iconify.design](https://iconify.design)
/// icon sets, and declares the accessors for them.
///
/// Enable each set as a feature of `aurora_iconify`, then list the sets here:
///
/// ```ignore
/// aurora_iconify::icon_sets!("mage", "lucide");
///
/// let svg: &str = Icon::mage().calendar_2_fill();
/// let svg: Option<&str> = Icon::mage().by_name("calendar-2-fill");
/// let svg: Option<&str> = Icon::from_set("mage").and_then(|s| s.by_name("calendar-2-fill"));
/// ```
///
/// Every icon of a set has an accessor, so editors can complete all of them,
/// but only icons that the crate's source refers to are embedded. To embed icons
/// that are only looked up by a name computed at runtime, list them or ask for
/// the whole set:
///
/// ```ignore
/// aurora_iconify::icon_sets!(
///     "mage",                         // icons referenced in this crate
///     "lucide": ["arrow-left", "x"],  // ...plus these, always
///     "tabler": all,                  // every icon in the set
/// );
/// ```
///
/// Icon names come from the catalog bundled with `aurora_iconify`, so the build
/// downloads only the SVGs, and only when they are not cached yet.
#[proc_macro]
pub fn icon_sets(input: TokenStream) -> TokenStream {
    let code = expand(&input.to_string()).unwrap_or_else(|message| compile_error(&message));
    code.parse().unwrap_or_else(|e| {
        compile_error(&format!("aurora_iconify generated invalid code: {e}"))
            .parse()
            .expect("compile_error! invocation parses")
    })
}

/// Generates the accessors of one icon set. The facade's `sets` module calls it
/// once per set, behind that set's feature, so the accessors live in
/// `aurora_iconify` and an editor indexes them there, not in the crates that use
/// icons. Not meant to be called directly.
#[doc(hidden)]
#[proc_macro]
pub fn __set_accessors(input: TokenStream) -> TokenStream {
    let code = set_accessors(&input.to_string()).unwrap_or_else(|message| compile_error(&message));
    code.parse().unwrap_or_else(|e| {
        compile_error(&format!("aurora_iconify generated invalid code: {e}"))
            .parse()
            .expect("compile_error! invocation parses")
    })
}

/// The accessor source for the set named by the string literal `input`.
fn set_accessors(input: &str) -> Result<String, String> {
    let prefix = input
        .trim()
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .ok_or("expected the name of an icon set as a string literal")?;
    let set =
        catalog::set(prefix).ok_or_else(|| format!("there is no icon set named \"{prefix}\""))?;
    Ok(catalog::facade_set_source(set, API_BASE))
}

fn compile_error(message: &str) -> String {
    format!("::core::compile_error!({message:?});")
}

fn expand(input: &str) -> Result<String, String> {
    let specs = input::parse(input)?;
    let cache = Cache::locate();
    let api = Api::new(API_BASE, cache.outage_marker());
    // The sources are only read if some set needs to know what is referenced.
    let mut words: Option<Words> = None;

    let sets = specs
        .iter()
        .map(|spec| {
            resolve(spec, &cache, &api, &mut words)
                .map_err(|e| format!("icon set '{}': {e}", spec.prefix))
        })
        .collect::<Result<Vec<SetCode>, String>>()?;
    Ok(codegen::generate(&sets))
}

/// Works out which icons of one declared set to embed, downloads what is
/// missing, and returns the data the generated code needs.
fn resolve(
    spec: &SetSpec,
    cache: &Cache,
    api: &Api,
    words: &mut Option<Words>,
) -> Result<SetCode, String> {
    let set = catalog::set(&spec.prefix)
        .ok_or_else(|| format!("there is no icon set named \"{}\"", spec.prefix))?;
    let prefix = set.prefix;
    // Sorted, so the position of a name is its icon ID.
    let names: Vec<&str> = set.names().collect();

    // The icons to embed, and whether the crate clearly asks for each one.
    let (wanted, required): (Vec<usize>, Vec<bool>) = match &spec.mode {
        Mode::All => (0..names.len()).map(|i| (i, false)).unzip(),
        Mode::Referenced { always } => {
            if let Some(unknown) = always
                .iter()
                .find(|icon| names.binary_search(&icon.as_str()).is_err())
            {
                return Err(format!("there is no icon named \"{unknown}\""));
            }
            let words = words.get_or_insert_with(scan_sources);
            let mut picked = Vec::new();
            for (i, name) in names.iter().enumerate() {
                let listed = always.iter().any(|icon| icon.as_str() == *name);
                let referenced = listed || words.references(name);
                if referenced || words.mentions(name) {
                    picked.push((i, referenced));
                }
            }
            picked.into_iter().unzip()
        }
    };

    // A selection that was built before is reused as it is.
    let key = table_key(prefix, &wanted, &names);
    if let Some((index, blob)) = cache.existing_table(prefix, &key) {
        return Ok(SetCode {
            prefix,
            data: Data::File { index, blob },
        });
    }

    let wanted_names: Vec<String> = wanted.iter().map(|&i| names[i].to_string()).collect();
    let (svgs, error) = cache.icons(prefix, &wanted_names, |missing| api.icons(prefix, missing));

    // An icon the crate clearly asks for must be available, or its accessor would
    // panic at run time. A word that only happens to equal an icon name is skipped.
    let mut needed = Vec::new();
    for ((&i, svg), &is_required) in wanted.iter().zip(&svgs).zip(&required) {
        if svg.is_none() && is_required {
            needed.push(names[i]);
        }
    }
    match error {
        // A whole set is only useful complete, so any request failure is an error.
        Some(error) if matches!(spec.mode, Mode::All) => {
            return Err(format!("could not download icons: {error}"));
        }
        Some(error) if !needed.is_empty() => {
            return Err(format!("could not download {}: {error}", needed.join(", ")));
        }
        None if !needed.is_empty() => {
            return Err(format!(
                "the Iconify API has no icon named {}; the icon catalog may be out of date (run `just iconify-catalog`)",
                needed.join(", ")
            ));
        }
        _ => {}
    }

    // Icons the API does not have are left out, and their accessors panic at run time.
    let mut blob = String::new();
    let mut entries = Vec::new();
    for (&i, svg) in wanted.iter().zip(svgs) {
        let Some(svg) = svg else { continue };
        entries.push(index::Entry {
            id: i as u32,
            name: names[i],
            start: blob.len() as u32,
            len: svg.len() as u32,
        });
        blob.push_str(&svg);
    }
    let index = index::write(&entries, blob.len() as u32);

    if matches!(spec.mode, Mode::Referenced { .. }) && blob.len() <= INLINE_LIMIT {
        return Ok(SetCode {
            prefix,
            data: Data::Inline { index, blob },
        });
    }
    let (index_path, blob_path) = cache.write_table(prefix, &key, &index, &blob)?;
    Ok(SetCode {
        prefix,
        data: Data::File {
            index: index_path,
            blob: blob_path,
        },
    })
}

/// The cache key of a selection. It depends on the catalog and on exactly which
/// icons were asked for, so a changed selection gets a new table.
fn table_key(prefix: &str, wanted: &[usize], names: &[&str]) -> String {
    let mut hash = catalog::Fnv1a::new();
    hash.update(&catalog::CATALOG_HASH.to_le_bytes());
    hash.update(prefix.as_bytes());
    for &i in wanted {
        hash.update(&(i as u32).to_le_bytes());
        hash.update(names[i].as_bytes());
    }
    format!("{:016x}", hash.finish())
}

/// Reads the sources of the crate being compiled. Without a manifest directory
/// there is nothing to scan, so no icon counts as referenced.
fn scan_sources() -> Words {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(|dir| Words::scan_dir(Path::new(&dir)))
        .unwrap_or_default()
}

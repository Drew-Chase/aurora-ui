#![doc = include_str!("../README.md")]

mod cache;
mod codegen;
mod fetch;
mod input;
mod names;
mod scan;

use cache::{Cache, Span};
use codegen::{Blob, IconSet};
use fetch::Api;
use input::{Mode, SetSpec};
use proc_macro::TokenStream;
use scan::Words;
use std::path::{Path, PathBuf};

const API_BASE: &str = match option_env!("ICONIFY_API_URL") {
    Some(v) => v,
    None => "https://api.iconify.design",
};

/// Generates a type-safe Rust API for icon sets from
/// [iconify.design](https://iconify.design), embedding the SVGs of the icons
/// your crate uses as string data.
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
/// Every icon in a set gets a method, so editors can complete all of them,
/// but only icons that the crate's source refers to are downloaded and
/// embedded. To embed icons that are only looked up by a name computed at
/// runtime, list them or ask for the whole set:
///
/// ```ignore
/// aurora_iconify::icon_sets!(
///     "mage",                         // icons referenced in this crate
///     "lucide": ["arrow-left", "x"],  // ...plus these, always
///     "tabler": all,                  // every icon in the set
/// );
/// ```
///
/// Icon name lists are cached for 7 days and downloaded icons indefinitely,
/// in `target/.iconify-cache/`.
#[proc_macro]
pub fn icon_sets(input: TokenStream) -> TokenStream {
    let code = expand(&input.to_string()).unwrap_or_else(|message| compile_error(&message));
    code.parse().unwrap_or_else(|e| {
        compile_error(&format!("aurora_iconify generated invalid code: {e}"))
            .parse()
            .expect("compile_error! invocation parses")
    })
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

    let resolved = specs
        .iter()
        .map(|spec| {
            resolve(spec, &cache, &api, &mut words)
                .map_err(|e| format!("icon set '{}': {e}", spec.prefix))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let sets: Vec<IconSet<'_>> = resolved.iter().map(Resolved::as_icon_set).collect();
    Ok(codegen::generate(&sets, API_BASE))
}

/// One icon set with everything needed to generate its code.
struct Resolved<'a> {
    prefix: &'a str,
    /// Every icon in the set, sorted.
    names: Vec<String>,
    /// Indices into `names` of the icons with data, ascending.
    embedded: Vec<usize>,
    /// Where each embedded icon sits in the blob; parallel to `embedded`.
    spans: Vec<Span>,
    blob: OwnedBlob,
}

enum OwnedBlob {
    Inline(String),
    File(PathBuf),
}

impl Resolved<'_> {
    fn as_icon_set(&self) -> IconSet<'_> {
        IconSet {
            prefix: self.prefix,
            names: &self.names,
            embedded: self
                .embedded
                .iter()
                .zip(&self.spans)
                .map(|(&i, &span)| (self.names[i].as_str(), span))
                .collect(),
            blob: match &self.blob {
                OwnedBlob::Inline(text) => Blob::Inline(text),
                OwnedBlob::File(path) => Blob::File(path),
            },
        }
    }
}

fn resolve<'a>(
    spec: &'a SetSpec,
    cache: &Cache,
    api: &Api,
    words: &mut Option<Words>,
) -> Result<Resolved<'a>, String> {
    let prefix = spec.prefix.as_str();
    let mut names = cache.names(prefix, || api.icon_names(prefix))?;
    // The generated lookup is a binary search, so order is load-bearing.
    names.sort_unstable();
    names.dedup();

    let fetch = |missing: &[String]| api.icons(prefix, missing);

    match &spec.mode {
        Mode::All => {
            let (path, spans) = cache.full_set(prefix, &names, fetch)?;
            Ok(Resolved {
                prefix,
                embedded: (0..names.len()).collect(),
                names,
                spans,
                blob: OwnedBlob::File(path),
            })
        }
        Mode::Referenced { always } => {
            if let Some(unknown) = always
                .iter()
                .find(|icon| names.binary_search(icon).is_err())
            {
                return Err(format!("there is no icon named \"{unknown}\""));
            }
            let words = words.get_or_insert_with(scan_sources);

            let wanted: Vec<usize> = (0..names.len())
                .filter(|&i| always.contains(&names[i]) || words.mentions(&names[i]))
                .collect();
            let wanted_names: Vec<String> = wanted.iter().map(|&i| names[i].clone()).collect();
            let (svgs, error) = cache.icons(prefix, &wanted_names, fetch);

            // A word that merely coincides with an icon name is not worth
            // failing the build over, but an icon the crate clearly asks for
            // is: its accessor would panic at runtime.
            if let Some(error) = error {
                let required: Vec<&str> = wanted_names
                    .iter()
                    .zip(&svgs)
                    .filter(|(name, svg)| {
                        svg.is_none() && (always.contains(name) || words.references(name))
                    })
                    .map(|(name, _)| name.as_str())
                    .collect();
                if !required.is_empty() {
                    return Err(format!(
                        "could not download {}: {error}",
                        required.join(", ")
                    ));
                }
            }

            let (embedded, svgs): (Vec<usize>, Vec<String>) = wanted
                .into_iter()
                .zip(svgs)
                .filter_map(|(i, svg)| Some((i, svg?)))
                .unzip();
            let (blob, spans) = cache::concat(&svgs);
            Ok(Resolved {
                prefix,
                names,
                embedded,
                spans,
                blob: OwnedBlob::Inline(blob),
            })
        }
    }
}

/// Reads the sources of the crate being compiled. Without a manifest
/// directory there is nothing to scan, so no icon counts as referenced.
fn scan_sources() -> Words {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(|dir| Words::scan_dir(Path::new(&dir)))
        .unwrap_or_default()
}

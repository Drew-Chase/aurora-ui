//! Maintenance commands for the `aurora_iconify` crates. Run from anywhere in the workspace:
//!
//! ```text
//! cargo run -p xtask -- iconify-catalog
//! cargo run -p xtask -- iconify-facade
//! ```
//!
//! `iconify-catalog` needs network access. It rewrites the icon name lists in
//! `crates/aurora_iconify_catalog/`, then the facade's set list (`src/sets.rs`) and
//! the set features in `crates/aurora_iconify/Cargo.toml`. `iconify-facade` does the
//! last step alone, from the catalog compiled into this binary, without network
//! access. Output is sorted and deterministic, so a refresh with no upstream
//! changes changes no files.

use aurora_iconify_catalog::{
    CATALOG_HASH, SETS, accessor_type_name, catalog_hash, feature_env_var, is_valid_name,
    sanitize_ident, sets_module_source, variant_name,
};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_API: &str = "https://api.iconify.design";
/// crates.io accepts at most 300 features per crate.
const MAX_FEATURES: usize = 300;
/// `default` and `previews` are not icon sets.
const FIXED_FEATURES: usize = 2;
const FEATURES_BEGIN: &str =
    "# BEGIN GENERATED SET FEATURES (cargo run -p xtask -- iconify-catalog)";
const FEATURES_END: &str = "# END GENERATED SET FEATURES";
/// Pause between the per-set requests. The public API rate-limits bursts (a `429`
/// with a `Retry-After` of about an hour), so use `ICONIFY_API_URL` for a mirror
/// that does not; a 1-second pause is enough there.
const REQUEST_GAP: Duration = Duration::from_secs(1);
/// Attempts per request before giving up.
const MAX_ATTEMPTS: u32 = 8;

/// One icon set as the API reports it.
struct FetchedSet {
    prefix: String,
    /// Sorted, unique, valid names.
    names: Vec<String>,
    /// The icon count the API reports. It can differ from `names.len()`
    /// because hidden icons and aliases are not listed.
    total: Option<u64>,
}

fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("iconify-catalog") => iconify_catalog(),
        Some("iconify-facade") => iconify_facade(),
        _ => Err("usage: cargo run -p xtask -- iconify-catalog | iconify-facade".to_string()),
    };
    if let Err(message) = result {
        eprintln!("xtask: {message}");
        std::process::exit(1);
    }
}

fn iconify_catalog() -> Result<(), String> {
    let root = workspace_root()?;
    let base = std::env::var("ICONIFY_API_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| DEFAULT_API.to_string());
    // Status codes are handled by `fetch`, so the agent must not turn them into errors.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .http_status_as_error(false)
        .build()
        .into();

    let listing: Value = serde_json::from_str(&fetch(&agent, &format!("{base}/collections"))?)
        .map_err(|e| format!("/collections is not JSON: {e}"))?;
    let listing = listing
        .as_object()
        .ok_or("/collections did not return an object")?;

    // Valid `/collection` responses are kept on disk, so a run that is interrupted
    // (or rate limited) resumes without asking the API for the same sets again.
    let cache_dir = root
        .join("target")
        .join("iconify-catalog")
        .join("collection");
    fs::create_dir_all(&cache_dir).map_err(|e| fail(&cache_dir, e))?;

    let mut sets = Vec::with_capacity(listing.len());
    let mut skipped = Vec::new();
    for (index, (prefix, info)) in listing.iter().enumerate() {
        if !is_valid_name(prefix) || prefix.contains('_') {
            return Err(format!("unsupported icon set name {prefix:?}"));
        }
        let cache_file = cache_dir.join(format!("{prefix}.json"));
        let (body, cached) = match fs::read_to_string(&cache_file) {
            Ok(body) => (body, true),
            Err(_) => {
                eprintln!("xtask: {}/{} {prefix}", index + 1, listing.len());
                let body = fetch(&agent, &format!("{base}/collection?prefix={prefix}"))?;
                std::thread::sleep(REQUEST_GAP);
                (body, false)
            }
        };
        // A response that does not describe any icons leaves the set out, with a
        // warning. A failed request, though, stops the run: it would otherwise
        // silently drop sets when the API is down.
        match names_from_collection(&body) {
            Ok(names) if !names.is_empty() => {
                if !cached {
                    write_if_changed(&cache_file, &body)?;
                }
                sets.push(FetchedSet {
                    prefix: prefix.clone(),
                    names,
                    total: info.get("total").and_then(Value::as_u64),
                });
            }
            Ok(_) => skipped.push(format!("{prefix} (no icons)")),
            Err(error) => skipped.push(format!("{prefix} ({error})")),
        }
    }
    if !skipped.is_empty() {
        eprintln!(
            "xtask: left out {} sets the API did not describe: {}",
            skipped.len(),
            skipped.join(", ")
        );
    }
    if sets.is_empty() {
        return Err("the API described no icon sets".to_string());
    }
    sets.sort_by(|a, b| a.prefix.cmp(&b.prefix));

    let prefixes: Vec<&str> = sets.iter().map(|set| set.prefix.as_str()).collect();
    check_prefixes(&prefixes)?;
    let catalog_hash = write_catalog(&root, &sets)?;
    write_facade(&root, &prefixes, catalog_hash)?;

    let names: usize = sets.iter().map(|set| set.names.len()).sum();
    let short = sets
        .iter()
        .filter(|set| {
            set.total
                .is_some_and(|total| total != set.names.len() as u64)
        })
        .count();
    println!(
        "catalog: {} sets, {names} icon names ({short} sets list fewer names than the API's total)",
        sets.len()
    );
    Ok(())
}

/// Regenerates the facade's set list and feature block from the catalog compiled
/// into this binary. Needs no network access.
fn iconify_facade() -> Result<(), String> {
    let root = workspace_root()?;
    let prefixes: Vec<&str> = SETS.iter().map(|set| set.prefix).collect();
    check_prefixes(&prefixes)?;
    write_facade(&root, &prefixes, CATALOG_HASH)?;
    println!("facade: {} icon sets", prefixes.len());
    Ok(())
}

/// The workspace root: the directory above this crate.
fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask has no parent directory".to_string())
}

/// GETs `url`. A `429` waits out the `Retry-After` the API asks for; server
/// errors and dropped connections retry with a growing pause.
fn fetch(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let mut failures: u32 = 0;
    loop {
        let wait = match agent.get(url).call() {
            Ok(mut response) if response.status().is_success() => {
                return response
                    .body_mut()
                    .read_to_string()
                    .map_err(|e| format!("reading {url}: {e}"));
            }
            Ok(response) if response.status().as_u16() == 429 => {
                let seconds = response
                    .headers()
                    .get("retry-after")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|text| text.trim().parse::<u64>().ok())
                    .unwrap_or(60);
                eprintln!("xtask: the Iconify API is rate limiting; waiting {seconds}s");
                Duration::from_secs(seconds)
            }
            Ok(response) if response.status().is_server_error() => {
                Duration::from_secs(1 << failures.min(6))
            }
            Ok(response) => return Err(format!("GET {url}: HTTP {}", response.status())),
            Err(error) => {
                eprintln!("xtask: GET {url} failed: {error}");
                Duration::from_secs(1 << failures.min(6))
            }
        };
        failures += 1;
        if failures >= MAX_ATTEMPTS {
            return Err(format!("GET {url}: giving up after {failures} attempts"));
        }
        std::thread::sleep(wait);
    }
}

/// The sorted, unique, valid icon names from a `/collection` response. Icons
/// are listed under `uncategorized` and/or `categories`.
fn names_from_collection(body: &str) -> Result<Vec<String>, String> {
    let parsed: Value = serde_json::from_str(body).map_err(|e| format!("JSON parse error: {e}"))?;

    let uncategorized = parsed
        .get("uncategorized")
        .and_then(Value::as_array)
        .into_iter();
    let categorized = parsed
        .get("categories")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|categories| categories.values())
        .filter_map(Value::as_array);

    let names: BTreeSet<&str> = uncategorized
        .chain(categorized)
        .flatten()
        .filter_map(Value::as_str)
        .filter(|name| is_valid_name(name))
        .collect();
    Ok(names.into_iter().map(str::to_string).collect())
}

/// Fails when the sets cannot all become Cargo features and identifiers without clashing.
fn check_prefixes(prefixes: &[&str]) -> Result<(), String> {
    if prefixes.len() + FIXED_FEATURES > MAX_FEATURES {
        return Err(format!(
            "{} sets need more Cargo features than crates.io allows ({MAX_FEATURES}); split the catalog",
            prefixes.len()
        ));
    }
    let mut features = HashMap::new();
    let mut variants = HashMap::new();
    let mut accessors = HashMap::new();
    let mut methods = HashMap::new();
    for prefix in prefixes {
        claim(&mut features, feature_env_var(prefix), prefix)?;
        claim(&mut variants, variant_name(prefix), prefix)?;
        claim(&mut accessors, accessor_type_name(prefix), prefix)?;
        let method = sanitize_ident(prefix);
        if method == "from_set" {
            return Err(format!("set {prefix} would shadow Icon::from_set"));
        }
        claim(&mut methods, method, prefix)?;
    }
    Ok(())
}

fn claim(seen: &mut HashMap<String, String>, key: String, prefix: &str) -> Result<(), String> {
    match seen.insert(key.clone(), prefix.to_string()) {
        Some(other) => Err(format!("sets {other} and {prefix} both map to `{key}`")),
        None => Ok(()),
    }
}

/// Writes one name list per set, the catalog module that includes them, and
/// removes lists for sets that no longer exist. Returns the catalog's fingerprint.
fn write_catalog(root: &Path, sets: &[FetchedSet]) -> Result<u64, String> {
    let crate_dir = root.join("crates").join("aurora_iconify_catalog");
    let data_dir = crate_dir.join("data");
    fs::create_dir_all(&data_dir).map_err(|e| fail(&data_dir, e))?;

    let wanted: BTreeSet<String> = sets
        .iter()
        .map(|set| format!("{}.txt", set.prefix))
        .collect();
    for entry in fs::read_dir(&data_dir).map_err(|e| fail(&data_dir, e))? {
        let path = entry.map_err(|e| fail(&data_dir, e))?.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let is_list = path.extension().is_some_and(|ext| ext == "txt");
        if is_list && !wanted.contains(&file_name) {
            fs::remove_file(&path).map_err(|e| fail(&path, e))?;
        }
    }
    for set in sets {
        let mut text = String::new();
        for name in &set.names {
            text.push_str(name);
            text.push('\n');
        }
        write_if_changed(&data_dir.join(format!("{}.txt", set.prefix)), &text)?;
    }

    let hash = catalog_hash(
        sets.iter()
            .map(|set| (set.prefix.as_str(), set.names.iter().map(String::as_str))),
    );
    let mut src = String::from(
        "//! Generated by `cargo run -p xtask -- iconify-catalog`. Do not edit by hand.\n\n\
         use crate::SetData;\n\n",
    );
    src.push_str(&format!(
        "/// Fingerprint of the name lists below. Icon IDs are only valid with this catalog.\n\
         pub const CATALOG_HASH: u64 = {hash:#x};\n\n"
    ));
    src.push_str(
        "/// Every icon set in the catalog, sorted by prefix.\npub static SETS: &[SetData] = &[\n",
    );
    for set in sets {
        src.push_str(&format!(
            "    SetData {{ prefix: {:?}, names_text: include_str!(\"../data/{}.txt\") }},\n",
            set.prefix, set.prefix
        ));
    }
    src.push_str("];\n");
    write_if_changed(&crate_dir.join("src").join("data.rs"), &src)?;
    Ok(hash)
}

/// Writes the facade's set list (`src/sets.rs`) and the set features in its manifest.
fn write_facade(root: &Path, prefixes: &[&str], catalog_hash: u64) -> Result<(), String> {
    let facade = root.join("crates").join("aurora_iconify");
    write_if_changed(
        &facade.join("src").join("sets.rs"),
        &sets_module_source(prefixes, catalog_hash),
    )?;
    write_features(&facade.join("Cargo.toml"), prefixes)
}

/// Rewrites the generated block of set features in the facade's manifest.
fn write_features(path: &Path, prefixes: &[&str]) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| fail(path, e))?;
    let begin = text
        .find(FEATURES_BEGIN)
        .ok_or_else(|| format!("{} has no `{FEATURES_BEGIN}` line", path.display()))?;
    let end = text
        .find(FEATURES_END)
        .ok_or_else(|| format!("{} has no `{FEATURES_END}` line", path.display()))?
        + FEATURES_END.len();
    if end < begin {
        return Err(format!(
            "{}: the feature markers are out of order",
            path.display()
        ));
    }

    let mut block = String::from(FEATURES_BEGIN);
    block.push('\n');
    for prefix in prefixes {
        block.push_str(&format!("{prefix} = []\n"));
    }
    block.push_str(FEATURES_END);

    let updated = format!("{}{block}{}", &text[..begin], &text[end..]);
    write_if_changed(path, &updated)
}

/// Writes `contents` unless the file already has exactly that text, so
/// unchanged files keep their timestamps.
fn write_if_changed(path: &Path, contents: &str) -> Result<(), String> {
    if fs::read_to_string(path).is_ok_and(|old| old == contents) {
        return Ok(());
    }
    fs::write(path, contents).map_err(|e| fail(path, e))
}

fn fail(path: &Path, error: std::io::Error) -> String {
    format!("{}: {error}", path.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_come_from_both_sections_sorted_and_unique() {
        let body = r#"{
            "prefix": "demo",
            "uncategorized": ["zebra", "apple"],
            "categories": {"A": ["mango", "apple"], "B": ["3d-box", "../evil"]}
        }"#;
        assert_eq!(
            names_from_collection(body).unwrap(),
            ["3d-box", "apple", "mango", "zebra"]
        );
    }

    #[test]
    fn a_collection_without_icons_has_no_names() {
        assert!(
            names_from_collection(r#"{"prefix": "demo"}"#)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn colliding_feature_names_are_rejected() {
        let error = check_prefixes(&["fa-6", "fa6"]).unwrap_err();
        assert!(error.contains("fa-6") && error.contains("fa6"), "{error}");
        assert!(check_prefixes(&["lucide", "mage"]).is_ok());
    }
}

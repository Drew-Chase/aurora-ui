//! On-disk cache for icon name lists and SVG data.
//!
//! Layout below the cache root, per icon set:
//!
//! ```text
//! <set>/names.txt          icon names, one per line; refreshed after CACHE_TTL
//! <set>/icons/<name>.svg   one file per icon that has ever been needed
//! <set>/all-<hash>.svgs    every icon concatenated, for sets embedded whole
//! <set>/all-<hash>.idx     "<start> <len>" per icon, in name order
//! ```
//!
//! cargo and an IDE may expand the macro at the same time, so every file is
//! written to a temporary name and renamed into place.

use crate::names::is_valid_name;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{env, fs, io};

const CACHE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

/// Byte range of one icon inside a blob of concatenated SVGs.
pub type Span = (u32, u32); // (start, len)

pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Uses `ICONIFY_CACHE_DIR` if set, otherwise
    /// `<workspace target dir>/.iconify-cache/v2`.
    pub fn locate() -> Self {
        let root = env::var_os("ICONIFY_CACHE_DIR")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_target_dir().join(".iconify-cache").join("v2"));
        Self { root }
    }

    #[cfg(test)]
    fn at(root: PathBuf) -> Self {
        Self { root }
    }

    /// Where the API client records that the API could not be reached.
    pub fn outage_marker(&self) -> PathBuf {
        self.root.join(".api-unreachable")
    }

    fn set_dir(&self, prefix: &str) -> PathBuf {
        self.root.join(prefix)
    }

    /// Returns the icon names of a set, calling `fetch` only when there is no
    /// fresh cached list. If `fetch` fails, an expired list is still used.
    pub fn names(
        &self,
        prefix: &str,
        fetch: impl FnOnce() -> Result<Vec<String>, String>,
    ) -> Result<Vec<String>, String> {
        let path = self.set_dir(prefix).join("names.txt");
        if is_fresh(&path)
            && let Some(names) = read_names(&path)
        {
            return Ok(names);
        }
        match fetch() {
            Ok(names) => {
                // A failed write only costs another fetch next time.
                let _ = write_atomic(&path, &names.join("\n"));
                Ok(names)
            }
            Err(e) => read_names(&path).ok_or(e),
        }
    }

    /// Returns the SVG for each of `names`, in order, calling `fetch` once
    /// with only the names that are not cached yet.
    ///
    /// `fetch` returns what it managed to download plus an optional error.
    /// Downloaded icons are cached even when the call as a whole fails, and
    /// the error is passed back alongside whatever is available (`None` for
    /// icons that could not be obtained) so the caller decides how much of a
    /// failure it is.
    pub fn icons(
        &self,
        prefix: &str,
        names: &[String],
        fetch: impl FnOnce(&[String]) -> (Vec<(String, String)>, Option<String>),
    ) -> (Vec<Option<String>>, Option<String>) {
        let dir = self.set_dir(prefix).join("icons");
        let icon_path = |name: &str| dir.join(format!("{name}.svg"));

        let mut svgs: Vec<Option<String>> = names
            .iter()
            .map(|name| fs::read_to_string(icon_path(name)).ok())
            .collect();

        let missing: Vec<String> = names
            .iter()
            .zip(&svgs)
            .filter(|(_, svg)| svg.is_none())
            .map(|(name, _)| name.clone())
            .collect();

        if missing.is_empty() {
            return (svgs, None);
        }

        let (fetched, error) = fetch(&missing);
        for (name, svg) in &fetched {
            // A failed write only costs another fetch next time.
            let _ = write_atomic(&icon_path(name), svg);
        }
        let mut fetched: HashMap<String, String> = fetched.into_iter().collect();
        for (name, svg) in names.iter().zip(&mut svgs) {
            if svg.is_none() {
                *svg = fetched.remove(name);
            }
        }
        (svgs, error)
    }

    /// Returns a file holding every icon of the set concatenated in `names`
    /// order, plus each icon's span within it. The file is built once per
    /// distinct name list and then reused without touching individual icons.
    pub fn full_set(
        &self,
        prefix: &str,
        names: &[String],
        fetch: impl FnOnce(&[String]) -> (Vec<(String, String)>, Option<String>),
    ) -> Result<(PathBuf, Vec<Span>), String> {
        let key = hash_str(&names.join("\n"));
        let set_dir = self.set_dir(prefix);
        let blob_path = set_dir.join(format!("all-{key}.svgs"));
        let index_path = set_dir.join(format!("all-{key}.idx"));

        if let Some(spans) = read_index(&index_path, &blob_path, names.len()) {
            return Ok((blob_path, spans));
        }

        let (svgs, error) = self.icons(prefix, names, fetch);
        if let Some(error) = error {
            return Err(error);
        }
        let svgs: Vec<String> = names
            .iter()
            .zip(svgs)
            .map(|(name, svg)| svg.ok_or_else(|| format!("no data for icon '{name}'")))
            .collect::<Result<_, _>>()?;
        let (blob, spans) = concat(&svgs);
        let index: String = spans
            .iter()
            .map(|(start, len)| format!("{start} {len}\n"))
            .collect();

        // The index is written last: its presence marks the blob as complete.
        write_atomic(&blob_path, &blob)
            .and_then(|()| write_atomic(&index_path, &index))
            .map_err(|e| format!("failed to write {}: {e}", blob_path.display()))?;

        Ok((blob_path, spans))
    }
}

/// Concatenates SVGs into one string and records where each one sits.
pub fn concat(svgs: &[String]) -> (String, Vec<Span>) {
    let mut blob = String::with_capacity(svgs.iter().map(String::len).sum());
    let mut spans = Vec::with_capacity(svgs.len());
    for svg in svgs {
        spans.push((blob.len() as u32, svg.len() as u32));
        blob.push_str(svg);
    }
    (blob, spans)
}

fn read_names(path: &Path) -> Option<Vec<String>> {
    let names: Vec<String> = fs::read_to_string(path)
        .ok()?
        .lines()
        .filter(|name| is_valid_name(name))
        .map(str::to_string)
        .collect();
    (!names.is_empty()).then_some(names)
}

/// Loads spans from an index file, accepting it only if it describes exactly
/// `count` icons that tile the blob file completely.
fn read_index(index_path: &Path, blob_path: &Path, count: usize) -> Option<Vec<Span>> {
    let blob_len = fs::metadata(blob_path).ok()?.len();
    let index = fs::read_to_string(index_path).ok()?;

    let mut spans = Vec::with_capacity(count);
    let mut end = 0u64;
    for line in index.lines() {
        let (start, len) = line.split_once(' ')?;
        let (start, len): Span = (start.parse().ok()?, len.parse().ok()?);
        if u64::from(start) != end {
            return None;
        }
        end += u64::from(len);
        spans.push((start, len));
    }

    (spans.len() == count && end == blob_len).then_some(spans)
}

fn is_fresh(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().unwrap_or(Duration::MAX) < CACHE_TTL)
        .unwrap_or(false)
}

/// Writes `contents` to a temporary sibling and renames it over `path`, so
/// concurrent readers never observe a partially written file.
fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;

    let tmp = dir.join(format!(
        ".tmp-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path).or_else(|e| {
        let _ = fs::remove_file(&tmp);
        // Another process holding the destination open has already written
        // the same content.
        if path.exists() { Ok(()) } else { Err(e) }
    })
}

/// FNV-1a 64-bit hash, used to fingerprint name lists without extra deps.
fn hash_str(value: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Finds the workspace `target/` directory: `CARGO_TARGET_DIR` if set, else
/// next to a workspace `Cargo.toml` found in the current directory or above
/// `CARGO_MANIFEST_DIR`.
fn workspace_target_dir() -> PathBuf {
    if let Some(dir) = env::var_os("CARGO_TARGET_DIR").filter(|dir| !dir.is_empty()) {
        return PathBuf::from(dir);
    }

    // Try CWD first — this is where the user ran `cargo build` from
    if let Ok(cwd) = env::current_dir()
        && is_workspace_root(&cwd)
    {
        return cwd.join("target");
    }

    // Fallback: walk up from CARGO_MANIFEST_DIR
    let start = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir());

    start
        .ancestors()
        .find(|dir| is_workspace_root(dir))
        .unwrap_or(&start)
        .join("target")
}

fn is_workspace_root(dir: &Path) -> bool {
    let cargo_toml = dir.join("Cargo.toml");
    cargo_toml.is_file()
        && fs::read_to_string(&cargo_toml)
            .map(|c| c.contains("[workspace]"))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn temp_cache(test: &str) -> Cache {
        let root = env::temp_dir().join(format!("iconify-cache-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        Cache::at(root)
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn expire(path: &Path) {
        let old = SystemTime::now() - CACHE_TTL - Duration::from_secs(60);
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(old)
            .unwrap();
    }

    fn no_fetch(_: &[String]) -> (Vec<(String, String)>, Option<String>) {
        panic!("unexpected fetch")
    }

    #[test]
    fn names_are_fetched_once_then_served_from_cache() {
        let cache = temp_cache("names-once");
        let fetched = cache.names("demo", || Ok(strings(&["a", "b"]))).unwrap();
        let cached = cache.names("demo", || panic!("unexpected fetch")).unwrap();
        assert_eq!(fetched, cached);
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn expired_names_are_refreshed() {
        let cache = temp_cache("names-refresh");
        cache.names("demo", || Ok(strings(&["a"]))).unwrap();
        expire(&cache.set_dir("demo").join("names.txt"));
        let names = cache.names("demo", || Ok(strings(&["a", "b"]))).unwrap();
        assert_eq!(names, ["a", "b"]);
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn expired_names_survive_a_failed_refresh() {
        let cache = temp_cache("names-stale");
        cache.names("demo", || Ok(strings(&["a", "b"]))).unwrap();
        expire(&cache.set_dir("demo").join("names.txt"));
        let names = cache.names("demo", || Err("offline".into())).unwrap();
        assert_eq!(names, ["a", "b"]);
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn missing_names_with_failed_fetch_is_an_error() {
        let cache = temp_cache("names-none");
        let err = cache.names("demo", || Err("offline".into())).unwrap_err();
        assert_eq!(err, "offline");
    }

    fn some(items: &[&str]) -> Vec<Option<String>> {
        items.iter().map(|s| Some(s.to_string())).collect()
    }

    #[test]
    fn only_uncached_icons_are_fetched() {
        let cache = temp_cache("icons-missing");
        let (first, error) = cache.icons("demo", &strings(&["a", "b"]), |missing| {
            assert_eq!(missing, ["a", "b"]);
            (
                vec![("a".into(), "<a/>".into()), ("b".into(), "<b/>".into())],
                None,
            )
        });
        assert_eq!((first, error), (some(&["<a/>", "<b/>"]), None));

        let (second, _) = cache.icons("demo", &strings(&["c", "a"]), |missing| {
            assert_eq!(missing, ["c"]);
            (vec![("c".into(), "<c/>".into())], None)
        });
        assert_eq!(second, some(&["<c/>", "<a/>"]));

        let (third, _) = cache.icons("demo", &strings(&["b", "c"]), no_fetch);
        assert_eq!(third, some(&["<b/>", "<c/>"]));
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn partial_downloads_are_kept_when_a_fetch_fails() {
        let cache = temp_cache("icons-partial");
        let (svgs, error) = cache.icons("demo", &strings(&["a", "b"]), |_| {
            (vec![("a".into(), "<a/>".into())], Some("boom".into()))
        });
        assert_eq!(svgs, [Some("<a/>".to_string()), None]);
        assert_eq!(error.as_deref(), Some("boom"));

        let (svgs, error) = cache.icons("demo", &strings(&["a", "b"]), |missing| {
            assert_eq!(missing, ["b"]);
            (vec![("b".into(), "<b/>".into())], None)
        });
        assert_eq!((svgs, error), (some(&["<a/>", "<b/>"]), None));
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn full_set_fails_when_an_icon_cannot_be_fetched() {
        let cache = temp_cache("full-set-fail");
        let err = cache
            .full_set("demo", &strings(&["a", "b"]), |_| {
                (vec![("a".into(), "<a/>".into())], Some("boom".into()))
            })
            .unwrap_err();
        assert_eq!(err, "boom");
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn full_set_blob_is_built_once_and_reused() {
        let cache = temp_cache("full-set");
        let names = strings(&["a", "b"]);
        let (path, spans) = cache
            .full_set("demo", &names, |_| {
                (
                    vec![("a".into(), "<a/>".into()), ("b".into(), "<bb/>".into())],
                    None,
                )
            })
            .unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "<a/><bb/>");
        assert_eq!(spans, [(0, 4), (4, 5)]);

        // Reused even if the per-icon files are gone.
        fs::remove_dir_all(cache.set_dir("demo").join("icons")).unwrap();
        let (again, again_spans) = cache.full_set("demo", &names, no_fetch).unwrap();
        assert_eq!((again, again_spans), (path, spans));
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn truncated_blob_is_rebuilt() {
        let cache = temp_cache("full-set-truncated");
        let names = strings(&["a", "b"]);
        let fetch = |_: &[String]| {
            (
                vec![("a".into(), "<a/>".into()), ("b".into(), "<b/>".into())],
                None,
            )
        };
        let (path, _) = cache.full_set("demo", &names, fetch).unwrap();
        fs::write(&path, "<a/>").unwrap();
        let (path, spans) = cache.full_set("demo", &names, no_fetch).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "<a/><b/>");
        assert_eq!(spans, [(0, 4), (4, 4)]);
        let _ = fs::remove_dir_all(&cache.root);
    }
}

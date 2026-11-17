//! On-disk cache for SVG data and generated icon tables.
//!
//! Layout below the cache root, per icon set:
//!
//! ```text
//! <set>/icons/<name>.svg     one file per icon that has ever been needed
//! <set>/table-<key>.svgs     SVG data for a group of icons, concatenated
//! <set>/table-<key>.idx      index into that data (see aurora_iconify_catalog::index)
//! ```
//!
//! An icon never changes once it is downloaded, so nothing here expires. `cargo`
//! and an IDE may expand the macro at the same time, so every file is written
//! under a temporary name and renamed into place. A table's index is written
//! last, since its presence marks the table complete.

use aurora_iconify_catalog::index;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, fs};

pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Uses `ICONIFY_CACHE_DIR` if set, otherwise `<workspace target dir>/.iconify-cache/v3`.
    pub fn locate() -> Self {
        let root = env::var_os("ICONIFY_CACHE_DIR")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_target_dir().join(".iconify-cache").join("v3"));
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

    /// Returns the SVG for each of `names`, in order, calling `fetch` once with
    /// only the names that are not cached yet.
    ///
    /// `fetch` returns what it managed to download plus an optional error.
    /// Downloaded icons are cached even when the call as a whole fails, and the
    /// error comes back alongside whatever is available (`None` for icons that
    /// could not be obtained), so the caller decides how much of a failure it is.
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
            let _ = write_atomic(&icon_path(name), svg.as_bytes());
        }
        let mut fetched: HashMap<String, String> = fetched.into_iter().collect();
        for (name, svg) in names.iter().zip(&mut svgs) {
            if svg.is_none() {
                *svg = fetched.remove(name);
            }
        }
        (svgs, error)
    }

    /// The complete table stored under `key`, if there is one: the paths of its
    /// index and of its SVG data.
    pub fn existing_table(&self, prefix: &str, key: &str) -> Option<(PathBuf, PathBuf)> {
        let (index_path, blob_path) = self.table_paths(prefix, key);
        let index = fs::read(&index_path).ok()?;
        let blob_len = fs::metadata(&blob_path).ok()?.len();
        (u64::from(index::blob_len(&index)?) == blob_len).then_some((index_path, blob_path))
    }

    /// Stores a table and returns the paths of its index and of its SVG data.
    pub fn write_table(
        &self,
        prefix: &str,
        key: &str,
        index: &[u8],
        blob: &str,
    ) -> Result<(PathBuf, PathBuf), String> {
        let (index_path, blob_path) = self.table_paths(prefix, key);
        write_atomic(&blob_path, blob.as_bytes()).map_err(|e| fail(&blob_path, e))?;
        write_atomic(&index_path, index).map_err(|e| fail(&index_path, e))?;
        Ok((index_path, blob_path))
    }

    fn table_paths(&self, prefix: &str, key: &str) -> (PathBuf, PathBuf) {
        let dir = self.set_dir(prefix);
        (
            dir.join(format!("table-{key}.idx")),
            dir.join(format!("table-{key}.svgs")),
        )
    }
}

fn fail(path: &Path, error: io::Error) -> String {
    format!("failed to write {}: {error}", path.display())
}

/// Writes `contents` to a temporary sibling and renames it over `path`, so
/// concurrent readers never observe a partially written file.
fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
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

/// Finds the workspace `target/` directory: `CARGO_TARGET_DIR` if set, else
/// next to a workspace `Cargo.toml` found in the current directory or above
/// `CARGO_MANIFEST_DIR`.
fn workspace_target_dir() -> PathBuf {
    if let Some(dir) = env::var_os("CARGO_TARGET_DIR").filter(|dir| !dir.is_empty()) {
        return PathBuf::from(dir);
    }

    // The current directory first: this is where the user ran `cargo build`.
    if let Ok(cwd) = env::current_dir()
        && is_workspace_root(&cwd)
    {
        return cwd.join("target");
    }

    // Otherwise walk up from CARGO_MANIFEST_DIR.
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
    use aurora_iconify_catalog::index::{Entry, write};

    fn temp_cache(test: &str) -> Cache {
        let root = env::temp_dir().join(format!("iconify-cache-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        Cache::at(root)
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn some(items: &[&str]) -> Vec<Option<String>> {
        items.iter().map(|s| Some(s.to_string())).collect()
    }

    fn no_fetch(_: &[String]) -> (Vec<(String, String)>, Option<String>) {
        panic!("unexpected fetch")
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
    fn a_table_is_written_once_and_found_again() {
        let cache = temp_cache("table-reuse");
        let blob = "<a/><bb/>";
        let index = write(
            &[
                Entry {
                    id: 0,
                    name: "a",
                    start: 0,
                    len: 4,
                },
                Entry {
                    id: 1,
                    name: "b",
                    start: 4,
                    len: 5,
                },
            ],
            blob.len() as u32,
        );
        let (index_path, blob_path) = cache.write_table("demo", "k1", &index, blob).unwrap();
        assert_eq!(fs::read_to_string(&blob_path).unwrap(), blob);
        assert_eq!(
            cache.existing_table("demo", "k1"),
            Some((index_path, blob_path))
        );
        assert_eq!(cache.existing_table("demo", "other"), None);
        let _ = fs::remove_dir_all(&cache.root);
    }

    #[test]
    fn a_truncated_table_is_not_reused() {
        let cache = temp_cache("table-truncated");
        // The index claims 8 bytes of SVG data, but only 4 are written.
        let index = write(
            &[Entry {
                id: 0,
                name: "a",
                start: 0,
                len: 4,
            }],
            8,
        );
        cache.write_table("demo", "k", &index, "<a/>").unwrap();
        assert_eq!(cache.existing_table("demo", "k"), None);
        let _ = fs::remove_dir_all(&cache.root);
    }
}

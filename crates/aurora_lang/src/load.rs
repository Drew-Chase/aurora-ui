use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Locale tag → { flat dotted key → string value }.
pub type LocaleMap = BTreeMap<String, BTreeMap<String, String>>;

/// Resolve a path relative to the calling crate's `CARGO_MANIFEST_DIR`.
pub fn resolve_path(relative: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(relative)
}

/// Load all locale data from either a directory of `.toml` files or a single
/// combined `.toml` file.
pub fn load_locales(path: &Path) -> Result<LocaleMap, String> {
    if path.is_dir() {
        load_directory(path)
    } else if path.is_file() {
        load_single_file(path)
    } else {
        Err(format!(
            "path `{}` is neither a file nor a directory",
            path.display()
        ))
    }
}

/// Collect all absolute paths of `.toml` files that were loaded, for rebuild tracking.
pub fn collect_toml_paths(path: &Path) -> Vec<PathBuf> {
    if path.is_dir() {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
            .collect();
        paths.sort();
        paths
    } else {
        vec![path.to_path_buf()]
    }
}

/// Directory mode: each `.toml` file is one locale, filename = locale tag.
fn load_directory(dir: &Path) -> Result<LocaleMap, String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory `{}`: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("toml"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        return Err(format!(
            "no .toml files found in directory `{}`",
            dir.display()
        ));
    }

    let mut map = LocaleMap::new();
    for entry in entries {
        let path = entry.path();
        let locale_tag = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid filename: {}", path.display()))?
            .to_lowercase();

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read `{}`: {}", path.display(), e))?;
        let table: toml::Value = content
            .parse::<toml::Value>()
            .map_err(|e| format!("TOML parse error in `{}`: {}", path.display(), e))?;

        let flat = flatten_toml_value(&table, "")?;
        if flat.is_empty() {
            return Err(format!(
                "locale file `{}` contains no string keys",
                path.display()
            ));
        }
        map.insert(locale_tag, flat);
    }

    Ok(map)
}

/// Single-file mode: top-level table keys are locale tags.
fn load_single_file(path: &Path) -> Result<LocaleMap, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read `{}`: {}", path.display(), e))?;
    let table: toml::Value = content
        .parse::<toml::Value>()
        .map_err(|e| format!("TOML parse error in `{}`: {}", path.display(), e))?;

    let root = table
        .as_table()
        .ok_or_else(|| "TOML root must be a table".to_string())?;

    if root.is_empty() {
        return Err(format!(
            "TOML file `{}` contains no locale sections",
            path.display()
        ));
    }

    let mut map = LocaleMap::new();
    for (locale_tag, section) in root {
        let flat = flatten_toml_value(section, "")?;
        if flat.is_empty() {
            return Err(format!(
                "locale section `[{}]` contains no string keys",
                locale_tag
            ));
        }
        map.insert(locale_tag.to_lowercase(), flat);
    }

    Ok(map)
}

/// Recursively flatten a TOML value into dotted-key → string pairs.
fn flatten_toml_value(val: &toml::Value, prefix: &str) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();

    match val {
        toml::Value::Table(table) => {
            for (key, child) in table {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                let sub = flatten_toml_value(child, &full_key)?;
                result.extend(sub);
            }
        }
        toml::Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        other => {
            return Err(format!(
                "key `{}` has non-string value type `{}`; only strings are supported",
                prefix,
                other.type_str()
            ));
        }
    }

    Ok(result)
}

//! Iconify HTTP API access.

use crate::names::is_valid_name;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

/// Icons requested per API call.
const BATCH_SIZE: usize = 80;
/// Concurrent API calls when a request spans several batches.
const MAX_WORKERS: usize = 6;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// How long to stop trying after the API could not be reached, so that a
/// build without network access does not wait for a timeout on every compile.
const OUTAGE_PAUSE: Duration = Duration::from_secs(60);

/// A connection-pooling client for one Iconify API host.
pub struct Api {
    agent: ureq::Agent,
    base: &'static str,
    /// File recording the most recent failure to reach the API. It lives on
    /// disk because every compilation runs the macro in a new process.
    outage_marker: PathBuf,
}

impl Api {
    pub fn new(base: &'static str, outage_marker: PathBuf) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build()
            .into();
        Self {
            agent,
            base,
            outage_marker,
        }
    }

    fn get(&self, url: &str) -> Result<String, String> {
        if let Some(failure) = self.recent_outage() {
            return Err(format!(
                "{failure} (not retrying for {}s after a failed request)",
                OUTAGE_PAUSE.as_secs()
            ));
        }
        match self.agent.get(url).call() {
            Ok(mut response) => response
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("failed to read response from {url}: {e}")),
            Err(e) => {
                let failure = format!("HTTP request failed for {url}: {e}");
                // A status code means the API answered; anything else means
                // it could not be reached at all.
                if !matches!(e, ureq::Error::StatusCode(_)) {
                    if let Some(dir) = self.outage_marker.parent() {
                        let _ = fs::create_dir_all(dir);
                    }
                    let _ = fs::write(&self.outage_marker, &failure);
                }
                Err(failure)
            }
        }
    }

    /// The failure message of an unreachable-API error from the last
    /// `OUTAGE_PAUSE`, if there was one.
    fn recent_outage(&self) -> Option<String> {
        let age = fs::metadata(&self.outage_marker)
            .and_then(|m| m.modified())
            .ok()?
            .elapsed()
            .ok()?;
        (age < OUTAGE_PAUSE).then(|| fs::read_to_string(&self.outage_marker).unwrap_or_default())
    }

    /// Fetches the sorted list of icon names in a set. One request.
    pub fn icon_names(&self, prefix: &str) -> Result<Vec<String>, String> {
        let url = format!("{}/collection?prefix={prefix}", self.base);
        let names = names_from_collection(&self.get(&url)?)?;
        if names.is_empty() {
            return Err(format!("icon set '{prefix}' has no icons"));
        }
        Ok(names)
    }

    /// Fetches complete SVG strings for `names`, batching and parallelising
    /// as needed.
    ///
    /// Returns everything that was fetched together with the first error, so
    /// the caller can cache partial progress before reporting the failure.
    pub fn icons(&self, prefix: &str, names: &[String]) -> (Vec<(String, String)>, Option<String>) {
        let batches: Vec<&[String]> = names.chunks(BATCH_SIZE).collect();
        let next = AtomicUsize::new(0);
        let failed = AtomicBool::new(false);
        let fetched = Mutex::new(Vec::with_capacity(names.len()));
        let error = Mutex::new(None);

        let work = || {
            while !failed.load(Ordering::Relaxed) {
                let Some(batch) = batches.get(next.fetch_add(1, Ordering::Relaxed)) else {
                    break;
                };
                let url = format!("{}/{prefix}.json?icons={}", self.base, batch.join(","));
                match self
                    .get(&url)
                    .and_then(|body| svgs_from_response(&body, batch))
                {
                    Ok(icons) => fetched.lock().unwrap().extend(icons),
                    Err(e) => {
                        failed.store(true, Ordering::Relaxed);
                        error.lock().unwrap().get_or_insert(e);
                    }
                }
            }
        };

        if batches.len() <= 1 {
            work();
        } else {
            std::thread::scope(|scope| {
                for _ in 0..batches.len().min(MAX_WORKERS) {
                    scope.spawn(work);
                }
            });
        }

        (fetched.into_inner().unwrap(), error.into_inner().unwrap())
    }
}

/// Extracts the sorted, de-duplicated icon names from a `/collection`
/// response. Icons are listed under `uncategorized` and/or `categories`.
pub fn names_from_collection(body: &str) -> Result<Vec<String>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("JSON parse error: {e}"))?;

    let uncategorized = parsed.get("uncategorized").and_then(|v| v.as_array());
    let categorized = parsed
        .get("categories")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|categories| categories.values())
        .filter_map(|v| v.as_array());

    let names: BTreeSet<&str> = uncategorized
        .into_iter()
        .chain(categorized)
        .flatten()
        .filter_map(|v| v.as_str())
        .filter(|name| is_valid_name(name))
        .collect();

    Ok(names.into_iter().map(str::to_string).collect())
}

/// Builds a full `<svg>` string for each of `requested` from an icon data
/// response. Fails if any requested icon is absent.
pub fn svgs_from_response(
    body: &str,
    requested: &[String],
) -> Result<Vec<(String, String)>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("JSON parse error: {e}"))?;

    let default_width = parsed.get("width").and_then(|v| v.as_u64()).unwrap_or(24);
    let default_height = parsed.get("height").and_then(|v| v.as_u64()).unwrap_or(24);
    let icons = parsed.get("icons").and_then(|v| v.as_object());

    let mut svgs = Vec::with_capacity(requested.len());
    let mut missing = Vec::new();

    for name in requested {
        let Some(data) = icons.and_then(|icons| icons.get(name.as_str())) else {
            missing.push(name.as_str());
            continue;
        };
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
        svgs.push((
            name.clone(),
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">{body}</svg>"
            ),
        ));
    }

    if missing.is_empty() {
        Ok(svgs)
    } else {
        Err(format!(
            "the Iconify API returned no data for: {}",
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn requests_are_skipped_shortly_after_the_api_was_unreachable() {
        let marker = std::env::temp_dir().join(format!("iconify-outage-{}", std::process::id()));
        fs::write(&marker, "HTTP request failed for <url>: timed out").unwrap();
        // The base URL is never contacted while the marker is recent.
        let api = Api::new("http://unreachable.invalid", marker.clone());

        let err = api.icon_names("mage").unwrap_err();
        assert!(
            err.starts_with("HTTP request failed for <url>: timed out (not retrying"),
            "{err}"
        );
        let (icons, err) = api.icons("mage", &["a".to_string()]);
        assert!(icons.is_empty() && err.is_some());

        let old = SystemTime::now() - OUTAGE_PAUSE - Duration::from_secs(1);
        let file = fs::File::options().write(true).open(&marker).unwrap();
        file.set_modified(old).unwrap();
        drop(file);
        assert_eq!(api.recent_outage(), None, "the pause expires");
        let _ = fs::remove_file(&marker);
    }

    #[test]
    fn collects_sorted_unique_names_from_both_sections() {
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
    fn builds_svgs_with_set_and_icon_dimensions() {
        let body = r#"{
            "prefix": "demo", "width": 16, "height": 16,
            "icons": {
                "a": {"body": "<path d=\"M0 0\"/>"},
                "b": {"body": "<g/>", "width": 32}
            }
        }"#;
        let svgs = svgs_from_response(body, &["a".to_string(), "b".to_string()]).unwrap();
        assert_eq!(
            svgs[0].1,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path d="M0 0"/></svg>"#
        );
        assert_eq!(
            svgs[1].1,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="16" viewBox="0 0 32 16"><g/></svg>"#
        );
    }

    #[test]
    fn reports_icons_missing_from_the_response() {
        let body = r#"{"prefix": "demo", "icons": {"a": {"body": ""}}, "not_found": ["nope"]}"#;
        let err = svgs_from_response(body, &["a".to_string(), "nope".to_string()]).unwrap_err();
        assert!(err.contains("nope"), "{err}");
    }
}

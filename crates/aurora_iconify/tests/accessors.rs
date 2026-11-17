//! The accessors pass the icon's catalog ID to the provider, and panic at the
//! caller's location when the provider has no data for it.
//!
//! Needs the `lucide` feature, which `cargo test -p aurora_iconify --features lucide` enables.

#![cfg(feature = "lucide")]

use aurora_iconify::{LucideIcons, Provider, SetId};
use std::panic;
use std::sync::{Arc, Mutex};

/// The catalog ID of the lucide icon `trash`.
fn trash_id() -> u32 {
    let set = aurora_iconify_catalog::set("lucide").expect("the catalog has lucide");
    set.names()
        .position(|name| name == "trash")
        .expect("lucide has a trash icon") as u32
}

const TRASH_SVG: &str = "<svg id=\"trash\"/>";

/// Embeds the `trash` icon and nothing else.
struct OneIcon;

impl Provider for OneIcon {
    fn get(set: SetId, id: u32) -> Option<&'static str> {
        (matches!(set, SetId::Lucide) && id == trash_id()).then_some(TRASH_SVG)
    }

    fn find(set: SetId, name: &str) -> Option<&'static str> {
        (matches!(set, SetId::Lucide) && name == "trash").then_some(TRASH_SVG)
    }
}

/// Embeds nothing.
struct NoIcons;

impl Provider for NoIcons {
    fn get(_: SetId, _: u32) -> Option<&'static str> {
        None
    }

    fn find(_: SetId, _: &str) -> Option<&'static str> {
        None
    }
}

#[test]
fn an_accessor_returns_the_svg_the_provider_has() {
    assert_eq!(LucideIcons::<OneIcon>::__handle().trash(), TRASH_SVG);
}

#[test]
fn by_name_finds_only_what_the_provider_has() {
    let icons = LucideIcons::<OneIcon>::__handle();
    assert_eq!(icons.by_name("trash"), Some(TRASH_SVG));
    assert_eq!(icons.by_name("not-an-icon"), None);
}

#[test]
fn an_icon_without_data_panics_at_the_call_site() {
    let reported: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&reported);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| (l.file().to_string(), l.line().to_string()));
        let message = info
            .payload()
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        *sink.lock().unwrap() = location.map(|(file, line)| (format!("{file}:{line}"), message));
    }));

    let result = panic::catch_unwind(|| LucideIcons::<NoIcons>::__handle().trash());

    panic::set_hook(previous);
    assert!(result.is_err());
    let (location, message) = reported
        .lock()
        .unwrap()
        .clone()
        .expect("the panic was reported");
    assert!(
        message.contains("icon `lucide:trash` is not embedded"),
        "{message}"
    );
    assert!(
        location.starts_with(file!()),
        "the panic should point at this test, not at the crate: {location}"
    );
}

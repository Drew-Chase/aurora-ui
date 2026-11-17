//! Placeholder until `cargo run -p xtask -- iconify-catalog` writes the real catalog.

use crate::SetData;

/// Fingerprint of the name lists. Placeholder: regenerate the catalog.
pub const CATALOG_HASH: u64 = 0x3d11b683a3ca76af;

/// Every icon set in the catalog. Placeholder: regenerate the catalog.
pub static SETS: &[SetData] = &[
    SetData {
        prefix: "lucide",
        names_text: include_str!("../data/lucide.txt"),
    },
    SetData {
        prefix: "mage",
        names_text: include_str!("../data/mage.txt"),
    },
    SetData {
        prefix: "material-symbols",
        names_text: include_str!("../data/material-symbols.txt"),
    },
];

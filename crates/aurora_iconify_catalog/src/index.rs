//! Binary index of the icons a crate embeds, read by `aurora_iconify::IconTable`.
//!
//! Every field is a little-endian `u32`:
//!
//! - header: `count`, `blob_len` (bytes of SVG data the index describes) and
//!   `names_at` (offset of the names region)
//! - `count` records sorted by icon ID: `id`, `start` and `len` (where the SVG
//!   sits in the blob), then `name_at` and `name_len` (where the name sits,
//!   relative to `names_at`)
//! - `count` record numbers, sorted by icon name
//! - the names, concatenated
//!
//! The facade reads this layout, so change both crates together.

/// Bytes of header before the records.
pub const HEADER_LEN: usize = 12;
/// Bytes per record.
pub const RECORD_LEN: usize = 20;

/// One embedded icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry<'a> {
    /// The icon's position in its set's sorted name list.
    pub id: u32,
    /// The icon's name, such as `arrow-left`.
    pub name: &'a str,
    /// Where the icon's SVG starts in the blob.
    pub start: u32,
    /// The length of the icon's SVG in bytes.
    pub len: u32,
}

/// Serializes `entries`, which must be sorted by `id` and have unique names.
/// `blob_len` is the length of the SVG blob they index.
pub fn write(entries: &[Entry<'_>], blob_len: u32) -> Vec<u8> {
    let count = u32::try_from(entries.len()).expect("an index holds fewer than 2^32 icons");
    let names_at = (HEADER_LEN + entries.len() * (RECORD_LEN + 4)) as u32;

    let mut names = Vec::new();
    let mut name_at = Vec::with_capacity(entries.len());
    for entry in entries {
        name_at.push(names.len() as u32);
        names.extend_from_slice(entry.name.as_bytes());
    }

    let mut by_name: Vec<u32> = (0..count).collect();
    by_name.sort_by(|&a, &b| entries[a as usize].name.cmp(entries[b as usize].name));

    let mut out = Vec::with_capacity(names_at as usize + names.len());
    for word in [count, blob_len, names_at] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    for (entry, at) in entries.iter().zip(&name_at) {
        let name_len = entry.name.len() as u32;
        for word in [entry.id, entry.start, entry.len, *at, name_len] {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    for index in by_name {
        out.extend_from_slice(&index.to_le_bytes());
    }
    out.extend_from_slice(&names);
    out
}

/// The blob length recorded in an index header, if the header is present.
pub fn blob_len(index: &[u8]) -> Option<u32> {
    let bytes = index.get(4..8)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_and_layout_match_the_documented_format() {
        let entries = [
            Entry {
                id: 1,
                name: "zebra",
                start: 0,
                len: 3,
            },
            Entry {
                id: 4,
                name: "apple",
                start: 3,
                len: 2,
            },
        ];
        let index = write(&entries, 5);
        assert_eq!(blob_len(&index), Some(5));
        // header + 2 records + 2 name slots + 10 name bytes
        assert_eq!(index.len(), HEADER_LEN + 2 * RECORD_LEN + 2 * 4 + 10);
        let count = u32::from_le_bytes([index[0], index[1], index[2], index[3]]);
        assert_eq!(count, 2);
        // `apple` (record 1, ID 4) sorts before `zebra` (record 0, ID 1).
        let by_name_at = HEADER_LEN + 2 * RECORD_LEN;
        let first = u32::from_le_bytes(index[by_name_at..by_name_at + 4].try_into().unwrap());
        assert_eq!(first, 1);
    }

    #[test]
    fn short_input_has_no_header() {
        assert_eq!(blob_len(&[1, 2, 3]), None);
    }
}

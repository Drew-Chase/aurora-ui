# aurora_fonts

Compile-time [Google Fonts](https://fonts.google.com) embedding for Rust.

Fetches font families from Google Fonts at build time, downloads all available weights and styles, and embeds the `.ttf` bytes directly in your binary. Over 1,600 font families available — no runtime HTTP requests, no asset files, no system font dependency.

## Usage

```rust,ignore
aurora_fonts::font_families!("Roboto", "Open Sans");

fn main() {
    // Named weight access — returns &'static [u8] (raw .ttf bytes):
    let regular: &[u8] = FontFamily::roboto().regular();
    let bold: &[u8] = FontFamily::roboto().bold();
    let italic: &[u8] = FontFamily::roboto().italic();
    let bold_italic: &[u8] = FontFamily::roboto().bold_italic();

    // Numeric weight access:
    let w500: Option<&[u8]> = FontFamily::roboto().weight(500);

    // Multiple weights at once:
    let fonts: Vec<&[u8]> = FontFamily::roboto().with_weights(&[100, 400, 700]);

    // Use with AuroraUI FontManager:
    app.font(FontFamily::roboto().regular())
}
```

Family names are **case-insensitive**: `"Roboto"`, `"roboto"`, and `"ROBOTO"` all work.

## How it works

1. Call `font_families!("Roboto")` anywhere in your crate
2. At compile time, the proc macro fetches the Google Fonts metadata catalog
3. For each requested family, it queries the CSS API to discover `.ttf` download URLs
4. Downloads each weight/style variant as a `.ttf` file
5. Embeds the raw bytes as `&'static [u8]` literals in the generated code
6. Results are cached locally for 30 days to keep builds fast

## API

### `font_families!` macro

Declares which font families to include. Generates:

- `FontFamily` struct with one method per family (e.g. `FontFamily::roboto()`)
- Per-family structs with named weight methods (e.g. `RobotoFamily::bold()`)
- `.weight(n)` and `.weight_italic(n)` for numeric access
- `FontFamily::by_name(name)` for dynamic selection

### Named weight methods

| Weight | Normal | Italic |
|--------|--------|--------|
| 100 | `thin()` | `thin_italic()` |
| 200 | `extra_light()` | `extra_light_italic()` |
| 300 | `light()` | `light_italic()` |
| 400 | `regular()` | `italic()` |
| 500 | `medium()` | `medium_italic()` |
| 600 | `semi_bold()` | `semi_bold_italic()` |
| 700 | `bold()` | `bold_italic()` |
| 800 | `extra_bold()` | `extra_bold_italic()` |
| 900 | `black()` | `black_italic()` |

Only methods for available weights are generated. Not all fonts have all 9 weights.

### Dynamic access

```rust,ignore
// By family name (case-insensitive):
let font = FontFamily::by_name("roboto").and_then(|f| f.weight(400));
```

## Caching

Font files are cached in `target/.fonts-cache/` relative to your project root. Cache entries expire after 30 days. The Google Fonts metadata catalog is also cached. If the API is unreachable and a cache exists, the stale cache is used.

## Requirements

- Network access at compile time (first build only; cached afterwards)
- Rust edition 2021 or later

## License

MIT OR Apache-2.0

# aurora_iconify

Compile-time icon embedding from [iconify.design](https://iconify.design) for Rust.

Fetches icon sets from the Iconify API at build time, generates type-safe Rust accessors, and embeds all SVGs directly in your binary as `&'static str`. Over 200,000 icons from 150+ open-source icon sets — no runtime HTTP requests, no asset files.

## Usage

```rust,ignore
aurora_iconify::icon_sets!("mage", "lucide");

fn main() {
    // Type-safe — one generated method per icon:
    let svg: &str = Icon::mage().calendar_2_fill();

    // Dynamic lookup by name:
    let svg: Option<&str> = Icon::mage().by_name("calendar-2-fill");

    // Dynamic set + name:
    let svg = Icon::from_set("mage")
        .and_then(|set| set.by_name("calendar-2-fill"));
}
```

## How it works

1. Call `icon_sets!("mage", "lucide")` anywhere in your crate
2. At compile time, the proc macro fetches icon data from the [Iconify API](https://api.iconify.design)
3. For each icon, a full SVG string is generated and embedded as a string literal
4. Type-safe accessor methods are generated for every icon in every requested set
5. Results are cached locally for 7 days to keep builds fast

## API

### `icon_sets!` macro

Declares which icon sets to include. Generates:

- `Icon` struct with one method per set (e.g. `Icon::mage()`)
- Per-set structs with one method per icon (e.g. `MageIcons::calendar_2_fill()`)
- `.by_name(name)` for dynamic lookup on each set
- `Icon::from_set(name)` for dynamic set selection

### Type-safe access

```rust,ignore
aurora_iconify::icon_sets!("tabler");

let svg = Icon::tabler().arrow_left();
```

Icon names are sanitized to valid Rust identifiers:

| Icon name | Method name |
|---|---|
| `calendar-2-fill` | `calendar_2_fill()` |
| `arrow-left` | `arrow_left()` |
| `3d-rotate` | `_3d_rotate()` |
| `box` | `box_icon()` |

### Dynamic access

```rust,ignore
// By name within a known set:
let svg: Option<&str> = Icon::mage().by_name("calendar-2-fill");

// By set name and icon name:
let svg = Icon::from_set("mage").and_then(|s| s.by_name("calendar-2-fill"));
```

## Caching

Fetched icon data is cached in `target/.iconify-cache/` relative to your project root. Cache entries expire after 7 days. If the Iconify API is unreachable and a stale cache exists, the stale cache is used with a compile-time warning.

## Available icon sets

Browse all available sets at [icon-sets.iconify.design](https://icon-sets.iconify.design). Popular sets include:

| Set | Prefix | Icons |
|---|---|---|
| Material Symbols | `material-symbols` | 15,000+ |
| Lucide | `lucide` | 1,500+ |
| Tabler Icons | `tabler` | 5,600+ |
| Phosphor | `ph` | 7,400+ |
| Mage Icons | `mage` | 1,000+ |
| Simple Icons | `simple-icons` | 3,100+ |
| Heroicons | `heroicons` | 290+ |
| Feather Icons | `feather` | 280+ |

## Requirements

- Network access at compile time (first build only; cached afterwards)
- Rust edition 2021 or later

## License

MIT OR Apache-2.0

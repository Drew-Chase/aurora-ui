# aurora_iconify

Compile-time icon embedding from [iconify.design](https://iconify.design) for Rust.

Generates type-safe Rust accessors for whole icon sets and embeds the SVGs of the icons you actually use directly in your binary as `&'static str`. Over 200,000 icons from 150+ open-source icon sets — no runtime HTTP requests, no asset files.

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
2. At compile time, the proc macro fetches the list of icon names for each set (one small request per set) and generates an accessor method for every icon, so your editor can complete all of them
3. It then scans your crate's `.rs` files for icon names and downloads only the icons it finds
4. Those SVGs are embedded in the binary; every other accessor is a stub with no data behind it

Adding `Icon::mage().some_new_icon()` to your code is all it takes: the next build downloads and embeds that icon.

## API

### `icon_sets!` macro

Declares which icon sets to include. Generates:

- `Icon` struct with one method per set (e.g. `Icon::mage()`)
- Per-set structs with one method per icon (e.g. `MageIcons::calendar_2_fill()`)
- `.by_name(name)` for dynamic lookup on each set
- `Icon::from_set(name)` for dynamic set selection

Each set can optionally say what to embed:

```rust,ignore
aurora_iconify::icon_sets!(
    "mage",                         // the icons this crate refers to
    "lucide": ["arrow-left", "x"],  // ...plus these, always
    "tabler": all,                  // every icon in the set
);
```

### Type-safe access

```rust,ignore
aurora_iconify::icon_sets!("tabler");

let svg = Icon::tabler().arrow_left();
```

Icon names are sanitized to valid Rust identifiers:

| Icon name         | Method name         |
|-------------------|---------------------|
| `calendar-2-fill` | `calendar_2_fill()` |
| `arrow-left`      | `arrow_left()`      |
| `3d-rotate`       | `_3d_rotate()`      |
| `box`             | `box_icon()`        |

### Dynamic access

```rust,ignore
// By name within a known set:
let svg: Option<&str> = Icon::mage().by_name("calendar-2-fill");

// By set name and icon name:
let svg = Icon::from_set("mage").and_then(|s| s.by_name("calendar-2-fill"));
```

`by_name` finds the icons that are embedded: those your crate calls by method, those whose name appears in a string literal (as above), and those listed in `icon_sets!`. For a name that only exists at runtime — read from a config file, built with `format!` — list the candidates or embed the set:

```rust,ignore
aurora_iconify::icon_sets!("lucide": ["arrow-left", "arrow-right"]);

let svg = Icon::lucide().by_name(&format!("arrow-{direction}"));
```

## Which icons get embedded

The macro cannot see how its output is used, so it reads the source text of the crate that invokes it (every `.rs` file under the package directory) and embeds any icon whose name or method name appears there. This errs on the side of including too much: a variable called `search` embeds the ~500-byte `search` icon. It will not miss an icon for ordinary code, but it cannot see a reference when:

- **another crate** calls a re-exported `Icon` — the scan only covers the crate containing `icon_sets!`
- the method name is assembled by a macro (e.g. `paste!`)
- the code lives outside the package directory (`#[path = "../shared.rs"]`) or in a non-`.rs` file pulled in with `include!`

In those cases, list the icons or use `"set": all`. Calling an accessor whose icon was not embedded panics with a message naming the icon and the fix.

## Caching

Downloads are cached in `target/.iconify-cache/` relative to your workspace root (override with the `ICONIFY_CACHE_DIR` environment variable):

- Icon name lists are refreshed after 7 days. If the Iconify API is unreachable, the expired list keeps being used.
- Downloaded icons are kept until the cache is deleted (`cargo clean` removes it).

A build needs network access only when it uses an icon that has not been downloaded before. If that download fails, the build fails for icons your code clearly asks for (`.icon_name()` calls, quoted names, the explicit list); a word that merely coincides with an icon name is skipped.

Set `ICONIFY_API_URL` when building to use a self-hosted Iconify API.

## Available icon sets

Browse all available sets at [icon-sets.iconify.design](https://icon-sets.iconify.design). Popular sets include:

| Set              | Prefix             | Icons   |
|------------------|--------------------|---------|
| Material Symbols | `material-symbols` | 15,000+ |
| Lucide           | `lucide`           | 1,500+  |
| Tabler Icons     | `tabler`           | 5,600+  |
| Phosphor         | `ph`               | 7,400+  |
| Mage Icons       | `mage`             | 1,000+  |
| Simple Icons     | `simple-icons`     | 3,100+  |
| Heroicons        | `heroicons`        | 290+    |
| Feather Icons    | `feather`          | 280+    |

Unless you ask for `all`, the size of a set does not affect your binary. It adds a little compile time for the generated methods: roughly a second per rebuild of the invoking crate for the 15,000-icon `material-symbols`, and proportionally less for smaller sets.

## Upgrading from 0.1

`icon_sets!("a", "b")` still compiles, but 0.1 embedded every icon of every set. If you look icons up with `by_name` using names that never appear in your source, add `: all` (or a list of names) to those sets.

## Requirements

- Network access at compile time, the first time an icon is used
- Rust edition 2021 or later

## License

MIT OR Apache-2.0

# aurora_iconify

Compile-time icon embedding from [iconify.design](https://iconify.design) for Rust.

Type-safe accessors for every icon in the icon sets you enable, and the SVGs of the icons your crate uses are embedded in your binary as `&'static str`. More than 550,000 icons from 280 open-source sets, with no runtime HTTP requests and no asset files.

## Usage

Enable each icon set you need as a feature, then list the sets in `icon_sets!`:

```toml
[dependencies]
aurora_iconify = { version = "0.4", features = ["lucide", "material-symbols"] }
```

```rust,ignore
aurora_iconify::icon_sets!("lucide", "material-symbols");

fn main() {
    // Type-safe: one accessor per icon, and each returns the SVG.
    let svg: &str = Icon::lucide().trash();

    // Dynamic lookup by name.
    let svg: Option<&str> = Icon::lucide().by_name("trash");

    // Dynamic set and name.
    let svg = Icon::from_set("material-symbols")
        .and_then(|set| set.by_name("home"));
}
```

## How it works

- **Icon names are bundled.** The list of icons in each set ships with this crate, so builds never need the network to know what exists. Your editor can complete every icon of every enabled set.
- **Only the SVGs you use are downloaded and embedded.** At compile time, `icon_sets!` scans your crate's `.rs` files for icon names and downloads the SVGs it finds. Each one is cached, so a build needs the network only for an icon it has not seen before.
- **Every other accessor panics with an explanation** if it is called for an icon that was not embedded.

Adding `Icon::lucide().some_new_icon()` to your code is all it takes: the next build embeds that icon.

## Why the IDE stays fast

Older versions generated one method per icon into each crate that called `icon_sets!`. For a set like `material-symbols` (over 15,000 icons) that meant megabytes of generated code in the crate being edited. Editors re-expand a crate's macros when its items change, so every item-level edit paid for it.

Now the accessors for an enabled set live in this crate, which the editor indexes once. What `icon_sets!` produces depends only on the icons the crate embeds, so editing your code stays fast however many sets you enable.

Enabling a set still costs build time once, when this crate is compiled with that feature. Hover previews add to it; turn them off with `default-features = false` (and re-enable the sets you need) if you need faster builds.

## API

### `icon_sets!`

Declares the icon sets to use. Generates:

- `Icon`, with one function per declared set (`Icon::lucide()`), and `Icon::from_set(name)`.
- Per-set accessors with one method per icon (`LucideIcons::trash()`), and `by_name` on each set.

Each set can say what to embed:

```rust,ignore
aurora_iconify::icon_sets!(
    "mage",                         // the icons this crate refers to
    "lucide": ["arrow-left", "x"],  // ...plus these, always
    "tabler": all,                  // every icon in the set
);
```

A set must be both enabled as a feature and listed in `icon_sets!`. Listing a set without its feature is a compile error that names the feature to add.

### Type-safe access

Icon names are sanitized to valid Rust identifiers:

| Icon name         | Method name         |
|-------------------|---------------------|
| `calendar-2-fill` | `calendar_2_fill()` |
| `arrow-left`      | `arrow_left()`      |
| `3d-rotate`       | `_3d_rotate()`      |
| `box`             | `box_icon()`        |

If two names sanitize to the same identifier, the first one gets the method; the other stays reachable through `by_name`.

### Dynamic access

`by_name` finds the icons that are embedded: those the crate calls by method, those whose name appears in a string literal, and those listed in `icon_sets!`. For a name known only at run time, list the candidates or embed the set:

```rust,ignore
aurora_iconify::icon_sets!("lucide": ["arrow-left", "arrow-right"]);

let svg = Icon::lucide().by_name(&format!("arrow-{direction}"));
```

### Which icons get embedded

The macro cannot see how its output is used, so it reads the source text of the crate that invokes it (every `.rs` file under the package directory) and embeds any icon whose name or method name appears there. This errs on the side of including too much: a variable called `search` embeds the `search` icon. It will not miss an icon for ordinary code, but it cannot see a reference when:

- **another crate** calls an accessor re-exported from it. The scan covers only the crate containing `icon_sets!`.
- the method name is assembled by a macro (for example `paste!`).
- the code lives outside the package directory (`#[path = "../shared.rs"]`), or in a non-`.rs` file pulled in with `include!`.

In those cases, list the icons or use `"set": all`.

## Catalog and updates

The icon names are a snapshot. An icon added upstream after this release is not in the catalog until a new release, because the names are bundled. The catalog is refreshed with:

```bash
just iconify-catalog
```

which runs `cargo run -p xtask -- iconify-catalog` in the aurora-ui repository and needs network access. The API downloads SVGs at compile time for the icons you use, as before; set `ICONIFY_API_URL` to use a self-hosted Iconify API.

## Caching

SVGs are cached in `target/.iconify-cache/` relative to your workspace root, override with `ICONIFY_CACHE_DIR`. Each downloaded icon is kept until the cache is deleted (`cargo clean` removes it). Nothing expires, because an icon does not change once it is published.

If the API is unreachable, a build fails only for icons your code clearly asks for (`.icon_name()` calls, quoted names, the explicit list). A word that merely coincides with an icon name is skipped.

## Upgrading

### From 0.3

The only change you have to make is to enable the icon sets you use as features. The call sites stay the same: `Icon::lucide().trash()`, `by_name` and `Icon::from_set` work as before.

```toml
aurora_iconify = { version = "0.4", features = ["lucide"] }
```

Without the feature, `icon_sets!` fails with a message naming the feature to add.

Other changes:

- **Generic accessor types.** `LucideIcons` is now `LucideIcons<P>`, and `DynIconSet` is `DynIconSet<P>`. Code that only calls methods on `Icon` doesn't change. Code that names these types in a struct field or signature needs a type alias, with `Provider` being the type that `icon_sets!` generates in the module that calls it:

  ```rust,ignore
  type Lucide = aurora_iconify::LucideIcons<crate::aurora::Provider>;
  ```

- **Bundled icon names.** `icon_sets!` no longer downloads icon name lists. SVGs are still downloaded and cached as before. An icon added upstream after this release is only available after the next release.
- **`"set": all` skips missing icons.** If the API no longer serves an icon, it is left out and its accessor panics at run time. Previously the build failed.
- **Cache location.** The cache directory is `target/.iconify-cache/v3`. The old `v2` directory is ignored and can be deleted.

### From 0.1.x

0.1.x embedded every icon of every set listed in `icon_sets!`. Since 0.3, only the icons your crate refers to are embedded, unless you ask for `"set": all`. If you looked up icons with `by_name` using names that never appear in your source, add `: all` (or a list of names) to those sets.

Upgrading from 0.1.x also brings the 0.3 changes above: enable the sets as features, and update any type names that refer to the accessor types.

## Requirements

- Network access at compile time, the first time an icon is used
- Rust edition 2021 or later

## License

MIT OR Apache-2.0

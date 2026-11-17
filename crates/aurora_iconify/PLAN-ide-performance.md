# Plan: make `aurora_iconify` free in the IDE, for any number of icon sets

Handoff document for the agent implementing this. Read it fully before starting.

## Problem

Calling `icon_sets!("lucide", "material-symbols")` in an application crate makes
RustRover lag on every item-level edit (typing `pub struct X`, `impl X`, adding
`fn` at module level). Edits inside function bodies are unaffected. Removing
the large set makes the lag disappear.

### Root cause (confirmed with RustRover CPU snapshots, 2026-11-17)

- In 0.3.0, `codegen::write_set` emits **one method per icon in the set**
  (`lucide` 2,019 + `material-symbols` 15,760 = ~17.8k methods), each with a
  `#[doc = "![name](https://api.iconify.design/...)"]` attribute.
- Item-level edits are *structural* changes. RustRover then rebuilds the
  edited crate's DefMap, which re-expands every macro call in that crate. The
  snapshot shows the hot path as:
  `DefCollector.expandMacro → MacroExpansionSharedCache.cachedExpand →
  readExpansionResultOk → IOUtil.readLongString` (deserializing the whole
  multi-MB expansion as one string), followed by `createExpansionStub` (building
  an index from it). **Cost scales with the expansion's text size**, on every
  structural keystroke.
- Edits inside function bodies don't trigger a DefMap rebuild, which is why `Rc::` is fast.
- Library crates' DefMaps are **not** rebuilt when you edit an app crate. That
  is why the old `aurora_leaf` wrapper crate didn't lag. 0.3.0 can't use a
  leaf crate, because source scanning (`CARGO_MANIFEST_DIR`) must run in the
  crate that uses the icons.

### Requirement

Users can include **any number of icon sets** (iconify has 280 sets, ~555k
icons as of 2026-11-17) with:

1. no IDE slowdown in the crate that calls the macro,
2. full completion of every icon in every included set,
3. only referenced icons embedded in the binary (keep 0.3.0's behaviour),
4. the existing call style kept: `Icon::lucide().trash() -> &'static str`.

## Design

**Rule: the macro expansion in the user's crate must stay small no matter how
many sets or icons are available. It may grow only with the number of icons
the crate actually embeds.** Everything sized by the catalog (all icon
names and methods) moves into a library crate, which RustRover indexes once.

### Crate layout (new, breaking → 0.4.0)

| Crate | Kind | Contents |
|---|---|---|
| `aurora_iconify` | lib (facade) | `Provider` trait, generic `Icon<P>` + one accessor type per set (`LucideIcons<P>` …) with **all** methods, `DynIconSet`, re-exports `icon_sets!`. One cargo feature per set. |
| `aurora_iconify_macros` | proc-macro | `icon_sets!`. Scans sources and emits only the provider: the blob plus index of **embedded** icons, and `pub type Icon = ::aurora_iconify::Icon<Provider>;` |
| `aurora_iconify_catalog` | lib (data) | Bundled icon name lists for every set (compressed), with set IDs. Used by the facade's `build.rs` **and** by the macro crate, so both see the identical list. |

Pin the three crates to each other with `=x.y.z` versions. Icon IDs are only
valid within one catalog version.

### How the pieces fit

- **Facade `build.rs`:** for each enabled set feature, read the bundled name
  list from `aurora_iconify_catalog` and write `$OUT_DIR/sets/<prefix>.rs`. That file holds:
  - `pub struct <Set>Icons<P>(PhantomData<P>);`
  - `impl<P: Provider> <Set>Icons<P> { #[inline] #[track_caller] pub fn <icon>(&self) -> &'static str { P::get(SetId::<Set>, <icon_id>).unwrap_or_else(|| missing(...)) } … }`
  - plus a `by_name`.

  Include the file with `include!(concat!(env!("OUT_DIR"), …))` behind
  `#[cfg(feature = "<prefix>")]`. **No network access in `build.rs`**: the
  names come from the catalog crate, so docs.rs and offline builds work.
  RustRover evaluates build scripts by default and indexes `OUT_DIR` includes.
- **`Icon<P>`:** add `impl<P: Provider> Icon<P> { pub fn lucide() -> LucideIcons<P> … }`, with each
  method cfg-gated by its feature. `Icon::from_set(&str) -> Option<DynIconSet<P>>`
  is kept.
- **The macro (`icon_sets!`):**
  1. Parse the input (keep the current grammar: `"set"`, `"set": [..]`, `"set": all`).
  2. Check that each listed set is an enabled feature. The facade forwards each set
     feature to the macros crate (`lucide = ["aurora_iconify_macros/lucide", …]`),
     so the macro can check with `cfg!(feature = …)`. If a set isn't enabled, emit a
     `compile_error!` that says which feature to add.
  3. Scan sources exactly as `scan.rs` does today and work out which icon IDs
     to embed. Use the catalog name list, **not** the downloaded `names.txt`.
  4. Download any missing SVGs into the cache (keep `cache.rs`/`fetch.rs` for SVGs).
  5. Emit only: `pub struct Provider;`, `impl ::aurora_iconify::Provider for Provider { fn get(set, id) … }`,
     the data, and the `Icon` type alias. **No per-icon items.**
- **Data for `"set": all`:** don't emit a 15k-entry `INDEX` literal in the
  expansion, because that's the same problem again. Write blob and index to cache files and
  load them with `include_bytes!`/`include_str!`, so the expansion stays a few
  lines for any set size. Do the same once the embedded count passes a
  threshold (e.g. > 64 icons). Below it, inline literals are fine.
- **Lookup:** `get(set: SetId, id: u32)` uses a `match` on `set`, then a binary search on a
  sorted `&[u32]` of embedded IDs. The calls are `#[inline]` with constant
  arguments; check that the binary doesn't regress (see Verification).
- **Hover previews:** the per-method `#[doc = "![…](url)"]` is now one-time
  library cost, so it may stay. Put it behind a default-on `previews` feature so
  users can drop it to shrink the catalog crate's compile time.

### Why not other options

- *Emit methods only for referenced icons:* the expansion becomes small, but you lose
  completion of icons you haven't used yet (fails requirement 2). RustRover also caches
  expansions keyed on the macro's input text, so newly referenced icons stay
  unresolved in the IDE until it re-expands.
- *Leaf crate per user:* breaks source scanning; this is the problem 0.3.0 set out to remove.
- *Generating catalog code inside the proc macro in the facade:* this works, but
  `build.rs` producing plain Rust avoids a proc-macro expansion layer in the IDE
  for 15k-method files, and it's simpler to debug.

## Implementation steps

1. **Workspace split:** create `crates/aurora_iconify_macros` (move `lib.rs`
   macro entry, `input.rs`, `scan.rs`, `cache.rs`, `fetch.rs`, `names.rs`) and
   `crates/aurora_iconify_catalog`. Turn `aurora_iconify` into a plain lib that
   re-exports the macro. Add the new members to `aurora-ui/Cargo.toml`.
2. **Catalog data + refresh tool:** add a Rust `xtask` (or `just` recipe calling
   a Rust bin; **no Python**) that downloads `https://api.iconify.design/collections`
   and each set's name list, sorts and dedups the names, and writes them as compressed
   files in `aurora_iconify_catalog`. It also regenerates the feature lists in the
   facade's and macros crate's `Cargo.toml`. Commit the generated data.
   - Check the packaged `.crate` size (`cargo package --list` / `cargo package`)
     against crates.io's 10 MB limit.
   - 280 sets is close to crates.io's 300-features-per-crate default limit.
     Options: request a higher limit, or split catalogs alphabetically into
     several crates. Decide before publishing and record the decision in the README.
3. **Shared naming:** method names come from `names::sanitize_ident` plus
   collision handling ("first one wins", plus `by_name` reserved). Move this into
   `aurora_iconify_catalog` so `build.rs` and the macro can't disagree.
   Icon IDs = index in the sorted, deduplicated catalog list.
4. **Facade `build.rs` codegen:** port `codegen::write_set`'s method loop to
   generate the generic accessor types. Use `cargo:rerun-if-changed` on nothing
   but the catalog version. Feature-gate per set.
5. **Rewrite the macro's codegen:** `codegen::generate` now emits only the provider,
   data and alias. Remove `MODULE_FUNCTIONS`' per-crate `missing`; the facade
   owns the panic message (`#[track_caller]` on the accessor keeps the caller
   location).
6. **Keep the dynamic API:** `Icon::from_set`, `DynIconSet::by_name`, and
   `<Set>Icons::by_name` resolve name → ID via the catalog's sorted name table
   (in the facade, behind the set's feature), then call `P::get`.
7. **Docs:** update the README for 0.4.0: the new `Cargo.toml` features, the
   migration from 0.3 (add the set features; the call site is unchanged), a section on why the
   IDE stays fast, and the catalog refresh process.
8. **Downstream migration (todo_app_throwdown):** bump to 0.4, add
   `features = ["lucide", "material-symbols"]` in the workspace dep, keep
   `aurora_iconify::icon_sets!(...)` in `apps/rust/{iced,gpui}/src/aurora.rs`.
   Also check `aurora_fonts::font_families!` for the same per-item expansion
   pattern.

## Verification

- **Expansion size test (the regression guard):** unit-test the macro codegen
  with 1 set vs. 10 sets, each referencing 3 icons. The output must be under ~4 KB
  and must **not** grow with the number of sets or catalog size. With `"set": all` it must
  still be under a few KB (data goes through `include_*!`).
- **Generated catalog parses:** `syn::parse_file` on every generated set file, as
  `output_is_valid_rust` does today.
- **Behaviour tests:** a `trybuild` or example crate that:
  - uses an embedded icon (returns its SVG),
  - calls an unreferenced icon (panics, and the location points at the caller),
  - uses `by_name` hits and misses, and `from_set`,
  - lists a set without its feature (clear `compile_error!`),
  - has two icon names that sanitize to the same identifier.
- **Binary size:** compare the release binary of `examples/` before and after with 6 icons
  embedded. The lookup must not pull in the catalog name tables unless the
  dynamic `by_name` API is used (check with `cargo bloat` or a symbol listing).
- **Build time:** `cargo build --timings`, clean, with `lucide` + `material-symbols`
  enabled. Record the facade crate's time and make sure it's reasonable (it's a one-time cost).
- **IDE (manual, the actual goal):** in todo_app_throwdown with 5+ large sets
  enabled, type `pub struct Foo` / `impl Foo` in `apps/rust/iced`. Capture a
  RustRover CPU snapshot (Help → Diagnostic Tools → CPU and memory snapshot) and
  check that `MacroExpansionSharedCache.readExpansionResultOk` no longer
  shows up meaningfully. Also check that completion after `Icon::material_symbols().` still lists every
  icon.
- `cargo clippy --all-targets` and `cargo test` across the aurora-ui workspace pass.

## Risks to watch

- **Catalog staleness:** new upstream icons appear only after a catalog
  refresh and release. Document this. Optional: an escape hatch to embed a
  name that isn't in the catalog via `by_name`, using the old download path.
- **Catalog version mismatch** between facade and macros crates would silently map IDs to the wrong
  icons. Prevent it with exact `=` version pins, and a const
  `CATALOG_HASH` that the macro emits and the facade checks with a const assertion.
- **The features limit** (see step 2).
- **Facade compile time** with many large sets enabled: if it's heavy, the `previews` feature
  off is the first lever. Generic methods are type-checked once and only monomorphized
  when called.

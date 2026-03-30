# Contributing to AuroraUI

## Branch Naming

All changes must be submitted via pull request from a named branch. Use the following format:

```
{username}/{type}-{description}
```

**Types:**

| Prefix     | Use when                                              |
|------------|-------------------------------------------------------|
| `feat`     | Adding new functionality                              |
| `fix`      | Fixing a bug                                          |
| `bug`      | Investigating or fixing a reported bug                |
| `issue`    | Addressing a specific GitHub issue (use the issue ID) |
| `refactor` | Restructuring code without changing behavior          |
| `docs`     | Documentation-only changes                            |
| `chore`    | Build, CI, or tooling changes                         |

**Examples:**

```
drew/feat-custom-titlebar-linux
drew/fix-softbuffer-resize-crash
drew/issue-42
drew/refactor-widget-event-dispatch
drew/docs-canvas-api
```

## Workflow

1. Create a branch from `master` using the naming convention above.
2. Make your changes with clear, focused commits.
3. Ensure CI passes locally before pushing:
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo fmt --all -- --check
   ```
   Or run the full Docker-based CI locally:
   ```bash
   just cicd
   ```
   Note: `just cicd` only covers Linux. macOS and Windows checks run on GitHub.
4. Push your branch and open a pull request against `master`.
5. CI must pass on all platforms (Ubuntu, macOS, Windows) before merging.

## CI Checks

Every pull request runs the following on all three platforms:

- **Clippy** with `-D warnings` (zero warnings policy)
- **Tests** for the full workspace and with `--no-default-features`
- **Release build** of the facade crate
- **Formatting** check (`cargo fmt`, Ubuntu only)
- **Benchmarks** (Ubuntu only)

## Code Style

- Run `cargo fmt` before committing.
- No `thiserror` or `anyhow` -- implement `Display` + `Error` manually.
- Keep `aurora_core` at zero external dependencies.
- Remove unused dependencies immediately.
- Feature-gate anything that increases binary size beyond a blank window.

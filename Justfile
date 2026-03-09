set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]
set shell := ["bash", "-c"]

# Builds the rustdocs for the aurora_ui package with the option to open it.
doc open="false":
    cargo doc --no-deps --package aurora_ui --release {{ if open == "true" { "--open" } else { "" } }}

# Runs the minimal example project in release mode
run-example-minimal:
    cargo run --package aurora_minimal_example --profile release
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]
set shell := ["bash", "-c"]

# Builds the rustdocs for the aurora_ui package with the option to open it.
doc open="false":
    cargo doc --no-deps --package aurora_ui --release {{ if open == "true" { "--open" } else { "" } }}

# Builds all examples in release mode and reports binary size, startup time, and memory usage
[windows]
benchmark:
    cargo build --release --workspace
    @echo ""
    @echo "Example                        Binary (KB)  Startup (ms)   Memory (KB)"
    @echo "-------                        -----------  ------------   -----------"
    @foreach ($bin in (Get-ChildItem target/release/*.exe | Where-Object { $_.Name -match '_example\.exe$' })) { $size = [math]::Round($bin.Length / 1024); $env:AURORA_BENCHMARK = "1"; $output = & $bin.FullName 2>&1 | Out-String; $startup = if ($output -match 'STARTUP_MS=([^\r\n]+)') { $Matches[1] } else { "N/A" }; $memory = if ($output -match 'MEMORY_KB=([^\r\n]+)') { $Matches[1] } else { "N/A" }; $name = $bin.Name -replace '\.exe$',''; Write-Host ("{0,-30} {1,12} {2,14} {3,12}" -f $name, $size, $startup, $memory) }

# Builds all examples in release mode and reports binary size, startup time, and memory usage
[linux]
[macos]
benchmark:
    #!/usr/bin/env bash
    set -e
    cargo build --release --workspace 2>/dev/null
    echo ""
    printf "%-30s %12s %14s %12s\n" "Example" "Binary (KB)" "Startup (ms)" "Memory (KB)"
    printf "%-30s %12s %14s %12s\n" "-------" "-----------" "------------" "-----------"
    for binary in target/release/*_example; do
        [ -f "$binary" ] || continue
        name=$(basename "$binary")
        size_kb=$(( $(wc -c < "$binary") / 1024 ))
        output=$(AURORA_BENCHMARK=1 timeout 15 "$binary" 2>&1 || true)
        startup=$(echo "$output" | grep "STARTUP_MS=" | head -1 | cut -d= -f2)
        memory=$(echo "$output" | grep "MEMORY_KB=" | head -1 | cut -d= -f2)
        printf "%-30s %12s %14s %12s\n" "$name" "${size_kb:-N/A}" "${startup:-N/A}" "${memory:-N/A}"
    done

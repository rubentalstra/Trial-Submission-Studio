#!/usr/bin/env bash
# Cross-platform clippy checks for supported targets
# Run this locally to verify platform-specific code compiles correctly
#
# LIMITATIONS:
# - Cross-compiling to different OS families (e.g., macOS → Windows) requires
#   C cross-compilation toolchains for crates with native dependencies (like 'ring')
# - This script checks targets within the same OS family natively
# - For full cross-platform validation, use CI (which runs on native runners)

set -euo pipefail

# Detect current OS
case "$(uname -s)" in
    Darwin*)
        OS="macos"
        TARGETS=(
            "aarch64-apple-darwin"
            "x86_64-apple-darwin"
        )
        ;;
    Linux*)
        OS="linux"
        TARGETS=(
            "x86_64-unknown-linux-gnu"
            # aarch64-unknown-linux-gnu requires cross-compilation toolchain
        )
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS="windows"
        TARGETS=(
            "x86_64-pc-windows-msvc"
            # aarch64-pc-windows-msvc requires ARM64 toolchain
        )
        ;;
    *)
        echo "Unsupported OS: $(uname -s)"
        exit 1
        ;;
esac

echo "Running cross-platform clippy checks on $OS..."
echo "Targets: ${TARGETS[*]}"
echo ""
echo "Note: For full 6-target validation, push to CI."
echo ""

FAILED=()

for target in "${TARGETS[@]}"; do
    echo "=========================================="
    echo "Checking target: $target"
    echo "=========================================="

    if cargo clippy --target "$target" --all-features -- -D warnings; then
        echo "✓ $target passed"
    else
        echo "✗ $target failed"
        FAILED+=("$target")
    fi
    echo ""
done

if [ ${#FAILED[@]} -eq 0 ]; then
    echo "=========================================="
    echo "All local cross-platform checks passed!"
    echo "=========================================="
    exit 0
else
    echo "=========================================="
    echo "FAILED targets: ${FAILED[*]}"
    echo "=========================================="
    exit 1
fi

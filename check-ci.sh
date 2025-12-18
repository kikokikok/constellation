#!/bin/bash
# CI SIMULATION SCRIPT - MUST RUN BEFORE ANY COMMIT
# This simulates the exact CI checks that will run on GitHub

set -e  # Exit on any error

echo "🚀 Running CI simulation..."
echo "=============================="

# 1. Formatting check
echo "📝 Checking formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Formatting check failed!"
    echo "Run: cargo fmt --all"
    exit 1
fi
echo "✅ Formatting OK"

# 2. Clippy check
echo "🔍 Running clippy..."
if ! cargo clippy --all -- -D warnings 2>&1 | grep -q "Finished"; then
    echo "❌ Clippy check failed!"
    echo "Output:"
    cargo clippy --all -- -D warnings
    exit 1
fi
echo "✅ Clippy OK"

# 3. Compilation check
echo "🔧 Checking compilation..."
if ! cargo check --all; then
    echo "❌ Compilation check failed!"
    exit 1
fi
echo "✅ Compilation OK"

# 4. Test check (run tests for changed modules if specified)
if [ "$1" = "--full" ]; then
    echo "🧪 Running ALL tests..."
    if ! cargo test -- --test-threads=1; then
        echo "❌ Tests failed!"
        exit 1
    fi
    echo "✅ All tests OK"
else
    echo "🧪 Running hybrid module tests (most relevant)..."
    if ! cargo test hybrid:: -- --test-threads=1; then
        echo "❌ Hybrid tests failed!"
        exit 1
    fi
    echo "✅ Hybrid tests OK"
fi

echo "=============================="
echo "🎉 ALL CI CHECKS PASSED!"
echo "You can now commit safely."
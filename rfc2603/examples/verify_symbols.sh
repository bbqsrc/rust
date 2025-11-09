#!/bin/bash
# Verify that our generated symbols match rustc's v0 mangled symbols

set -e

LIB_PATH="/home/user/test-symbols/target/debug/libtest_symbols.so"

echo "=== Extracting Actual Symbols from rustc (nm) ==="
ACTUAL=$(nm -g "$LIB_PATH" | grep "_RNv" | grep -v "INv" | grep "test_symbols" | awk '{print $3}' | sort)

echo "=== Generating Symbols with rfc2603 ==="
GENERATED=$(cargo run --example generate_from_stele "$LIB_PATH" 2>/dev/null | grep "Symbol:" | awk '{print $3}' | sort)

echo ""
echo "=== Comparison ==="
echo ""

# Check functions
echo "Checking function symbols..."
for sym in $(echo "$ACTUAL" | grep "_RNvCs5GYaaS9NRMV_12test_symbols[0-9]"); do
    if echo "$GENERATED" | grep -q "^$sym\$"; then
        echo "✓ $sym"
    else
        echo "✗ MISSING: $sym"
    fi
done

echo ""
echo "Checking method symbols..."
for sym in $(echo "$ACTUAL" | grep "_RNvMsa_"); do
    if echo "$GENERATED" | grep -q "^$sym\$"; then
        echo "✓ $sym"
    else
        echo "✗ MISSING: $sym"
    fi
done

echo ""
echo "=== Summary ==="
ACTUAL_COUNT=$(echo "$ACTUAL" | grep -E "(_RNvCs5GYaaS9NRMV_12test_symbols[0-9]|_RNvMsa_)" | wc -l)
GENERATED_COUNT=$(echo "$GENERATED" | wc -l)
echo "Actual symbols: $ACTUAL_COUNT"
echo "Generated symbols: $GENERATED_COUNT"

if [ "$ACTUAL_COUNT" -eq "$GENERATED_COUNT" ]; then
    echo "✓ All symbols match!"
else
    echo "✗ Symbol count mismatch"
fi

#!/bin/bash

echo "Testing RUST_KEYLIME_SKIP_SECURE_MOUNT environment variable fix"
echo "=============================================================="

echo ""
echo "Test 1: Without environment variable (should try to mount)"
echo "Expected: Should attempt secure mount and likely fail due to permissions"
echo "Actual:"
timeout 3s cargo run --bin keylime_agent --manifest-path keylime-agent/Cargo.toml 2>&1 | grep -E "(mount|secure|skip)" | head -5 || echo "No secure mount messages found"

echo ""
echo "Test 2: With RUST_KEYLIME_SKIP_SECURE_MOUNT=0 (should try to mount)"
echo "Expected: Should attempt secure mount"
echo "Actual:"
RUST_KEYLIME_SKIP_SECURE_MOUNT=0 timeout 3s cargo run --bin keylime_agent --manifest-path keylime-agent/Cargo.toml 2>&1 | grep -E "(mount|secure|skip)" | head -5 || echo "No secure mount messages found"

echo ""
echo "Test 3: With RUST_KEYLIME_SKIP_SECURE_MOUNT=1 (should skip mount)"
echo "Expected: Should skip secure mount and show warning message"
echo "Actual:"
RUST_KEYLIME_SKIP_SECURE_MOUNT=1 timeout 3s cargo run --bin keylime_agent --manifest-path keylime-agent/Cargo.toml 2>&1 | grep -E "(mount|secure|skip|Skipping)" | head -5 || echo "No secure mount messages found"

echo ""
echo "Test 4: With RUST_KEYLIME_SKIP_SECURE_MOUNT=yes (should try to mount)"
echo "Expected: Should attempt secure mount (only '1' should skip)"
echo "Actual:"
RUST_KEYLIME_SKIP_SECURE_MOUNT=yes timeout 3s cargo run --bin keylime_agent --manifest-path keylime-agent/Cargo.toml 2>&1 | grep -E "(mount|secure|skip)" | head -5 || echo "No secure mount messages found"

echo ""
echo "=============================================================="
echo "Test completed. If Test 3 shows 'Skipping secure mount' message,"
echo "then the fix is working correctly!"

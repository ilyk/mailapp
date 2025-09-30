#!/bin/bash

echo "Testing single-instance behavior..."
echo ""

echo "Starting first instance (should register with DBus and stay running):"
echo "----------------------------------------"
cd /home/ilyk/projects/pets/mailapp
cargo run --manifest-path app/Cargo.toml --bin asgard-mail -- --demo &
FIRST_PID=$!

echo "First instance PID: $FIRST_PID"
echo "Waiting 3 seconds for first instance to start..."
sleep 3

echo ""
echo "Starting second instance (should detect first instance and exit):"
echo "----------------------------------------"
cargo run --manifest-path app/Cargo.toml --bin asgard-mail -- --demo &
SECOND_PID=$!

echo "Second instance PID: $SECOND_PID"
echo "Waiting 2 seconds for second instance to exit..."
sleep 2

echo ""
echo "Checking if first instance is still running:"
if kill -0 $FIRST_PID 2>/dev/null; then
    echo "✅ First instance is still running (PID: $FIRST_PID)"
else
    echo "❌ First instance has exited"
fi

echo ""
echo "Checking if second instance has exited:"
if kill -0 $SECOND_PID 2>/dev/null; then
    echo "❌ Second instance is still running (PID: $SECOND_PID)"
else
    echo "✅ Second instance has exited as expected"
fi

echo ""
echo "Cleaning up..."
kill $FIRST_PID 2>/dev/null || true
kill $SECOND_PID 2>/dev/null || true

echo "Test completed!"

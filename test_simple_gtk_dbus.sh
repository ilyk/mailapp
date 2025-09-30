#!/bin/bash

echo "🧪 Testing DBus Single-Instance Behavior in Simple GTK App"
echo "========================================================"
echo ""

echo "📋 Instructions:"
echo "1. First instance should register with DBus and show window"
echo "2. Second instance should detect first instance and exit"
echo "3. X button should hide window (app stays running)"
echo "4. Exit menu should quit application"
echo "5. DBus can show hidden window"
echo ""

echo "🚀 Starting first instance..."
echo "----------------------------------------"
cd /home/ilyk/projects/pets/mailapp
cargo run --bin simple-gtk-app &
FIRST_PID=$!

echo "First instance PID: $FIRST_PID"
echo "Waiting 5 seconds for first instance to start..."
sleep 5

echo ""
echo "🚀 Starting second instance (should detect first and exit)..."
echo "----------------------------------------"
cargo run --bin simple-gtk-app &
SECOND_PID=$!

echo "Second instance PID: $SECOND_PID"
echo "Waiting 3 seconds for second instance to exit..."
sleep 3

echo ""
echo "🔍 Checking process status:"
echo "----------------------------------------"
if kill -0 $FIRST_PID 2>/dev/null; then
    echo "✅ First instance is still running (PID: $FIRST_PID)"
else
    echo "❌ First instance has exited"
fi

if kill -0 $SECOND_PID 2>/dev/null; then
    echo "❌ Second instance is still running (PID: $SECOND_PID)"
else
    echo "✅ Second instance has exited as expected"
fi

echo ""
echo "🧹 Cleaning up..."
kill $FIRST_PID 2>/dev/null || true
kill $SECOND_PID 2>/dev/null || true

echo ""
echo "✅ Test completed!"
echo ""
echo "📝 Manual testing:"
echo "1. Run: cargo run --bin simple-gtk-app"
echo "2. Try closing window with X button (should hide)"
echo "3. Use Exit menu to quit"
echo "4. Test DBus: dbus-send --session --dest=com.asgard.Mail --type=method_call --print-reply /com/asgard/Mail com.asgard.Mail.ShowWindow"

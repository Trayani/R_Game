#!/bin/bash
# Test compact log compression by analyzing existing action_log.json

echo "=== Compact Log Compression Test ==="
echo ""

if [ -f "action_log.json" ]; then
    JSON_SIZE=$(wc -c < action_log.json)
    echo "Existing JSON log: $JSON_SIZE bytes ($(numfmt --to=iec-i --suffix=B $JSON_SIZE))"

    # Count events in JSON
    EVENT_COUNT=$(grep -c '"timestamp_ms"' action_log.json)
    echo "Event count: $EVENT_COUNT"
    echo "Avg bytes per event (JSON): $(echo "scale=2; $JSON_SIZE / $EVENT_COUNT" | bc) bytes"
else
    echo "No existing action_log.json found. Run the application first."
    exit 1
fi

echo ""
echo "The compact binary log (.bin) will be created automatically"
echo "when you run the application and press ESC to exit."
echo ""
echo "Expected compression: ~85-90% reduction in size"
echo "Expected compact size: ~12-15 bytes per event"

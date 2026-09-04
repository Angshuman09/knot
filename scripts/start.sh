#!/bin/bash

trap 'kill $(jobs -p) 2>/dev/null; exit 0' SIGINT SIGTERM EXIT

echo "Starting Knot leader..."
cargo run -q -p node -- leader &
LEADER_PID=$!

echo "Starting Knot follower..."
cargo run -q -p node -- follower &
FOLLOWER_PID=$!

echo "Leader PID: $LEADER_PID"
echo "Follower PID: $FOLLOWER_PID"
echo "Press Ctrl+C to stop all nodes."

wait
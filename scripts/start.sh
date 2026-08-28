#!/bin/bash

echo "Starting Knot leader..."
cargo run -p node -- leader \
  --client-addr 127.0.0.1:9000 \
  --follower-addr 127.0.0.1:9001 &

LEADER_PID=$!

echo "Starting Knot follower..."
cargo run -p node -- follower \
  --client-addr 127.0.0.1:9010 \
  --leader-client-addr 127.0.0.1:9000 \
  --leader-follower-addr 127.0.0.1:9001 &

FOLLOWER_PID=$!

echo "Leader PID: $LEADER_PID"
echo "Follower PID: $FOLLOWER_PID"

wait
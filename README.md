# Knot

Knot is a distributed event-sourced ledger engine built in Rust.

The system includes a leader that accepts operations and followers that replicate the event log. Followers can catch up on missed events and receive new events in real time.

The ledger supports deposits, withdrawals, transfers, and balance queries, with the state machine keeping account balances in sync with the event log.


Single-leader replication, logical (row-based) log shipping:

- One leader accepts client writes (deposit, withdraw, transfer), assigns
  each one the next offset in an in-memory event log, and applies it to a
  local account-balance table.
- Any number of followers connect to the leader on a separate port, catch
  up on whatever they are missing, and then receive new entries as they are
  committed. Followers replay the same log independently to arrive at the
  same balances, no balance data is ever sent directly.
- Writes are only accepted by the leader.

Full wire protocol and message formats: see `SPEC.md`.


## design notes

- Amounts are `i64` cents, never floating point.
- Log offsets are 1-based. Offset 0 means "nothing applied yet" and is
  the default subscribe position for a brand new follower.
- A transfer is represented as two separate events, `TransferDebit` and
  `TransferCredit`, sharing one `transfer_id`, rather than as one atomic
  event. This is deliberate: it makes a follower's partial replication of
  a transfer (debit applied, credit not yet arrived) directly observable
  once `follower.rs` exists.
- `StateMachine::apply` never rejects an event. Validation (e.g.
  insufficient funds) happens once, on the leader, before an event is
  appended, once something is in the log, every replica is obligated to
  apply it identically, or replicas would diverge.

## run it

### Start the Nodes
Open two terminal windows (or use scripts/start.sh):

Terminal-1 **Leader Node**:

```sh
cargo run -p node -- leader
```

Terminal-2 **Follower Node** (replicates in real-time):

```sh
cargo run -p node -- follower 
```

### Test Operations with the Client CLI
You can use (or use scripts/client.sh):

1. **Deposit**

Deposit funds into accounts on the leader:

```sh
# Deposit $100.00 into alice
cargo run -p client -- deposit alice 100.00
# Output: ok, committed at offset 1
```
```sh
# Deposit $50.00 into bob
cargo run -p client -- deposit bob 50.00
# Output: ok, committed at offset 2
```

2. **Withdraw**
Withdraw funds from an account (with balance check):
```sh
# Successful withdrawal
cargo run -p client -- withdraw alice 25.00
# Output: ok, committed at offset 3

# Insufficient funds check (alice now has $75.00)
cargo run -p client -- withdraw alice 200.00
# Output: error: insufficinet funds in alice
```

3. **Transfer**
Transfer funds between two accounts (generates two paired events: TransferDebit and TransferCredit):
```sh
# Transfer $30.00 from alice to bob
cargo run -p client -- transfer alice bob 30.00
# Output: ok, committed at offset 5 (offset 4 was Debit, offset 5 was Credit)

# Insufficient funds transfer attempt
cargo run -p client -- transfer alice bob 500.00
# Output: error: insufficient funds in alice
```

4. **Check Balance** (Read Path & Replication Verification)
Read directly from the Leader (:9000):
```sh
cargo run -p client -- balance alice
# Output: alice: $45.00 (as of offset 5)

cargo run -p client -- balance bob
# Output: bob: $80.00 (as of offset 5)
```

Read from the Follower (:9010) to verify replication:
```sh
cargo run -p client -- 127.0.0.1:9010 balance alice
# Output: alice: $45.00 (as of offset 5)
```

5. **Test Read-Your-Own-Writes Consistency** (--read-after)

You can ask the follower to ensure it has synced up to at least a specific offset before returning the balance:
```sh
cargo run -p client -- 127.0.0.1:9010 balance alice --read-after 5
# Output: alice: $45.00 (as of offset 5)
```

6.  Test Single-Leader Write Rejection
If a client attempts to send a write directly to a follower:
```sh
cargo run -p client -- 127.0.0.1:9010 deposit alice 10.00
# Output: not the leader — writes go to 127.0.0.1:9000
```

## architecture & data flow
[ARCHITECTURE.md](docs/ARCHITECTURE.md)

## references: 

- Chapter-5(Replication) of **Designing Data-Intensive Applications** by Martin Kleppmann
- [Event Sourcing Blog by Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Replication Blog by Arpit Bhayani](https://arpitbhayani.me/blogs/read-your-write-consistency/)

## license

MIT. See `LICENSE`.
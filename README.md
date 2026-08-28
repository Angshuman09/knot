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

## Sequence Diagram

```mermaid
sequenceDiagram
    autonumber

    actor Client
    participant L as Leader :9000
    participant LL as Leader Ledger
    participant F as Follower :9010
    participant FL as Follower Ledger

    Note over F,L: Background Replication Setup
    F->>L: Connect to :9001 and Subscribe from offset 0

    Note over Client,L: 1. Write Path
    Client->>L: Deposit alice $50
    L->>LL: Append event
    LL->>LL: Apply event to StateMachine
    L-->>Client: WriteAck offset 1

    Note over L,F: 2. Replication
    L->>F: Replicate LogEntry offset 1
    F->>FL: Append entry and apply event

    Note over Client,F: 3. Read Path
    Client->>F: GetBalance alice, read_after 1
    Note over F: Wait until local offset >= 1
    F->>FL: Read alice balance
    FL-->>F: $50.00
    F-->>Client: Balance $50.00, offset 1
    ```

## Design notes

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

## References: 

- **Designing Data-Intensive Applications** by Martin Kleppmann
  - Chapter 5: Replication
- [Event Sourcing Blog](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Replication Blog](https://arpitbhayani.me/blogs/read-your-write-consistency/)

## License

MIT. See `LICENSE`.
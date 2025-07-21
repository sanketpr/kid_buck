# 🐐 Kid-Buck

**Kid-Buck** is a fast, memory-efficient payment processor written in 🦀 Rust. It ingests transactions via CSV  and models client account states with full lifecycle logic—including deposits, withdrawals, disputes, resolutions, and chargebacks.

Thanks to Rust’s [`csv`](https://docs.rs/csv/latest/csv/) crate, **Kid-Buck** leverages its default line-by-line streaming behavior. This means transactions are read and processed incrementally without buffering the entire file into memory—allowing it to handle very large datasets with minimal memory overhead.


---

## 📁 Features

- Efficient CSV streaming, built for handling large datasets
- Implements transaction validation rules that are enforced even before we begin processing transaction.
- Support for all transaction types:
  - `deposit` — Adds funds to available balance
  - `withdrawal` — Deducts funds from available balance if sufficient
  - `dispute` — Moves funds from available to held
  - `resolve` — Releases held funds back to available
  - `chargeback` — Removes held funds and freezes client account
- Maintains per-client accounts with:
  - `available` balance
  - `held` balance
  - `total` balance
  - `locked` status

---

## 🚀 How to Run

1. **Prepare your CSV**
   - Format:
     ```csv
     type,client,tx,amount
     deposit,1,1,100.00
     withdrawal,1,2,50.00
     dispute,1,1,
     ```

2. **Run with CSV input**
   ```bash
   cargo run -- transactions.csv
    ```

## ⚠️ Known Limitations

This processor is designed for learning purposes, and while it handles all major transaction types, there are several known limitations:

- **Chargeback without prior dispute**  
  `chargeback` actions can be executed even if the associated transaction was never disputed, which breaks the intended flow of `dispute → chargeback`.

- **Synchronous processing limits concurrency**  
  The current implementation processes each transaction sequentially, which can be limiting for responsiveness and scalability—especially when integrated with streaming systems like NATS. Refactoring to use `async` functions and Rust's `tokio` runtime would allow concurrent transaction handling, better I/O responsiveness, and improved throughput in real-time environments.

- **Transaction ID duplication**  
  The system assumes the transaction ids are not duplicated and does not enforce uniqueness for `tx` IDs. This means multiple transactions may share the same ID, which could lead to ambiguity and logic conflicts.

- **Dispute can cause negative available balance**  
  Disputing a transaction subtracts its amount from the `available` balance—even if that balance isn't sufficient—potentially resulting in a negative value.

- **Multiple disputes per transaction**  
  Transactions can be disputed multiple times because there's no state-tracking to prevent redundant dispute actions on the same `tx`.

- **Limited error propagation**  
  The processor uses `String` messages or early `return`s for invalid transactions instead of leveraging Rust's rich `Result<T, E>` patterns and custom error enums, which would improve robustness and diagnostics.

- **Suboptimal lifetime and borrowing**  
  Some temporary allocations and borrowed values could benefit from more precise lifetime annotations or refactoring to reduce unnecessary cloning and improve memory footprint.

---

## 🚧 Suggestions for Improvement

To harden the processor and increase correctness:

- Track the lifecycle of each transaction (`Disputed`, `Resolved`, `ChargedBack`)
- Enforce one-time dispute actions per transaction
- Non-blocking or asynchronous processing of transaction.
- Ensure `tx` uniqueness across the dataset
- Block disputes when available balance is insufficient
- Improve validation and error feedback via custom error types
- Improve memory usage with tighter control over variable mutability, borrowing and lifetimes

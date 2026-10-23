# Simple Summary Implementation Plan

**Goal:** Send fitting transcripts once; split only on context/output limits, combine once, never ask AI to review or repair.
**Spec:** User instruction in this chat, 2026-10-23; supersedes complete-part consistency sweeps.
**Architecture:** Keep provider transports, stable source IDs, cancellation and atomic cache writes. Use one bounded generation per source packet and one final combination for long input. Reject incomplete/invalid responses locally without paid repair. No fixed 24KiB target; whole rows fit the actual request budget. Output truncation splits source with bounded depth, never continues partial text. Final combination does not retry or recursively reduce.
**Tech Stack:** Existing Rust/Tauri/Vue, no dependencies.
**Constraints:** No push/release; preserve production data; live tests use unchanged original transcript, isolated stores and existing DeepSeek credentials. No other agents; existing chat is idle.

- [x] Add loopback regressions proving one direct call beyond 24KiB, full long-source coverage, single combination, no repair, cancellation/cache preservation, output-limit splitting and bounded failure.
- [x] Replace generation/pipeline, retire consistency/layout/replacement production modules, update obsolete tests and progress UI.
- [x] Run fmt/clippy/cargo tests, frontend typecheck/lint/tests; test original transcript in three formats, inspect outputs against source; record exact request counts and limitations.

Review focus: provider finish markers; indivisible large rows; citations absent from supplied excerpt; cancellation before save; final combination overflow. These are exercised through real pipeline/loopback tests and existing transport tests. No semantic correctness claim follows from mock tests.

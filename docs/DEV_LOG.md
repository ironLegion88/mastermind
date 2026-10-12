# Development Log (DEV_LOG.md)

This log is maintained chronologically throughout the development of the Master Mind / Bulls and Cows project. It records development phases, objectives, decisions, commands executed, test results, limitations, and Git commits.

---

## 2026-10-12: Phase 0 — Workspace Inspection & Project Setup

### Objective
Inspect the existing project workspace, determine whether an existing Rust project and Git repository exist, check Rust toolchain versions, examine existing code and configuration, and establish the baseline project state before making any modifications.

### Actions & Findings
1. **Directory Inspection**:
   - Workspace root: `d:\Ashoka\Monsoon 2026\Programming Lab\Mid-term`
   - Found an existing subfolder `mastermind`.
   - Contents of `mastermind`: `.git/`, `.gitignore`, `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `target/`.
2. **Git Repository Status**:
   - Checked `git status` inside `mastermind`:
     - Initialized repository on branch `main` with 0 commits.
     - Untracked files: `.gitignore`, `Cargo.toml`, `Cargo.lock`, `src/main.rs`.
3. **Toolchain & Configuration**:
   - `Cargo.toml` specification:
     - Name: `mastermind`
     - Version: `0.1.0`
     - Edition: `2024`
     - Dependencies: None initially.
   - Rust toolchain:
     - `rustc 1.98.1 (48a229cea 2026-09-01)`
     - `cargo 1.98.1 (797e8a9bc 2026-08-05)`
   - Pre-existing code:
     - `src/main.rs` contained default `fn main() { println!("Hello, world!"); }`.
     - No pre-existing user modifications or existing tests were present.
4. **Execution & Baseline Check**:
   - Executed `cargo check`: Finished successfully in 0.05s.
   - Staged initial project files and created the initial repository commit:
     - Command: `git add .gitignore Cargo.toml Cargo.lock src/main.rs`
     - Command: `git commit -m "chore: initial cargo project structure"`
     - Commit Hash: `29f5a99`

### Status & Next Steps
- Baseline workspace clean and verified.
- Proceed to Phase 1: Record pre-implementation design baseline.

---

## 2026-10-12: Phase 1 — Pre-Implementation Design Baseline

### Objective
Formulate and document the pre-implementation design decisions, architectural boundaries, data structure trade-offs, duplicate-safe scoring algorithm, error handling model, and testing strategy prior to writing implementation code.

### Actions & Design Decisions
1. **Design Documentation (`docs/DESIGN.md`)**:
   - Formulated Section 1: Problem and Requirements (Bulls and Cows rules, duplicate handling constraint, terminal interface requirements).
   - Formulated Section 2: Pre-implementation Design Baseline (`M = 4`, `N = 6`, `[u32; M]` representation, `score_guess` slice signature).
   - Formulated Section 3: Program Architecture (`generate_target`, `parse_guess`, `score_guess`, `play_game`, `main`).
   - Formulated Section 4: Data Structures (Trade-off analysis between fixed arrays `[u32; M]` and dynamic vectors `Vec<u32>`, temporary vector buffering during parsing, and future runtime difficulty extension).
   - Formulated Section 5: Mutability and Borrowing (Immutability by default for target and guess, borrowed slices for scoring, localized mutation for matching markers and input buffers).
   - Formulated Section 6: Functions vs. Macros (Rationale for using standard functions for algorithms and logic, and reserving macros exclusively for formatted I/O and test assertions).
   - Formulated Section 7: Duplicate-Safe Scoring Algorithm (Detailed two-pass algorithm with boolean match markers preventing duplicate overcounting, accompanied by correctness proof).
   - Formulated Section 8: Input Validation and Control Flow (Pipeline for tokenization, length validation, integer parsing, range verification, and custom `GuessError` enum).
   - Formulated Section 9: Alternatives & Trade-offs comparison table.
   - Formulated Section 10: Three-tier testing strategy (Deterministic unit tests, reproducible randomized differential testing with seeded `StdRng`, and manual/stream integration tests).
   - Formulated Section 11 & 12: Limitations and deviations.
2. **Commit Baseline**:
   - Staged documentation files (`docs/DESIGN.md` and `docs/DEV_LOG.md`) and committed.
   - Commit Hash: `400ddf9`

### Status & Next Steps
- Baseline design documented and committed.
- Proceed to Phase 2: Incremental implementation starting with constants, project configuration, and duplicate-safe scoring algorithm.

---

## 2026-10-12: Phase 2, Step 1 — Constants and Project Configuration

### Objective
Define the compile-time sequence length ($M = 4$) and peg value upper bound ($N = 6$), and configure the project dependency for random number generation (`rand = "0.8"`).

### Actions & Design Decisions
1. **Dependency Configuration (`Cargo.toml`)**:
   - Added `rand = "0.8"` under `[dependencies]` to support random target generation and seeded PRNG testing.
   - Built dependencies via `cargo check`, compiling `rand v0.8.8` and its sub-crates.
2. **Compile-time Constants (`src/main.rs`)**:
   - Defined `pub const M: usize = 4;` (sequence length).
   - Defined `pub const N: u32 = 6;` (peg range upper bound, valid values $1..=N$).
   - Kept $M$ and $N$ as centrally defined constants with doc-comments to avoid magic numbers throughout the codebase.
3. **Verification**:
   - Executed `cargo check`: Finished successfully.
   - Executed `cargo fmt --check`: Clean formatting verified.

### Status & Next Steps
- Project configured and constants defined.
- Commit Step 1.
- Proceed to Step 2: Implement the duplicate-safe scoring function `score_guess`.


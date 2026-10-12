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
- Commit Hash: `160f6c4`
- Proceed to Step 2: Implement the duplicate-safe scoring function `score_guess`.

---

## 2026-10-12: Phase 2, Steps 2–4 — Duplicate-Safe Scoring Function & Testing

### Objective
Implement the duplicate-safe `score_guess` function using the two-pass marker algorithm, write deterministic unit tests covering all edge cases, implement an independent multiset frequency reference scorer, and perform reproducible randomized differential testing.

### Actions & Design Decisions
1. **Scoring Function Implementation (`src/main.rs`)**:
   - Implemented `pub fn score_guess(target: &[u32], guess: &[u32]) -> (usize, usize)`.
   - Function borrows both sequences as immutable slices `&[u32]`, enforcing non-consumption and non-mutation of inputs.
   - Preconditions enforced with `assert_eq!` validating length equals $M$.
   - Allocated stack-based boolean markers `target_used: [bool; M]` and `guess_used: [bool; M]` initialized to `false` (zero heap allocation).
   - **Pass 1 (Bulls)**: Identified exact index matches `target[i] == guess[i]`, marked both indices as `true`, incremented `bulls`.
   - **Pass 2 (Cows)**: For unmatched guess indices, scanned for unmatched target indices with identical values. Upon first match, marked both as `true`, incremented `cows`, and executed `break` to prevent multiple matches for the same peg.
2. **Deterministic Unit Tests (`src/main.rs`)**:
   - Implemented 10 deterministic test functions covering:
     - All Bulls (`[1, 2, 3, 4]` vs `[1, 2, 3, 4]`, and all identical `[3, 3, 3, 3]`).
     - All Cows (`[1, 2, 3, 4]` vs `[4, 3, 2, 1]`, and misplaced pairs `[1, 1, 2, 2]` vs `[2, 2, 1, 1]`).
     - Mixed Bulls and Cows (`[1, 2, 3, 4]` vs `[1, 3, 2, 5]` $\to (1, 2)$).
     - No matches (`[1, 2, 3, 4]` vs `[5, 6, 5, 6]`).
     - Repeated values in target vs single in guess (`[1, 1, 2, 3]` vs `[1, 4, 5, 6]` and `[4, 5, 1, 6]`).
     - Repeated values in guess vs single in target (`[1, 2, 3, 4]` vs `[1, 1, 1, 1]`).
     - Duplicate overcounting traps (`[1, 2, 2, 3]` vs `[2, 2, 4, 2]` $\to (1, 1)$).
     - Entirely outside matching set (`[1, 2, 3, 4]` vs `[6, 6, 6, 6]`).
3. **Reference Scorer & Randomized Differential Testing**:
   - Implemented `reference_score_guess(target: &[u32], guess: &[u32]) -> (usize, usize)` using multiset frequency counts:
     - $\text{bulls} = \sum [target[i] == guess[i]]$
     - $\text{total\_matches} = \sum_{v=1}^N \min(\text{freq}_T(v), \text{freq}_G(v))$
     - $\text{cows} = \text{total\_matches} - \text{bulls}$
   - Created reproducible randomized test with `StdRng::seed_from_u64(42)` executing 10,000 random target/guess pairs.
   - Verified 5 essential invariants across all 10,000 cases:
     - Exact agreement between `score_guess` and `reference_score_guess`.
     - $\text{bulls} \le M$ and $\text{cows} \le M$.
     - $\text{bulls} + \text{cows} \le M$.
     - Self-comparison: $\text{score}(A, A) = (M, 0)$.
     - Reverse-permutation multiset invariant: $\text{bulls} + \text{cows} = M$.
4. **Test Failures & Resolutions**:
   - *First test run*: 10 tests passed, 2 failed.
     - `test_repeated_values_in_target_single_in_guess`: Test setup had `target = [1, 1, 2, 3]` and `guess = [4, 1, 5, 6]` expecting `(0, 1)`, but at index 1 both target and guess contained 1 (an exact Bull). Corrected guess to `[4, 5, 1, 6]` so peg 1 was placed at index 2 where target had 2.
     - `test_duplicate_overcounting_edge_cases`: Test setup had `target = [1, 1, 1, 2]` and `guess = [3, 4, 1, 1]` expecting `(0, 2)`, but index 2 was an exact match (Bull). Corrected target to `[1, 1, 2, 2]` with guess `[3, 4, 1, 1]`.
   - *Second test run*: All 12 unit and randomized tests passed cleanly in 0.03s.
5. **Verification**:
   - `cargo check`: Success.
   - `cargo fmt --check`: Clean.
   - `cargo test`: 12 passed, 0 failed.

### Status & Next Steps
- Scoring algorithm and tests completed and verified.
- Commit Hash: `eaac432`
- Proceed to Step 5: Input parsing and validation (`parse_guess`).

---

## 2026-10-12: Phase 2, Step 5 — Input Parsing and Validation

### Objective
Implement input parsing and validation logic in `parse_guess`, supporting whitespace separation, quit command detection, integer parsing, token count verification, peg range validation, safe conversion into `[u32; M]`, and informative error reporting via `GuessError`.

### Actions & Design Decisions
1. **Error Enum (`src/main.rs`)**:
   - Defined `GuessError` with variants:
     - `Quit`: player requested graceful game termination.
     - `EmptyInput`: player entered an empty line or only whitespace.
     - `InvalidLength { expected: usize, actual: usize }`: count mismatch.
     - `NonInteger(String)`: token cannot parse as an integer.
     - `OutOfRange { value: u32, min: u32, max: u32 }`: integer parsed outside $1..=N$.
   - Implemented `std::fmt::Display` and `std::error::Error` for user-facing terminal messages.
2. **Parsing Function (`src/main.rs`)**:
   - Implemented `pub fn parse_guess(input: &str) -> Result<[u32; M], GuessError>`.
   - Tokenization via `split_whitespace()` natively supports single spaces, multiple spaces, and tabs.
   - Checked `quit` (case-insensitive) prior to numeric parsing.
   - Parsed tokens into a temporary `Vec<u32>` buffer.
   - Validated length against $M$ and value range against $1..=N$.
   - Safely copied elements from `Vec<u32>` into fixed array `[u32; M]` using `copy_from_slice()`.
3. **Unit Tests for Parsing**:
   - Implemented 8 dedicated unit test functions:
     - `test_parse_guess_valid`: valid inputs and repetitions.
     - `test_parse_guess_whitespace_variations`: multiple spaces, tabs, leading/trailing whitespace.
     - `test_parse_guess_quit_command`: case-insensitive variations ("quit", "QUIT", "  quit  \t").
     - `test_parse_guess_empty_input`: empty strings and whitespace-only lines.
     - `test_parse_guess_invalid_length`: too few (1, 3) and too many (5, 6) numbers.
     - `test_parse_guess_non_integer_tokens`: alphabetic strings, floats, negatives.
     - `test_parse_guess_out_of_range`: below 1, above 6, large numbers.
     - `test_guess_error_display`: formatted error messages.
4. **Verification**:
   - `cargo check`: Passed.
   - `cargo fmt --check`: Clean formatting verified.
   - `cargo test`: All 20 tests passed cleanly in 0.03s.

### Status & Next Steps
- Input parsing and validation complete and verified.
- Commit Step 5.
- Proceed to Step 6: Random target generation (`generate_target`).




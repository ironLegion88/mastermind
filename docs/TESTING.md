# Mastermind (Bulls and Cows) — Testing Methodology & Verification

This document records the comprehensive testing strategy, test cases, and actual observed results for the Master Mind / Bulls and Cows implementation.

---

## 1. Testing Strategy Overview

The testing methodology is organized into three distinct, complementary tiers:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Deterministic Unit Tests                                │
│    - score_guess: edge cases, duplicates, overcounting traps│
│    - parse_guess: whitespace, quit, bounds, non-integers    │
│    - generate_target: range, length, repetition freedom    │
├─────────────────────────────────────────────────────────────┤
│ 2. Reproducible Randomized Differential Testing            │
│    - 10,000 seeded PRNG iterations (StdRng seed 42)         │
│    - Cross-validated against independent reference scorer   │
│    - 5 mathematical invariants checked on every case        │
├─────────────────────────────────────────────────────────────┤
│ 3. Automated Stream Integration & Manual Testing            │
│    - Generic Reader/Writer simulation via Cursor<&[u8]>     │
│    - Error recovery without attempt penalty                 │
│    - Full session lifecycle: welcome, play, replay, quit   │
│    - Interactive CLI piping verification                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Deterministic Unit Tests (`score_guess`)

Deterministic unit tests verify known inputs against analytically determined outputs. All tests reside in `src/main.rs` under `#[cfg(test)] mod tests` and execute via `cargo test`.

### Test Case Verification Table

| Test Function | Target | Guess | Expected (Bulls, Cows) | Observed Result | Pass/Fail | Rationale & Focus |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `test_all_bulls_unique` | `[1, 2, 3, 4]` | `[1, 2, 3, 4]` | `(4, 0)` | `(4, 0)` | **PASS** | Exact identity with unique elements. |
| `test_all_bulls_with_duplicates` | `[2, 2, 5, 5]` | `[2, 2, 5, 5]` | `(4, 0)` | `(4, 0)` | **PASS** | Identity with duplicate pairs. |
| `test_all_bulls_with_duplicates` | `[3, 3, 3, 3]` | `[3, 3, 3, 3]` | `(4, 0)` | `(4, 0)` | **PASS** | All elements identical. |
| `test_all_cows_complete_misplacement` | `[1, 2, 3, 4]` | `[4, 3, 2, 1]` | `(0, 4)` | `(0, 4)` | **PASS** | Reversed order; 0 Bulls, 4 Cows. |
| `test_all_cows_complete_misplacement` | `[1, 2, 3, 4]` | `[2, 3, 4, 1]` | `(0, 4)` | `(0, 4)` | **PASS** | Cyclically shifted; 0 Bulls, 4 Cows. |
| `test_all_cows_with_duplicates` | `[1, 1, 2, 2]` | `[2, 2, 1, 1]` | `(0, 4)` | `(0, 4)` | **PASS** | Swapped duplicate pairs; 4 Cows. |
| `test_all_cows_with_duplicates` | `[1, 2, 1, 2]` | `[2, 1, 2, 1]` | `(0, 4)` | `(0, 4)` | **PASS** | Alternating duplicate pairs; 4 Cows. |
| `test_mixed_bulls_and_cows` | `[1, 2, 3, 4]` | `[1, 3, 2, 5]` | `(1, 2)` | `(1, 2)` | **PASS** | 1 Bull at pos 0; 2 Cows at pos 1, 2; 5 absent. |
| `test_mixed_bulls_and_cows` | `[1, 2, 3, 4]` | `[1, 2, 4, 3]` | `(2, 2)` | `(2, 2)` | **PASS** | 2 Bulls (pos 0, 1); 2 Cows (pos 2, 3). |
| `test_no_matches` | `[1, 2, 3, 4]` | `[5, 6, 5, 6]` | `(0, 0)` | `(0, 0)` | **PASS** | Disjoint sets of values. |
| `test_repeated_target_single_guess` | `[1, 1, 2, 3]` | `[1, 4, 5, 6]` | `(1, 0)` | `(1, 0)` | **PASS** | Target has two 1s; guess has one 1 (Bull); no extra Cow. |
| `test_repeated_target_single_guess` | `[1, 1, 2, 3]` | `[4, 5, 1, 6]` | `(0, 1)` | `(0, 1)` | **PASS** | Misplaced 1; single Cow awarded despite two target 1s. |
| `test_repeated_guess_single_target` | `[1, 2, 3, 4]` | `[1, 1, 1, 1]` | `(1, 0)` | `(1, 0)` | **PASS** | Guess has four 1s; pos 0 is Bull; three extra 1s NOT counted as Cows. |
| `test_repeated_guess_single_target` | `[1, 2, 3, 4]` | `[5, 1, 1, 1]` | `(0, 1)` | `(0, 1)` | **PASS** | Three 1s in guess match single 1 in target $\to$ exactly 1 Cow. |
| `test_duplicate_overcounting_edge_cases` | `[1, 2, 2, 3]` | `[2, 2, 4, 2]` | `(1, 1)` | `(1, 1)` | **PASS** | Classic trap: pos 1 Bull consumes one 2; single remaining 2 awards 1 Cow; third 2 in guess ignored. |
| `test_duplicate_overcounting_edge_cases` | `[1, 1, 2, 2]` | `[3, 4, 1, 1]` | `(0, 2)` | `(0, 2)` | **PASS** | Guess has two 1s matching two target 1s off-position. |
| `test_duplicate_overcounting_edge_cases` | `[1, 1, 3, 4]` | `[1, 5, 1, 1]` | `(1, 1)` | `(1, 1)` | **PASS** | Pos 0 Bull; pos 2 Cow; third 1 ignored. |
| `test_outside_matching_set` | `[1, 2, 3, 4]` | `[6, 6, 6, 6]` | `(0, 0)` | `(0, 0)` | **PASS** | Single non-matching value repeated 4 times. |

---

## 3. Randomized Differential Testing

To guarantee correctness beyond hand-picked test cases, we implement differential testing against an independently written multiset frequency scorer.

### 3.1 Independent Reference Algorithm
The reference algorithm (`reference_score_guess`) operates on completely different principles:
1. **Bulls**: Coordinate-wise equality $\sum_{i=0}^{M-1} [target[i] == guess[i]]$.
2. **Total Matches**: Multiset intersection cardinality $\sum_{v=1}^N \min(f_\text{target}(v), f_\text{guess}(v))$.
3. **Cows**: Derived mathematically by subtraction: $\text{cows} = \text{total\_matches} - \text{bulls}$.

### 3.2 Methodology & Parameters
- **RNG**: `rand::rngs::StdRng` initialized with fixed seed `42`.
- **Reproducibility**: 100% deterministic; will produce the identical sequence of test cases on any platform or machine.
- **Iterations**: 10,000 randomly generated target/guess pairs.
- **Target & Guess Distribution**: $M = 4$, values independently drawn with replacement from uniform distribution $1..=N$ ($1..=6$).

### 3.3 Invariants Verified per Iteration
Across every one of the 10,000 iterations, the following 5 invariants were checked:
1. **Differential Equivalence**: `score_guess(&target, &guess) == reference_score_guess(&target, &guess)`
2. **Individual Bounds**: $\text{bulls} \le M$ and $\text{cows} \le M$
3. **Total Bounds**: $\text{bulls} + \text{cows} \le M$
4. **Reflexivity (Self-Comparison)**: $\text{score}(A, A) = (M, 0)$
5. **Permutation Multiset Conservation**: If guess is a permutation (e.g. reverse) of target, then $\text{bulls} + \text{cows} = M$.

**Observed Result**: 10,000 out of 10,000 iterations passed with zero failures in 0.03 seconds.

---

## 4. Input Parsing & Validation Unit Tests (`parse_guess`)

| Input String | Expected Result | Observed Result | Pass/Fail | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| `"1 2 3 4"` | `Ok([1, 2, 3, 4])` | `Ok([1, 2, 3, 4])` | **PASS** | Valid standard input. |
| `"2 2 5 5"` | `Ok([2, 2, 5, 5])` | `Ok([2, 2, 5, 5])` | **PASS** | Valid repeated numbers allowed. |
| `"1   2    3     4"` | `Ok([1, 2, 3, 4])` | `Ok([1, 2, 3, 4])` | **PASS** | Multiple spaces between tokens. |
| `"   1 2 3 4   "` | `Ok([1, 2, 3, 4])` | `Ok([1, 2, 3, 4])` | **PASS** | Leading and trailing spaces. |
| `"\t1 \t 2  3\t 4\n"` | `Ok([1, 2, 3, 4])` | `Ok([1, 2, 3, 4])` | **PASS** | Tabs and newlines. |
| `"quit"` | `Err(GuessError::Quit)` | `Err(GuessError::Quit)` | **PASS** | Lowercase quit command. |
| `"QUIT"` | `Err(GuessError::Quit)` | `Err(GuessError::Quit)` | **PASS** | Uppercase quit command. |
| `"Quit"` | `Err(GuessError::Quit)` | `Err(GuessError::Quit)` | **PASS** | Mixed-case quit command. |
| `"  quit  \t"` | `Err(GuessError::Quit)` | `Err(GuessError::Quit)` | **PASS** | Quit with surrounding whitespace. |
| `""` | `Err(GuessError::EmptyInput)` | `Err(GuessError::EmptyInput)` | **PASS** | Empty input string. |
| `"   "` | `Err(GuessError::EmptyInput)` | `Err(GuessError::EmptyInput)` | **PASS** | Whitespace-only input string. |
| `"1 2 3"` | `Err(InvalidLength { 4, 3 })` | `Err(InvalidLength { 4, 3 })` | **PASS** | Too few tokens (3 instead of 4). |
| `"1 2 3 4 5"` | `Err(InvalidLength { 4, 5 })` | `Err(InvalidLength { 4, 5 })` | **PASS** | Too many tokens (5 instead of 4). |
| `"1 2 abc 4"` | `Err(NonInteger("abc"))` | `Err(NonInteger("abc"))` | **PASS** | Alphabetic token. |
| `"1 2.5 3 4"` | `Err(NonInteger("2.5"))` | `Err(NonInteger("2.5"))` | **PASS** | Floating-point token. |
| `"1 -2 3 4"` | `Err(NonInteger("-2"))` | `Err(NonInteger("-2"))` | **PASS** | Negative number token. |
| `"0 2 3 4"` | `Err(OutOfRange { 0, 1, 6 })` | `Err(OutOfRange { 0, 1, 6 })` | **PASS** | Below lower bound ($0 < 1$). |
| `"1 2 7 4"` | `Err(OutOfRange { 7, 1, 6 })` | `Err(OutOfRange { 7, 1, 6 })` | **PASS** | Above upper bound ($7 > 6$). |

---

## 5. Automated Stream Integration Tests

The interactive gameplay loop was tested by mocking standard I/O with `std::io::Cursor<&[u8]>` (reader) and `Vec<u8>` (writer):

1. **`test_play_game_win_first_guess`**:
   - Target: `[1, 2, 3, 4]`.
   - Input: `"1 2 3 4\n"`.
   - Result: `GameOutcome::Won { attempts: 1 }`.
   - Verification: Terminal output contained `"Result: 4 Bulls, 0 Cows"` and victory banner.
2. **`test_play_game_multi_guess_with_invalid_attempts`**:
   - Target: `[1, 2, 3, 4]`.
   - Inputs:
     1. `"1 2\n"` $\to$ Error: expected 4, got 2 (attempts = 0)
     2. `"1 2 9 4\n"` $\to$ Error: value 9 out of range (attempts = 0)
     3. `"1 2 x 4\n"` $\to$ Error: invalid token 'x' (attempts = 0)
     4. `"   \n"` $\to$ Error: input was empty (attempts = 0)
     5. `"2 1 4 3\n"` $\to$ Valid guess 1: 0 Bulls, 4 Cows (attempts = 1)
     6. `"1 2 3 4\n"` $\to$ Valid guess 2: 4 Bulls, 0 Cows (attempts = 2)
   - Result: `GameOutcome::Won { attempts: 2 }`.
   - Verification: Verified that malformed attempts were rejected with informative messages and were **never counted** toward the player's attempt tally.
3. **`test_play_game_quit_midway`**:
   - Target: `[1, 2, 3, 4]`.
   - Inputs: `"2 2 5 5\nquit\n"`.
   - Result: `GameOutcome::Quit`.
   - Verification: Output contained `"Result: 1 Bull, 0 Cows"`, revealed the secret code upon quit, and terminated cleanly.
4. **`test_play_game_eof_handling`**:
   - Simulated closed input stream (0 bytes).
   - Result: `GameOutcome::Eof`.
   - Verification: Handled cleanly without hanging or infinite loop.
5. **`test_run_session_quit_immediately`**:
   - Full coordinator test with immediate `"quit\n"`.
   - Result: `Ok(())`.
   - Verification: Printed welcome banner, accepted quit, printed goodbye.

---

## 6. Manual CLI End-to-End Verification

Executed `cargo run` in PowerShell with piped input to test actual process execution:

### Command
```powershell
"1 2`n1 2 7 4`n1 2 x 4`nquit`n" | cargo run
```

### Observed Terminal Output
```
================================================
     Welcome to Master Mind / Bulls & Cows      
================================================
Game Configuration: M = 4 pegs, N = 6 max value (range 1..=6)
Rules:
 - Bulls: Correct number in the correct position.
 - Cows:  Correct number in an incorrect position.
 - Repeated numbers are permitted in targets and guesses.
 - Type 'quit' at any prompt to exit.
================================================

A new secret code has been generated (4 numbers between 1 and 6).
Enter 4 numbers separated by spaces (or type 'quit' to exit):

Guess #1: Error: Invalid number of values: expected 4, but received 2.
Please enter your guess again.

Guess #1: Error: Value 7 is out of range: each number must be between 1 and 6 (inclusive).
Please enter your guess again.

Guess #1: Error: Invalid token 'x': must be a positive integer.
Please enter your guess again.

Guess #1: 
You decided to quit. The secret code was: [3, 1, 6, 4].
Thank you for playing Master Mind! Goodbye.
```

### Analysis of Observed Output
- Welcome banner displayed configuration parameters $M=4$ and $N=6$.
- Malformed inputs were cleanly rejected with clear diagnostic explanations.
- The prompt counter (`Guess #1:`) remained at 1 across all invalid inputs.
- The `quit` command was recognized gracefully, revealed the target sequence, and exited with status 0.

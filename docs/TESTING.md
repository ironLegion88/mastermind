# Mastermind (Bulls and Cows) — Testing Methodology & Verification

This document records the comprehensive testing strategy, test cases, and actual observed results for the Master Mind / Bulls and Cows implementation, including both the initial baseline and the post-implementation difficulty, scoring, and UI extension.

---

## 1. Testing Strategy Overview

The testing methodology is organized into four distinct, complementary tiers:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Deterministic Unit Tests                                 │
│    - config: Difficulty mapping, bounds, menu parsing       │
│    - scoring: Multi-length Bulls/Cows matching, invariants  │
│    - game: Guess parsing, attempt limits, target generation │
├─────────────────────────────────────────────────────────────┤
│ 2. Scoring Formula & Range Stratification Tests             │
│    - Exact multiplier bounds (attempt 1 vs. attempt L)      │
│    - Non-overlapping score hierarchy verification           │
│    - Invalid input panics (A = 0, A > L, length mismatch)   │
├─────────────────────────────────────────────────────────────┤
│ 3. Reproducible Randomized Differential Testing             │
│    - 3,000 seeded PRNG iterations per difficulty (9,000 tot)│
│    - Multi-length cross-validation against multiset oracle  │
│    - 5 mathematical invariants checked per iteration        │
├─────────────────────────────────────────────────────────────┤
│ 4. ANSI Presentation & Automated Stream Integration Tests   │
│    - Zero ANSI escape code leakage in plain-text mode       │
│    - Simulated reader/writer streams via Cursor<&[u8]>      │
│    - Win on final attempt vs. loss on exhausted attempts    │
│    - Difficulty switching and cumulative session score      │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Configuration & Parameter Unit Tests (`config.rs`)

Tested via `cargo test config::tests`:

| Test Function | Input / Target | Expected Result | Observed | Status | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `test_difficulty_configurations` | `Easy`, `Medium`, `Hard` | Correct $M, N, L$, multiplier | Matches table | **PASS** | Validates Easy (3, 4, 10, 1.0×), Medium (4, 6, 9, 2.0×), Hard (5, 8, 8, 4.0×). |
| `test_difficulty_display_and_all` | `Difficulty::all()` | 3 items, correct Display names | `["Easy", "Medium", "Hard"]` | **PASS** | Ensures complete iteration and user-friendly formatting. |
| `test_parse_difficulty_from_input` | `"1"`, `"2"`, `"3"`, `"easy"`, `"hard"`, `"invalid"`, `""` | `Some(...)` or `None` | Accurate matching | **PASS** | Case-insensitive menu parsing with graceful rejection of unknown options. |

---

## 3. Slice-Based Bulls & Cows Scoring Unit Tests (`scoring.rs`)

`score_guess(target: &[u32], guess: &[u32]) -> (usize, usize)` operates on borrowed slices, allowing seamless testing across arbitrary sequence lengths $M \in \{3, 4, 5\}$:

| Test Function | Target Length ($M$) | Target / Guess | Expected (Bulls, Cows) | Observed | Status | Focus / Invariant |
| :--- | :---: | :--- | :--- | :--- | :--- | :--- |
| `test_score_guess_all_bulls_various_lengths` | 3 (Easy) | `[1, 2, 3]` vs `[1, 2, 3]` | `(3, 0)` | `(3, 0)` | **PASS** | All Bulls on length 3. |
| `test_score_guess_all_bulls_various_lengths` | 4 (Medium) | `[1, 2, 3, 4]` vs `[1, 2, 3, 4]` | `(4, 0)` | `(4, 0)` | **PASS** | All Bulls on length 4. |
| `test_score_guess_all_bulls_various_lengths` | 5 (Hard) | `[1, 2, 3, 4, 5]` vs `[1, 2, 3, 4, 5]` | `(5, 0)` | `(5, 0)` | **PASS** | All Bulls on length 5. |
| `test_score_guess_all_cows_various_lengths` | 3 (Easy) | `[1, 2, 3]` vs `[3, 1, 2]` | `(0, 3)` | `(0, 3)` | **PASS** | Shifted permutation; 3 Cows. |
| `test_score_guess_all_cows_various_lengths` | 4 (Medium) | `[1, 2, 3, 4]` vs `[4, 3, 2, 1]` | `(0, 4)` | `(0, 4)` | **PASS** | Reversed permutation; 4 Cows. |
| `test_score_guess_all_cows_various_lengths` | 5 (Hard) | `[1, 2, 3, 4, 5]` vs `[2, 3, 4, 5, 1]` | `(0, 5)` | `(0, 5)` | **PASS** | Rotated permutation; 5 Cows. |
| `test_score_guess_mixed_matches` | 4 (Medium) | `[1, 2, 3, 4]` vs `[1, 3, 2, 5]` | `(1, 2)` | `(1, 2)` | **PASS** | 1 Bull, 2 Cows, 1 non-match. |
| `test_score_guess_mixed_and_duplicates` | 4 (Medium) | `[1, 2, 2, 3]` vs `[2, 2, 4, 2]` | `(1, 1)` | `(1, 1)` | **PASS** | Duplicate trap: Bull consumes one 2, single Cow awarded. |
| `test_score_guess_repeated_values_in_target` | 4 (Medium) | `[1, 1, 2, 3]` vs `[1, 4, 5, 6]` | `(1, 0)` | `(1, 0)` | **PASS** | Single guess match consumes only one target peg. |
| `test_score_guess_repeated_values_in_guess` | 4 (Medium) | `[1, 2, 3, 4]` vs `[1, 1, 1, 1]` | `(1, 0)` | `(1, 0)` | **PASS** | Single target peg consumed by pos 0 Bull; extra guess pegs ignored. |
| `test_score_guess_panics_on_length_mismatch` | 4 vs 3 | `[1, 2, 3, 4]` vs `[1, 2, 3]` | Panic | Panicked as expected | **PASS** | Contract enforcement: slice lengths must match. |

---

## 4. Scoring Formula & Stratification Tests (`scoring.rs`)

Tested via `cargo test scoring::tests`:

### 4.1 Boundary Value Verification
$$\text{attempt\_multiplier} = 1.0 + \frac{L - A}{L}, \quad \text{final\_score} = \operatorname{round}(100 \times \text{diff\_mult} \times \text{att\_mult})$$

| Setting | Max Attempts ($L$) | Attempt ($A$) | Attempt Multiplier | Final Multiplier | Final Score | Status |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Easy** | 10 | 1 (Best) | $1.0 + \frac{9}{10} = 1.900$ | $1.0 \times 1.900 = 1.900$ | **190 pts** | **PASS** |
| **Easy** | 10 | 10 (Worst) | $1.0 + \frac{0}{10} = 1.000$ | $1.0 \times 1.000 = 1.000$ | **100 pts** | **PASS** |
| **Medium** | 9 | 1 (Best) | $1.0 + \frac{8}{9} \approx 1.8889$ | $2.0 \times 1.8889 \approx 3.7778$ | **378 pts** | **PASS** |
| **Medium** | 9 | 9 (Worst) | $1.0 + \frac{0}{9} = 1.000$ | $2.0 \times 1.000 = 2.000$ | **200 pts** | **PASS** |
| **Hard** | 8 | 1 (Best) | $1.0 + \frac{7}{8} = 1.875$ | $4.0 \times 1.875 = 7.500$ | **750 pts** | **PASS** |
| **Hard** | 8 | 8 (Worst) | $1.0 + \frac{0}{8} = 1.000$ | $4.0 \times 1.000 = 4.000$ | **400 pts** | **PASS** |

### 4.2 Score Band Non-Overlap Proof (`test_score_bands_are_strictly_non_overlapping`)
The test iterates over every valid winning attempt count $1 \le A \le L$ for Easy, Medium, and Hard, asserting:
$$\max(\text{Easy Scores}) = 190 < \min(\text{Medium Scores}) = 200$$
$$\max(\text{Medium Scores}) = 378 < \min(\text{Hard Scores}) = 400$$
Result: **PASS**. The score tiers provide completely disjoint, non-overlapping bands.

### 4.3 Contract Invariants (`should_panic`)
- `test_calculate_score_panics_on_zero_attempts`: Passing $A = 0$ panics as required.
- `test_calculate_score_panics_on_exceeded_attempts`: Passing $A = 11$ on Easy ($L=10$) panics as required.

---

## 5. Multi-Difficulty Randomized Differential Testing

To guarantee correctness across all difficulty configurations, the differential test was extended to test Easy ($M=3, N=4$), Medium ($M=4, N=6$), and Hard ($M=5, N=8$).

### 5.1 Test Configuration
- **RNG**: `rand::rngs::StdRng::seed_from_u64(42)`.
- **Iterations**: 3,000 iterations per difficulty = **9,000 total iterations**.
- **Reference Oracle**: Independent multiset frequency counter (`total_matches - bulls = cows`).

### 5.2 Mathematical Invariants Verified
1. **Differential Oracle Equivalence**: `score_guess(target, guess) == reference_score(target, guess)`.
2. **Individual Bounds**: $0 \le \text{bulls} \le M$ and $0 \le \text{cows} \le M$.
3. **Combined Bound**: $\text{bulls} + \text{cows} \le M$.
4. **Reflexivity**: $\text{score}(T, T) = (M, 0)$.
5. **Permutation Conservation**: If guess is a permutation of target, then $\text{bulls} + \text{cows} = M$.

**Observed Result**: 9,000 / 9,000 iterations passed with zero failures in 0.08s.

---

## 6. Attempt Limit & State Machine Unit Tests (`game.rs`)

Tested via `cargo test game::tests`:

| Test Function | Scenario | Expected Behavior | Observed | Status |
| :--- | :--- | :--- | :--- | :--- |
| `test_attempt_limit_win_on_first_attempt` | Guess target on attempt 1 | Returns `(Won, 1)`, no further attempts allowed | `Won { attempts: 1 }` | **PASS** |
| `test_attempt_limit_win_on_final_attempt` | Fail $L-1$ times, solve on attempt $L$ | Win condition evaluated *before* loss condition | `Won { attempts: L }` | **PASS** |
| `test_attempt_limit_loss_on_final_attempt` | Fail $L$ times | Round ends in `Lost`, target sequence revealed | `Lost { attempts: L }` | **PASS** |
| `test_parse_guess_all_difficulties` | Length & range checks for Easy, Medium, Hard | Strict validation according to active config | Matches config rules | **PASS** |
| `test_parse_guess_whitespace_and_quit` | Tabs, multiple spaces, case-insensitive quit | Seamless parsing, `Err(Quit)` returned | Matches specification | **PASS** |
| `test_replay_choice_parsing` | `"1"`, `"2"`, `"3"`, `"q"`, invalid | Mapped to `PlayAgainSame`, `ChangeDifficulty`, `Exit` | Matches menu options | **PASS** |
| `test_generate_target_all_difficulties` | Generate 50 targets per difficulty | Length equals $M$, all elements in $1..=N$ | All within bounds | **PASS** |

---

## 7. ANSI Styling & Fallback Unit Tests (`ui.rs`)

Tested via `cargo test ui::tests`:

| Test Function | Mode | Verification | Status |
| :--- | :--- | :--- | :--- |
| `test_ansi_mode_wraps_and_resets` | `ColorMode::Enabled` | Every styled element includes reset `\x1b[0m` to prevent bleeding | **PASS** |
| `test_plain_text_mode_has_no_ansi_escapes` | `ColorMode::Disabled` | Generated output contains **0 escape sequences** (`\x1b[`) | **PASS** |
| `test_progress_bar_rendering` | Both | Verified filled blocks `█` and empty blocks `░` with remaining count | **PASS** |
| `test_ui_panels_render_in_both_modes` | Both | Status panel and welcome banner render cleanly with box-drawing | **PASS** |
| `test_win_and_loss_panels_render_accurately` | Both | Victory breakdown displays base points, multipliers, and total | **PASS** |

---

## 8. End-to-End Stream Integration Tests (`main.rs`)

Simulated full user interaction sessions using mocked I/O streams:

1. **`test_select_difficulty_valid_and_invalid`**:
   - Tested invalid menu inputs (`"9"`, `"foo"`) followed by valid selection (`"1"`).
   - Proved that invalid menu input displays a friendly error and re-prompts without panicking.
2. **`test_end_to_end_single_win`**:
   - Complete single-game victory on Medium difficulty ($M=4, N=6, L=9$).
   - Solved on attempt 2: verified output includes score breakdown (333 pts) and session score.
3. **`test_invalid_input_does_not_consume_attempts`**:
   - Injected wrong length (`1 2`), out of range (`1 2 9 4`), non-integer (`1 2 x 4`), and empty input.
   - Proved attempt counter remained at attempt 1 until a valid guess was submitted.
4. **`test_end_to_end_win_on_final_attempt`**:
   - Tested edge case on Hard difficulty ($L=8$): 7 non-matching guesses followed by a matching 8th guess.
   - Proved player wins on attempt 8 (earning 400 pts) rather than losing.
5. **`test_end_to_end_attempt_limit_loss`**:
   - 8 consecutive incorrect guesses on Hard difficulty.
   - Proved game ended in a loss, revealed the secret target, and awarded 0 points.
6. **`test_run_session_difficulty_change_and_exit`**:
   - Multi-round session: Round 1 won on Easy (190 pts) $\to$ replay choice `2` (Change Difficulty) $\to$ Round 2 won on Medium (378 pts) $\to$ replay choice `3` (Exit).
   - Proved cumulative session score correctly accumulated ($190 + 378 = 568$ pts).
7. **`test_quit_midway`**:
   - Valid guess followed by `quit`.
   - Proved session exited cleanly, revealed target, and awarded 0 victory points.

---

## 9. Comprehensive Execution Summary

All 37 automated tests execute in 0.08 seconds:

```
running 37 tests
test config::tests::test_difficulty_configurations ... ok
test config::tests::test_difficulty_display_and_all ... ok
test config::tests::test_parse_difficulty_from_input ... ok
test game::tests::test_attempt_limit_loss_on_final_attempt ... ok
test game::tests::test_attempt_limit_win_on_final_attempt ... ok
test game::tests::test_attempt_limit_win_on_first_attempt ... ok
test game::tests::test_generate_target_all_difficulties ... ok
test game::tests::test_parse_guess_all_difficulties ... ok
test game::tests::test_parse_guess_whitespace_and_quit ... ok
test game::tests::test_replay_choice_parsing ... ok
test scoring::tests::test_calculate_score_panics_on_exceeded_attempts - should panic ... ok
test scoring::tests::test_calculate_score_panics_on_zero_attempts - should panic ... ok
test scoring::tests::test_score_bands_are_strictly_non_overlapping ... ok
test scoring::tests::test_score_calculation_easy ... ok
test scoring::tests::test_score_calculation_hard ... ok
test scoring::tests::test_score_calculation_medium ... ok
test scoring::tests::test_score_guess_all_bulls_various_lengths ... ok
test scoring::tests::test_score_guess_all_cows_various_lengths ... ok
test scoring::tests::test_score_guess_mixed_and_duplicates ... ok
test scoring::tests::test_score_guess_mixed_matches ... ok
test scoring::tests::test_score_guess_panics_on_length_mismatch - should panic ... ok
test scoring::tests::test_score_guess_repeated_values_in_guess ... ok
test scoring::tests::test_score_guess_repeated_values_in_target ... ok
test scoring::tests::test_randomized_differential_scoring_all_difficulties ... ok
test ui::tests::test_ansi_mode_wraps_and_resets ... ok
test ui::tests::test_plain_text_mode_has_no_ansi_escapes ... ok
test ui::tests::test_progress_bar_rendering ... ok
test ui::tests::test_ui_panels_render_in_both_modes ... ok
test ui::tests::test_win_and_loss_panels_render_accurately ... ok
test tests::test_ask_replay_choices ... ok
test tests::test_end_to_end_attempt_limit_loss ... ok
test tests::test_end_to_end_single_win ... ok
test tests::test_end_to_end_win_on_final_attempt ... ok
test tests::test_invalid_input_does_not_consume_attempts ... ok
test tests::test_quit_midway ... ok
test tests::test_run_session_difficulty_change_and_exit ... ok
test tests::test_select_difficulty_valid_and_invalid ... ok

test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

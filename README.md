# Master Mind / Bulls and Cows in Rust

A robust, terminal-based implementation of the classic **Master Mind (Bulls and Cows)** code-breaking game written in idiomatic Rust for CS2102 Programming Lab.

Features runtime configurable difficulty levels, limited attempts, a difficulty- and attempt-efficiency-aware scoring system, and an elegant ANSI terminal interface with plain-text fallback.

---

## 1. Project Overview & Rules

The computer secretly generates a target sequence of $M$ integers, each in the inclusive range $[1, N]$. The player deduces this secret code by submitting guesses and receiving Bulls and Cows feedback after each attempt.

- **Bulls**: Numbers in your guess that match the secret code at the **exact same position** (identical index and value).
- **Cows**: Numbers in your guess that appear in the secret code at a **different position**, counted *after* all Bulls have been resolved.
- **Repeated Numbers**: Repetitions are explicitly **permitted** in both the secret code and player guesses.
- **Duplicate Safety**: Every peg in the target and in your guess can be matched at most once. A peg that is scored as a Bull is never counted as a Cow, and multiple instances of a number in a guess cannot match a single instance in the target.

---

## 2. Difficulty Levels & Runtime Configuration

At the start of the game (or before any replay round), players select from three difficulty settings:

| Setting | Sequence Length ($M$) | Peg Range ($N$) | Max Valid Attempts ($L$) | Difficulty Multiplier | Score Range ($A=L \to A=1$) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Easy** | 3 | $1..=4$ | 10 | 1.0× | 100 – 190 pts |
| **Medium** | 4 | $1..=6$ | 9 | 2.0× | 200 – 378 pts |
| **Hard** | 5 | $1..=8$ | 8 | 4.0× | 400 – 750 pts |

### Progression Rationale
- **Easy**: Shortest sequence (3 pegs), smallest number domain (1–4), and the most attempts (10).
- **Medium**: Preserves the original project's classic $M = 4$ and $N = 6$ settings, with 9 allowed attempts.
- **Hard**: Longest sequence (5 pegs), largest number domain (1–8), and the fewest attempts (8).

All difficulty settings are centrally maintained in `src/config.rs` as the single authoritative source of truth.

---

## 3. Attempt Limits & Game Rules

1. **Attempt Accounting**: Only valid, evaluated guesses consume an attempt.
2. **Invalid Input Is Free**: Non-integer tokens, wrong token counts, and out-of-range values produce informative error messages and **do not** consume an attempt.
3. **Win-Before-Loss Rule**: If your final allowed attempt ($A = L$) correctly solves the code ($M$ Bulls), it is counted as a **victory**.
4. **Loss Condition**: If your final allowed attempt fails to solve the code, the game ends in a loss and reveals the secret code.
5. **Graceful Quit**: Typing `quit` (case-insensitive) at any prompt terminates the current round gracefully, reveals the secret code, and consumes no attempts.
6. **Replay Flow**: At the end of each round, you can:
   - `1`: Play again with the current difficulty.
   - `2`: Change difficulty setting.
   - `3` (or `q`): Exit the game session.

---

## 4. Scoring & Multiplier System

The scoring system rewards both the chosen difficulty and attempt efficiency.

### Base Score
$$\text{BASE\_POINTS} = 100$$

### Difficulty Multiplier
- **Easy**: $1.0\times$
- **Medium**: $2.0\times$
- **Hard**: $4.0\times$

### Attempt-Efficiency Multiplier
Let $A$ be the 1-based number of valid attempts used to win ($1 \le A \le L$), and $L$ be the maximum allowed attempts:
$$\text{attempt\_multiplier} = 1.0 + \frac{L - A}{L}$$

- Winning on attempt 1 awards the maximum efficiency bonus: $1.0 + \frac{L-1}{L} < 2.0$.
- Winning on the final allowed attempt ($A = L$) yields an efficiency multiplier of exactly $1.0$.

### Final Round Score
$$\text{final\_score} = \operatorname{round}(\text{BASE\_POINTS} \times \text{difficulty\_multiplier} \times \text{attempt\_multiplier})$$

### Worked Examples
- **Hard win on Attempt 1 of 8**:
  $$\text{attempt\_multiplier} = 1.0 + \frac{8 - 1}{8} = 1.875$$
  $$\text{final\_score} = \operatorname{round}(100 \times 4.0 \times 1.875) = \mathbf{750\text{ points}}$$
- **Hard win on Attempt 8 of 8**:
  $$\text{attempt\_multiplier} = 1.0 + \frac{8 - 8}{8} = 1.0$$
  $$\text{final\_score} = \operatorname{round}(100 \times 4.0 \times 1.0) = \mathbf{400\text{ points}}$$
- **Easy win on Attempt 1 of 10**:
  $$\text{attempt\_multiplier} = 1.0 + \frac{10 - 1}{10} = 1.9$$
  $$\text{final\_score} = \operatorname{round}(100 \times 1.0 \times 1.9) = \mathbf{190\text{ points}}$$

### Stratified Score Hierarchy
Because the difficulty multipliers are scaled (1.0×, 2.0×, 4.0×), the score intervals never overlap:
$$\text{Easy: } [100, 190] < \text{Medium: } [200, 378] < \text{Hard: } [400, 750]$$
The minimum score achievable on a harder setting exceeds the maximum score on a lower setting.

### Cumulative Session Score
The program tracks cumulative points across successful rounds within a running session. A loss or voluntary quit awards 0 points and preserves the current session total. The total resets cleanly when the program terminates.

---

## 5. Terminal Presentation & ANSI Support

The user interface features a clean, restrained visual theme:
- Prominent welcome panel with Unicode box-drawing characters (`╭───╮`, `│`, `╰───╯`).
- Status panels displaying difficulty, code length, value range, attempts used, and session score.
- Dynamic attempt progress bar (`[████████░░] 8 remaining`).
- Color-coded feedback for Bulls (green), Cows (yellow), errors (red), victories (bold green), and losses (red).
- Formatted victory score breakdown showing each multiplier step.

### Plain-Text Fallback & `NO_COLOR`
ANSI escape codes are strictly managed via `ui::ColorMode`:
- If the `NO_COLOR` environment variable is present, color sequences are automatically disabled.
- In automated test pipelines or redirected streams where output is non-interactive, styling cleanly falls back to pure ASCII/Unicode text without escape sequence leakage.

---

## 6. Prerequisites & Setup

### Prerequisites
- [Rust](https://www.rust-lang.org/) toolchain (version 1.80+ or latest stable; verified on `rustc 1.98.1` / Edition 2024).
- Cargo (Rust package manager).

### Installation & Check
Navigate to the project root:
```bash
cd mastermind
cargo check
```

---

## 7. How to Run the Game

### Debug Run
```bash
cargo run
```

### Optimized Release Build
```bash
cargo run --release
```

To build the standalone release binary:
```bash
cargo build --release
```
The compiled binary will be placed at:
- Windows: `target/release/mastermind.exe`
- Linux/macOS: `target/release/mastermind`

---

## 8. How to Run Tests

The test suite contains 37 automated tests across unit, property-based differential, and integration testing tiers.

Run all tests:
```bash
cargo test
```

Run tests with verbose output:
```bash
cargo test -- --nocapture
```

Verify formatting compliance:
```bash
cargo fmt --check
```

---

## 9. Project & Source Code Structure

The project is structured into focused, single-responsibility modules:

```
mastermind/
├── Cargo.toml            # Project metadata and dependencies (rand 0.8)
├── Cargo.lock            # Exact dependency lockfile
├── src/
│   ├── config.rs         # Difficulty enum, DifficultyConfig, base points, parsing
│   ├── scoring.rs        # Slice-based Bulls/Cows matching, score formula, ScoreBreakdown
│   ├── game.rs           # Guess parsing, target generation, GameSession state machine
│   ├── ui.rs             # ColorMode, ANSI styling, box drawing, progress bars, panels
│   └── main.rs           # Entry point, interactive loop, replay coordination, E2E tests
├── docs/
│   ├── DESIGN.md         # Architecture rationale, slice borrowing, and design evolution
│   ├── DEV_LOG.md        # Chronological development milestones and Git commit records
│   └── TESTING.md        # Detailed test matrices, differential proofs, and benchmarks
└── README.md             # This comprehensive user guide
```

---

## 10. Additional Documentation

For complete design rationale and historical development logs, consult:
- [DESIGN.md](docs/DESIGN.md): Detailed architectural rationale, vector transition, borrowing continuity, and scoring theory.
- [TESTING.md](docs/TESTING.md): Verification matrices, seeded differential testing results (3,000 cases per tier), and invariant proofs.
- [DEV_LOG.md](docs/DEV_LOG.md): Full chronological milestone history and Git commit record.

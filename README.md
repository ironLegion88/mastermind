# Master Mind / Bulls and Cows in Rust

A robust, terminal-based implementation of the classic **Master Mind (Bulls and Cows)** code-breaking game written in idiomatic Rust for CS2102 Programming Lab.

---

## 1. Project Overview & Rules

The computer generates a secret target sequence of $M$ integers, each in the inclusive range $[1, N]$. The player's objective is to deduce this secret code by submitting guesses and using the Bulls and Cows feedback provided after each attempt.

- **Bulls**: Numbers in your guess that match the secret code at the **exact same position** (same index and same value).
- **Cows**: Numbers in your guess that appear in the secret code at a **different position**, counted *after* all Bulls have already been accounted for.
- **Repeated Numbers**: Repetitions are explicitly **permitted** in both the secret code and player guesses.
- **Duplicate Safety**: Every peg in the target and in your guess can be matched at most once. A peg that is scored as a Bull is never counted as a Cow, and multiple instances of a number in a guess cannot match a single instance in the target.

### Current Game Configuration
- **Sequence Length ($M$)**: `4` pegs
- **Peg Range ($N$)**: `1..=6` (integers from 1 through 6)
- **Repetitions Allowed**: Yes (both in target and in guesses)

---

## 2. Prerequisites & Setup

### Prerequisites
- [Rust](https://www.rust-lang.org/) toolchain (version 1.80+ or latest stable; tested on `rustc 1.98.1` / Edition 2024).
- Cargo (Rust package manager).

### Installation
Clone or navigate to the project directory:
```bash
cd mastermind
```

Verify the build environment:
```bash
cargo check
```

---

## 3. How to Run the Game

### Normal Debug Run
```bash
cargo run
```

### Optimized Release Build
To compile and run an optimized release binary:
```bash
cargo run --release
```

To build standalone release executables without running:
```bash
cargo build --release
```
The optimized executable will be located at:
- Windows: `target/release/mastermind.exe`
- Linux/macOS: `target/release/mastermind`

---

## 4. Gameplay Instructions

1. Upon starting the game, you will see a welcome screen displaying the current game configuration ($M=4, N=6$) and rules.
2. The computer secretly generates $M$ numbers.
3. At the `Guess #...:` prompt, enter exactly $4$ integers separated by whitespace.
   - Valid examples:
     - `1 2 3 4`
     - `2 2 5 5` (repeated numbers are valid)
     - `1   4   2   6` (multiple spaces and tabs are accepted)
4. Feedback is displayed as:
   ```text
   Result: 1 Bull, 2 Cows
   ```
5. **Quitting**: You may type `quit` (case-insensitive, e.g., `quit`, `QUIT`, `Quit`) at any guess prompt to gracefully exit the game. When you quit, the secret code is revealed.
6. **Error Recovery**: If you enter an invalid token (e.g. letters, floats), the wrong number of values, or values outside $1..=6$, a descriptive error message will appear and you will be prompted again without increasing your guess counter.
7. **Victory & Replay**: When you score $4$ Bulls, your victory is celebrated along with your total attempt count. You will be asked if you want to play another game (`y/n`).

---

## 5. How to Run Tests

The test suite includes deterministic unit tests, randomized differential testing, and stream integration tests.

Run all tests:
```bash
cargo test
```

Run tests with test output displayed:
```bash
cargo test -- --nocapture
```

Verify code formatting:
```bash
cargo fmt --check
```

---

## 6. Project & Source Code Structure

```
mastermind/
├── Cargo.toml            # Project metadata and dependencies (rand 0.8)
├── Cargo.lock            # Exact dependency lockfile
├── src/
│   └── main.rs           # Core game implementation, algorithms, and tests
├── docs/
│   ├── DESIGN.md         # Comprehensive design document & architectural trade-offs
│   ├── DEV_LOG.md        # Chronological development log of all phases
│   └── TESTING.md        # Detailed testing methodology and verification results
└── README.md             # This user guide
```

### Main Components in `src/main.rs`
- **`M` / `N`**: Central compile-time constants for sequence length and peg range.
- **`score_guess(target: &[u32], guess: &[u32]) -> (usize, usize)`**: Duplicate-safe two-pass scoring algorithm utilizing stack-allocated boolean markers.
- **`parse_guess(input: &str) -> Result<[u32; M], GuessError>`**: Whitespace-tolerant input parser and validator returning structured error variants.
- **`generate_target() -> [u32; M]`**: Uniform random sequence generator using `rand`.
- **`play_game<R, W>(target, reader, writer) -> io::Result<GameOutcome>`**: Single game session loop decoupled from concrete stdin/stdout for testability.
- **`run<R, W>(reader, writer) -> io::Result<()>`**: Top-level coordinator managing welcome screen and replay options.
- **`#[cfg(test)] mod tests`**: Comprehensive unit, differential, and integration test suite (29 tests).

---

## 7. Additional Documentation

For deeper details regarding the implementation decisions, algorithm proofs, and development history, see:
- [DESIGN.md](docs/DESIGN.md): Detailed architectural rationale, fixed arrays vs. vectors, functions vs. macros, and correctness proofs.
- [TESTING.md](docs/TESTING.md): Test verification matrices, differential testing parameters, and invariant proofs.
- [DEV_LOG.md](docs/DEV_LOG.md): Full chronological milestone history and Git commit record.

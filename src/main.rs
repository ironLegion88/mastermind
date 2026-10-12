//! CS2102 Programming Lab: Mid-Term Project
//! Terminal-based Master Mind / Bulls and Cows game in Rust.

/// The number of pegs (sequence length) in the secret target and each guess.
pub const M: usize = 4;

/// The maximum integer value for each peg (valid range is 1..=N).
pub const N: u32 = 6;

/// Calculates the number of Bulls and Cows between a secret target and a player guess.
///
/// - A **Bull** is an exact match at the same position (`target[i] == guess[i]`).
/// - A **Cow** is a match at different positions (`target[t] == guess[g]` where `t != g`),
///   counted only after all Bulls have been accounted for.
/// - **Duplicate Safety**: Neither target pegs nor guess pegs may be counted more than once.
///   Each peg index can participate in at most one match (either Bull or Cow).
///
/// # Arguments
/// * `target` - Slice representing the secret code sequence.
/// * `guess` - Slice representing the player's guess sequence.
///
/// # Returns
/// A tuple `(bulls, cows)`.
pub fn score_guess(target: &[u32], guess: &[u32]) -> (usize, usize) {
    assert_eq!(
        target.len(),
        M,
        "target length ({}) must equal M ({})",
        target.len(),
        M
    );
    assert_eq!(
        guess.len(),
        M,
        "guess length ({}) must equal M ({})",
        guess.len(),
        M
    );

    let mut bulls = 0;
    let mut cows = 0;

    // Temporary boolean marker arrays to track matched positions.
    // Stack-allocated: zero heap allocation overhead, fixed compile-time size M.
    let mut target_used = [false; M];
    let mut guess_used = [false; M];

    // Pass 1: Identify Bulls (exact position and value matches).
    // Marking both target and guess positions ensures they cannot be re-used as Cows.
    for i in 0..M {
        if target[i] == guess[i] {
            bulls += 1;
            target_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Pass 2: Identify Cows (matching values at different positions).
    // For each unmatched guess peg, search for an unmatched target peg with the same value.
    for g in 0..M {
        if !guess_used[g] {
            for t in 0..M {
                if !target_used[t] && guess[g] == target[t] {
                    cows += 1;
                    target_used[t] = true;
                    guess_used[g] = true;
                    // Stop searching once this guess peg is paired with an unmatched target peg
                    break;
                }
            }
        }
    }

    (bulls, cows)
}

/// Errors that can occur when parsing and validating a user's guess input.
#[derive(Debug, PartialEq, Eq)]
pub enum GuessError {
    /// The user requested to quit the game (e.g., entered "quit").
    Quit,
    /// The user submitted an empty line (or only whitespace).
    EmptyInput,
    /// The number of tokens was not equal to M.
    InvalidLength { expected: usize, actual: usize },
    /// A token could not be parsed as a positive integer.
    NonInteger(String),
    /// A parsed integer is outside the valid range 1..=N.
    OutOfRange { value: u32, min: u32, max: u32 },
}

impl std::fmt::Display for GuessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuessError::Quit => write!(f, "Game quit requested."),
            GuessError::EmptyInput => write!(
                f,
                "Input was empty. Please enter {M} numbers separated by spaces."
            ),
            GuessError::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "Invalid number of values: expected {expected}, but received {actual}."
                )
            }
            GuessError::NonInteger(token) => {
                write!(f, "Invalid token '{token}': must be a positive integer.")
            }
            GuessError::OutOfRange { value, min, max } => {
                write!(
                    f,
                    "Value {value} is out of range: each number must be between {min} and {max} (inclusive)."
                )
            }
        }
    }
}

impl std::error::Error for GuessError {}

/// Parses and validates a raw input string into a fixed-size guess array `[u32; M]`.
///
/// # Validation Steps:
/// 1. Recognizes the `quit` command (case-insensitive) before numeric parsing.
/// 2. Splits input across any whitespace (spaces, multiple spaces, tabs).
/// 3. Parses all tokens into a temporary `Vec<u32>`.
/// 4. Verifies that exactly `M` numbers were provided.
/// 5. Verifies that every number lies within the inclusive range `1..=N`.
/// 6. Safely converts the validated numbers into the fixed-size array `[u32; M]`.
///
/// Repeated numbers are explicitly allowed and preserved.
pub fn parse_guess(input: &str) -> Result<[u32; M], GuessError> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return Err(GuessError::EmptyInput);
    }

    // 1. Recognize quit command before numeric parsing
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("quit") {
        return Err(GuessError::Quit);
    }

    // 2. Parse tokens into integers, collecting into temporary Vec<u32>
    let mut values = Vec::with_capacity(tokens.len());
    for token in &tokens {
        match token.parse::<u32>() {
            Ok(val) => values.push(val),
            Err(_) => return Err(GuessError::NonInteger((*token).to_string())),
        }
    }

    // 3. Verify exactly M values were provided
    if values.len() != M {
        return Err(GuessError::InvalidLength {
            expected: M,
            actual: values.len(),
        });
    }

    // 4. Verify every value lies in 1..=N
    for &val in &values {
        if !(1..=N).contains(&val) {
            return Err(GuessError::OutOfRange {
                value: val,
                min: 1,
                max: N,
            });
        }
    }

    // 5. Explicit, safe conversion into [u32; M]
    let mut validated = [0u32; M];
    validated.copy_from_slice(&values);
    Ok(validated)
}

fn main() {
    println!("Master Mind / Bulls and Cows (M = {M}, N = {N})");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    // -------------------------------------------------------------------------
    // Deterministic Unit Tests for score_guess
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_bulls_unique() {
        let target = [1, 2, 3, 4];
        let guess = [1, 2, 3, 4];
        assert_eq!(score_guess(&target, &guess), (4, 0));
    }

    #[test]
    fn test_all_bulls_with_duplicates() {
        let target = [2, 2, 5, 5];
        let guess = [2, 2, 5, 5];
        assert_eq!(score_guess(&target, &guess), (4, 0));

        let all_same = [3, 3, 3, 3];
        assert_eq!(score_guess(&all_same, &all_same), (4, 0));
    }

    #[test]
    fn test_all_cows_complete_misplacement() {
        let target = [1, 2, 3, 4];
        let guess = [4, 3, 2, 1];
        assert_eq!(score_guess(&target, &guess), (0, 4));

        let guess_shifted = [2, 3, 4, 1];
        assert_eq!(score_guess(&target, &guess_shifted), (0, 4));
    }

    #[test]
    fn test_all_cows_with_duplicates() {
        // Misplaced pairs: target has two 1s and two 2s; guess has two 2s and two 1s
        let target = [1, 1, 2, 2];
        let guess = [2, 2, 1, 1];
        assert_eq!(score_guess(&target, &guess), (0, 4));

        let target2 = [1, 2, 1, 2];
        let guess2 = [2, 1, 2, 1];
        assert_eq!(score_guess(&target2, &guess2), (0, 4));
    }

    #[test]
    fn test_mixed_bulls_and_cows() {
        let target = [1, 2, 3, 4];
        // Pos 0 is Bull (1 == 1); Pos 1 and 2 are swapped Cows (3 vs 2); 5 is absent
        let guess = [1, 3, 2, 5];
        assert_eq!(score_guess(&target, &guess), (1, 2));

        // Two Bulls, two Cows
        let guess2 = [1, 2, 4, 3];
        assert_eq!(score_guess(&target, &guess2), (2, 2));
    }

    #[test]
    fn test_no_matches() {
        let target = [1, 2, 3, 4];
        let guess = [5, 6, 5, 6];
        assert_eq!(score_guess(&target, &guess), (0, 0));
    }

    #[test]
    fn test_repeated_values_in_target_single_in_guess() {
        // Target has two 1s; guess only has one 1
        let target = [1, 1, 2, 3];
        let guess_exact = [1, 4, 5, 6]; // Bull at pos 0; no extra Cow
        assert_eq!(score_guess(&target, &guess_exact), (1, 0));

        // Peg 1 placed at pos 2 (target has 2 at pos 2, so no Bull) -> exactly 1 Cow
        let guess_misplaced = [4, 5, 1, 6];
        assert_eq!(score_guess(&target, &guess_misplaced), (0, 1));
    }

    #[test]
    fn test_repeated_values_in_guess_single_in_target() {
        // Target has one 1; guess has four 1s
        let target = [1, 2, 3, 4];
        let guess = [1, 1, 1, 1];
        // Pos 0 is a Bull. The remaining three 1s in guess must NOT be counted as Cows!
        assert_eq!(score_guess(&target, &guess), (1, 0));

        // Target has one 1; guess has three 1s, none at index 0
        let guess_no_bull = [5, 1, 1, 1];
        // Only one of the 1s can match the single 1 in target
        assert_eq!(score_guess(&target, &guess_no_bull), (0, 1));
    }

    #[test]
    fn test_duplicate_overcounting_edge_cases() {
        // Classic trap case: target [1, 2, 2, 3], guess [2, 2, 4, 2]
        // Exact match at pos 1 (Bull for 2).
        // Guess has two other 2s (pos 0 and 3). Target has only one unmatched 2 (pos 2).
        // Only ONE Cow should be awarded, not two!
        let target = [1, 2, 2, 3];
        let guess = [2, 2, 4, 2];
        assert_eq!(score_guess(&target, &guess), (1, 1));

        // Target has two 1s and two 2s; guess has 3, 3, 1, 1 (two misplaced 1s, no Bulls)
        let target2 = [1, 1, 2, 2];
        let guess2 = [3, 4, 1, 1];
        assert_eq!(score_guess(&target2, &guess2), (0, 2));

        // Target has two 1s; guess has three 1s (one Bull, one Cow, one ignored)
        let target3 = [1, 1, 3, 4];
        let guess3 = [1, 5, 1, 1];
        assert_eq!(score_guess(&target3, &guess3), (1, 1));
    }

    #[test]
    fn test_outside_matching_set() {
        let target = [1, 2, 3, 4];
        let guess = [6, 6, 6, 6];
        assert_eq!(score_guess(&target, &guess), (0, 0));
    }

    // -------------------------------------------------------------------------
    // Reference Implementation for Randomized Differential Testing
    // -------------------------------------------------------------------------

    /// Independent reference scorer based on multiset frequency counting.
    ///
    /// Algorithm:
    /// 1. Exact matches at identical indices are counted as Bulls.
    /// 2. For each distinct peg value v in 1..=N, the total possible matches
    ///    between target and guess is min(count_target(v), count_guess(v)).
    /// 3. The total matches across all values equals (bulls + cows).
    /// 4. Therefore, cows = total_matches - bulls.
    fn reference_score_guess(target: &[u32], guess: &[u32]) -> (usize, usize) {
        let mut bulls = 0;
        let mut target_freq = [0usize; (N + 1) as usize];
        let mut guess_freq = [0usize; (N + 1) as usize];

        for i in 0..M {
            if target[i] == guess[i] {
                bulls += 1;
            }
            if target[i] <= N {
                target_freq[target[i] as usize] += 1;
            }
            if guess[i] <= N {
                guess_freq[guess[i] as usize] += 1;
            }
        }

        let mut total_matches = 0;
        for v in 1..=(N as usize) {
            total_matches += target_freq[v].min(guess_freq[v]);
        }

        let cows = total_matches - bulls;
        (bulls, cows)
    }

    #[test]
    fn test_reference_scorer_matches_known_cases() {
        let cases = [
            ([1, 2, 3, 4], [1, 2, 3, 4], (4, 0)),
            ([1, 2, 3, 4], [4, 3, 2, 1], (0, 4)),
            ([1, 2, 3, 4], [1, 3, 2, 5], (1, 2)),
            ([1, 2, 2, 3], [2, 2, 4, 2], (1, 1)),
            ([1, 2, 3, 4], [1, 1, 1, 1], (1, 0)),
            ([1, 1, 2, 2], [2, 2, 1, 1], (0, 4)),
            ([5, 6, 5, 6], [1, 2, 3, 4], (0, 0)),
        ];

        for (target, guess, expected) in cases {
            assert_eq!(reference_score_guess(&target, &guess), expected);
            assert_eq!(score_guess(&target, &guess), expected);
        }
    }

    // -------------------------------------------------------------------------
    // Randomized Differential Testing with Seeded PRNG
    // -------------------------------------------------------------------------

    #[test]
    fn test_randomized_differential_scoring() {
        // Seeded RNG ensures 100% deterministic reproducibility across all test runs
        const SEED: u64 = 42;
        const ITERATIONS: usize = 10_000;

        let mut rng = StdRng::seed_from_u64(SEED);

        for iteration in 0..ITERATIONS {
            let mut target = [0u32; M];
            let mut guess = [0u32; M];

            for i in 0..M {
                target[i] = rng.gen_range(1..=N);
                guess[i] = rng.gen_range(1..=N);
            }

            let actual = score_guess(&target, &guess);
            let expected = reference_score_guess(&target, &guess);

            // Invariant 1: Two-pass marker algorithm matches frequency-counting reference
            assert_eq!(
                actual, expected,
                "Mismatch on iteration {iteration}: target={target:?}, guess={guess:?}"
            );

            let (bulls, cows) = actual;

            // Invariant 2: Individual counts cannot exceed sequence length M
            assert!(
                bulls <= M,
                "Bulls ({bulls}) exceeded M ({M}) on target={target:?}, guess={guess:?}"
            );
            assert!(
                cows <= M,
                "Cows ({cows}) exceeded M ({M}) on target={target:?}, guess={guess:?}"
            );

            // Invariant 3: Total matches cannot exceed sequence length M
            assert!(
                bulls + cows <= M,
                "Total matches ({}) exceeded M ({M}) on target={target:?}, guess={guess:?}",
                bulls + cows
            );

            // Invariant 4: Self-comparison yields exactly (M, 0)
            assert_eq!(
                score_guess(&target, &target),
                (M, 0),
                "Self-comparison invariant failed for target={target:?}"
            );
            assert_eq!(
                score_guess(&guess, &guess),
                (M, 0),
                "Self-comparison invariant failed for guess={guess:?}"
            );

            // Invariant 5: Multiset permutation check: if guess is a permutation of target,
            // then total matches (bulls + cows) must equal exactly M.
            let mut permuted_target = target;
            // Simple deterministic permutation: reverse target
            permuted_target.reverse();
            let (p_bulls, p_cows) = score_guess(&target, &permuted_target);
            assert_eq!(
                p_bulls + p_cows,
                M,
                "Permutation invariant failed: target={target:?}, permuted={permuted_target:?}"
            );
        }
    }

    // -------------------------------------------------------------------------
    // Unit Tests for parse_guess
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_guess_valid() {
        assert_eq!(parse_guess("1 2 3 4"), Ok([1, 2, 3, 4]));
        assert_eq!(parse_guess("2 2 5 5"), Ok([2, 2, 5, 5]));
        assert_eq!(parse_guess("6 1 6 1"), Ok([6, 1, 6, 1]));
    }

    #[test]
    fn test_parse_guess_whitespace_variations() {
        // Multiple spaces
        assert_eq!(parse_guess("1   2    3     4"), Ok([1, 2, 3, 4]));
        // Leading and trailing spaces
        assert_eq!(parse_guess("   1 2 3 4   "), Ok([1, 2, 3, 4]));
        // Tabs and mixed whitespace
        assert_eq!(parse_guess("\t1 \t 2  3\t 4\n"), Ok([1, 2, 3, 4]));
    }

    #[test]
    fn test_parse_guess_quit_command() {
        assert_eq!(parse_guess("quit"), Err(GuessError::Quit));
        assert_eq!(parse_guess("QUIT"), Err(GuessError::Quit));
        assert_eq!(parse_guess("Quit"), Err(GuessError::Quit));
        assert_eq!(parse_guess("  quit  \t"), Err(GuessError::Quit));
    }

    #[test]
    fn test_parse_guess_empty_input() {
        assert_eq!(parse_guess(""), Err(GuessError::EmptyInput));
        assert_eq!(parse_guess("   "), Err(GuessError::EmptyInput));
        assert_eq!(parse_guess("\t  \n"), Err(GuessError::EmptyInput));
    }

    #[test]
    fn test_parse_guess_invalid_length() {
        // Too few tokens
        assert_eq!(
            parse_guess("1 2 3"),
            Err(GuessError::InvalidLength {
                expected: 4,
                actual: 3
            })
        );
        assert_eq!(
            parse_guess("1"),
            Err(GuessError::InvalidLength {
                expected: 4,
                actual: 1
            })
        );

        // Too many tokens
        assert_eq!(
            parse_guess("1 2 3 4 5"),
            Err(GuessError::InvalidLength {
                expected: 4,
                actual: 5
            })
        );
        assert_eq!(
            parse_guess("1 2 3 4 5 6"),
            Err(GuessError::InvalidLength {
                expected: 4,
                actual: 6
            })
        );
    }

    #[test]
    fn test_parse_guess_non_integer_tokens() {
        assert_eq!(
            parse_guess("1 2 abc 4"),
            Err(GuessError::NonInteger("abc".to_string()))
        );
        assert_eq!(
            parse_guess("1 2.5 3 4"),
            Err(GuessError::NonInteger("2.5".to_string()))
        );
        assert_eq!(
            parse_guess("1 -2 3 4"),
            Err(GuessError::NonInteger("-2".to_string()))
        );
    }

    #[test]
    fn test_parse_guess_out_of_range() {
        // Below minimum (0 < 1)
        assert_eq!(
            parse_guess("0 2 3 4"),
            Err(GuessError::OutOfRange {
                value: 0,
                min: 1,
                max: 6
            })
        );

        // Above maximum (7 > 6)
        assert_eq!(
            parse_guess("1 2 7 4"),
            Err(GuessError::OutOfRange {
                value: 7,
                min: 1,
                max: 6
            })
        );

        // High value
        assert_eq!(
            parse_guess("1 2 3 99"),
            Err(GuessError::OutOfRange {
                value: 99,
                min: 1,
                max: 6
            })
        );
    }

    #[test]
    fn test_guess_error_display() {
        // Verify user-facing error message formatting
        assert_eq!(GuessError::Quit.to_string(), "Game quit requested.");
        assert_eq!(
            GuessError::EmptyInput.to_string(),
            "Input was empty. Please enter 4 numbers separated by spaces."
        );
        assert_eq!(
            GuessError::InvalidLength {
                expected: 4,
                actual: 2
            }
            .to_string(),
            "Invalid number of values: expected 4, but received 2."
        );
        assert_eq!(
            GuessError::NonInteger("xyz".to_string()).to_string(),
            "Invalid token 'xyz': must be a positive integer."
        );
        assert_eq!(
            GuessError::OutOfRange {
                value: 9,
                min: 1,
                max: 6
            }
            .to_string(),
            "Value 9 is out of range: each number must be between 1 and 6 (inclusive)."
        );
    }
}

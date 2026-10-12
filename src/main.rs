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
}

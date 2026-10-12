//! Scoring algorithms and calculation of points for Mastermind.

use crate::config::{BASE_POINTS, Difficulty, DifficultyConfig};

/// Calculates the number of Bulls and Cows between a secret target and a player guess.
///
/// - A **Bull** is an exact match at the same position (`target[i] == guess[i]`).
/// - A **Cow** is a match at different positions (`target[t] == guess[g]` where `t != g`),
///   counted only after all Bulls have been accounted for.
/// - **Duplicate Safety**: Neither target pegs nor guess pegs may be counted more than once.
///   Each peg index participates in at most one match (either Bull or Cow).
///
/// Accepts arbitrary-length slices to support dynamic sequence lengths at runtime.
///
/// # Panics
/// Panics if `target.len() != guess.len()`.
pub fn score_guess(target: &[u32], guess: &[u32]) -> (usize, usize) {
    assert_eq!(
        target.len(),
        guess.len(),
        "Target length ({}) and guess length ({}) must match",
        target.len(),
        guess.len()
    );

    let len = target.len();
    let mut bulls = 0;
    let mut cows = 0;

    // Dynamically sized matching markers sized to the runtime sequence length.
    let mut target_used = vec![false; len];
    let mut guess_used = vec![false; len];

    // Pass 1: Identify Bulls (exact position and value matches).
    for i in 0..len {
        if target[i] == guess[i] {
            bulls += 1;
            target_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Pass 2: Identify Cows (matching values at different positions).
    for g in 0..len {
        if !guess_used[g] {
            for t in 0..len {
                if !target_used[t] && guess[g] == target[t] {
                    cows += 1;
                    target_used[t] = true;
                    guess_used[g] = true;
                    break;
                }
            }
        }
    }

    (bulls, cows)
}

/// Detailed breakdown of points and multipliers awarded for a winning round.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreBreakdown {
    /// The difficulty played.
    pub difficulty: Difficulty,
    /// Number of valid attempts used (1-based, 1..=max_attempts).
    pub attempts_used: usize,
    /// Remaining attempts that were not needed (max_attempts - attempts_used).
    pub attempts_remaining: usize,
    /// Base points (100).
    pub base_points: u32,
    /// Difficulty multiplier (e.g., 1.0, 2.0, 4.0).
    pub difficulty_multiplier: f64,
    /// Attempt-efficiency multiplier: `1.0 + (L - A) / L`.
    pub attempt_multiplier: f64,
    /// Combined effective multiplier: `difficulty_multiplier * attempt_multiplier`.
    pub effective_multiplier: f64,
    /// Final rounded score awarded: `round(BASE_POINTS * effective_multiplier)`.
    pub final_score: u32,
}

/// Calculates the points and multiplier breakdown for a victorious game.
///
/// # Mathematical Formula
/// - `A` = valid attempts used (`1..=L`)
/// - `L` = maximum allowed attempts
/// - `attempt_multiplier = 1.0 + (L - A) as f64 / L as f64`
/// - `effective_multiplier = difficulty_multiplier * attempt_multiplier`
/// - `final_score = round(BASE_POINTS * effective_multiplier)`
///
/// # Panics
/// Panics if `attempts_used == 0` or `attempts_used > config.max_attempts`.
pub fn calculate_score(config: &DifficultyConfig, attempts_used: usize) -> ScoreBreakdown {
    assert!(
        attempts_used >= 1 && attempts_used <= config.max_attempts,
        "attempts_used ({}) must be between 1 and max_attempts ({})",
        attempts_used,
        config.max_attempts
    );

    let attempts_remaining = config.max_attempts - attempts_used;
    let attempt_multiplier = 1.0 + (attempts_remaining as f64) / (config.max_attempts as f64);
    let effective_multiplier = config.difficulty_multiplier * attempt_multiplier;
    let final_score = ((BASE_POINTS as f64) * effective_multiplier).round() as u32;

    ScoreBreakdown {
        difficulty: config.difficulty,
        attempts_used,
        attempts_remaining,
        base_points: BASE_POINTS,
        difficulty_multiplier: config.difficulty_multiplier,
        attempt_multiplier,
        effective_multiplier,
        final_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    // -------------------------------------------------------------------------
    // Deterministic Unit Tests for score_guess across various lengths
    // -------------------------------------------------------------------------

    #[test]
    fn test_score_guess_all_bulls_various_lengths() {
        // Length 3 (Easy)
        assert_eq!(score_guess(&[1, 2, 3], &[1, 2, 3]), (3, 0));
        // Length 4 (Medium)
        assert_eq!(score_guess(&[1, 2, 3, 4], &[1, 2, 3, 4]), (4, 0));
        // Length 5 (Hard)
        assert_eq!(score_guess(&[1, 2, 3, 4, 5], &[1, 2, 3, 4, 5]), (5, 0));
        // With duplicates
        assert_eq!(score_guess(&[2, 2, 5, 5, 1], &[2, 2, 5, 5, 1]), (5, 0));
    }

    #[test]
    fn test_score_guess_all_cows_various_lengths() {
        assert_eq!(score_guess(&[1, 2, 3], &[3, 1, 2]), (0, 3));
        assert_eq!(score_guess(&[1, 2, 3, 4], &[4, 3, 2, 1]), (0, 4));
        assert_eq!(score_guess(&[1, 2, 3, 4, 5], &[5, 4, 1, 2, 3]), (0, 5));
    }

    #[test]
    fn test_score_guess_mixed_and_duplicates() {
        // Target [1, 2, 2, 3], Guess [2, 2, 4, 2] -> 1 Bull, 1 Cow (not 2 cows!)
        assert_eq!(score_guess(&[1, 2, 2, 3], &[2, 2, 4, 2]), (1, 1));

        // Target [1, 1, 2, 2], Guess [2, 2, 1, 1] -> 0 Bulls, 4 Cows
        assert_eq!(score_guess(&[1, 1, 2, 2], &[2, 2, 1, 1]), (0, 4));

        // Target [1, 2, 3, 4, 5], Guess [1, 5, 4, 3, 2] -> 1 Bull, 4 Cows
        assert_eq!(score_guess(&[1, 2, 3, 4, 5], &[1, 5, 4, 3, 2]), (1, 4));

        // Disjoint sets
        assert_eq!(score_guess(&[1, 2, 3], &[4, 4, 4]), (0, 0));
        assert_eq!(score_guess(&[1, 2, 3, 4], &[6, 6, 6, 6]), (0, 0));
    }

    #[test]
    fn test_score_guess_repeated_values_in_target() {
        let target = [1, 1, 2, 3];
        // Pos 0 Bull, no extra cow
        assert_eq!(score_guess(&target, &[1, 4, 5, 6]), (1, 0));
        // Pos 2 Cow (misplaced 1), single cow awarded
        assert_eq!(score_guess(&target, &[4, 5, 1, 6]), (0, 1));
    }

    #[test]
    fn test_score_guess_repeated_values_in_guess() {
        let target = [1, 2, 3, 4];
        // Pos 0 Bull; remaining three 1s must not be counted as Cows
        assert_eq!(score_guess(&target, &[1, 1, 1, 1]), (1, 0));
        // Single cow awarded for one matching 1
        assert_eq!(score_guess(&target, &[5, 1, 1, 1]), (0, 1));
    }

    #[test]
    fn test_score_guess_mixed_matches() {
        let target = [1, 2, 3, 4];
        assert_eq!(score_guess(&target, &[1, 3, 2, 5]), (1, 2));
        assert_eq!(score_guess(&target, &[1, 2, 4, 3]), (2, 2));
    }

    #[test]
    #[should_panic(expected = "Target length (2) and guess length (3) must match")]
    fn test_score_guess_panics_on_length_mismatch() {
        score_guess(&[1, 2], &[1, 2, 3]);
    }

    // -------------------------------------------------------------------------
    // Reference Scorer for Multi-Length Differential Testing
    // -------------------------------------------------------------------------

    fn reference_score_guess(target: &[u32], guess: &[u32], max_val: u32) -> (usize, usize) {
        assert_eq!(target.len(), guess.len());
        let len = target.len();
        let mut bulls = 0;
        let mut target_freq = vec![0usize; (max_val + 1) as usize];
        let mut guess_freq = vec![0usize; (max_val + 1) as usize];

        for i in 0..len {
            if target[i] == guess[i] {
                bulls += 1;
            }
            if target[i] <= max_val {
                target_freq[target[i] as usize] += 1;
            }
            if guess[i] <= max_val {
                guess_freq[guess[i] as usize] += 1;
            }
        }

        let mut total_matches = 0;
        for v in 1..=(max_val as usize) {
            total_matches += target_freq[v].min(guess_freq[v]);
        }

        let cows = total_matches - bulls;
        (bulls, cows)
    }

    #[test]
    fn test_randomized_differential_scoring_all_difficulties() {
        const SEED: u64 = 42;
        const ITERATIONS_PER_DIFFICULTY: usize = 3_000;

        let mut rng = StdRng::seed_from_u64(SEED);

        for diff in Difficulty::all() {
            let config = diff.config();
            let m = config.sequence_length;
            let n = config.max_value;

            for _ in 0..ITERATIONS_PER_DIFFICULTY {
                let mut target = vec![0u32; m];
                let mut guess = vec![0u32; m];

                for i in 0..m {
                    target[i] = rng.gen_range(1..=n);
                    guess[i] = rng.gen_range(1..=n);
                }

                let actual = score_guess(&target, &guess);
                let expected = reference_score_guess(&target, &guess, n);

                assert_eq!(actual, expected);

                let (bulls, cows) = actual;
                assert!(bulls <= m);
                assert!(cows <= m);
                assert!(bulls + cows <= m);
                assert_eq!(score_guess(&target, &target), (m, 0));

                let mut permuted = target.clone();
                permuted.reverse();
                let (p_b, p_c) = score_guess(&target, &permuted);
                assert_eq!(p_b + p_c, m);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Unit Tests for calculate_score
    // -------------------------------------------------------------------------

    #[test]
    fn test_score_calculation_easy() {
        let config = Difficulty::Easy.config(); // L = 10, mult = 1.0

        // Win on Attempt 1: 1.0 + 9/10 = 1.9 -> round(100 * 1.0 * 1.9) = 190
        let s1 = calculate_score(&config, 1);
        assert_eq!(s1.attempts_used, 1);
        assert_eq!(s1.attempts_remaining, 9);
        assert!((s1.attempt_multiplier - 1.9).abs() < f64::EPSILON);
        assert_eq!(s1.final_score, 190);

        // Win on Attempt 10: 1.0 + 0/10 = 1.0 -> round(100 * 1.0 * 1.0) = 100
        let s10 = calculate_score(&config, 10);
        assert_eq!(s10.attempts_used, 10);
        assert_eq!(s10.attempts_remaining, 0);
        assert!((s10.attempt_multiplier - 1.0).abs() < f64::EPSILON);
        assert_eq!(s10.final_score, 100);
    }

    #[test]
    fn test_score_calculation_medium() {
        let config = Difficulty::Medium.config(); // L = 9, mult = 2.0

        // Win on Attempt 1: 1.0 + 8/9 = 1.888888... -> round(100 * 2.0 * 1.888888...) = 378
        let s1 = calculate_score(&config, 1);
        assert_eq!(s1.attempts_used, 1);
        assert_eq!(s1.attempts_remaining, 8);
        assert_eq!(s1.final_score, 378);

        // Win on Attempt 9: 1.0 + 0/9 = 1.0 -> round(100 * 2.0 * 1.0) = 200
        let s9 = calculate_score(&config, 9);
        assert_eq!(s9.attempts_used, 9);
        assert_eq!(s9.attempts_remaining, 0);
        assert_eq!(s9.final_score, 200);
    }

    #[test]
    fn test_score_calculation_hard() {
        let config = Difficulty::Hard.config(); // L = 8, mult = 4.0

        // Win on Attempt 1: 1.0 + 7/8 = 1.875 -> round(100 * 4.0 * 1.875) = 750
        let s1 = calculate_score(&config, 1);
        assert_eq!(s1.attempts_used, 1);
        assert_eq!(s1.attempts_remaining, 7);
        assert!((s1.attempt_multiplier - 1.875).abs() < f64::EPSILON);
        assert_eq!(s1.final_score, 750);

        // Win on Attempt 8: 1.0 + 0/8 = 1.0 -> round(100 * 4.0 * 1.0) = 400
        let s8 = calculate_score(&config, 8);
        assert_eq!(s8.attempts_used, 8);
        assert_eq!(s8.attempts_remaining, 0);
        assert_eq!(s8.final_score, 400);
    }

    #[test]
    fn test_score_bands_are_strictly_non_overlapping() {
        let easy_max = calculate_score(&Difficulty::Easy.config(), 1).final_score; // 190
        let med_min = calculate_score(&Difficulty::Medium.config(), 9).final_score; // 200
        let med_max = calculate_score(&Difficulty::Medium.config(), 1).final_score; // 378
        let hard_min = calculate_score(&Difficulty::Hard.config(), 8).final_score; // 400
        let hard_max = calculate_score(&Difficulty::Hard.config(), 1).final_score; // 750

        assert!(
            easy_max < med_min,
            "Easy max ({easy_max}) must be < Medium min ({med_min})"
        );
        assert!(
            med_max < hard_min,
            "Medium max ({med_max}) must be < Hard min ({hard_min})"
        );
        assert!(hard_min < hard_max);
    }

    #[test]
    #[should_panic(expected = "attempts_used (0) must be between 1 and max_attempts")]
    fn test_calculate_score_panics_on_zero_attempts() {
        calculate_score(&Difficulty::Easy.config(), 0);
    }

    #[test]
    #[should_panic(expected = "attempts_used (11) must be between 1 and max_attempts")]
    fn test_calculate_score_panics_on_exceeded_attempts() {
        calculate_score(&Difficulty::Easy.config(), 11);
    }
}

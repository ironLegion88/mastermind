//! Game configuration and difficulty definitions for Mastermind.

use std::fmt;

/// The base points awarded for successfully solving a secret code.
pub const BASE_POINTS: u32 = 100;

/// The available game difficulty levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Returns all available difficulty variants in ascending order.
    pub const fn all() -> &'static [Difficulty] {
        &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }

    /// Returns the human-readable display name of the difficulty.
    pub const fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }

    /// Returns the complete gameplay configuration for this difficulty setting.
    pub const fn config(&self) -> DifficultyConfig {
        match self {
            Difficulty::Easy => DifficultyConfig {
                difficulty: Difficulty::Easy,
                sequence_length: 3,
                max_value: 4,
                max_attempts: 10,
                difficulty_multiplier: 1.0,
            },
            Difficulty::Medium => DifficultyConfig {
                difficulty: Difficulty::Medium,
                sequence_length: 4,
                max_value: 6,
                max_attempts: 9,
                difficulty_multiplier: 2.0,
            },
            Difficulty::Hard => DifficultyConfig {
                difficulty: Difficulty::Hard,
                sequence_length: 5,
                max_value: 8,
                max_attempts: 8,
                difficulty_multiplier: 4.0,
            },
        }
    }

    /// Parses user menu input into a `Difficulty` variant.
    ///
    /// Accepts numeric selections ("1", "2", "3") as well as full names or abbreviations
    /// ("easy", "e", "medium", "m", "hard", "h"), case-insensitively.
    pub fn from_input(input: &str) -> Option<Difficulty> {
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("1")
            || trimmed.eq_ignore_ascii_case("easy")
            || trimmed.eq_ignore_ascii_case("e")
        {
            Some(Difficulty::Easy)
        } else if trimmed.eq_ignore_ascii_case("2")
            || trimmed.eq_ignore_ascii_case("medium")
            || trimmed.eq_ignore_ascii_case("med")
            || trimmed.eq_ignore_ascii_case("m")
        {
            Some(Difficulty::Medium)
        } else if trimmed.eq_ignore_ascii_case("3")
            || trimmed.eq_ignore_ascii_case("hard")
            || trimmed.eq_ignore_ascii_case("h")
        {
            Some(Difficulty::Hard)
        } else {
            None
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// The runtime configuration parameters associated with a chosen difficulty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifficultyConfig {
    /// The associated difficulty variant.
    pub difficulty: Difficulty,
    /// Sequence length (M): number of pegs in target and guesses.
    pub sequence_length: usize,
    /// Maximum peg value (N): numbers are sampled and validated in range 1..=N.
    pub max_value: u32,
    /// Maximum number of valid attempts (L) before game over.
    pub max_attempts: usize,
    /// Difficulty score multiplier.
    pub difficulty_multiplier: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_configurations() {
        let easy = Difficulty::Easy.config();
        assert_eq!(easy.sequence_length, 3);
        assert_eq!(easy.max_value, 4);
        assert_eq!(easy.max_attempts, 10);
        assert!((easy.difficulty_multiplier - 1.0).abs() < f64::EPSILON);

        let med = Difficulty::Medium.config();
        assert_eq!(med.sequence_length, 4);
        assert_eq!(med.max_value, 6);
        assert_eq!(med.max_attempts, 9);
        assert!((med.difficulty_multiplier - 2.0).abs() < f64::EPSILON);

        let hard = Difficulty::Hard.config();
        assert_eq!(hard.sequence_length, 5);
        assert_eq!(hard.max_value, 8);
        assert_eq!(hard.max_attempts, 8);
        assert!((hard.difficulty_multiplier - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_difficulty_from_input() {
        // Numeric choices
        assert_eq!(Difficulty::from_input("1"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::from_input("2"), Some(Difficulty::Medium));
        assert_eq!(Difficulty::from_input("3"), Some(Difficulty::Hard));

        // Name choices (case-insensitive with whitespace)
        assert_eq!(Difficulty::from_input("  easy  "), Some(Difficulty::Easy));
        assert_eq!(Difficulty::from_input("MEDIUM"), Some(Difficulty::Medium));
        assert_eq!(Difficulty::from_input("Hard"), Some(Difficulty::Hard));

        // Abbreviations
        assert_eq!(Difficulty::from_input("e"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::from_input("m"), Some(Difficulty::Medium));
        assert_eq!(Difficulty::from_input("h"), Some(Difficulty::Hard));

        // Invalid choices
        assert_eq!(Difficulty::from_input("0"), None);
        assert_eq!(Difficulty::from_input("4"), None);
        assert_eq!(Difficulty::from_input("extreme"), None);
        assert_eq!(Difficulty::from_input(""), None);
        assert_eq!(Difficulty::from_input("   "), None);
    }

    #[test]
    fn test_difficulty_display_and_all() {
        assert_eq!(Difficulty::Easy.to_string(), "Easy");
        assert_eq!(Difficulty::Medium.to_string(), "Medium");
        assert_eq!(Difficulty::Hard.to_string(), "Hard");

        let all = Difficulty::all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], Difficulty::Easy);
        assert_eq!(all[1], Difficulty::Medium);
        assert_eq!(all[2], Difficulty::Hard);
    }
}

//! Core game state, input parsing, target generation, and game outcomes for Mastermind.

use crate::config::DifficultyConfig;
use crate::scoring::{ScoreBreakdown, calculate_score, score_guess};
use std::fmt;

/// Errors that can occur when parsing a player's guess input.
#[derive(Debug, PartialEq, Eq)]
pub enum GuessError {
    /// The player entered the command to quit the current game.
    Quit,
    /// The player submitted an empty line or only whitespace.
    EmptyInput,
    /// The number of tokens entered did not match the expected sequence length.
    InvalidLength { expected: usize, actual: usize },
    /// A token could not be parsed as a positive integer.
    NonInteger(String),
    /// A number entered was outside the valid peg value range `1..=max_value`.
    OutOfRange { value: u32, min: u32, max: u32 },
}

impl fmt::Display for GuessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuessError::Quit => write!(f, "Game quit requested."),
            GuessError::EmptyInput => {
                write!(f, "Input was empty. Please enter your guess numbers.")
            }
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

/// Parses and validates a raw input string into a guess vector `Vec<u32>`
/// according to the specified difficulty configuration.
///
/// # Validation Steps:
/// 1. Splits input by arbitrary whitespace (spaces, multiple spaces, tabs).
/// 2. Recognizes the case-insensitive `quit` command before numeric parsing.
/// 3. Parses tokens into integers.
/// 4. Verifies that the count of numbers equals `config.sequence_length`.
/// 5. Verifies that each number lies in the inclusive range `1..=config.max_value`.
///
/// Repeated numbers are explicitly allowed and preserved.
pub fn parse_guess(input: &str, config: &DifficultyConfig) -> Result<Vec<u32>, GuessError> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.is_empty() {
        return Err(GuessError::EmptyInput);
    }

    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("quit") {
        return Err(GuessError::Quit);
    }

    let mut values = Vec::with_capacity(tokens.len());
    for token in &tokens {
        match token.parse::<u32>() {
            Ok(val) => values.push(val),
            Err(_) => return Err(GuessError::NonInteger((*token).to_string())),
        }
    }

    if values.len() != config.sequence_length {
        return Err(GuessError::InvalidLength {
            expected: config.sequence_length,
            actual: values.len(),
        });
    }

    for &val in &values {
        if !(1..=config.max_value).contains(&val) {
            return Err(GuessError::OutOfRange {
                value: val,
                min: 1,
                max: config.max_value,
            });
        }
    }

    Ok(values)
}

/// Generates a secret target sequence of length `config.sequence_length`,
/// with numbers randomly sampled in `1..=config.max_value`.
///
/// Repeated numbers are permitted in the target sequence.
pub fn generate_target(config: &DifficultyConfig) -> Vec<u32> {
    generate_target_with_rng(config, &mut rand::thread_rng())
}

/// Helper function generating a secret target sequence using an injected RNG.
///
/// Enables deterministic target generation in tests via seeded RNG implementations.
pub fn generate_target_with_rng<R: rand::Rng + ?Sized>(
    config: &DifficultyConfig,
    rng: &mut R,
) -> Vec<u32> {
    let mut target = Vec::with_capacity(config.sequence_length);
    for _ in 0..config.sequence_length {
        target.push(rng.gen_range(1..=config.max_value));
    }
    target
}

/// The result of an evaluated valid guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuessEvaluation {
    /// The guess sequence submitted by the player.
    pub guess: Vec<u32>,
    /// Number of Bulls scored.
    pub bulls: usize,
    /// Number of Cows scored.
    pub cows: usize,
    /// The attempt index (1-based).
    pub attempt_number: usize,
}

/// Represents the terminal outcome of a game session.
#[derive(Debug, Clone, PartialEq)]
pub enum GameOutcome {
    /// The player cracked the code before or on the final allowed attempt.
    Won {
        attempts: usize,
        score: ScoreBreakdown,
    },
    /// The player exhausted all allowed attempts without solving the code.
    Lost { attempts: usize, secret: Vec<u32> },
    /// The player voluntarily quit the game midway.
    Quit { attempts: usize, secret: Vec<u32> },
    /// Standard input closed (end-of-file reached).
    Eof,
}

/// Represents the player's choice at the end-of-round replay menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayChoice {
    /// Start a new game with the same difficulty setting.
    PlayAgainSame,
    /// Change the difficulty setting before starting the next game.
    ChangeDifficulty,
    /// Exit the application.
    Exit,
}

impl ReplayChoice {
    /// Parses user input for the replay decision menu.
    pub fn from_input(input: &str) -> Option<ReplayChoice> {
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("1")
            || trimmed.eq_ignore_ascii_case("same")
            || trimmed.eq_ignore_ascii_case("s")
            || trimmed.eq_ignore_ascii_case("y")
            || trimmed.eq_ignore_ascii_case("yes")
        {
            Some(ReplayChoice::PlayAgainSame)
        } else if trimmed.eq_ignore_ascii_case("2")
            || trimmed.eq_ignore_ascii_case("change")
            || trimmed.eq_ignore_ascii_case("c")
            || trimmed.eq_ignore_ascii_case("diff")
            || trimmed.eq_ignore_ascii_case("d")
        {
            Some(ReplayChoice::ChangeDifficulty)
        } else if trimmed.eq_ignore_ascii_case("3")
            || trimmed.eq_ignore_ascii_case("exit")
            || trimmed.eq_ignore_ascii_case("quit")
            || trimmed.eq_ignore_ascii_case("q")
            || trimmed.eq_ignore_ascii_case("n")
            || trimmed.eq_ignore_ascii_case("no")
        {
            Some(ReplayChoice::Exit)
        } else {
            None
        }
    }
}

/// Encapsulates active round state and game-play transition rules.
#[derive(Debug, Clone)]
pub struct GameSession {
    /// The active difficulty configuration.
    pub config: DifficultyConfig,
    /// The secret target sequence to be solved.
    pub secret: Vec<u32>,
    /// Number of valid, evaluated attempts used so far.
    pub attempts_used: usize,
    /// History of past evaluated guesses in this round.
    pub history: Vec<GuessEvaluation>,
}

impl GameSession {
    /// Creates a new game session with a given difficulty configuration and secret code.
    pub fn new(config: DifficultyConfig, secret: Vec<u32>) -> Self {
        assert_eq!(
            secret.len(),
            config.sequence_length,
            "Secret length must match difficulty sequence_length"
        );
        Self {
            config,
            secret,
            attempts_used: 0,
            history: Vec::new(),
        }
    }

    /// Returns the number of attempts remaining (`max_attempts - attempts_used`).
    pub fn attempts_remaining(&self) -> usize {
        self.config.max_attempts.saturating_sub(self.attempts_used)
    }

    /// Evaluates a valid guess sequence.
    ///
    /// Consumes one attempt, computes Bulls and Cows, records the evaluation in history,
    /// and checks win/loss conditions.
    ///
    /// # Edge Case Invariant
    /// A guess that matches all pegs on the *final* attempt is strictly treated as a **Win**,
    /// not a Loss.
    pub fn submit_guess(&mut self, guess: Vec<u32>) -> (GuessEvaluation, Option<GameOutcome>) {
        assert_eq!(
            guess.len(),
            self.config.sequence_length,
            "Guess length must match configured sequence length"
        );

        self.attempts_used += 1;
        let (bulls, cows) = score_guess(&self.secret, &guess);

        let eval = GuessEvaluation {
            guess,
            bulls,
            cows,
            attempt_number: self.attempts_used,
        };
        self.history.push(eval.clone());

        // Check victory first
        if bulls == self.config.sequence_length {
            let score = calculate_score(&self.config, self.attempts_used);
            let outcome = GameOutcome::Won {
                attempts: self.attempts_used,
                score,
            };
            return (eval, Some(outcome));
        }

        // If not won and attempts are exhausted, player loses
        if self.attempts_used >= self.config.max_attempts {
            let outcome = GameOutcome::Lost {
                attempts: self.attempts_used,
                secret: self.secret.clone(),
            };
            return (eval, Some(outcome));
        }

        (eval, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Difficulty;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_parse_guess_all_difficulties() {
        let easy_cfg = Difficulty::Easy.config();
        assert_eq!(parse_guess("1 2 3", &easy_cfg), Ok(vec![1, 2, 3]));
        assert_eq!(
            parse_guess("1 2 3 4", &easy_cfg),
            Err(GuessError::InvalidLength {
                expected: 3,
                actual: 4
            })
        );
        assert_eq!(
            parse_guess("1 2 5", &easy_cfg),
            Err(GuessError::OutOfRange {
                value: 5,
                min: 1,
                max: 4
            })
        );

        let hard_cfg = Difficulty::Hard.config();
        assert_eq!(parse_guess("1 2 3 4 5", &hard_cfg), Ok(vec![1, 2, 3, 4, 5]));
        assert_eq!(parse_guess("8 8 1 2 7", &hard_cfg), Ok(vec![8, 8, 1, 2, 7]));
        assert_eq!(
            parse_guess("1 2 3 4 9", &hard_cfg),
            Err(GuessError::OutOfRange {
                value: 9,
                min: 1,
                max: 8
            })
        );
        assert_eq!(
            parse_guess("1 2 3 4", &hard_cfg),
            Err(GuessError::InvalidLength {
                expected: 5,
                actual: 4
            })
        );
    }

    #[test]
    fn test_parse_guess_whitespace_and_quit() {
        let med_cfg = Difficulty::Medium.config();
        assert_eq!(parse_guess("quit", &med_cfg), Err(GuessError::Quit));
        assert_eq!(parse_guess("  QUIT  ", &med_cfg), Err(GuessError::Quit));
        assert_eq!(
            parse_guess("\t1   2 \t 3  4\n", &med_cfg),
            Ok(vec![1, 2, 3, 4])
        );
        assert_eq!(parse_guess("   ", &med_cfg), Err(GuessError::EmptyInput));
        assert_eq!(
            parse_guess("1 2 abc 4", &med_cfg),
            Err(GuessError::NonInteger("abc".to_string()))
        );
    }

    #[test]
    fn test_generate_target_all_difficulties() {
        let mut rng = StdRng::seed_from_u64(12345);

        for diff in Difficulty::all() {
            let cfg = diff.config();
            for _ in 0..100 {
                let target = generate_target_with_rng(&cfg, &mut rng);
                assert_eq!(target.len(), cfg.sequence_length);
                for val in target {
                    assert!((1..=cfg.max_value).contains(&val));
                }
            }
        }
    }

    #[test]
    fn test_attempt_limit_win_on_first_attempt() {
        let cfg = Difficulty::Medium.config();
        let target = vec![1, 2, 3, 4];
        let mut session = GameSession::new(cfg, target.clone());

        assert_eq!(session.attempts_remaining(), 9);
        let (eval, outcome) = session.submit_guess(vec![1, 2, 3, 4]);

        assert_eq!(eval.bulls, 4);
        assert_eq!(eval.cows, 0);
        assert_eq!(session.attempts_used, 1);
        assert_eq!(session.attempts_remaining(), 8);

        match outcome {
            Some(GameOutcome::Won { attempts, score }) => {
                assert_eq!(attempts, 1);
                assert_eq!(score.final_score, 378);
            }
            _ => panic!("Expected GameOutcome::Won"),
        }
    }

    #[test]
    fn test_attempt_limit_win_on_final_attempt() {
        let cfg = Difficulty::Hard.config(); // L = 8
        let target = vec![1, 2, 3, 4, 5];
        let mut session = GameSession::new(cfg, target.clone());

        // Submit 7 incorrect guesses
        for i in 1..=7 {
            let (eval, outcome) = session.submit_guess(vec![1, 1, 1, 1, 1]);
            assert_eq!(eval.attempt_number, i);
            assert_eq!(outcome, None);
            assert_eq!(session.attempts_remaining(), 8 - i);
        }

        // 8th (final) attempt: correct guess
        let (eval, outcome) = session.submit_guess(vec![1, 2, 3, 4, 5]);
        assert_eq!(eval.attempt_number, 8);
        assert_eq!(session.attempts_remaining(), 0);

        // Crucial requirement: Must win on final attempt!
        match outcome {
            Some(GameOutcome::Won { attempts, score }) => {
                assert_eq!(attempts, 8);
                assert_eq!(score.final_score, 400); // 100 * 4.0 * 1.0 = 400
            }
            _ => panic!("Expected victory on final attempt, but got: {:?}", outcome),
        }
    }

    #[test]
    fn test_attempt_limit_loss_on_final_attempt() {
        let cfg = Difficulty::Easy.config(); // L = 10
        let target = vec![1, 2, 3];
        let mut session = GameSession::new(cfg, target.clone());

        // Submit 9 non-winning guesses
        for _ in 1..=9 {
            let (_, outcome) = session.submit_guess(vec![4, 4, 4]);
            assert_eq!(outcome, None);
        }

        assert_eq!(session.attempts_remaining(), 1);

        // 10th (final) non-winning guess
        let (_, outcome) = session.submit_guess(vec![4, 4, 4]);
        assert_eq!(session.attempts_remaining(), 0);

        match outcome {
            Some(GameOutcome::Lost { attempts, secret }) => {
                assert_eq!(attempts, 10);
                assert_eq!(secret, vec![1, 2, 3]);
            }
            _ => panic!("Expected GameOutcome::Lost on final attempt"),
        }
    }

    #[test]
    fn test_replay_choice_parsing() {
        assert_eq!(
            ReplayChoice::from_input("1"),
            Some(ReplayChoice::PlayAgainSame)
        );
        assert_eq!(
            ReplayChoice::from_input("same"),
            Some(ReplayChoice::PlayAgainSame)
        );
        assert_eq!(
            ReplayChoice::from_input("y"),
            Some(ReplayChoice::PlayAgainSame)
        );

        assert_eq!(
            ReplayChoice::from_input("2"),
            Some(ReplayChoice::ChangeDifficulty)
        );
        assert_eq!(
            ReplayChoice::from_input("change"),
            Some(ReplayChoice::ChangeDifficulty)
        );
        assert_eq!(
            ReplayChoice::from_input("d"),
            Some(ReplayChoice::ChangeDifficulty)
        );

        assert_eq!(ReplayChoice::from_input("3"), Some(ReplayChoice::Exit));
        assert_eq!(ReplayChoice::from_input("exit"), Some(ReplayChoice::Exit));
        assert_eq!(ReplayChoice::from_input("q"), Some(ReplayChoice::Exit));
        assert_eq!(ReplayChoice::from_input("no"), Some(ReplayChoice::Exit));

        assert_eq!(ReplayChoice::from_input("unknown"), None);
        assert_eq!(ReplayChoice::from_input(""), None);
    }
}

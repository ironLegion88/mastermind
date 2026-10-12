//! CS2102 Programming Lab: Mid-Term Project
//! Terminal-based Master Mind / Bulls and Cows game in Rust.

use std::io::{self, BufRead, Write};

pub mod config;
pub mod game;
pub mod scoring;
pub mod ui;

// Re-exports for clean API access and backward compatibility
pub use config::{BASE_POINTS, Difficulty, DifficultyConfig};
pub use game::{
    GameOutcome, GameSession, GuessError, GuessEvaluation, ReplayChoice, generate_target,
    generate_target_with_rng, parse_guess,
};
pub use scoring::{ScoreBreakdown, calculate_score, score_guess};
pub use ui::{
    ColorMode, render_banner, render_difficulty_menu, render_error, render_feedback,
    render_loss_summary, render_progress_bar, render_quit_summary, render_replay_menu,
    render_status_panel, render_win_summary,
};

/// Default sequence length for Medium difficulty (preserved for backwards compatibility).
pub const M: usize = 4;

/// Default peg maximum value for Medium difficulty (preserved for backwards compatibility).
pub const N: u32 = 6;

/// Prompts the user to select a difficulty level, returning `None` on EOF.
pub fn select_difficulty<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    color: ColorMode,
) -> io::Result<Option<Difficulty>> {
    let mut buffer = String::new();
    loop {
        render_difficulty_menu(writer, color)?;
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            // Default to Medium if the user simply presses Enter
            return Ok(Some(Difficulty::Medium));
        }
        if let Some(diff) = Difficulty::from_input(trimmed) {
            return Ok(Some(diff));
        }
        writeln!(
            writer,
            "  {}",
            color.red("Invalid choice. Please enter 1, 2, or 3.")
        )?;
    }
}

/// Plays a single interactive round with the provided `GameSession`.
///
/// Prompts the player for guesses, validates input, tracks attempts against the limit,
/// evaluates Bulls and Cows, renders feedback, and checks win/loss conditions.
pub fn play_round<R: BufRead, W: Write>(
    session: &mut GameSession,
    reader: &mut R,
    writer: &mut W,
    cumulative_score: u32,
    color: ColorMode,
) -> io::Result<GameOutcome> {
    let mut line_buffer = String::new();

    loop {
        render_status_panel(writer, session, cumulative_score, color)?;

        write!(
            writer,
            "  Enter guess ({} numbers, 1–{}, or 'quit'): ",
            session.config.sequence_length, session.config.max_value
        )?;
        writer.flush()?;

        line_buffer.clear();
        let bytes_read = reader.read_line(&mut line_buffer)?;
        if bytes_read == 0 {
            writeln!(writer, "\n[End of input reached. Terminating session.]")?;
            return Ok(GameOutcome::Eof);
        }

        match parse_guess(&line_buffer, &session.config) {
            Ok(guess) => {
                let (eval, outcome) = session.submit_guess(guess);
                render_feedback(writer, &eval, color)?;

                if let Some(outcome) = outcome {
                    match &outcome {
                        GameOutcome::Won { score, .. } => {
                            render_win_summary(
                                writer,
                                score,
                                cumulative_score + score.final_score,
                                color,
                            )?;
                        }
                        GameOutcome::Lost { secret, attempts } => {
                            render_loss_summary(writer, secret, *attempts, color)?;
                        }
                        _ => {}
                    }
                    return Ok(outcome);
                }
            }
            Err(GuessError::Quit) => {
                render_quit_summary(writer, &session.secret, color)?;
                return Ok(GameOutcome::Quit {
                    attempts: session.attempts_used,
                    secret: session.secret.clone(),
                });
            }
            Err(err) => {
                // Invalid input does NOT consume an attempt
                render_error(writer, &err, color)?;
            }
        }
    }
}

/// Prompts the player with the post-game replay menu.
pub fn ask_replay<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    current_difficulty: Difficulty,
    color: ColorMode,
) -> io::Result<Option<ReplayChoice>> {
    let mut buffer = String::new();
    loop {
        render_replay_menu(writer, current_difficulty, color)?;
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            // Default to playing again on the same difficulty
            return Ok(Some(ReplayChoice::PlayAgainSame));
        }
        if let Some(choice) = ReplayChoice::from_input(trimmed) {
            return Ok(Some(choice));
        }
        writeln!(
            writer,
            "  {}",
            color.red("Invalid choice. Please enter 1, 2, or 3.")
        )?;
    }
}

/// Runs the Mastermind game loop with an explicit color mode.
pub fn run_with_color<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    color: ColorMode,
) -> io::Result<()> {
    render_banner(writer, color)?;

    let mut current_difficulty = match select_difficulty(reader, writer, color)? {
        Some(diff) => diff,
        None => return Ok(()),
    };

    let mut cumulative_score = 0u32;

    loop {
        let config = current_difficulty.config();
        let secret = generate_target(&config);
        let mut session = GameSession::new(config, secret);

        let outcome = play_round(&mut session, reader, writer, cumulative_score, color)?;

        match outcome {
            GameOutcome::Won { score, .. } => {
                cumulative_score += score.final_score;
            }
            GameOutcome::Lost { .. } | GameOutcome::Quit { .. } => {
                // No points awarded for losses or quits
            }
            GameOutcome::Eof => {
                break;
            }
        }

        match ask_replay(reader, writer, current_difficulty, color)? {
            Some(ReplayChoice::PlayAgainSame) => {
                continue;
            }
            Some(ReplayChoice::ChangeDifficulty) => {
                if let Some(new_diff) = select_difficulty(reader, writer, color)? {
                    current_difficulty = new_diff;
                } else {
                    break;
                }
            }
            Some(ReplayChoice::Exit) | None => {
                writeln!(
                    writer,
                    "\nThank you for playing Mastermind! Final session score: {} pts. Goodbye!",
                    cumulative_score
                )?;
                break;
            }
        }
    }

    Ok(())
}

/// Top-level entry point auto-detecting terminal color capabilities.
pub fn run<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    run_with_color(reader, writer, ColorMode::auto_detect())
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    if let Err(err) = run(&mut reader, &mut writer) {
        eprintln!("I/O Error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_end_to_end_single_win() {
        let cfg = Difficulty::Medium.config();
        let mut session = GameSession::new(cfg, vec![1, 2, 3, 4]);

        // Input: 1 wrong guess ("1 2 4 3"), 1 correct guess ("1 2 3 4")
        let input_bytes = b"1 2 4 3\n1 2 3 4\n";
        let mut reader = Cursor::new(input_bytes);
        let mut output = Vec::new();

        let outcome = play_round(
            &mut session,
            &mut reader,
            &mut output,
            0,
            ColorMode::Disabled,
        )
        .unwrap();

        match outcome {
            GameOutcome::Won { attempts, score } => {
                assert_eq!(attempts, 2);
                assert_eq!(score.final_score, 356); // 100 * 2.0 * (1 + 7/9) = 355.55... -> 356
            }
            _ => panic!("Expected GameOutcome::Won"),
        }

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.contains("VICTORY! CODE CRACKED"));
        assert!(out_str.contains("+356 pts"));
    }

    #[test]
    fn test_end_to_end_attempt_limit_loss() {
        let cfg = Difficulty::Easy.config(); // L = 10, M = 3, N = 4
        let mut session = GameSession::new(cfg, vec![1, 2, 3]);

        // 10 incorrect guesses
        let mut input = Vec::new();
        for _ in 0..10 {
            input.extend_from_slice(b"4 4 4\n");
        }
        let mut reader = Cursor::new(input);
        let mut output = Vec::new();

        let outcome = play_round(
            &mut session,
            &mut reader,
            &mut output,
            0,
            ColorMode::Disabled,
        )
        .unwrap();

        match outcome {
            GameOutcome::Lost { attempts, secret } => {
                assert_eq!(attempts, 10);
                assert_eq!(secret, vec![1, 2, 3]);
            }
            _ => panic!("Expected GameOutcome::Lost"),
        }

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.contains("GAME OVER - ATTEMPTS EXHAUSTED"));
        assert!(out_str.contains("The secret code was: [1 2 3]"));
    }

    #[test]
    fn test_end_to_end_win_on_final_attempt() {
        let cfg = Difficulty::Hard.config(); // L = 8, M = 5, N = 8
        let mut session = GameSession::new(cfg, vec![1, 2, 3, 4, 5]);

        let mut input = Vec::new();
        // 7 wrong guesses
        for _ in 0..7 {
            input.extend_from_slice(b"8 8 8 8 8\n");
        }
        // 8th guess: correct!
        input.extend_from_slice(b"1 2 3 4 5\n");

        let mut reader = Cursor::new(input);
        let mut output = Vec::new();

        let outcome = play_round(
            &mut session,
            &mut reader,
            &mut output,
            0,
            ColorMode::Disabled,
        )
        .unwrap();

        match outcome {
            GameOutcome::Won { attempts, score } => {
                assert_eq!(attempts, 8);
                assert_eq!(score.final_score, 400); // 100 * 4.0 * 1.0 = 400
            }
            _ => panic!("Expected win on final attempt"),
        }

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.contains("VICTORY! CODE CRACKED"));
    }

    #[test]
    fn test_invalid_input_does_not_consume_attempts() {
        let cfg = Difficulty::Easy.config(); // L = 10, M = 3, N = 4
        let mut session = GameSession::new(cfg, vec![1, 2, 3]);

        // 4 invalid inputs, followed by correct guess
        let input_bytes = b"1 2\n1 2 9\n1 2 x\n   \n1 2 3\n";
        let mut reader = Cursor::new(input_bytes);
        let mut output = Vec::new();

        let outcome = play_round(
            &mut session,
            &mut reader,
            &mut output,
            0,
            ColorMode::Disabled,
        )
        .unwrap();

        match outcome {
            GameOutcome::Won { attempts, score } => {
                assert_eq!(attempts, 1);
                assert_eq!(score.final_score, 190); // 100 * 1.0 * 1.9 = 190
            }
            _ => panic!("Expected win with 1 consumed attempt"),
        }
    }

    #[test]
    fn test_quit_midway() {
        let cfg = Difficulty::Medium.config();
        let mut session = GameSession::new(cfg, vec![1, 2, 3, 4]);

        let input_bytes = b"quit\n";
        let mut reader = Cursor::new(input_bytes);
        let mut output = Vec::new();

        let outcome = play_round(
            &mut session,
            &mut reader,
            &mut output,
            0,
            ColorMode::Disabled,
        )
        .unwrap();

        match outcome {
            GameOutcome::Quit { attempts, secret } => {
                assert_eq!(attempts, 0);
                assert_eq!(secret, vec![1, 2, 3, 4]);
            }
            _ => panic!("Expected GameOutcome::Quit"),
        }
    }

    #[test]
    fn test_select_difficulty_valid_and_invalid() {
        let input_bytes = b"invalid\n4\n1\n";
        let mut reader = Cursor::new(input_bytes);
        let mut output = Vec::new();

        let diff = select_difficulty(&mut reader, &mut output, ColorMode::Disabled).unwrap();
        assert_eq!(diff, Some(Difficulty::Easy));

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.contains("Invalid choice"));
    }

    #[test]
    fn test_ask_replay_choices() {
        let input_same = b"1\n";
        let mut r1 = Cursor::new(input_same);
        let mut out1 = Vec::new();
        assert_eq!(
            ask_replay(&mut r1, &mut out1, Difficulty::Easy, ColorMode::Disabled).unwrap(),
            Some(ReplayChoice::PlayAgainSame)
        );

        let input_change = b"2\n";
        let mut r2 = Cursor::new(input_change);
        let mut out2 = Vec::new();
        assert_eq!(
            ask_replay(&mut r2, &mut out2, Difficulty::Easy, ColorMode::Disabled).unwrap(),
            Some(ReplayChoice::ChangeDifficulty)
        );

        let input_exit = b"3\n";
        let mut r3 = Cursor::new(input_exit);
        let mut out3 = Vec::new();
        assert_eq!(
            ask_replay(&mut r3, &mut out3, Difficulty::Easy, ColorMode::Disabled).unwrap(),
            Some(ReplayChoice::Exit)
        );
    }

    #[test]
    fn test_run_session_difficulty_change_and_exit() {
        // Initial difficulty: Easy (1)
        // Round 1: quit immediately
        // Replay: Change difficulty (2)
        // New difficulty: Medium (2)
        // Round 2: quit immediately
        // Replay: Exit (3)
        let input_bytes = b"1\nquit\n2\n2\nquit\n3\n";
        let mut reader = Cursor::new(input_bytes);
        let mut output = Vec::new();

        let res = run_with_color(&mut reader, &mut output, ColorMode::Disabled);
        assert!(res.is_ok());

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.contains("M A S T E R M I N D"));
        assert!(out_str.contains("Select Difficulty"));
        assert!(out_str.contains("You quit the round"));
        assert!(out_str.contains("Final session score: 0 pts. Goodbye!"));
        // Confirm no ANSI codes leaked into plain text mode
        assert!(!out_str.contains("\x1b["));
    }
}

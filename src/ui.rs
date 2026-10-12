//! Terminal user interface, ANSI styling, Unicode box-drawing, and plain-text fallback.

use crate::config::Difficulty;
use crate::game::{GameSession, GuessError, GuessEvaluation};
use crate::scoring::ScoreBreakdown;
use std::io::{self, Write};

/// Color formatting mode for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// ANSI escape sequences enabled for rich colors.
    Enabled,
    /// Plain-text formatting with zero ANSI escape sequences.
    Disabled,
}

impl ColorMode {
    /// Automatically detects color mode based on environment variables.
    ///
    /// Respects the `NO_COLOR` standard (https://no-color.org).
    pub fn auto_detect() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            ColorMode::Disabled
        } else {
            ColorMode::Enabled
        }
    }

    /// Wraps text with an ANSI escape code and reliable reset, or returns plain text if disabled.
    fn style(&self, code: &str, text: &str) -> String {
        match self {
            ColorMode::Enabled => format!("\x1b[{code}m{text}\x1b[0m"),
            ColorMode::Disabled => text.to_string(),
        }
    }

    pub fn bold(&self, text: &str) -> String {
        self.style("1", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.style("2", text)
    }

    pub fn red(&self, text: &str) -> String {
        self.style("31", text)
    }

    pub fn green(&self, text: &str) -> String {
        self.style("32", text)
    }

    pub fn yellow(&self, text: &str) -> String {
        self.style("33", text)
    }

    pub fn blue(&self, text: &str) -> String {
        self.style("34", text)
    }

    pub fn magenta(&self, text: &str) -> String {
        self.style("35", text)
    }

    pub fn cyan(&self, text: &str) -> String {
        self.style("36", text)
    }

    /// Colorizes a difficulty name according to its tier.
    pub fn difficulty_color(&self, diff: Difficulty) -> String {
        match diff {
            Difficulty::Easy => self.green(diff.name()),
            Difficulty::Medium => self.yellow(diff.name()),
            Difficulty::Hard => self.magenta(diff.name()),
        }
    }
}

/// Renders a visual progress bar indicating remaining attempts.
///
/// Example: `[████████░░] 8 / 10 remaining`
pub fn render_progress_bar(
    remaining: usize,
    total: usize,
    bar_width: usize,
    color: ColorMode,
) -> String {
    if total == 0 {
        return String::new();
    }
    let filled_count = ((remaining as f64 / total as f64) * (bar_width as f64)).round() as usize;
    let filled_count = filled_count.min(bar_width);
    let empty_count = bar_width.saturating_sub(filled_count);

    let filled_str: String = "█".repeat(filled_count);
    let empty_str: String = "░".repeat(empty_count);

    let colored_bar = if remaining > total / 2 {
        color.green(&filled_str)
    } else if remaining > total / 4 {
        color.yellow(&filled_str)
    } else {
        color.red(&filled_str)
    };

    format!(
        "[{}{}] {} remaining",
        colored_bar,
        color.dim(&empty_str),
        remaining
    )
}

/// Renders the prominent Mastermind title banner and welcome panel.
pub fn render_banner<W: Write>(writer: &mut W, color: ColorMode) -> io::Result<()> {
    writeln!(
        writer,
        "{}",
        color.cyan("╭──────────────────────────────────────────────────╮")
    )?;
    writeln!(
        writer,
        "{}        {}        {}",
        color.cyan("│"),
        color.bold("M A S T E R M I N D   (Bulls & Cows)"),
        color.cyan("│")
    )?;
    writeln!(
        writer,
        "{}             {}            {}",
        color.cyan("│"),
        color.dim("Deduce the secret numeric code"),
        color.cyan("│")
    )?;
    writeln!(
        writer,
        "{}",
        color.cyan("╰──────────────────────────────────────────────────╯")
    )?;
    Ok(())
}

/// Renders the interactive difficulty selection menu.
pub fn render_difficulty_menu<W: Write>(writer: &mut W, color: ColorMode) -> io::Result<()> {
    writeln!(writer, "\n{}", color.bold("Select Difficulty:"))?;
    for (i, diff) in Difficulty::all().iter().enumerate() {
        let cfg = diff.config();
        let num = format!("{}.", i + 1);
        writeln!(
            writer,
            "  {} {:<6} [Length: {}, Values: 1–{}, Max Attempts: {}, Mult: {:.1}×]",
            color.cyan(&num),
            color.difficulty_color(*diff),
            cfg.sequence_length,
            cfg.max_value,
            cfg.max_attempts,
            cfg.difficulty_multiplier
        )?;
    }
    write!(writer, "Enter choice (1-3) [default: 2]: ")?;
    writer.flush()?;
    Ok(())
}

/// Renders the compact round status panel before guess input.
pub fn render_status_panel<W: Write>(
    writer: &mut W,
    session: &GameSession,
    cumulative_score: u32,
    color: ColorMode,
) -> io::Result<()> {
    let cfg = session.config;
    let attempts_used = session.attempts_used;
    let remaining = session.attempts_remaining();
    let bar = render_progress_bar(remaining, cfg.max_attempts, 10, color);

    writeln!(
        writer,
        "\n{}",
        color.dim("──────────────────────────────────────────────────")
    )?;
    writeln!(
        writer,
        "  Difficulty: {:<12} Code Length: {}    Values: 1–{}",
        color.difficulty_color(cfg.difficulty),
        color.bold(&cfg.sequence_length.to_string()),
        color.bold(&cfg.max_value.to_string())
    )?;
    writeln!(
        writer,
        "  Attempt:    {:<12} Progress:    {}",
        format!("{}/{}", attempts_used + 1, cfg.max_attempts),
        bar
    )?;
    if cumulative_score > 0 {
        writeln!(
            writer,
            "  Session Score: {}",
            color.yellow(&format!("{cumulative_score} pts"))
        )?;
    }
    writeln!(
        writer,
        "{}",
        color.dim("──────────────────────────────────────────────────")
    )?;
    Ok(())
}

/// Renders feedback for a submitted valid guess.
pub fn render_feedback<W: Write>(
    writer: &mut W,
    eval: &GuessEvaluation,
    color: ColorMode,
) -> io::Result<()> {
    let guess_str: String = eval
        .guess
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    let bulls_label = color.green(&format!(
        "{} Bull{}",
        eval.bulls,
        if eval.bulls == 1 { "" } else { "s" }
    ));
    let cows_label = color.yellow(&format!(
        "{} Cow{}",
        eval.cows,
        if eval.cows == 1 { "" } else { "s" }
    ));

    writeln!(
        writer,
        "  Guess #{}: [{}] -> {}, {}",
        eval.attempt_number,
        color.bold(&guess_str),
        bulls_label,
        cows_label
    )?;
    Ok(())
}

/// Renders the end-of-round victory panel with detailed score breakdown.
pub fn render_win_summary<W: Write>(
    writer: &mut W,
    score: &ScoreBreakdown,
    cumulative_score: u32,
    color: ColorMode,
) -> io::Result<()> {
    writeln!(
        writer,
        "\n{}",
        color.green("╭──────────────────────────────────────────────────╮")
    )?;
    writeln!(
        writer,
        "{}              {}             {}",
        color.green("│"),
        color.bold("VICTORY! CODE CRACKED"),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}",
        color.green("├──────────────────────────────────────────────────┤")
    )?;
    writeln!(
        writer,
        "{}  Difficulty:             {:<23} {}",
        color.green("│"),
        color.difficulty_color(score.difficulty),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}  Attempts Used:          {:<23} {}",
        color.green("│"),
        format!(
            "{} of {}",
            score.attempts_used,
            score.attempts_used + score.attempts_remaining
        ),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}  Base Points:            {:<23} {}",
        color.green("│"),
        format!("{} pts", score.base_points),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}  Difficulty Multiplier:  {:<23} {}",
        color.green("│"),
        format!("{:.1}×", score.difficulty_multiplier),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}  Efficiency Multiplier:  {:<23} {}",
        color.green("│"),
        format!("{:.4}×", score.attempt_multiplier),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}  Combined Multiplier:    {:<23} {}",
        color.green("│"),
        format!("{:.4}×", score.effective_multiplier),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}",
        color.green("├──────────────────────────────────────────────────┤")
    )?;
    writeln!(
        writer,
        "{}  Round Score:            {:<23} {}",
        color.green("│"),
        color.yellow(&format!("+{} pts", score.final_score)),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}  Total Session Score:    {:<23} {}",
        color.green("│"),
        color.bold(&format!("{cumulative_score} pts")),
        color.green("│")
    )?;
    writeln!(
        writer,
        "{}",
        color.green("╰──────────────────────────────────────────────────╯")
    )?;
    Ok(())
}

/// Renders the end-of-round loss panel when attempts are exhausted.
pub fn render_loss_summary<W: Write>(
    writer: &mut W,
    secret: &[u32],
    attempts: usize,
    color: ColorMode,
) -> io::Result<()> {
    let secret_str: String = secret
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    writeln!(
        writer,
        "\n{}",
        color.red("╭──────────────────────────────────────────────────╮")
    )?;
    writeln!(
        writer,
        "{}               {}              {}",
        color.red("│"),
        color.bold("GAME OVER - ATTEMPTS EXHAUSTED"),
        color.red("│")
    )?;
    writeln!(
        writer,
        "{}",
        color.red("├──────────────────────────────────────────────────┤")
    )?;
    writeln!(
        writer,
        "{}  You used all {} allowed attempts.           {}",
        color.red("│"),
        attempts,
        color.red("│")
    )?;
    writeln!(
        writer,
        "{}  The secret code was: [{}]               {}",
        color.red("│"),
        color.yellow(&secret_str),
        color.red("│")
    )?;
    writeln!(
        writer,
        "{}",
        color.red("╰──────────────────────────────────────────────────╯")
    )?;
    Ok(())
}

/// Renders the exit panel when the player quits voluntarily.
pub fn render_quit_summary<W: Write>(
    writer: &mut W,
    secret: &[u32],
    color: ColorMode,
) -> io::Result<()> {
    let secret_str: String = secret
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    writeln!(
        writer,
        "\n{} You quit the round. The secret code was [{}].",
        color.dim("[Aborted]"),
        color.yellow(&secret_str)
    )?;
    Ok(())
}

/// Renders user-facing error feedback.
pub fn render_error<W: Write>(
    writer: &mut W,
    error: &GuessError,
    color: ColorMode,
) -> io::Result<()> {
    writeln!(writer, "  {} {}", color.red("Error:"), error)?;
    writeln!(writer, "  {}", color.dim("Please enter your guess again."))?;
    Ok(())
}

/// Renders the replay decision menu after a completed round.
pub fn render_replay_menu<W: Write>(
    writer: &mut W,
    current_difficulty: Difficulty,
    color: ColorMode,
) -> io::Result<()> {
    writeln!(writer, "\n{}", color.bold("Play Again?"))?;
    writeln!(
        writer,
        "  1. Play again on {} difficulty",
        color.difficulty_color(current_difficulty)
    )?;
    writeln!(writer, "  2. Change difficulty setting")?;
    writeln!(writer, "  3. Exit Mastermind")?;
    write!(writer, "Enter choice (1-3) [default: 1]: ")?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::calculate_score;

    #[test]
    fn test_plain_text_mode_has_no_ansi_escapes() {
        let plain = ColorMode::Disabled;
        assert_eq!(plain.bold("test"), "test");
        assert_eq!(plain.green("hello"), "hello");
        assert_eq!(plain.red("error"), "error");
        assert!(!plain.bold("test").contains("\x1b["));
    }

    #[test]
    fn test_ansi_mode_wraps_and_resets() {
        let ansi = ColorMode::Enabled;
        let styled = ansi.green("success");
        assert!(styled.starts_with("\x1b[32m"));
        assert!(styled.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_progress_bar_rendering() {
        let bar_full = render_progress_bar(10, 10, 10, ColorMode::Disabled);
        assert!(bar_full.contains("██████████"));
        assert!(bar_full.contains("10 remaining"));

        let bar_empty = render_progress_bar(0, 10, 10, ColorMode::Disabled);
        assert!(bar_empty.contains("░░░░░░░░░░"));
        assert!(bar_empty.contains("0 remaining"));

        let bar_half = render_progress_bar(5, 10, 10, ColorMode::Disabled);
        assert!(bar_half.contains("█████"));
        assert!(bar_half.contains("░░░░░"));
    }

    #[test]
    fn test_ui_panels_render_in_both_modes() {
        let mut buf_ansi = Vec::new();
        let mut buf_plain = Vec::new();

        render_banner(&mut buf_ansi, ColorMode::Enabled).unwrap();
        render_banner(&mut buf_plain, ColorMode::Disabled).unwrap();

        let str_ansi = String::from_utf8(buf_ansi).unwrap();
        let str_plain = String::from_utf8(buf_plain).unwrap();

        assert!(str_ansi.contains("\x1b["));
        assert!(!str_plain.contains("\x1b["));
        assert!(str_plain.contains("M A S T E R M I N D"));
    }

    #[test]
    fn test_win_and_loss_panels_render_accurately() {
        let cfg = Difficulty::Medium.config();
        let score = calculate_score(&cfg, 3);
        let mut buf = Vec::new();
        render_win_summary(&mut buf, &score, 500, ColorMode::Disabled).unwrap();

        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("VICTORY! CODE CRACKED"));
        assert!(out.contains("Medium"));
        assert!(out.contains("3 of 9"));
        assert!(out.contains("500 pts"));

        let mut loss_buf = Vec::new();
        render_loss_summary(&mut loss_buf, &[1, 2, 3, 4], 9, ColorMode::Disabled).unwrap();
        let loss_out = String::from_utf8(loss_buf).unwrap();
        assert!(loss_out.contains("GAME OVER"));
        assert!(loss_out.contains("1 2 3 4"));
    }
}

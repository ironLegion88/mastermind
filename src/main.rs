//! CS2102 Programming Lab: Mid-Term Project
//! Terminal-based Master Mind / Bulls and Cows game in Rust.

/// The number of pegs (sequence length) in the secret target.
pub const M: usize = 4;

/// The maximum integer value for each peg (valid range is 1..=N).
pub const N: u32 = 6;

fn main() {
    println!("Master Mind / Bulls and Cows (M = {M}, N = {N})");
}

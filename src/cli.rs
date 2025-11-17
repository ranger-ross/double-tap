use clap::Parser;

/// Keyboard chattering filter configuration
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Threshold value for filtering in milliseconds
    #[arg(long)]
    pub threshold: Option<u64>,

    /// Optional keyboard name to filter by
    #[arg(long)]
    pub keyboard_name: Option<String>,
}

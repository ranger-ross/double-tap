use clap::Parser;

/// Keyboard chattering filter configuration
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Threshold value for filtering in milliseconds
    #[arg(long, default_value_t = 30)]
    pub threshold: u64,

    /// Optional keyboard name to filter by
    #[arg(long)]
    pub keyboard_name: Option<String>,
}

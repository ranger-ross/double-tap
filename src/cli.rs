use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "double-tap",
    about = "Fix keyboard chattering at the Linux input layer",
    after_help = "Requires root privileges to grab input devices."
)]
pub struct Args {
    #[arg(short, long, help = "Keyboard device name (auto-detected if omitted)")]
    pub keyboard: Option<String>,

    #[arg(
        short,
        long,
        default_value = "50",
        help = "Filter threshold in milliseconds"
    )]
    pub threshold: u32,

    #[arg(
        short,
        long,
        default_value = "1",
        value_parser = clap::value_parser!(u8).range(0..=2),
        help = "Verbosity level: 0=error, 1=info, 2=debug"
    )]
    pub verbosity: u8,
}

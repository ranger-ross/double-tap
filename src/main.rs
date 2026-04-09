mod cli;
mod device;
mod error;
mod filter;
mod keyboard;

use std::process;

use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    let args = cli::Args::parse();

    init_logging(args.verbosity);

    info!("double-tap starting");

    let keyboard_name = match keyboard::select_keyboard(args.keyboard.clone()) {
        Ok(name) => name,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    };

    if let Err(e) = device::run_event_loop(&keyboard_name, args.threshold) {
        error!("{}", e);
        process::exit(1);
    }
}

fn init_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => EnvFilter::new("error"),
        1 => EnvFilter::new("info"),
        _ => EnvFilter::new("debug"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(filter)
        .init();
}

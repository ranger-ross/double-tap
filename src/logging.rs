use anyhow::Result;
use flexi_logger::{AdaptiveFormat, Age, Cleanup, Criterion, Duplicate, FileSpec, Naming};

pub fn configure_logging() -> Result<()> {
    let mut log_dir = std::env::temp_dir();
    log_dir.push("double-tap");

    println!("Logging to {}", log_dir.display());

    flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(FileSpec::default().directory(log_dir))
        // Keep 6 hrs of logs as they will probably build up pretty quickly
        .rotate(
            Criterion::Age(Age::Hour),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(6),
        )
        .duplicate_to_stdout(Duplicate::All)
        .adaptive_format_for_stdout(AdaptiveFormat::Opt)
        .start()?;
    Ok(())
}

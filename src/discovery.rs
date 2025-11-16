use anyhow::{Context, Result, bail};
use log::{info, warn};
use std::fs;
use std::path::{Path, PathBuf};

const INPUT_DEVICES_PATH: &str = "/dev/input/by-id";
const KEYBOARD_NAME_SUFFIX: &str = "-kbd";

/// Retrieve the name of a keyboard from /dev/input/by-id.
pub fn retrieve_keyboard_name() -> Result<String> {
    let entries = fs::read_dir(INPUT_DEVICES_PATH)
        .with_context(|| format!("Failed to read directory {}", INPUT_DEVICES_PATH))?;

    let mut keyboard_devices: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(KEYBOARD_NAME_SUFFIX))
        .collect();

    let n_devices = keyboard_devices.len();

    if n_devices == 0 {
        bail!("Couldn't find a keyboard in '{}'", INPUT_DEVICES_PATH);
    }

    if n_devices == 1 {
        info!("Found keyboard: {}", keyboard_devices[0]);
        return Ok(keyboard_devices.remove(0));
    }

    warn!(
        "multiple keyboards found, selecting the first: {}",
        keyboard_devices[0]
    );
    Ok(keyboard_devices[0].clone())
}

/// Produce an absolute keyboard device path
pub fn abs_keyboard_path(device: &str) -> PathBuf {
    Path::new(INPUT_DEVICES_PATH).join(device)
}

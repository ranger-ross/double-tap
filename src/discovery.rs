use anyhow::{Context, Result, bail};
use dialoguer::Input;
use log::info;
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

    // Multiple devices → ask user to select
    println!("Select a device:");
    for (idx, device) in keyboard_devices.iter().enumerate() {
        println!("{}. {}", idx + 1, device);
    }

    let selected_idx: usize = loop {
        let input: String = Input::new()
            .with_prompt("Enter your choice (number)")
            .interact_text()?;

        match input.parse::<usize>() {
            Ok(num) if num >= 1 && num <= n_devices => break num,
            _ => println!("Please select a number between 1 and {}", n_devices),
        }
    };

    Ok(keyboard_devices[selected_idx - 1].clone())
}

/// Produce an absolute keyboard device path
pub fn abs_keyboard_path(device: &str) -> PathBuf {
    Path::new(INPUT_DEVICES_PATH).join(device)
}

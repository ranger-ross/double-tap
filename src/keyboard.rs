use std::path::Path;

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::error::Error;

const INPUT_BY_ID_PATH: &str = "/dev/input/by-id";

pub fn discover_keyboards() -> Result<Vec<String>> {
    let path = Path::new(INPUT_BY_ID_PATH);

    if !path.exists() {
        return Err(Error::NoKeyboardFound).context("no keyboards found");
    }

    let mut keyboards: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(path).context("failed to read /dev/input/by-id")? {
        let entry = entry.context("failed to read directory entry")?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.ends_with("-kbd") {
            debug!("found keyboard: {}", name_str);
            keyboards.push(name_str.to_string());
        }
    }

    Ok(keyboards)
}

pub fn select_keyboard(keyboard_arg: Option<String>) -> Result<String> {
    let keyboards = discover_keyboards().context("failed to discover keyboards")?;

    match keyboards.len() {
        0 => Err(Error::NoKeyboardFound).context("no keyboards found"),
        1 => {
            let keyboard = &keyboards[0];
            info!("using keyboard: {}", keyboard);
            Ok(keyboard.clone())
        }
        _ => {
            if let Some(ref name) = keyboard_arg {
                if keyboards.contains(name) {
                    info!("using specified keyboard: {}", name);
                    return Ok(name.clone());
                } else {
                    return Err(Error::KeyboardNotFound(name.clone()))
                        .context("keyboard not found in available devices");
                }
            }

            Err(Error::MultipleKeyboards(keyboards))
                .context("multiple keyboards found, please specify one with -k")
        }
    }
}

use std::{collections::HashMap, time::SystemTime};

use anyhow::{Context, Result};
use clap::Parser;
use evdev::{Device, EventSummary, EventType, uinput::VirtualDevice};
use input_event_codes::{EV_MSC, EV_SYN};
use log::Level;

use crate::{
    cli::Args,
    config::AppConfig,
    discovery::{abs_keyboard_path, retrieve_keyboard_name},
};

mod cli;
mod config;
mod discovery;

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let keyboard = match &args.keyboard_name {
        Some(k) => k.to_string(),
        None => retrieve_keyboard_name()?,
    };

    let mut config = AppConfig::load()?.unwrap_or_else(AppConfig::default);
    config.merge_args(&args);

    log::info!("KEYBOARD: {keyboard}");

    let kb_path = abs_keyboard_path(&keyboard);
    log::info!("path: {kb_path:?}");

    let mut device = Device::open(kb_path)?;

    // Hijack all inputs from this keyboard so we can filter out duplicates.
    device.grab()?;

    // Create a virtual keyboard to emit final events if we decide to not filter them.
    let vd = VirtualDevice::builder()?
        .name("double-tap virtual keyboard")
        .with_keys(device.supported_keys().context("keyboard without keys?")?)?
        .build()?;

    main_loop(device, vd, config)
}

fn main_loop(mut device: Device, mut vd: VirtualDevice, config: AppConfig) -> Result<()> {
    let mut last_key_up: HashMap<u16, SystemTime> = HashMap::new();
    let mut key_pressed: HashMap<u16, bool> = HashMap::new();

    while let Ok(events) = device.fetch_events() {
        for event in events {
            let code = event.code();
            let value = event.value();

            let threshold_ms = config
                .key_overrides
                .iter()
                .find(|k| k.code == code)
                .map(|k| k.threshold_ms)
                .unwrap_or(config.threshold_ms) as u128;

            let mut forward = |level: Level, reason: &str| -> Result<()> {
                log::log!(level, "Forwarding {code} ({reason})");
                vd.emit(&[event])?;
                Ok(())
            };

            if !matches!(event.destructure(), EventSummary::Key(..)) {
                forward(Level::Trace, "non-key event")?;
                continue;
            };

            if code == EV_SYN!() || code == EV_MSC!() {
                // `.emit()` already emits these for us
                continue;
            }

            if event.event_type() != EventType::KEY {
                forward(Level::Trace, "non-key event")?;
                continue;
            }

            if value > 1 {
                forward(Level::Debug, "hold")?;
                continue;
            }

            // key down
            if value == 0 {
                if *key_pressed.get(&code).unwrap_or(&false) {
                    last_key_up.insert(code, event.timestamp());
                    key_pressed.insert(code, false);
                    forward(Level::Debug, "key up")?;
                } else {
                    log::info!("FILTERING {} up: key not pressed beforehand", code);
                }
                continue;
            }

            let Some(prev) = last_key_up.get(&code) else {
                key_pressed.insert(code, true);
                forward(Level::Debug, "first press")?;
                continue;
            };

            let now = event.timestamp();
            let duration_between = now.duration_since(*prev)?.as_millis();

            if duration_between > threshold_ms {
                key_pressed.insert(code, true);
                forward(Level::Debug, "key down")?;
                continue;
            }
            log::info!(
                "FILTERED {code} down: last key up event happened {duration_between} ms ago"
            );
        }
    }

    Ok(())
}

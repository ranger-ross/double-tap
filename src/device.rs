
use anyhow::Result;
use evdev::uinput::VirtualDevice;
use evdev::{Device, EventType, InputEvent};
use tracing::{debug, info, warn};

use crate::error::Error;
use crate::filter::ChatteringFilter;

pub struct KeyboardDevice {
    device: Device,
}

impl KeyboardDevice {
    pub fn open(keyboard_name: &str) -> Result<Self> {
        let device_path = format!("/dev/input/by-id/{}", keyboard_name);

        let device = Device::open(&device_path).map_err(|e| Error::DeviceOpen(e))?;

        let name = device.name().unwrap_or("(unknown)");

        info!("opened device: {} ({})", name, device_path);

        Ok(Self { device })
    }

    pub fn grab(&mut self) -> Result<()> {
        self.device
            .grab()
            .map_err(|e| Error::DeviceGrab(format!("{:?}", e)))?;

        info!("grabbed device successfully");
        Ok(())
    }

    pub fn create_uinput(&self) -> Result<VirtualDevice> {
        let supported = self.device.supported_events();

        let builder = VirtualDevice::builder()
            .map_err(|e| Error::UinputCreate(format!("{:?}", e)))?
            .name("double-tap");

        let builder = if supported.contains(EventType::KEY) {
            if let Some(keys) = self.device.supported_keys() {
                builder
                    .with_keys(keys)
                    .map_err(|e| Error::UinputConfigure(format!("{:?}", e)))?
            } else {
                builder
            }
        } else {
            builder
        };

        let uinput = builder
            .build()
            .map_err(|e| Error::UinputCreate(format!("{:?}", e)))?;

        info!("created uinput device");
        Ok(uinput)
    }

    pub fn into_event_stream(self) -> Device {
        self.device
    }
}

pub fn run_event_loop(keyboard_name: &str, threshold_ms: u32) -> Result<()> {
    let mut device = KeyboardDevice::open(keyboard_name)?;
    device.grab()?;

    let mut uinput = device.create_uinput()?;

    let filter = ChatteringFilter::new(threshold_ms);
    filter.reset_state();

    info!("starting event loop (threshold: {}ms)", threshold_ms);

    let mut device = device.into_event_stream();

    loop {
        for event in device.fetch_events()? {
            process_event(&event, &mut uinput, &filter);
        }
    }
}

fn process_event(event: &InputEvent, uinput: &mut VirtualDevice, filter: &ChatteringFilter) {
    let timestamp = event.timestamp();

    let event_type = event.event_type();

    let code = event.code();
    let value = event.value();

    let should_forward = if event_type == EventType::KEY {
        filter.should_forward(code, value, &timestamp)
    } else {
        true
    };

    if should_forward {
        if let Err(e) = uinput.emit(&[*event]) {
            warn!("failed to write event: {}", e);
        } else {
            debug!(
                "forwarded: type={:?}, code={}, value={}",
                event_type, code, value
            );
        }
    }
}

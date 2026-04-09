#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to open device: {0}")]
    DeviceOpen(#[from] std::io::Error),

    #[error("no keyboard device found in /dev/input/by-id")]
    NoKeyboardFound,

    #[error("multiple keyboards found: {0:?}, use -k to specify")]
    MultipleKeyboards(Vec<String>),

    #[error("keyboard not found: {0}")]
    KeyboardNotFound(String),

    #[error("failed to grab device: {0}")]
    DeviceGrab(String),

    #[error("failed to create uinput device: {0}")]
    UinputCreate(String),

    #[error("failed to configure uinput device: {0}")]
    UinputConfigure(String),
}

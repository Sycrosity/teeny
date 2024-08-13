use core::fmt::Display;

use crate::display::TeenyDisplayError;

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum TeenyError {
    I2C,
    Unknown,
    InterfaceError,
    DisplayError(TeenyDisplayError),
}

impl Display for TeenyError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self {
            Self::I2C => write!(f, "I2C error"),
            Self::Unknown => write!(f, "Unknown error"),
            Self::InterfaceError => write!(f, "Interface error"),
            Self::DisplayError(e) => write!(f, "Display error: {:?}", e),
        }
    }
}


impl core::error::Error for TeenyError {}

use std::fmt;

use crate::{DDS_NAME, LPC_NAME, PHY_NAME};

#[derive(Debug)]
pub enum Error {
    NoSuchDevice(DevicePart),
    NoChannelOnDevice,
    GeneralIIOError(industrial_io::Error),
    NoRxBuff,
    NoTxBuff,
    UnexpectedStringValue(String),
    OutOfRangeIntValue(i64),
    OutOfRangeFloatValue(f64),
}

impl From<industrial_io::Error> for Error {
    fn from(error: industrial_io::Error) -> Self {
        Self::GeneralIIOError(error)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum DevicePart {
    Phy,
    Dds,
    Lpc,
}

impl fmt::Debug for DevicePart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phy => write!(f, "{PHY_NAME}"),
            Self::Dds => write!(f, "{DDS_NAME}"),
            Self::Lpc => write!(f, "{LPC_NAME}"),
        }
    }
}

use industrial_io as iio;

use iio::{Channel, Context, Device};
use std::fmt;

const PHY_NAME: &str = "ad9361-phy";
const DDS_NAME: &str = "cf-ad9361-dds-core-lpc";
const LPC_NAME: &str = "cf-ad9361-lpc";

#[derive(Debug)]
pub struct AD9361 {
    control_device: Device,
    rx_device: Device,
    tx_device: Device,
    rx_control_channels: [Channel; 2],
    tx_control_channels: [Channel; 2],
    rx_lo: Channel,
    tx_lo: Channel,
    rx_channels: [IQChannel; 2],
    tx_channels: [IQChannel; 2],
}

impl AD9361 {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        // Accuire devices
        let control_device = ctx
            .find_device(PHY_NAME)
            .ok_or(Error::NoSuchDevice(DevicePart::Phy))?;
        let rx_device = ctx
            .find_device(LPC_NAME)
            .ok_or(Error::NoSuchDevice(DevicePart::Lpc))?;
        let tx_device = ctx
            .find_device(DDS_NAME)
            .ok_or(Error::NoSuchDevice(DevicePart::Dds))?;

        // Accuire control channels
        let rx_control_channels = [
            control_device
                .find_channel("voltage0", false)
                .ok_or(Error::NoChannelOnDevice)?,
            control_device
                .find_channel("voltage1", false)
                .ok_or(Error::NoChannelOnDevice)?,
        ];
        let tx_control_channels = [
            control_device
                .find_channel("voltage0", true)
                .ok_or(Error::NoChannelOnDevice)?,
            control_device
                .find_channel("voltage1", true)
                .ok_or(Error::NoChannelOnDevice)?,
        ];

        // Accuire local oscillator channels
        let rx_lo = control_device
            .find_channel("altvoltage0", true)
            .ok_or(Error::NoChannelOnDevice)?;
        let tx_lo = control_device
            .find_channel("altvoltage1", true)
            .ok_or(Error::NoChannelOnDevice)?;

        // Accuire channels
        //TODO: This should be rewritten without code duplication
        let rx_channels = [
            IQChannel {
                i: rx_device
                    .find_channel("voltage0", false)
                    .ok_or(Error::NoChannelOnDevice)?,
                q: rx_device
                    .find_channel("voltage1", false)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
            IQChannel {
                i: rx_device
                    .find_channel("voltage2", false)
                    .ok_or(Error::NoChannelOnDevice)?,
                q: rx_device
                    .find_channel("voltage3", false)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
        ];
        let tx_channels = [
            IQChannel {
                i: tx_device
                    .find_channel("voltage0", true)
                    .ok_or(Error::NoChannelOnDevice)?,
                q: tx_device
                    .find_channel("voltage1", true)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
            IQChannel {
                i: tx_device
                    .find_channel("voltage2", true)
                    .ok_or(Error::NoChannelOnDevice)?,
                q: tx_device
                    .find_channel("voltage3", true)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
        ];

        Ok(Self {
            control_device,
            rx_device,
            tx_device,
            rx_control_channels,
            tx_control_channels,
            rx_lo,
            tx_lo,
            rx_channels,
            tx_channels,
        })
    }
}

#[derive(Debug)]
struct IQChannel {
    i: Channel,
    q: Channel,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    NoSuchDevice(DevicePart),
    NoChannelOnDevice,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum DevicePart {
    Phy,
    Dds,
    Lpc,
}

impl fmt::Debug for DevicePart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phy => write!(f, "{}", PHY_NAME),
            Self::Dds => write!(f, "{}", DDS_NAME),
            Self::Lpc => write!(f, "{}", LPC_NAME),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TxPortSelect {
    A,
    B,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RxPortSelect {
    ABalanced,
    AN,
    AP,
    BBalanced,
    BN,
    BP,
    CBalanced,
    CN,
    CP,
    TxMonitor1,
    TxMonitor12,
    TxMonitor2,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GainControlMode {
    FastAttack,
    Hybrid,
    Manual,
    SlowAttack,
}

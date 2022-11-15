use industrial_io as iio;

use iio::{Buffer, Channel, Context, Device};
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
    rx_buffer: Option<Buffer>,
    tx_buffer: Option<Buffer>,
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
            rx_buffer: None,
            tx_buffer: None,
        })
    }

    pub fn set_rx_rf_bandwidth(&self, chan_id: usize, bandwidth: i64) -> Result<(), Error> {
        self.rx_control_channels[chan_id].attr_write_int("rf_bandwidth", bandwidth)?;
        Ok(())
    }

    pub fn set_rx_sampling_frequency(&self, chan_id: usize, samplerate: i64) -> Result<(), Error> {
        self.rx_control_channels[chan_id].attr_write_int("sampling_frequency", samplerate)?;
        Ok(())
    }

    pub fn set_rx_lo(&self, freq: i64) -> Result<(), Error> {
        self.rx_lo.attr_write_int("frequency", freq)?;
        Ok(())
    }

    pub fn set_rx_port(&self, chan_id: usize, port: RxPortSelect) -> Result<(), Error> {
        self.rx_control_channels[chan_id].attr_write_str("rf_port_select", port.to_str())?;
        Ok(())
    }

    pub fn rx_enable(&self, chan_id: usize) {
        self.rx_channels[chan_id].i.enable();
        self.rx_channels[chan_id].q.enable();
    }

    pub fn rx_disable(&self, chan_id: usize) {
        self.rx_channels[chan_id].i.disable();
        self.rx_channels[chan_id].q.disable();
    }

    pub fn create_rx_buffer(&mut self, sample_count: usize, cyclic: bool) -> Result<(), Error> {
        let buffer = self.rx_device.create_buffer(sample_count, cyclic)?;
        self.rx_buffer = Some(buffer);
        Ok(())
    }

    pub fn destroy_rx_buffer(&mut self) {
        self.rx_buffer = None;
    }

    pub fn pool_samples_to_buff(&mut self) -> Result<usize, Error> {
        let Some(buf) = &mut self.rx_buffer else {return Err(Error::NoRxBuff);};
        let result = buf.refill()?;
        Ok(result)
    }

    pub fn read(&self, chan_id: usize) -> Result<Signal, Error> {
        let Some(buf) = &self.rx_buffer else {return Err(Error::NoRxBuff);};
        let i_channel: Vec<i16> = self.rx_channels[chan_id].i.read(&buf)?;
        let q_channel: Vec<i16> = self.rx_channels[chan_id].q.read(&buf)?;
        Ok(Signal {
            i_channel,
            q_channel,
        })
    }

    pub fn set_tx_rf_bandwidth(&self, chan_id: usize, bandwidth: i64) -> Result<(), Error> {
        self.tx_control_channels[chan_id].attr_write_int("rf_bandwidth", bandwidth)?;
        Ok(())
    }

    pub fn set_tx_sampling_frequency(&self, chan_id: usize, samplerate: i64) -> Result<(), Error> {
        self.tx_control_channels[chan_id].attr_write_int("sampling_frequency", samplerate)?;
        Ok(())
    }

    pub fn set_tx_lo(&self, freq: i64) -> Result<(), Error> {
        self.tx_lo.attr_write_int("frequency", freq)?;
        Ok(())
    }

    pub fn set_tx_port(&self, chan_id: usize, port: TxPortSelect) -> Result<(), Error> {
        self.tx_control_channels[chan_id].attr_write_str("rf_port_select", port.to_str())?;
        Ok(())
    }

    pub fn tx_enable(&self, chan_id: usize) {
        self.tx_channels[chan_id].i.enable();
        self.tx_channels[chan_id].q.enable();
    }

    pub fn tx_disable(&self, chan_id: usize) {
        self.tx_channels[chan_id].i.disable();
        self.tx_channels[chan_id].q.disable();
    }

    pub fn create_tx_buffer(&mut self, sample_count: usize, cyclic: bool) -> Result<(), Error> {
        let buffer = self.tx_device.create_buffer(sample_count, cyclic)?;
        self.tx_buffer = Some(buffer);
        Ok(())
    }

    pub fn destroy_tx_buffer(&mut self) {
        self.tx_buffer = None;
    }

    pub fn push_samples_to_device(&mut self) -> Result<usize, Error> {
        let Some(buf) = &mut self.tx_buffer else {return Err(Error::NoTxBuff);};
        let result = buf.push()?;
        Ok(result)
    }

    pub fn write(&self, chan_id: usize, signal: &Signal) -> Result<(usize, usize), Error> {
        let Some(buf) = &self.tx_buffer else {return Err(Error::NoTxBuff);};
        let write_i = self.tx_channels[chan_id].i.write(&buf, &signal.i_channel)?;
        let write_q = self.tx_channels[chan_id].q.write(&buf, &signal.q_channel)?;
        Ok((write_i, write_q))
    }
}

#[derive(Debug)]
struct IQChannel {
    i: Channel,
    q: Channel,
}

#[derive(Debug)]
pub enum Error {
    NoSuchDevice(DevicePart),
    NoChannelOnDevice,
    GeneralIIOError(industrial_io::Error),
    NoRxBuff,
    NoTxBuff,
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
            Self::Phy => write!(f, "{}", PHY_NAME),
            Self::Dds => write!(f, "{}", DDS_NAME),
            Self::Lpc => write!(f, "{}", LPC_NAME),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum TxPortSelect {
    A,
    B,
}

impl TxPortSelect {
    fn to_str(&self) -> &'static str {
        use TxPortSelect::*;
        match self {
            A => "A",
            B => "B",
        }
    }
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

impl RxPortSelect {
    fn to_str(&self) -> &'static str {
        use RxPortSelect::*;
        match self {
            ABalanced => "A_BALANCED",
            AN => "A_N",
            AP => "A_P",
            BBalanced => "B_BALANCED",
            BN => "B_N ",
            BP => "B_P",
            CBalanced => "C_BALANCED",
            CN => "C_N",
            CP => "C_P",
            TxMonitor1 => "TX_MONITOR1",
            TxMonitor12 => "TX_MONITOR1_2",
            TxMonitor2 => "TX_MONITOR2",
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GainControlMode {
    FastAttack,
    Hybrid,
    Manual,
    SlowAttack,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Signal {
    i_channel: Vec<i16>,
    q_channel: Vec<i16>,
}

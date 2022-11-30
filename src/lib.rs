use industrial_io as iio;

use iio::{Buffer, Channel as IIOChannel, Context, Device};
use std::cell::RefCell;

mod error;
mod rx_port_select;
mod tx_port_select;

pub use error::{DevicePart, Error};
pub use rx_port_select::RxPortSelect;
pub use tx_port_select::TxPortSelect;

const PHY_NAME: &str = "ad9361-phy";
const DDS_NAME: &str = "cf-ad9361-dds-core-lpc";
const LPC_NAME: &str = "cf-ad9361-lpc";

#[derive(Debug)]
pub struct AD9361 {
    control_device: Device,
    pub rx: RefCell<Transceiver<Rx>>,
    pub tx: RefCell<Transceiver<Tx>>,
}

impl AD9361 {
    pub fn from_ctx(ctx: &Context) -> Result<Self, Error> {
        // Acquire devices
        let control_device = ctx
            .find_device(PHY_NAME)
            .ok_or(Error::NoSuchDevice(DevicePart::Phy))?;
        let rx_device = ctx
            .find_device(LPC_NAME)
            .ok_or(Error::NoSuchDevice(DevicePart::Lpc))?;
        let tx_device = ctx
            .find_device(DDS_NAME)
            .ok_or(Error::NoSuchDevice(DevicePart::Dds))?;

        // Acquire local oscillator channels
        let rx_lo = control_device
            .find_channel("altvoltage0", true)
            .ok_or(Error::NoChannelOnDevice)?;
        let tx_lo = control_device
            .find_channel("altvoltage1", true)
            .ok_or(Error::NoChannelOnDevice)?;

        // Acquire channels
        //TODO: This should be rewritten without code duplication
        let rx_channels = [
            Channel {
                data: IQChannel {
                    i: rx_device
                        .find_channel("voltage0", false)
                        .ok_or(Error::NoChannelOnDevice)?,
                    q: rx_device
                        .find_channel("voltage1", false)
                        .ok_or(Error::NoChannelOnDevice)?,
                },
                _direction: Rx {},
                control: control_device
                    .find_channel("voltage0", false)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
            Channel {
                data: IQChannel {
                    i: rx_device
                        .find_channel("voltage2", false)
                        .ok_or(Error::NoChannelOnDevice)?,
                    q: rx_device
                        .find_channel("voltage3", false)
                        .ok_or(Error::NoChannelOnDevice)?,
                },
                _direction: Rx {},
                control: control_device
                    .find_channel("voltage1", false)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
        ];

        let tx_channels = [
            Channel {
                data: IQChannel {
                    i: tx_device
                        .find_channel("voltage0", true)
                        .ok_or(Error::NoChannelOnDevice)?,
                    q: tx_device
                        .find_channel("voltage1", true)
                        .ok_or(Error::NoChannelOnDevice)?,
                },
                _direction: Tx {},
                control: control_device
                    .find_channel("voltage0", true)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
            Channel {
                data: IQChannel {
                    i: tx_device
                        .find_channel("voltage2", true)
                        .ok_or(Error::NoChannelOnDevice)?,
                    q: tx_device
                        .find_channel("voltage3", true)
                        .ok_or(Error::NoChannelOnDevice)?,
                },
                _direction: Tx {},
                control: control_device
                    .find_channel("voltage1", true)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
        ];

        let rx = RefCell::new(Transceiver {
            channels: rx_channels,
            buffer: None,
            device: rx_device,
            lo: rx_lo,
        });

        let tx = RefCell::new(Transceiver {
            channels: tx_channels,
            buffer: None,
            device: tx_device,
            lo: tx_lo,
        });

        Ok(Self {
            control_device,
            rx,
            tx,
        })
    }
}

#[derive(Debug)]
struct IQChannel {
    i: IIOChannel,
    q: IIOChannel,
}

#[derive(Debug)]
struct Channel<T> {
    control: IIOChannel,
    data: IQChannel,
    _direction: T,
}

#[derive(Debug)]
struct ValueRange<T> {
    min: T,
    max: T,
    step: T,
}

// Marker structs for directioning
#[derive(Debug)]
pub struct Tx;
#[derive(Debug)]
pub struct Rx;

#[derive(Debug)]
pub struct Transceiver<T> {
    device: Device,
    lo: IIOChannel,
    channels: [Channel<T>; 2],
    buffer: Option<Buffer>,
}

impl<T> Transceiver<T> {
    pub fn set_rf_bandwidth(&self, chan_id: usize, bandwidth: i64) -> Result<(), Error> {
        self.channels[chan_id]
            .control
            .attr_write_int("rf_bandwidth", bandwidth)?;
        Ok(())
    }

    pub fn rf_bandwidth(&self, chan_id: usize) -> Result<i64, Error> {
        self.channels[chan_id]
            .control
            .attr_read_int("rf_bandwidth")
            .map_err(Error::from)
    }

    pub fn set_sampling_frequency(&self, chan_id: usize, samplerate: i64) -> Result<(), Error> {
        self.channels[chan_id]
            .control
            .attr_write_int("sampling_frequency", samplerate)?;
        Ok(())
    }

    pub fn sampling_frequency(&self, chan_id: usize) -> Result<i64, Error> {
        self.channels[chan_id]
            .control
            .attr_read_int("sampling_frequency")
            .map_err(Error::from)
    }

    pub fn set_lo(&self, freq: i64) -> Result<(), Error> {
        self.lo.attr_write_int("frequency", freq)?;
        Ok(())
    }

    pub fn lo(&self) -> Result<i64, Error> {
        self.lo.attr_read_int("frequency").map_err(Error::from)
    }

    pub fn enable(&self, chan_id: usize) {
        self.channels[chan_id].data.i.enable();
        self.channels[chan_id].data.q.enable();
    }

    pub fn disable(&self, chan_id: usize) {
        self.channels[chan_id].data.i.disable();
        self.channels[chan_id].data.q.disable();
    }

    pub fn create_buffer(&mut self, sample_count: usize, cyclic: bool) -> Result<(), Error> {
        let buffer = self.device.create_buffer(sample_count, cyclic)?;
        self.buffer = Some(buffer);
        Ok(())
    }

    pub fn destroy_buffer(&mut self) {
        self.buffer = None;
    }
}

impl Transceiver<Rx> {
    pub fn set_port(&self, chan_id: usize, port: RxPortSelect) -> Result<(), Error> {
        self.channels[chan_id]
            .control
            .attr_write_str("rf_port_select", port.to_str())?;
        Ok(())
    }

    pub fn port(&self, chan_id: usize) -> Result<RxPortSelect, Error> {
        let string = self.channels[chan_id]
            .control
            .attr_read_str("rf_port_select")?;
        RxPortSelect::try_from(string)
    }

    pub fn pool_samples_to_buff(&mut self) -> Result<usize, Error> {
        let Some(buf) = &mut self.buffer else {return Err(Error::NoRxBuff);};
        let result = buf.refill()?;
        Ok(result)
    }

    pub fn read(&self, chan_id: usize) -> Result<Signal, Error> {
        let Some(buf) = &self.buffer else {return Err(Error::NoRxBuff);};
        let i_channel: Vec<i16> = self.channels[chan_id].data.i.read(buf)?;
        let q_channel: Vec<i16> = self.channels[chan_id].data.q.read(buf)?;
        Ok(Signal {
            i_channel,
            q_channel,
        })
    }
}

impl Transceiver<Tx> {
    pub fn set_port(&self, chan_id: usize, port: TxPortSelect) -> Result<(), Error> {
        self.channels[chan_id]
            .control
            .attr_write_str("rf_port_select", port.to_str())?;
        Ok(())
    }

    pub fn port(&self, chan_id: usize) -> Result<TxPortSelect, Error> {
        let string = self.channels[chan_id]
            .control
            .attr_read_str("rf_port_select")?;
        TxPortSelect::try_from(string)
    }

    pub fn push_samples_to_device(&mut self) -> Result<usize, Error> {
        let Some(buf) = &mut self.buffer else {return Err(Error::NoTxBuff);};
        let result = buf.push()?;
        Ok(result)
    }

    pub fn write(&self, chan_id: usize, signal: &Signal) -> Result<(usize, usize), Error> {
        let Some(buf) = &self.buffer else {return Err(Error::NoTxBuff);};
        let write_i = self.channels[chan_id]
            .data
            .i
            .write(buf, &signal.i_channel)?;
        let write_q = self.channels[chan_id]
            .data
            .q
            .write(buf, &signal.q_channel)?;
        Ok((write_i, write_q))
    }
}

impl<T> Drop for Transceiver<T> {
    fn drop(&mut self) {
        self.buffer = None;
        self.disable(0);
        self.disable(1);
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
    pub i_channel: Vec<i16>,
    pub q_channel: Vec<i16>,
}

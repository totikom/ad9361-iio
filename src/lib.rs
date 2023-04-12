use industrial_io::{Buffer, Channel as IIOChannel, Context, Device};
use std::cell::RefCell;
use std::ops::{Range, RangeInclusive};

mod calib_mode;
mod channel;
mod ensm_mode;
mod error;

pub use calib_mode::CalibMode;
pub use channel::{GainControlMode, Rx, RxPortSelect, Tx, TxPortSelect};
pub use ensm_mode::ENSMMode;
pub use error::{DevicePart, Error};

use channel::Channel;

const DDS_NAME: &str = "cf-ad9361-dds-core-lpc";
const LPC_NAME: &str = "cf-ad9361-lpc";
const PHY_NAME: &str = "ad9361-phy";

const DCXO_COARSE_RANGE: Range<i64> = 1..64;
const DCXO_FINE_RANGE: Range<i64> = 1..8192;
const LO_FREQUENCY_RANGE: RangeInclusive<i64> = 46_875_001..=6_000_000_000;

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
            Channel::<Rx>::new(&rx_device, &control_device, 0)?,
            Channel::<Rx>::new(&rx_device, &control_device, 1)?,
        ];

        let tx_channels = [
            Channel::<Tx>::new(&tx_device, &control_device, 0)?,
            Channel::<Tx>::new(&tx_device, &control_device, 1)?,
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

    pub fn set_ensm_mode(&self, mode: ENSMMode) -> Result<(), Error> {
        self.control_device
            .attr_write_str("ensm_mode", mode.to_str())?;
        Ok(())
    }

    pub fn ensm_mode(&self) -> Result<ENSMMode, Error> {
        let string = self.control_device.attr_read_str("ensm_mode")?;
        ENSMMode::try_from(string)
    }

    pub fn set_calib_mode(&self, mode: CalibMode) -> Result<(), Error> {
        self.control_device
            .attr_write_str("calib_mode", mode.to_str())?;
        Ok(())
    }

    pub fn calib_mode(&self) -> Result<CalibMode, Error> {
        let string = self.control_device.attr_read_str("calib_mode")?;
        CalibMode::try_from(string)
    }

    pub fn set_dcxo_tune_fine(&self, dcxo: i64) -> Result<(), Error> {
        if DCXO_FINE_RANGE.contains(&dcxo) {
            self.control_device.attr_write_int("dcxo_tune_fine", dcxo)?;
            Ok(())
        } else {
            Err(Error::OutOfRangeIntValue(dcxo))
        }
    }

    pub fn dcxo_tune_fine(&self) -> Result<i64, Error> {
        self.control_device
            .attr_read_int("dcxo_tune_fine")
            .map_err(Error::from)
    }

    pub fn set_dcxo_tune_coarse(&self, dcxo: i64) -> Result<(), Error> {
        if DCXO_COARSE_RANGE.contains(&dcxo) {
            self.control_device
                .attr_write_int("dcxo_tune_coarse", dcxo)?;
            Ok(())
        } else {
            Err(Error::OutOfRangeIntValue(dcxo))
        }
    }

    pub fn dcxo_tune_coarse(&self) -> Result<i64, Error> {
        self.control_device
            .attr_read_int("dcxo_tune_coarse")
            .map_err(Error::from)
    }
}

#[derive(Debug)]
pub struct Transceiver<T> {
    device: Device,
    lo: IIOChannel,
    channels: [Channel<T>; 2],
    buffer: Option<Buffer>,
}

impl<T> Transceiver<T> {
    pub fn set_rf_bandwidth(&self, chan_id: usize, bandwidth: i64) -> Result<(), Error> {
        self.channels[chan_id].set_rf_bandwidth(bandwidth)
    }

    pub fn rf_bandwidth(&self, chan_id: usize) -> Result<i64, Error> {
        self.channels[chan_id].rf_bandwidth()
    }

    pub fn set_sampling_frequency(&self, chan_id: usize, samplerate: i64) -> Result<(), Error> {
        self.channels[chan_id].set_sampling_frequency(samplerate)
    }

    pub fn sampling_frequency(&self, chan_id: usize) -> Result<i64, Error> {
        self.channels[chan_id].sampling_frequency()
    }

    pub fn set_lo(&self, freq: i64) -> Result<(), Error> {
        if LO_FREQUENCY_RANGE.contains(&freq) {
            self.lo.attr_write_int("frequency", freq)?;
            Ok(())
        } else {
            Err(Error::OutOfRangeIntValue(freq))
        }
    }

    pub fn lo(&self) -> Result<i64, Error> {
        self.lo.attr_read_int("frequency").map_err(Error::from)
    }

    pub fn hardware_gain(&self, chan_id: usize) -> Result<f64, Error> {
        self.channels[chan_id].hardware_gain()
    }

    pub fn enable(&self, chan_id: usize) {
        self.channels[chan_id].enable();
    }

    pub fn disable(&self, chan_id: usize) {
        self.channels[chan_id].disable();
    }

    pub fn create_buffer(&mut self, sample_count: usize, cyclic: bool) -> Result<(), Error> {
        let buffer = self.device.create_buffer(sample_count, cyclic)?;
        self.buffer = Some(buffer);
        Ok(())
    }

    pub fn destroy_buffer(&mut self) {
        self.buffer = None;
    }

    pub fn rssi(&self, chan_id: usize) -> Result<f64, Error> {
        self.channels[chan_id].rssi()
        
    }
}

impl Transceiver<Rx> {
    pub fn set_port(&self, chan_id: usize, port: RxPortSelect) -> Result<(), Error> {
        self.channels[chan_id].set_port(port)
    }

    pub fn port(&self, chan_id: usize) -> Result<RxPortSelect, Error> {
        self.channels[chan_id].port()
    }

    pub fn set_hardware_gain(&self, chan_id: usize, gain: f64) -> Result<(), Error> {
        self.channels[chan_id].set_hardware_gain(gain)
    }

    pub fn pool_samples_to_buff(&mut self) -> Result<usize, Error> {
        let Some(buf) = &mut self.buffer else {return Err(Error::NoRxBuff);};
        let result = buf.refill()?;
        Ok(result)
    }

    pub fn read(&self, chan_id: usize) -> Result<Signal, Error> {
        let Some(buf) = &self.buffer else {return Err(Error::NoRxBuff);};
        self.channels[chan_id].read(buf)
    }
}

impl Transceiver<Tx> {
    pub fn set_gain_control_mode(
        &self,
        chan_id: usize,
        gain: GainControlMode,
    ) -> Result<(), Error> {
        self.channels[chan_id].set_gain_control_mode(gain)
    }

    pub fn gain_control_mode(&self, chan_id: usize) -> Result<GainControlMode, Error> {
        self.channels[chan_id].gain_control_mode()
    }

    pub fn set_port(&self, chan_id: usize, port: TxPortSelect) -> Result<(), Error> {
        self.channels[chan_id].set_port(port)
    }

    pub fn port(&self, chan_id: usize) -> Result<TxPortSelect, Error> {
        self.channels[chan_id].port()
    }

    pub fn set_hardware_gain(&self, chan_id: usize, gain: f64) -> Result<(), Error> {
        self.channels[chan_id].set_hardware_gain(gain)
    }

    pub fn push_samples_to_device(&mut self) -> Result<usize, Error> {
        let Some(buf) = &mut self.buffer else {return Err(Error::NoTxBuff);};
        let result = buf.push()?;
        Ok(result)
    }

    pub fn write(&self, chan_id: usize, signal: &Signal) -> Result<(usize, usize), Error> {
        let Some(buf) = &self.buffer else {return Err(Error::NoTxBuff);};
        self.channels[chan_id].write(signal, buf)
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
pub struct Signal {
    pub i_channel: Vec<i16>,
    pub q_channel: Vec<i16>,
}

#[cfg(debug_assertions)]
pub fn print_ctx(ctx: &Context, show_df: bool) {
    for dev in ctx.devices() {
        if let Some(name) = dev.name() {
            println!("Device: {}", name);
            if dev.has_attrs() {
                println!("Attributes:");
                for (name, value) in dev.attr_read_all().expect("Can't read attributes") {
                    println!("\t{}: {}", name, value);
                }
            }
            println!("Channels:");
            for channel in dev.channels() {
                println!(
                    "\tname: {}, id:{}, is_output: {}, type: {:?}",
                    channel.name().unwrap_or("None".to_owned()),
                    channel.id().unwrap(),
                    channel.is_output(),
                    channel.channel_type()
                );
                if show_df {
                    println!("{:#?}", channel.data_format());
                }
                if channel.has_attrs() {
                    println!("\tAttributes:");
                    for (name, value) in channel.attr_read_all().expect("Can't read attributes") {
                        println!("\t\t{}: {}", name, value);
                    }
                }
            }
        }
    }
}

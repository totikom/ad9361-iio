use industrial_io::{Buffer, Channel as IIOChannel, Device};
use std::ops::Range;

mod gain_control_mode;
mod rx_port_select;
mod tx_port_select;

use crate::error::Error;
use crate::Signal;

pub use gain_control_mode::GainControlMode;
pub use rx_port_select::RxPortSelect;
pub use tx_port_select::TxPortSelect;

const RF_BANDWIDTH_RANGE: Range<i64> = 200000..56000000;
const SAMPLING_FREQUENCY_RANGE: Range<i64> = 2083333..61440000;

// Marker structs for directioning
#[derive(Debug)]
pub struct Tx;
#[derive(Debug)]
pub struct Rx;

#[derive(Debug)]
struct IQChannel {
    i: IIOChannel,
    q: IIOChannel,
}

#[derive(Debug)]
pub struct Channel<T> {
    control: IIOChannel,
    data: IQChannel,
    _direction: T,
}

impl<T> Channel<T> {
    pub fn set_rf_bandwidth(&self, bandwidth: i64) -> Result<(), Error> {
        if RF_BANDWIDTH_RANGE.contains(&bandwidth) {
            self.control.attr_write_int("rf_bandwidth", bandwidth)?;
            Ok(())
        } else {
            Err(Error::OutOfRangeIntValue(bandwidth))
        }
    }

    pub fn rf_bandwidth(&self) -> Result<i64, Error> {
        self.control
            .attr_read_int("rf_bandwidth")
            .map_err(Error::from)
    }

    pub fn set_sampling_frequency(&self, samplerate: i64) -> Result<(), Error> {
        if SAMPLING_FREQUENCY_RANGE.contains(&samplerate) {
            self.control
                .attr_write_int("sampling_frequency", samplerate)?;
            Ok(())
        } else {
            Err(Error::OutOfRangeIntValue(samplerate))
        }
    }

    pub fn sampling_frequency(&self) -> Result<i64, Error> {
        self.control
            .attr_read_int("sampling_frequency")
            .map_err(Error::from)
    }

    pub fn enable(&self) {
        self.data.i.enable();
        self.data.q.enable();
    }

    pub fn disable(&self) {
        self.data.i.disable();
        self.data.q.disable();
    }
}

impl Channel<Rx> {
    pub fn set_port(&self, port: RxPortSelect) -> Result<(), Error> {
        self.control
            .attr_write_str("rf_port_select", port.to_str())?;
        Ok(())
    }

    pub fn port(&self) -> Result<RxPortSelect, Error> {
        let string = self.control.attr_read_str("rf_port_select")?;
        RxPortSelect::try_from(string)
    }

    pub fn read(&self, buf: &Buffer) -> Result<Signal, Error> {
        let i_channel: Vec<i16> = self.data.i.read(buf)?;
        let q_channel: Vec<i16> = self.data.q.read(buf)?;
        Ok(Signal {
            i_channel,
            q_channel,
        })
    }

    pub fn new(rx_device: &Device, control_device: &Device, index: usize) -> Result<Self, Error> {
        Ok(Channel {
            data: IQChannel {
                i: rx_device
                    .find_channel(format!("voltage{}", 2 * index).as_str(), false)
                    .ok_or(Error::NoChannelOnDevice)?,
                q: rx_device
                    .find_channel(format!("voltage{}", 2 * index + 1).as_str(), false)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
            _direction: Rx {},
            control: control_device
                .find_channel(format!("voltage{}", index).as_str(), false)
                .ok_or(Error::NoChannelOnDevice)?,
        })
    }
}

impl Channel<Tx> {
    pub fn set_gain_control_mode(&self, gain: GainControlMode) -> Result<(), Error> {
        self.control
            .attr_write_str("gain_control_mode", gain.to_str())?;
        Ok(())
    }

    pub fn gain_control_mode(&self) -> Result<GainControlMode, Error> {
        let string = self.control.attr_read_str("gain_control_mode")?;
        GainControlMode::try_from(string)
    }

    pub fn set_port(&self, port: TxPortSelect) -> Result<(), Error> {
        self.control
            .attr_write_str("rf_port_select", port.to_str())?;
        Ok(())
    }

    pub fn port(&self) -> Result<TxPortSelect, Error> {
        let string = self.control.attr_read_str("rf_port_select")?;
        TxPortSelect::try_from(string)
    }

    pub fn write(&self, signal: &Signal, buf: &Buffer) -> Result<(usize, usize), Error> {
        let write_i = self.data.i.write(buf, &signal.i_channel)?;
        let write_q = self.data.q.write(buf, &signal.q_channel)?;
        Ok((write_i, write_q))
    }

    pub fn new(tx_device: &Device, control_device: &Device, index: usize) -> Result<Self, Error> {
        Ok(Channel {
            data: IQChannel {
                i: tx_device
                    .find_channel(format!("voltage{}", 2 * index).as_str(), true)
                    .ok_or(Error::NoChannelOnDevice)?,
                q: tx_device
                    .find_channel(format!("voltage{}", 2 * index + 1).as_str(), true)
                    .ok_or(Error::NoChannelOnDevice)?,
            },
            _direction: Tx {},
            control: control_device
                .find_channel(format!("voltage{}", index).as_str(), true)
                .ok_or(Error::NoChannelOnDevice)?,
        })
    }
}

use crate::error::Error;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CalibMode {
    Auto,
    Manual,
    ManualTxQuad,
    TxQuad,
    RFdcOffs,
    RSSIGainStep,
}

impl TryFrom<String> for CalibMode {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        use CalibMode::*;
        match string.as_str() {
            "auto" => Ok(Auto),
            "manual" => Ok(Manual),
            "manual_tx_quad" => Ok(ManualTxQuad),
            "tx_quad" => Ok(TxQuad),
            "rf_dc_offs" => Ok(RFdcOffs),
            "rssi_gain_step" => Ok(RSSIGainStep),
            val => Err(Error::UnexpectedStringValue(val.to_string())),
        }
    }
}

impl CalibMode {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        use CalibMode::*;
        match self {
            Auto => "auto",
            Manual => "manual",
            ManualTxQuad => "manual_tx_quad",
            TxQuad => "tx_quad",
            RFdcOffs => "rf_dc_offs",
            RSSIGainStep => "rssi_gain_step",
        }
    }
}

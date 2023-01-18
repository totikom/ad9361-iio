use crate::error::Error;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ENSMMode {
    //ensm_mode_available: sleep wait alert fdd pinctrl pinctrl_fdd_indep
    Sleep,
    Wait,
    Alert,
    FDD,
    PinCtrl,
    PinCtrlFDDIndep,
}

impl TryFrom<String> for ENSMMode {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        use ENSMMode::*;
        match string.as_str() {
            "sleep" => Ok(Sleep),
            "wait" => Ok(Wait),
            "alert" => Ok(Alert),
            "fdd" => Ok(FDD),
            "pinctrl" => Ok(PinCtrl),
            "pinctrl_fdd_indep" => Ok(PinCtrlFDDIndep),
            val => Err(Error::UnexpectedStringValue(val.to_string())),
        }
    }
}

impl ENSMMode {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        use ENSMMode::*;
        match self {
            Sleep => "sleep",
            Wait => "wait",
            Alert => "alert",
            FDD => "fdd",
            PinCtrl => "pinctrl",
            PinCtrlFDDIndep => "pinctrl_fdd_indep",
        }
    }
}

use super::error::Error;

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

impl TryFrom<String> for RxPortSelect {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        use RxPortSelect::*;
        match string.as_str() {
            "A_BALANCED" => Ok(ABalanced),
            "A_N" => Ok(AN),
            "A_P" => Ok(AP),
            "B_BALANCED" => Ok(BBalanced),
            "B_N" => Ok(BN),
            "B_P" => Ok(BP),
            "C_BALANCED" => Ok(CBalanced),
            "C_N" => Ok(CN),
            "C_P" => Ok(CP),
            "TX_MONITOR1" => Ok(TxMonitor1),
            "TX_MONITOR1_2" => Ok(TxMonitor12),
            "TX_MONITOR2" => Ok(TxMonitor2),
            val => Err(Error::UnexpectedStringValue(val.to_string())),
        }
    }
}

impl RxPortSelect {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        use RxPortSelect::*;
        match self {
            ABalanced => "A_BALANCED",
            AN => "A_N",
            AP => "A_P",
            BBalanced => "B_BALANCED",
            BN => "B_N",
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

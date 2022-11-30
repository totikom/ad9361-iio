use super::error::Error;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GainControlMode {
    FastAttack,
    Hybrid,
    Manual,
    SlowAttack,
}

impl TryFrom<String> for GainControlMode {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        use GainControlMode::*;
        match string.as_str() {
            "fast_attack" => Ok(FastAttack),
            "hybrid" => Ok(Hybrid),
            "manual" => Ok(Manual),
            "slow_attack" => Ok(SlowAttack),
            val => Err(Error::UnexpectedStringValue(val.to_string())),
        }
    }
}

impl GainControlMode {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        use GainControlMode::*;
        match self {
            FastAttack => "fast_attack",
            Hybrid => "hybrid",
            Manual => "manual",
            SlowAttack => "slow_attack",
        }
    }
}

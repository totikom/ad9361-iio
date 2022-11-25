use super::error::Error;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum TxPortSelect {
    A,
    B,
}

impl TxPortSelect {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        use TxPortSelect::*;
        match self {
            A => "A",
            B => "B",
        }
    }
}

impl TryFrom<String> for TxPortSelect {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        use TxPortSelect::*;
        match string.as_str() {
            "A" => Ok(A),
            "B" => Ok(B),
            val => Err(Error::UnexpectedStringValue(val.to_string())),
        }
    }
}

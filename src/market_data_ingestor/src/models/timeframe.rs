use serde::{Deserialize, Serialize};
use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum TimeFrameError {
    #[snafu(display("Invalid amount for {unit:?}: {message}"))]
    InvalidAmount {
        unit: TimeFrameUnit,
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid input: {message}"))]
    InvalidInput {
        message: String,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeFrameUnit {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimeFrame {
    pub amount: u32,
    pub unit: TimeFrameUnit,
}

impl TimeFrame {
    pub fn new(amount: u32, unit: TimeFrameUnit) -> Self {
        Self { amount, unit }
    }
}

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeFrameError {
    #[error("Invalid amount for {:?}: {}", unit, message)]
    InvalidAmount {
        unit: TimeFrameUnit,
        message: String,
    },

    #[error("Invalid input: {}", message)]
    InvalidInput { message: String },
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

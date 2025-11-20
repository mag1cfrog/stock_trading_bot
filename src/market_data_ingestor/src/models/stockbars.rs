use crate::models::timeframe::TimeFrame;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct StockBarsParams {
    pub symbols: Vec<String>,
    pub timeframe: TimeFrame,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

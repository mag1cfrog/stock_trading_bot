use chrono::{DateTime, Utc};

use crate::models::{asset::AssetClass, timeframe::TimeFrame};

#[derive(Clone, Debug)]
pub struct BarsRequestParams {
    pub symbols: Vec<String>,
    pub timeframe: TimeFrame,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub asset_class: AssetClass,
}
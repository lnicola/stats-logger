use serde::Deserialize;
use time::{serde::timestamp, OffsetDateTime};

#[derive(Deserialize)]
pub struct Stats {
    #[serde(deserialize_with = "timestamp::deserialize")]
    pub time: OffsetDateTime,
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Deserialize)]
pub struct Stats2 {
    #[serde(deserialize_with = "timestamp::deserialize")]
    pub time: OffsetDateTime,
    pub temperature: f32,
    pub co2: u16,
}

#[derive(Deserialize)]
pub struct Tabs {
    #[serde(deserialize_with = "timestamp::deserialize")]
    pub time: OffsetDateTime,
    pub tabs: u16,
}

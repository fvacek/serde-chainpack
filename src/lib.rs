use chrono::{DateTime, FixedOffset};

pub mod de;
pub mod error;
pub mod ser;
pub mod types;
pub mod cpdatetime;

#[derive(Debug, PartialEq)]
pub struct CPDateTime(pub DateTime<FixedOffset>);

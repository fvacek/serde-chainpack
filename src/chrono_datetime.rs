use chrono::{DateTime, FixedOffset};
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

use crate::types;

pub fn serialize<S>(dt: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut bytes = [0u8; 12];
    let mut cursor = Cursor::new(&mut bytes[..]);
    cursor.write_i64::<LittleEndian>(dt.timestamp_millis()).unwrap();
    cursor.write_i32::<LittleEndian>(dt.offset().local_minus_utc()).unwrap();
    serializer.serialize_newtype_struct("DateTime", &bytes)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_newtype_struct("DateTime", DateTimeVisitor)
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<FixedOffset>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 13-byte array representing a ChainPack DateTime")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() != 13 || value[0] != types::CP_DATETIME {
            return Err(E::invalid_value(de::Unexpected::Bytes(value), &self));
        }
        let mut cursor = Cursor::new(&value[1..]);
        let msecs = cursor.read_i64::<LittleEndian>().map_err(E::custom)?;
        let offset_secs = cursor.read_i32::<LittleEndian>().map_err(E::custom)?;

        let offset = FixedOffset::east_opt(offset_secs)
            .ok_or_else(|| E::custom(format!("invalid timezone offset: {}", offset_secs)))?;

        let naive_dt = DateTime::from_timestamp(msecs / 1000, (msecs.rem_euclid(1000) * 1_000_000) as u32)
            .ok_or_else(|| E::custom(format!("invalid timestamp milliseconds: {}", msecs)))?;

        Ok(naive_dt.with_timezone(&offset))
    }
}
